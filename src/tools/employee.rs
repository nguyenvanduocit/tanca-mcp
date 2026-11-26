use serde::Deserialize;
use serde_json::Value;

use super::{ToolEntry, ToolError, bind, entry, failed, number, string};
use crate::format;
use crate::services::{GetEmployeeListParams, tanca_client};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ListEmployeesInput {
    page: i64,
    limit: i64,
    department_id: String,
    start_date: String,
    end_date: String,
}

pub(super) fn tools() -> Vec<ToolEntry> {
    vec![
        entry(
            "tanca_list_employees",
            "List all employees from Tanca. Returns employee name, ID, phone, department, position, and timekeeping status.",
            &[
                number("page", "Page number for pagination (default: 1)"),
                number(
                    "limit",
                    "Number of employees per page (default: 30, max: 100)",
                ),
                string(
                    "department_id",
                    "Filter by department ID (MongoDB ObjectId)",
                ),
                string("start_date", "Filter start date (YYYY-MM-DD format)"),
                string("end_date", "Filter end date (YYYY-MM-DD format)"),
            ],
            |a| Box::pin(list_employees(a)),
        ),
        entry(
            "tanca_list_timekeeping_employees",
            "List all employees that require timekeeping (excludes exempt employees). Auto-paginates to get all employees.",
            &[],
            |a| Box::pin(list_timekeeping_employees(a)),
        ),
    ]
}

async fn list_employees(args: Value) -> Result<String, ToolError> {
    let input: ListEmployeesInput = bind(args)?;
    let mut params = GetEmployeeListParams {
        page: input.page,
        limit: input.limit,
        start_date: input.start_date,
        end_date: input.end_date,
        ..Default::default()
    };
    if !input.department_id.is_empty() {
        params.department_ids = vec![input.department_id];
    }
    let data = tanca_client()
        .get_employee_list(params)
        .await
        .map_err(|e| failed("failed to list employees", e))?;
    Ok(format::employees(&data))
}

async fn list_timekeeping_employees(_args: Value) -> Result<String, ToolError> {
    let employees = tanca_client()
        .get_timekeeping_employees()
        .await
        .map_err(|e| failed("failed to list timekeeping employees", e))?;
    Ok(format::timekeeping_employees(&employees))
}
