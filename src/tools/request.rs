use serde::Deserialize;
use serde_json::Value;

use super::{ToolEntry, ToolError, bind, entry, failed, number, required_string, string};
use crate::format::{self, Flavor};
use crate::services::{ListRequestsParams, MakeShiftChangeRequestParams, tanca_client};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct MakeShiftChangeRequestInput {
    user_id: String,
    employee_shift_id: String,
    working_day: String,
    checkin_time: String,
    checkout_time: String,
    reason: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ListRequestsInput {
    from_date: String,
    to_date: String,
    #[serde(rename = "type")]
    kind: String,
    status: Option<i64>,
    page: i64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GetRequestDetailInput {
    request_id: String,
}

pub(super) fn tools() -> Vec<ToolEntry> {
    vec![
        entry(
            "tanca_make_shift_change_request",
            "Submit a time-change request (đề xuất chỉnh giờ vào/ra) for an employee shift. Goes through the approval workflow — status=0 pending until approved. Use tanca_get_shift_summary to find employee_shift_id.",
            &[
                required_string("user_id", "Employee's user ID"),
                required_string(
                    "employee_shift_id",
                    "The employee shift ID for the working day to adjust (from tanca_get_shift_summary)",
                ),
                required_string("working_day", "Working day in YYYY-MM-DD format"),
                required_string(
                    "checkin_time",
                    "Proposed check-in datetime (YYYY-MM-DD HH:mm:ss)",
                ),
                required_string(
                    "checkout_time",
                    "Proposed check-out datetime (YYYY-MM-DD HH:mm:ss)",
                ),
                required_string(
                    "reason",
                    "Reason for the time change (e.g. 'Quên chấm công')",
                ),
            ],
            |a| Box::pin(make_shift_change_request(a)),
        ),
        entry(
            "tanca_list_requests",
            "List submitted requests (yêu cầu) within a date range. Filter by type ('employee-shift', 'onleave', 'overtime', etc.) and status (0=pending, 1=approved, 2=rejected).",
            &[
                required_string("from_date", "Start date (YYYY-MM-DD)"),
                required_string("to_date", "End date (YYYY-MM-DD)"),
                string(
                    "type",
                    "Request type filter: 'employee-shift', 'onleave', 'overtime', 'business-trip', etc. Empty for all.",
                ),
                number(
                    "status",
                    "Status filter: 0=pending, 1=approved, 2=rejected. Omit for all.",
                ),
                number("page", "Page number (default 1)"),
            ],
            |a| Box::pin(list_requests(a)),
        ),
        entry(
            "tanca_get_request_detail",
            "Get full details of a single request by its ID.",
            &[required_string("request_id", "The request ID")],
            |a| Box::pin(get_request_detail(a)),
        ),
    ]
}

async fn make_shift_change_request(args: Value) -> Result<String, ToolError> {
    let input: MakeShiftChangeRequestInput = bind(args)?;
    let req = tanca_client()
        .make_shift_change_request(MakeShiftChangeRequestParams {
            user_id: input.user_id,
            employee_shift_id: input.employee_shift_id,
            working_day: input.working_day,
            checkin_time: input.checkin_time,
            checkout_time: input.checkout_time,
            reason: input.reason,
        })
        .await
        .map_err(|e| failed("failed to submit shift change request", e))?;
    Ok(format::shift_change_request(&req, Flavor::Mcp))
}

async fn list_requests(args: Value) -> Result<String, ToolError> {
    let input: ListRequestsInput = bind(args)?;
    let data = tanca_client()
        .list_requests(ListRequestsParams {
            from_date: input.from_date.clone(),
            to_date: input.to_date.clone(),
            kind: input.kind,
            status: input.status,
            page: input.page,
        })
        .await
        .map_err(|e| failed("failed to list requests", e))?;
    Ok(format::requests(
        &input.from_date,
        &input.to_date,
        &data,
        Flavor::Mcp,
    ))
}

async fn get_request_detail(args: Value) -> Result<String, ToolError> {
    let input: GetRequestDetailInput = bind(args)?;
    let r = tanca_client()
        .get_request_detail(&input.request_id)
        .await
        .map_err(|e| failed("failed to get request detail", e))?;
    Ok(format::request_detail(&r, Flavor::Mcp))
}
