//! `tanca-cli`: command-line access to the Tanca API.

use std::future::Future;
use std::process::exit;

use chrono::NaiveDateTime;
use serde::Serialize;
use tanca::format::{self, Flavor};
use tanca::goflag::{FlagSet, Values};
use tanca::services::{
    Client, GetEmployeeListParams, GetEmployeeShiftSummaryParams, ListRequestsParams,
    MakeShiftChangeRequestParams, now, parse_time, tanca_client,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        exit(1);
    }
    let rest = &args[2..];
    match args[1].as_str() {
        "list-employees" => run(list_employees(rest)),
        "list-timekeeping-employees" => run(list_timekeeping_employees(rest)),
        "list-shifts" => run(list_shifts(rest)),
        "get-shift-summary" => run(get_shift_summary(rest)),
        "get-today-shift" => run(get_today_shift(rest)),
        "get-shift-by-day" => run(get_shift_by_day(rest)),
        "check-in" => run(check_in(rest)),
        "check-out" => run(check_out(rest)),
        "get-clock-logs" => run(get_clock_logs(rest)),
        "get-shop-info" => run(get_shop_info(rest)),
        "make-shift-change-request" => run(make_shift_change_request(rest)),
        "list-requests" => run(list_requests(rest)),
        "get-request-detail" => run(get_request_detail(rest)),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("unknown command: {other}");
            exit(1);
        }
    }
}

fn run(fut: impl Future<Output = ()>) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to start async runtime")
        .block_on(fut);
}

fn print_usage() {
    print!(
        "tanca-cli - Tanca API command line interface

Usage:
  tanca-cli <command> [flags]

Commands:
  list-employees               List all employees
  list-timekeeping-employees   List employees requiring timekeeping
  list-shifts                  List all available shift assignments
  get-shift-summary            Get shift summary for an employee
  get-today-shift              Get today's shift for an employee
  get-shift-by-day             Get an employee's shift for a specific day (YYYY-MM-DD)
  check-in                     Record a check-in
  check-out                    Record a check-out
  get-clock-logs               Get clock logs for a shift
  get-shop-info                Get company/shop information
  make-shift-change-request    Submit a time-change request (đề xuất chỉnh giờ vào/ra)
  list-requests                List submitted requests filtered by date/type/status
  get-request-detail           Get full details of a request by ID

Global flags (available on all commands):
  --env string     Path to .env file
  --output string  Output format: text (default) or json
"
    );
}

/// A flag set with the global `--env` and `--output` flags.
fn flag_set(name: &str) -> FlagSet {
    let mut fs = FlagSet::new(name);
    fs.string("env", "", "Path to .env file").string(
        "output",
        "text",
        "Output format: text or json",
    );
    fs
}

/// Loads the env file (if any) and returns the shared client.
fn load_env_and_client(flags: &Values) -> &'static Client {
    let env_file = flags.str("env");
    if !env_file.is_empty()
        && let Err(e) = dotenvy::from_path(&env_file)
    {
        eprintln!("warning: failed to load env file {env_file}: {e}");
    }
    tanca_client()
}

/// Exits with status 1 unless every listed flag is non-empty.
fn require(flags: &Values, names: &[&str], message: &str) {
    if names.iter().any(|n| flags.str(n).is_empty()) {
        eprintln!("error: {message}");
        exit(1);
    }
}

/// Unwraps an API result or prints the error and exits with status 1.
fn check<T>(result: anyhow::Result<T>) -> T {
    result.unwrap_or_else(|e| {
        eprintln!("error: {e:#}");
        exit(1);
    })
}

/// Prints `data` as JSON when `--output json` was given, else `text()`.
fn emit<T: Serialize + ?Sized>(flags: &Values, data: &T, text: impl FnOnce() -> String) {
    if flags.str("output") == "json" {
        match tanca::json::to_pretty_string(data) {
            Ok(s) => print!("{s}"),
            Err(e) => {
                eprintln!("json encode error: {e}");
                exit(1);
            }
        }
    } else {
        print!("{}", text());
    }
}

/// Parses an optional time flag, defaulting to now.
fn time_or_now(flags: &Values, name: &str) -> NaiveDateTime {
    let value = flags.str(name);
    if value.is_empty() {
        return now();
    }
    parse_time(&value).unwrap_or_else(|e| {
        eprintln!("error: invalid --{name} format (use YYYY-MM-DD HH:mm:ss): {e}");
        exit(1);
    })
}

async fn list_employees(args: &[String]) {
    let mut fs = flag_set("list-employees");
    fs.int("page", 0, "Page number for pagination")
        .int("limit", 0, "Number of employees per page (max 100)")
        .string("department-id", "", "Filter by department ID")
        .string("start-date", "", "Filter start date (YYYY-MM-DD)")
        .string("end-date", "", "Filter end date (YYYY-MM-DD)");
    let f = fs.parse_or_exit(args);

    let client = load_env_and_client(&f);
    let mut params = GetEmployeeListParams {
        page: f.int("page"),
        limit: f.int("limit"),
        start_date: f.str("start-date"),
        end_date: f.str("end-date"),
        ..Default::default()
    };
    let department_id = f.str("department-id");
    if !department_id.is_empty() {
        params.department_ids = vec![department_id];
    }
    let data = check(client.get_employee_list(params).await);
    emit(&f, &data, || format::employees(&data));
}

async fn list_timekeeping_employees(args: &[String]) {
    let f = flag_set("list-timekeeping-employees").parse_or_exit(args);
    let client = load_env_and_client(&f);
    let employees = check(client.get_timekeeping_employees().await);
    emit(&f, &employees, || format::timekeeping_employees(&employees));
}

async fn list_shifts(args: &[String]) {
    let f = flag_set("list-shifts").parse_or_exit(args);
    let client = load_env_and_client(&f);
    let shifts = check(client.get_shift_assignments().await);
    emit(&f, &shifts, || format::shifts(&shifts));
}

async fn get_shift_summary(args: &[String]) {
    let mut fs = flag_set("get-shift-summary");
    fs.string("user-id", "", "Employee's user ID (required)")
        .string("start-date", "", "Start date (YYYY-MM-DD) (required)")
        .string("end-date", "", "End date (YYYY-MM-DD) (required)");
    let f = fs.parse_or_exit(args);
    require(
        &f,
        &["user-id", "start-date", "end-date"],
        "--user-id, --start-date, and --end-date are required",
    );

    let client = load_env_and_client(&f);
    let summary = check(
        client
            .get_employee_shift_summary(GetEmployeeShiftSummaryParams {
                user_id: f.str("user-id"),
                start_date: f.str("start-date"),
                end_date: f.str("end-date"),
            })
            .await,
    );
    emit(&f, &summary, || format::shift_summary(&summary));
}

async fn get_today_shift(args: &[String]) {
    let mut fs = flag_set("get-today-shift");
    fs.string("user-id", "", "Employee's user ID (required)");
    let f = fs.parse_or_exit(args);
    require(&f, &["user-id"], "--user-id is required");

    let client = load_env_and_client(&f);
    let user_id = f.str("user-id");
    let shift = check(client.get_today_shift(&user_id).await);
    emit(&f, &shift, || format::today_shift(&user_id, &shift));
}

async fn get_shift_by_day(args: &[String]) {
    let mut fs = flag_set("get-shift-by-day");
    fs.string("user-id", "", "Employee's user ID (required)")
        .string("working-day", "", "Working day YYYY-MM-DD (required)");
    let f = fs.parse_or_exit(args);
    require(
        &f,
        &["user-id", "working-day"],
        "--user-id and --working-day are required",
    );

    let client = load_env_and_client(&f);
    let (user_id, working_day) = (f.str("user-id"), f.str("working-day"));
    let shifts = check(
        client
            .list_employee_shifts_for_day(&user_id, &working_day)
            .await,
    );
    emit(&f, &shifts, || {
        format::shifts_by_day(&user_id, &working_day, &shifts)
    });
}

fn check_flags(name: &str, time_flag: &str, time_usage: &str) -> FlagSet {
    let mut fs = flag_set(name);
    fs.string("shift-assignment-id", "", "Shift assignment ID (required)")
        .string("user-id", "", "Employee's user ID (required)")
        .string("branch-id", "", "Branch ID (required)")
        .string(time_flag, "", time_usage);
    fs
}

const CHECK_REQUIRED: [&str; 3] = ["shift-assignment-id", "user-id", "branch-id"];
const CHECK_REQUIRED_MSG: &str = "--shift-assignment-id, --user-id, and --branch-id are required";

async fn check_in(args: &[String]) {
    let f = check_flags(
        "check-in",
        "checkin-time",
        "Check-in time (YYYY-MM-DD HH:mm:ss). Defaults to now.",
    )
    .parse_or_exit(args);
    require(&f, &CHECK_REQUIRED, CHECK_REQUIRED_MSG);
    let t = time_or_now(&f, "checkin-time");

    let client = load_env_and_client(&f);
    let resp = check(
        client
            .check_in(
                &f.str("shift-assignment-id"),
                &f.str("user-id"),
                &f.str("branch-id"),
                t,
            )
            .await,
    );
    emit(&f, &resp, || format::check_in(&resp));
}

async fn check_out(args: &[String]) {
    let f = check_flags(
        "check-out",
        "checkout-time",
        "Check-out time (YYYY-MM-DD HH:mm:ss). Defaults to now.",
    )
    .parse_or_exit(args);
    require(&f, &CHECK_REQUIRED, CHECK_REQUIRED_MSG);
    let t = time_or_now(&f, "checkout-time");

    let client = load_env_and_client(&f);
    let resp = check(
        client
            .check_out(
                &f.str("shift-assignment-id"),
                &f.str("user-id"),
                &f.str("branch-id"),
                t,
            )
            .await,
    );
    emit(&f, &resp, || format::check_out(&resp));
}

async fn get_clock_logs(args: &[String]) {
    let mut fs = flag_set("get-clock-logs");
    fs.string("employee-shift-id", "", "Employee shift ID (required)");
    let f = fs.parse_or_exit(args);
    require(
        &f,
        &["employee-shift-id"],
        "--employee-shift-id is required",
    );

    let client = load_env_and_client(&f);
    let id = f.str("employee-shift-id");
    let logs = check(client.get_checked_time(&id).await);
    emit(&f, &logs, || format::clock_logs(&id, &logs));
}

async fn get_shop_info(args: &[String]) {
    let f = flag_set("get-shop-info").parse_or_exit(args);
    let client = load_env_and_client(&f);
    let shop = check(client.get_shop_detail().await);
    emit(&f, &shop, || format::shop(&shop));
}

async fn make_shift_change_request(args: &[String]) {
    let mut fs = flag_set("make-shift-change-request");
    fs.string("user-id", "", "Employee's user ID (required)")
        .string(
            "employee-shift-id",
            "",
            "Employee shift ID for the working day (required)",
        )
        .string("working-day", "", "Working day YYYY-MM-DD (required)")
        .string(
            "checkin-time",
            "",
            "Proposed check-in datetime YYYY-MM-DD HH:mm:ss (required)",
        )
        .string(
            "checkout-time",
            "",
            "Proposed check-out datetime YYYY-MM-DD HH:mm:ss (required)",
        )
        .string("reason", "", "Reason for the change (required)");
    let f = fs.parse_or_exit(args);
    require(
        &f,
        &[
            "user-id",
            "employee-shift-id",
            "working-day",
            "checkin-time",
            "checkout-time",
            "reason",
        ],
        "--user-id, --employee-shift-id, --working-day, --checkin-time, --checkout-time, and --reason are required",
    );

    let client = load_env_and_client(&f);
    let req = check(
        client
            .make_shift_change_request(MakeShiftChangeRequestParams {
                user_id: f.str("user-id"),
                employee_shift_id: f.str("employee-shift-id"),
                working_day: f.str("working-day"),
                checkin_time: f.str("checkin-time"),
                checkout_time: f.str("checkout-time"),
                reason: f.str("reason"),
            })
            .await,
    );
    emit(&f, &req, || format::shift_change_request(&req, Flavor::Cli));
}

async fn list_requests(args: &[String]) {
    let mut fs = flag_set("list-requests");
    fs.string("from-date", "", "Start date YYYY-MM-DD (required)")
        .string("to-date", "", "End date YYYY-MM-DD (required)")
        .string(
            "type",
            "",
            "Type filter: employee-shift, onleave, overtime, etc.",
        )
        .int(
            "status",
            -1,
            "Status filter: 0=pending, 1=approved, 2=rejected. -1 for all.",
        )
        .int("page", 1, "Page number");
    let f = fs.parse_or_exit(args);
    require(
        &f,
        &["from-date", "to-date"],
        "--from-date and --to-date are required",
    );

    let client = load_env_and_client(&f);
    let (from_date, to_date) = (f.str("from-date"), f.str("to-date"));
    let status = f.int("status");
    let data = check(
        client
            .list_requests(ListRequestsParams {
                from_date: from_date.clone(),
                to_date: to_date.clone(),
                kind: f.str("type"),
                status: (status >= 0).then_some(status),
                page: f.int("page"),
            })
            .await,
    );
    emit(&f, &data, || {
        format::requests(&from_date, &to_date, &data, Flavor::Cli)
    });
}

async fn get_request_detail(args: &[String]) {
    let mut fs = flag_set("get-request-detail");
    fs.string("request-id", "", "Request ID (required)");
    let f = fs.parse_or_exit(args);
    require(&f, &["request-id"], "--request-id is required");

    let client = load_env_and_client(&f);
    let r = check(client.get_request_detail(&f.str("request-id")).await);
    emit(&f, &r, || format::request_detail(&r, Flavor::Cli));
}
