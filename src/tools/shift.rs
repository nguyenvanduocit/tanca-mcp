use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::Value;

use super::{ToolEntry, ToolError, bind, entry, failed, required_string, string};
use crate::format;
use crate::services::{GetEmployeeShiftSummaryParams, now, parse_time, tanca_client};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GetShiftSummaryInput {
    user_id: String,
    start_date: String,
    end_date: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GetTodayShiftInput {
    user_id: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GetShiftByDayInput {
    user_id: String,
    working_day: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CheckInInput {
    shift_assignment_id: String,
    user_id: String,
    branch_id: String,
    /// `YYYY-MM-DD HH:mm:ss`; defaults to now.
    checkin_time: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CheckOutInput {
    shift_assignment_id: String,
    user_id: String,
    branch_id: String,
    /// `YYYY-MM-DD HH:mm:ss`; defaults to now.
    checkout_time: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GetClockLogsInput {
    employee_shift_id: String,
}

pub(super) fn tools() -> Vec<ToolEntry> {
    vec![
        entry(
            "tanca_list_shifts",
            "List all available shift types/assignments configured in Tanca.",
            &[],
            |a| Box::pin(list_shifts(a)),
        ),
        entry(
            "tanca_get_shift_summary",
            "Get an employee's shift assignments for a date range. Returns shift details including check-in/out times and status.",
            &[
                required_string("user_id", "Employee's user ID"),
                required_string("start_date", "Start date (YYYY-MM-DD format)"),
                required_string("end_date", "End date (YYYY-MM-DD format)"),
            ],
            |a| Box::pin(get_shift_summary(a)),
        ),
        entry(
            "tanca_get_today_shift",
            "Get an employee's shift for today. Returns the shift assignment ID needed for check-in/out.",
            &[required_string("user_id", "Employee's user ID")],
            |a| Box::pin(get_today_shift(a)),
        ),
        entry(
            "tanca_get_shift_by_day",
            "Get an employee's shifts for a specific working day. Returns the employee_shift_id needed by tanca_make_shift_change_request.",
            &[
                required_string("user_id", "Employee's user ID"),
                required_string("working_day", "Target working day in YYYY-MM-DD format"),
            ],
            |a| Box::pin(get_shift_by_day(a)),
        ),
        entry(
            "tanca_check_in",
            "Record a check-in for an employee's shift. Requires the shift assignment ID from tanca_get_today_shift.",
            &[
                required_string(
                    "shift_assignment_id",
                    "The shift assignment ID from tanca_get_shift_summary",
                ),
                required_string("user_id", "Employee's user ID"),
                required_string("branch_id", "Branch ID where check-in occurs"),
                string(
                    "checkin_time",
                    "Check-in time (YYYY-MM-DD HH:mm:ss). Defaults to current time.",
                ),
            ],
            |a| Box::pin(check_in(a)),
        ),
        entry(
            "tanca_check_out",
            "Record a check-out for an employee's shift. Requires the shift assignment ID from tanca_get_today_shift.",
            &[
                required_string(
                    "shift_assignment_id",
                    "The shift assignment ID from tanca_get_shift_summary",
                ),
                required_string("user_id", "Employee's user ID"),
                required_string("branch_id", "Branch ID where check-out occurs"),
                string(
                    "checkout_time",
                    "Check-out time (YYYY-MM-DD HH:mm:ss). Defaults to current time.",
                ),
            ],
            |a| Box::pin(check_out(a)),
        ),
        entry(
            "tanca_get_clock_logs",
            "Get all clock logs (check-in/out entries) for a specific shift assignment.",
            &[required_string(
                "employee_shift_id",
                "The employee shift ID to get logs for",
            )],
            |a| Box::pin(get_clock_logs(a)),
        ),
    ]
}

async fn list_shifts(_args: Value) -> Result<String, ToolError> {
    let shifts = tanca_client()
        .get_shift_assignments()
        .await
        .map_err(|e| failed("failed to list shifts", e))?;
    Ok(format::shifts(&shifts))
}

async fn get_shift_summary(args: Value) -> Result<String, ToolError> {
    let input: GetShiftSummaryInput = bind(args)?;
    let summary = tanca_client()
        .get_employee_shift_summary(GetEmployeeShiftSummaryParams {
            user_id: input.user_id,
            start_date: input.start_date,
            end_date: input.end_date,
        })
        .await
        .map_err(|e| failed("failed to get shift summary", e))?;
    Ok(format::shift_summary(&summary))
}

async fn get_today_shift(args: Value) -> Result<String, ToolError> {
    let input: GetTodayShiftInput = bind(args)?;
    let shift = tanca_client()
        .get_today_shift(&input.user_id)
        .await
        .map_err(|e| failed("failed to get today's shift", e))?;
    Ok(format::today_shift(&input.user_id, &shift))
}

async fn get_shift_by_day(args: Value) -> Result<String, ToolError> {
    let input: GetShiftByDayInput = bind(args)?;
    let shifts = tanca_client()
        .list_employee_shifts_for_day(&input.user_id, &input.working_day)
        .await
        .map_err(|e| failed("failed to get shift by day", e))?;
    Ok(format::shifts_by_day(
        &input.user_id,
        &input.working_day,
        &shifts,
    ))
}

/// Parses an optional `YYYY-MM-DD HH:mm:ss` argument, defaulting to now.
fn time_or_now(value: &str, field: &str) -> Result<NaiveDateTime, ToolError> {
    if value.is_empty() {
        return Ok(now());
    }
    parse_time(value).map_err(|e| {
        ToolError::Failed(format!(
            "invalid {field} format (use YYYY-MM-DD HH:mm:ss): {e}"
        ))
    })
}

async fn check_in(args: Value) -> Result<String, ToolError> {
    let input: CheckInInput = bind(args)?;
    let t = time_or_now(&input.checkin_time, "checkin_time")?;
    let resp = tanca_client()
        .check_in(
            &input.shift_assignment_id,
            &input.user_id,
            &input.branch_id,
            t,
        )
        .await
        .map_err(|e| failed("failed to check in", e))?;
    Ok(format::check_in(&resp))
}

async fn check_out(args: Value) -> Result<String, ToolError> {
    let input: CheckOutInput = bind(args)?;
    let t = time_or_now(&input.checkout_time, "checkout_time")?;
    let resp = tanca_client()
        .check_out(
            &input.shift_assignment_id,
            &input.user_id,
            &input.branch_id,
            t,
        )
        .await
        .map_err(|e| failed("failed to check out", e))?;
    Ok(format::check_out(&resp))
}

async fn get_clock_logs(args: Value) -> Result<String, ToolError> {
    let input: GetClockLogsInput = bind(args)?;
    let logs = tanca_client()
        .get_checked_time(&input.employee_shift_id)
        .await
        .map_err(|e| failed("failed to get clock logs", e))?;
    Ok(format::clock_logs(&input.employee_shift_id, &logs))
}
