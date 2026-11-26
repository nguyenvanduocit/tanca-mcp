use anyhow::{Context, Result, anyhow};
use chrono::NaiveDateTime;

use super::client::{Client, Query, decode};
use super::types::{
    CheckInOutRequest, CheckInOutResponse, ClockLog, EmployeeShift, EmployeeShiftSummary,
    PaginatedData, ShiftAssignment, format_date, format_time, format_working_day, now,
};

/// Parameters for the shift summary query.
#[derive(Debug, Clone, Default)]
pub struct GetEmployeeShiftSummaryParams {
    pub user_id: String,
    /// `YYYY-MM-DD`.
    pub start_date: String,
    /// `YYYY-MM-DD`.
    pub end_date: String,
}

const CHECK_REASON: &str = "MCP Admin";

impl Client {
    /// Retrieves all shift assignment types.
    pub async fn get_shift_assignments(&self) -> Result<Vec<ShiftAssignment>> {
        let body = self
            .get("/shift/list-shift-assignment-with-shift-v2", None)
            .await
            .context("get shift assignments")?;
        let data: PaginatedData<ShiftAssignment> = decode(&body)?;
        Ok(data.items)
    }

    /// Retrieves an employee's shift assignments for a date range.
    pub async fn get_employee_shift_summary(
        &self,
        params: GetEmployeeShiftSummaryParams,
    ) -> Result<EmployeeShiftSummary> {
        let shop_id = self.get_shop_id().await?;
        let query = Query::from([
            ("shop_id".into(), shop_id),
            ("user_id".into(), params.user_id.clone()),
            ("start_date".into(), params.start_date),
            ("end_date".into(), params.end_date),
            ("is_shift".into(), "1".into()),
            ("is_shift_only".into(), "1".into()),
            ("is_status_shift".into(), "1".into()),
        ]);
        let body = self
            .get("/shift/summary-employee-shift", Some(&query))
            .await
            .context("get employee shift summary")?;
        let data: PaginatedData<EmployeeShiftSummary> = decode(&body)?;
        data.items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no shift data found for user {}", params.user_id))
    }

    /// Returns the employee's first shift today.
    pub async fn get_today_shift(&self, user_id: &str) -> Result<EmployeeShift> {
        let today = format_date(&now());
        let mut summary = self
            .get_employee_shift_summary(GetEmployeeShiftSummaryParams {
                user_id: user_id.into(),
                start_date: today.clone(),
                end_date: today.clone(),
            })
            .await?;
        // Keys look like "YYYY-MM-DD 00:00:00".
        let key = format!("{today} 00:00:00");
        summary
            .shifts
            .remove(&key)
            .and_then(|shifts| shifts.into_iter().next())
            .ok_or_else(|| anyhow!("no shift assigned for {user_id} on {today}"))
    }

    /// Returns the shifts assigned to a user on a working day. Use this to
    /// find the `employee_shift_id` for [`Client::make_shift_change_request`].
    pub async fn list_employee_shifts_for_day(
        &self,
        user_id: &str,
        working_day: &str,
    ) -> Result<Vec<EmployeeShift>> {
        let query = Query::from([
            ("is_unique_shift".into(), "1".into()),
            ("is_available".into(), "1".into()),
            ("user_id".into(), user_id.into()),
            ("working_day".into(), working_day.into()),
        ]);
        let body = self
            .get("/shift/list-employee-shift", Some(&query))
            .await
            .context("list employee shifts for day")?;
        decode(&body)
    }

    /// Records a check-in for an employee's shift.
    pub async fn check_in(
        &self,
        shift_assignment_id: &str,
        user_id: &str,
        branch_id: &str,
        checkin_time: NaiveDateTime,
    ) -> Result<CheckInOutResponse> {
        self.check_in_out_request(CheckInOutRequest {
            id: shift_assignment_id.into(),
            user_id: user_id.into(),
            branch_id: branch_id.into(),
            working_day: format_working_day(&checkin_time),
            checkin_time: format_time(&checkin_time),
            // Required but ignored when is_checkout = 0.
            checkout_time: format_time(&checkin_time),
            is_checkin: 1,
            is_checkout: 0,
            reason: CHECK_REASON.into(),
        })
        .await
    }

    /// Records a check-out for an employee's shift.
    pub async fn check_out(
        &self,
        shift_assignment_id: &str,
        user_id: &str,
        branch_id: &str,
        checkout_time: NaiveDateTime,
    ) -> Result<CheckInOutResponse> {
        self.check_in_out_request(CheckInOutRequest {
            id: shift_assignment_id.into(),
            user_id: user_id.into(),
            branch_id: branch_id.into(),
            working_day: format_working_day(&checkout_time),
            // Required but ignored when is_checkin = 0.
            checkin_time: format_time(&checkout_time),
            checkout_time: format_time(&checkout_time),
            is_checkin: 0,
            is_checkout: 1,
            reason: CHECK_REASON.into(),
        })
        .await
    }

    /// Records both check-in and check-out in one call.
    pub async fn check_in_out(
        &self,
        shift_assignment_id: &str,
        user_id: &str,
        branch_id: &str,
        checkin_time: NaiveDateTime,
        checkout_time: NaiveDateTime,
    ) -> Result<CheckInOutResponse> {
        self.check_in_out_request(CheckInOutRequest {
            id: shift_assignment_id.into(),
            user_id: user_id.into(),
            branch_id: branch_id.into(),
            working_day: format_working_day(&checkin_time),
            checkin_time: format_time(&checkin_time),
            checkout_time: format_time(&checkout_time),
            is_checkin: 1,
            is_checkout: 1,
            reason: CHECK_REASON.into(),
        })
        .await
    }

    async fn check_in_out_request(&self, req: CheckInOutRequest) -> Result<CheckInOutResponse> {
        let body = self
            .post("/shift/check-in-out-shift", &req)
            .await
            .context("check-in/out")?;
        decode(&body)
    }

    /// Retrieves clock logs for a shift assignment.
    pub async fn get_checked_time(&self, employee_shift_id: &str) -> Result<Vec<ClockLog>> {
        let query = Query::from([("employee_shift_id".into(), employee_shift_id.into())]);
        let body = self
            .get("/shift/get-checked-time-employee-shift", Some(&query))
            .await
            .context("get checked time")?;
        decode(&body)
    }

    /// Whether the shift has any clock entry that is not deleted.
    pub async fn has_checked_in(&self, employee_shift_id: &str) -> Result<bool> {
        let logs = self.get_checked_time(employee_shift_id).await?;
        Ok(logs.iter().any(|l| l.is_trashed == 0))
    }
}
