use anyhow::{Context, Result};

use super::client::{Client, Query, decode};
use super::types::{Employee, PaginatedData};

/// Filters for the employee list.
#[derive(Debug, Clone, Default)]
pub struct GetEmployeeListParams {
    pub page: i64,
    pub limit: i64,
    /// `YYYY-MM-DD`.
    pub start_date: String,
    /// `YYYY-MM-DD`.
    pub end_date: String,
    /// 0 = needs timekeeping, 1 = exempt.
    pub is_no_need_timekeeping: Option<i64>,
    /// 0 = active only, 1 = include inactive.
    pub is_inactive: Option<i64>,
    pub department_ids: Vec<String>,
    /// 1 = filter by task status.
    pub task_status_filter: Option<i64>,
    /// e.g. `timekeeping`.
    pub trans_type: String,
}

impl Client {
    /// Retrieves a page of employees.
    pub async fn get_employee_list(
        &self,
        mut params: GetEmployeeListParams,
    ) -> Result<PaginatedData<Employee>> {
        let shop_id = self.get_shop_id().await?;

        let mut query = Query::new();
        query.insert("shop_id".into(), shop_id);

        if params.page <= 0 {
            params.page = 1;
        }
        if params.limit <= 0 {
            params.limit = 30;
        }
        query.insert("page".into(), params.page.to_string());
        query.insert("limit".into(), params.limit.to_string());

        if !params.start_date.is_empty() {
            query.insert("start_date".into(), params.start_date);
        }
        if !params.end_date.is_empty() {
            query.insert("end_date".into(), params.end_date);
        }
        if let Some(v) = params.is_no_need_timekeeping {
            query.insert("is_no_need_timekeeping".into(), v.to_string());
        }
        if let Some(v) = params.is_inactive {
            query.insert("is_inactive".into(), v.to_string());
        }
        // PHP-style array notation.
        for (i, id) in params.department_ids.into_iter().enumerate() {
            query.insert(format!("department_ids[{i}]"), id);
        }
        if let Some(v) = params.task_status_filter {
            query.insert("task_status_filter".into(), v.to_string());
        }
        if !params.trans_type.is_empty() {
            query.insert("trans_type".into(), params.trans_type);
        }

        let body = self
            .get("/employee/list", Some(&query))
            .await
            .context("get employee list")?;
        decode(&body)
    }

    /// Fetches every page for `params`, starting at page 1.
    async fn collect_employees(&self, params: GetEmployeeListParams) -> Result<Vec<Employee>> {
        let mut all = Vec::new();
        let mut page = 1;
        loop {
            let data = self
                .get_employee_list(GetEmployeeListParams {
                    page,
                    ..params.clone()
                })
                .await?;
            let empty = data.items.is_empty();
            all.extend(data.items);
            if page >= data.meta.total_pages || empty {
                break;
            }
            page += 1;
        }
        Ok(all)
    }

    /// Retrieves all employees, following pagination.
    pub async fn get_all_employees(&self) -> Result<Vec<Employee>> {
        self.collect_employees(GetEmployeeListParams {
            limit: 100,
            ..Default::default()
        })
        .await
    }

    /// Retrieves only employees that need timekeeping, following pagination.
    pub async fn get_timekeeping_employees(&self) -> Result<Vec<Employee>> {
        self.collect_employees(GetEmployeeListParams {
            limit: 100,
            is_no_need_timekeeping: Some(0),
            ..Default::default()
        })
        .await
    }

    /// Retrieves active, timekept employees of one department.
    pub async fn get_employees_by_department(
        &self,
        department_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<Employee>> {
        self.collect_employees(GetEmployeeListParams {
            limit: 30,
            start_date: start_date.into(),
            end_date: end_date.into(),
            is_no_need_timekeeping: Some(0),
            is_inactive: Some(0),
            department_ids: vec![department_id.into()],
            task_status_filter: Some(1),
            trans_type: "timekeeping".into(),
            ..Default::default()
        })
        .await
    }
}
