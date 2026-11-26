//! Client tests against a local mock of the Tanca API.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use tanca::format::{self, Flavor};
use tanca::services::{Client, Config, GetEmployeeListParams, MakeShiftChangeRequestParams};

#[derive(Debug, Clone)]
struct Seen {
    method: String,
    path: String,
    query: String,
    headers: axum::http::HeaderMap,
    body: Vec<u8>,
}

type Reply = Box<dyn Fn(&Seen) -> (StatusCode, String) + Send + Sync>;

#[derive(Clone)]
struct Mock {
    seen: Arc<Mutex<Vec<Seen>>>,
    reply: Arc<Reply>,
}

async fn handle(State(mock): State<Mock>, req: Request) -> Response {
    let (parts, body) = req.into_parts();
    let seen = Seen {
        method: parts.method.to_string(),
        path: parts.uri.path().to_string(),
        query: parts.uri.query().unwrap_or("").to_string(),
        headers: parts.headers,
        body: to_bytes(body, usize::MAX).await.unwrap().to_vec(),
    };
    let (status, text) = (mock.reply)(&seen);
    mock.seen.lock().unwrap().push(seen);
    (status, Body::from(text)).into_response()
}

/// Starts a mock API and returns a client pointed at it plus the request log.
async fn start(
    shop_id: &str,
    reply: impl Fn(&Seen) -> (StatusCode, String) + Send + Sync + 'static,
) -> (Client, Arc<Mutex<Vec<Seen>>>) {
    let mock = Mock {
        seen: Arc::default(),
        reply: Arc::new(Box::new(reply)),
    };
    let seen = mock.seen.clone();
    let app = Router::new().fallback(handle).with_state(mock);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = Client::new(Config {
        base_url: format!("http://{addr}/api/v4"),
        token: "Bearer T".into(),
        shop_id: shop_id.into(),
        device_header: "DEV".into(),
        ..Default::default()
    })
    .with_retry(2, Duration::from_millis(1));
    (client, seen)
}

fn ok(data: &str) -> (StatusCode, String) {
    (
        StatusCode::OK,
        format!(r#"{{"error_code":0,"message":[],"data":{data}}}"#),
    )
}

#[tokio::test]
async fn employee_list_builds_query_and_headers() {
    let (client, seen) = start("S1", |_| {
        ok(
            r#"{"meta":{"total":1,"current_page":1,"total_pages":1},"items":[
            {"name":"An","user_id":"U1","username":"090","identification":null,
             "is_no_need_timekeeping":1,"working_date":"2024-01-02"}]}"#,
        )
    })
    .await;

    let data = client
        .get_employee_list(GetEmployeeListParams {
            department_ids: vec!["D 1".into()],
            ..Default::default()
        })
        .await
        .unwrap();

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    let req = &seen[0];
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/api/v4/employee/list");
    assert_eq!(
        req.query,
        "department_ids%5B0%5D=D+1&limit=30&page=1&shop_id=S1"
    );
    let h = |k: &str| req.headers.get(k).unwrap().to_str().unwrap().to_string();
    assert_eq!(h("authorization"), "Bearer T");
    assert_eq!(h("device"), "DEV");
    assert_eq!(h("is-admin"), "1");
    assert_eq!(h("lang"), "vi");
    assert_eq!(h("timezone"), "Asia/Saigon");
    assert_eq!(h("accept"), "application/json");

    assert_eq!(
        format::employees(&data),
        "Employees (Page 1/1, Total: 1)\n\n- An ()\n  User ID: U1\n  Phone: 090\n  Branch ID: \n  Department ID: \n  Position ID: \n  Needs Timekeeping: No\n  Working Date: 2024-01-02\n\n"
    );
}

#[tokio::test]
async fn shop_id_is_fetched_once_and_cached() {
    let (client, seen) = start("", |req| match req.path.as_str() {
        "/api/v4/shop/detail" => ok(r#"{"id":"SHOP","name":"Acme"}"#),
        _ => ok(r#"{"meta":{"total_pages":1},"items":[]}"#),
    })
    .await;

    for _ in 0..2 {
        client
            .get_employee_list(GetEmployeeListParams::default())
            .await
            .unwrap();
    }
    let seen = seen.lock().unwrap();
    let paths: Vec<&str> = seen.iter().map(|s| s.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "/api/v4/shop/detail",
            "/api/v4/employee/list",
            "/api/v4/employee/list"
        ]
    );
    assert!(seen[2].query.contains("shop_id=SHOP"));
}

#[tokio::test]
async fn server_errors_are_retried_and_client_errors_are_not() {
    let (client, seen) = start("S", |_| (StatusCode::BAD_GATEWAY, "down".into())).await;
    let err = client.get_shop_detail().await.unwrap_err();
    assert_eq!(
        format!("{err:#}"),
        "get shop detail: max retries exceeded: tanca API error 502: down"
    );
    assert_eq!(seen.lock().unwrap().len(), 3);

    let (client, seen) = start("S", |_| (StatusCode::UNAUTHORIZED, "no".into())).await;
    let err = client.get_shop_detail().await.unwrap_err();
    assert_eq!(
        format!("{err:#}"),
        "get shop detail: tanca API error 401: no"
    );
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn api_error_codes_become_errors() {
    let (client, _) = start("S", |_| {
        (
            StatusCode::OK,
            r#"{"error_code":5,"message":["bad","input"],"data":null}"#.into(),
        )
    })
    .await;
    let err = client.get_shift_assignments().await.unwrap_err();
    assert_eq!(format!("{err:#}"), "API error 5: [bad input]");
}

#[tokio::test]
async fn check_in_posts_json_body() {
    let (client, seen) = start("S", |_| {
        ok(r#"{"name":"Ca sáng","checkin_time":"2024-05-06 08:01:02","checkin_type":"admin","status":{"name":"OK"}}"#)
    })
    .await;
    let t = NaiveDate::from_ymd_opt(2024, 5, 6)
        .unwrap()
        .and_hms_opt(8, 1, 2)
        .unwrap();
    let resp = client.check_in("SA", "U", "B", t).await.unwrap();

    let seen = seen.lock().unwrap();
    let req = &seen[0];
    assert_eq!(req.method, "POST");
    assert_eq!(req.path, "/api/v4/shift/check-in-out-shift");
    assert_eq!(req.headers.get("content-type").unwrap(), "application/json");
    assert_eq!(
        String::from_utf8(req.body.clone()).unwrap(),
        r#"{"id":"SA","user_id":"U","branch_id":"B","working_day":"2024-05-06 00:00:00","checkin_time":"2024-05-06 08:01:02","checkout_time":"2024-05-06 08:01:02","is_checkin":1,"is_checkout":0,"reason":"MCP Admin"}"#
    );
    assert_eq!(
        format::check_in(&resp),
        "Check-in recorded successfully!\n\nShift: Ca sáng\nCheck-in Time: 2024-05-06 08:01:02\nCheck-in Type: admin\nStatus: OK\n"
    );
}

#[tokio::test]
async fn shift_change_request_posts_multipart_form() {
    let (client, seen) = start("S", |_| {
        ok(
            r#"{"id":"R1","serial":"DX-1","status":0,"checkin_datetime":"2024-05-06 08:00:00",
               "checkout_datetime":"2024-05-06 17:00:00","reason":"Quên chấm công",
               "user_obj":{"name":"An","username":"090"},
               "approver_obj":[{"name":"Bình"},{"name":"Chi"}]}"#,
        )
    })
    .await;
    let req = client
        .make_shift_change_request(MakeShiftChangeRequestParams {
            user_id: "U".into(),
            employee_shift_id: "ES".into(),
            working_day: "2024-05-06".into(),
            checkin_time: "2024-05-06 08:00:00".into(),
            checkout_time: "2024-05-06 17:00:00".into(),
            reason: "Quên chấm công".into(),
        })
        .await
        .unwrap();

    let seen = seen.lock().unwrap();
    let sent = &seen[0];
    assert_eq!(sent.path, "/api/v4/request/make-request");
    let ct = sent.headers.get("content-type").unwrap().to_str().unwrap();
    assert!(ct.starts_with("multipart/form-data; boundary="), "{ct}");
    let body = String::from_utf8(sent.body.clone()).unwrap();
    for (k, v) in [
        ("type", "employee-shift"),
        ("employee_shift_id", "ES"),
        ("working_day", "2024-05-06"),
        ("user_id", "U"),
        ("reason", "Quên chấm công"),
        ("checkin_time", "2024-05-06 08:00:00"),
        ("checkout_time", "2024-05-06 17:00:00"),
    ] {
        assert!(
            body.contains(&format!("name=\"{k}\"\r\n\r\n{v}\r\n")),
            "missing {k}"
        );
    }

    assert_eq!(
        format::shift_change_request(&req, Flavor::Mcp),
        "Time change request submitted successfully!\n\nRequest ID: R1\nSerial: DX-1\nStatus: Pending (chờ duyệt)\nWorking Day: 2024-05-06\nCheck-in: 2024-05-06 08:00:00\nCheck-out: 2024-05-06 17:00:00\nReason: Quên chấm công\nEmployee: An (090)\nApprover(s): Bình, Chi\n"
    );
    assert_eq!(
        format::shift_change_request(&req, Flavor::Cli),
        "Time change request submitted successfully!\n\nRequest ID: R1\nSerial: DX-1\nStatus: Pending\nCheck-in: 2024-05-06 08:00:00\nCheck-out: 2024-05-06 17:00:00\nReason: Quên chấm công\nEmployee: An (090)\nApprover(s): Bình, Chi\n"
    );
}

#[tokio::test]
async fn timekeeping_employees_follow_pagination() {
    let (client, seen) = start("S", |req| {
        let page = if req.query.contains("page=2") { 2 } else { 1 };
        ok(&format!(
            r#"{{"meta":{{"total_pages":2}},"items":[{{"name":"E{page}"}}]}}"#
        ))
    })
    .await;
    let all = client.get_timekeeping_employees().await.unwrap();
    assert_eq!(
        all.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
        ["E1", "E2"]
    );
    let seen = seen.lock().unwrap();
    assert_eq!(
        seen[0].query,
        "is_no_need_timekeeping=0&limit=100&page=1&shop_id=S"
    );
}

#[tokio::test]
async fn list_requests_query_and_json_output() {
    let (client, seen) = start("S", |_| {
        ok(
            r#"{"meta":{"current_page":1,"total_pages":1,"total":1},"items":[
            {"id":"R","type":"onleave","status":1,"serial":"NP-1","from_date":"2024-01-01",
             "to_date":"2024-01-02","created_at":"c","user_obj":null}]}"#,
        )
    })
    .await;
    let data = client
        .list_requests(tanca::services::ListRequestsParams {
            from_date: "2024-01-01".into(),
            to_date: "2024-01-31".into(),
            status: Some(1),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        seen.lock().unwrap()[0].query,
        "from_date=2024-01-01&page=1&status=1&to_date=2024-01-31&type="
    );
    assert_eq!(
        format::requests("2024-01-01", "2024-01-31", &data, Flavor::Cli),
        "Requests 2024-01-01 → 2024-01-31 (Page 1/1, Total: 1)\n\n- [NP-1] Nghỉ phép — Approved\n  ID: R\n  Range: 2024-01-01 → 2024-01-02\n  Created: c\n\n"
    );
    // JSON output keeps Go's field names and omitempty rules.
    let json = tanca::json::to_pretty_string(&data.items[0]).unwrap();
    assert_eq!(
        json,
        "{\n  \"id\": \"R\",\n  \"type\": \"onleave\",\n  \"status\": 1,\n  \"serial\": \"NP-1\",\n  \"reason\": \"\",\n  \"created_at\": \"c\",\n  \"from_date\": \"2024-01-01\",\n  \"to_date\": \"2024-01-02\"\n}\n"
    );
}
