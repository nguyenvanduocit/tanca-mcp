use anyhow::{Context, Result};

use super::client::{Client, Query, decode};
use super::types::{
    ListRequestsParams, MakeShiftChangeRequestParams, PaginatedData, RequestSummary,
    ShiftChangeRequest,
};

impl Client {
    /// Creates an employee-shift time-change request
    /// (`POST /request/make-request`, `type=employee-shift`). The request
    /// enters the approval flow with status 0 (pending).
    pub async fn make_shift_change_request(
        &self,
        params: MakeShiftChangeRequestParams,
    ) -> Result<ShiftChangeRequest> {
        let fields = [
            ("type", "employee-shift".to_string()),
            ("employee_shift_id", params.employee_shift_id),
            ("working_day", params.working_day),
            ("user_id", params.user_id),
            ("reason", params.reason),
            ("checkin_time", params.checkin_time),
            ("checkout_time", params.checkout_time),
        ];
        let body = self
            .post_form("/request/make-request", &fields)
            .await
            .context("make shift change request")?;
        decode(&body)
    }

    /// Retrieves submitted requests filtered by date range, type and status.
    pub async fn list_requests(
        &self,
        params: ListRequestsParams,
    ) -> Result<PaginatedData<RequestSummary>> {
        let page = if params.page <= 0 { 1 } else { params.page };
        let mut query = Query::new();
        query.insert("from_date".into(), params.from_date);
        query.insert("to_date".into(), params.to_date);
        query.insert("type".into(), params.kind);
        query.insert("page".into(), page.to_string());
        if let Some(s) = params.status {
            query.insert("status".into(), s.to_string());
        }
        let body = self
            .get("/request/list-request", Some(&query))
            .await
            .context("list requests")?;
        decode(&body)
    }

    /// Retrieves a single request by ID.
    pub async fn get_request_detail(&self, request_id: &str) -> Result<RequestSummary> {
        let query = Query::from([("id".to_string(), request_id.to_string())]);
        let body = self
            .get("/request/detail", Some(&query))
            .await
            .context("get request detail")?;
        decode(&body)
    }
}
