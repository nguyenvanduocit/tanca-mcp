//! Tanca API payload types.
//!
//! Every struct uses `#[serde(default)]` and responses are passed through
//! [`crate::json::strip_nulls`] before decoding, so a missing field or a
//! `null` value decodes to the type's zero value. `Option` fields stay `None`
//! in both cases.

use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::json::{go_float, is_zero_f64, is_zero_i64};

/// Wraps all Tanca API responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Response<T> {
    pub error_code: i64,
    pub message: Vec<String>,
    pub from_cache: Option<i64>,
    pub data: T,
}

/// Wraps paginated list responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PaginatedData<T> {
    pub meta: Meta,
    pub items: Vec<T>,
    pub from_cache: i64,
}

/// Pagination info.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Meta {
    pub total: i64,
    pub count: i64,
    pub per_page: i64,
    pub current_page: i64,
    pub total_pages: i64,
}

/// Employee from `/employee/list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Employee {
    pub name: String,
    pub user_id: String,
    /// Phone number.
    pub username: String,
    pub region_id: String,
    pub branch_id: String,
    pub department_id: String,
    pub position_id: String,
    pub identification: String,
    pub is_no_need_timekeeping: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub working_date: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub labour_end_date: String,
}

/// Shift assignment from `/shift/list-shift-assignment-with-shift-v2`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ShiftAssignment {
    pub id: String,
    pub title: String,
    pub sort_index: i64,
    pub shift: Shift,
}

/// Shift details embedded in [`ShiftAssignment`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Shift {
    pub id: String,
    pub name: String,
    pub shift_key: String,
    pub symbol: Option<String>,
    pub color: Option<String>,
    pub sort_index: i64,
    pub working_hour: i64,
    pub start_hour_obj: TimeOption,
    pub start_minute_obj: TimeOption,
    pub end_hour_obj: TimeOption,
    pub end_minute_obj: TimeOption,
    pub is_overtime_shift: i64,
    pub timezone: String,
}

/// Hour/minute option.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TimeOption {
    pub id: String,
    pub name: String,
    pub value: String,
    #[serde(rename = "type")]
    pub kind: String,
}

/// Summary from `/shift/summary-employee-shift`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EmployeeShiftSummary {
    pub user_id: String,
    pub name: String,
    pub phone: String,
    pub identification: String,
    pub branch_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_obj: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_obj: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_obj: Option<OrgRef>,
    /// Date (`YYYY-MM-DD 00:00:00`) to shifts.
    pub shifts: BTreeMap<String, Vec<EmployeeShift>>,
    pub total_working_hour: i64,
    pub real_working_hour: i64,
}

/// Reference to an org unit (branch, department, position, ...).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OrgRef {
    pub id: String,
    pub name: String,
}

/// A single shift assignment for an employee.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EmployeeShift {
    /// Shift assignment ID, used for check-in/out.
    pub id: String,
    pub name: String,
    pub shift_key: String,
    pub shift_id: String,
    /// `hard` or `soft`.
    pub shift_type: String,
    pub start_time: String,
    pub end_time: String,
    pub working_hour: i64,
    pub working_day: String,
    pub user_id: String,
    pub branch_id: String,
    pub checkin_time: Option<String>,
    pub checkout_time: Option<String>,
    pub checkin_type: String,
    pub checkout_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ShiftStatus>,
}

/// Current attendance status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ShiftStatus {
    pub color: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub status_color: Vec<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub detail: Vec<String>,
}

/// Body for `POST /shift/check-in-out-shift`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CheckInOutRequest {
    /// Shift assignment ID.
    pub id: String,
    pub user_id: String,
    pub branch_id: String,
    /// `YYYY-MM-DD HH:mm:ss`.
    pub working_day: String,
    /// `YYYY-MM-DD HH:mm:ss`.
    pub checkin_time: String,
    /// `YYYY-MM-DD HH:mm:ss`.
    pub checkout_time: String,
    /// 1 to record, 0 to skip.
    pub is_checkin: i64,
    /// 1 to record, 0 to skip.
    pub is_checkout: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reason: String,
}

/// Response from `POST /shift/check-in-out-shift`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CheckInOutResponse {
    pub id: String,
    pub name: String,
    pub shift_key: String,
    pub shift_id: String,
    pub shift_type: String,
    pub start_time: String,
    pub end_time: String,
    pub working_hour: i64,
    pub working_day: String,
    pub week_of_year: i64,
    pub branch_id: String,
    pub user_id: String,
    pub checkin_time: String,
    pub checkout_time: String,
    pub is_confirm: i64,
    pub is_overtime_shift: i64,
    pub shop_id: String,
    pub checkin_type: String,
    pub checkout_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub checkin_log_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub checkout_log_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ShiftStatus>,
}

/// Clock log from `/shift/get-checked-time-employee-shift`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockLog {
    pub time: String,
    pub timekeeper_log_id: String,
    pub is_trashed: i64,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_user: Option<ClockUser>,
    /// mobile, admin, gps, wifi, beacon.
    pub clock_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<GeoLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<ClockOption>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reason: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
}

/// User who created a clock entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockUser {
    pub name: String,
    pub username: String,
    pub user_id: String,
}

/// Location of a GPS-based clock entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GeoLocation {
    pub latitude: String,
    pub longitude: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub accuracy: String,
}

/// Check-in method.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClockOption {
    /// gps, wifi, beacon.
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub radius: i64,
    #[serde(skip_serializing_if = "is_zero_f64", serialize_with = "go_float")]
    pub lat: f64,
    #[serde(skip_serializing_if = "is_zero_f64", serialize_with = "go_float")]
    pub long: f64,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub type_name: String,
}

/// Company/shop information from `/shop/detail`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Shop {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub username: String,
    pub address: String,
    pub address_lat: Option<String>,
    pub address_lng: Option<String>,
    pub tel: String,
    pub phone: String,
    pub email: String,
    pub website: String,
    pub description: String,
    pub logo: String,
    pub brand_logo: String,
    pub brand_color: String,
    pub code: String,
    pub tax_code: String,
    pub bank: String,
    pub bank_account: String,
    pub founded_date: String,
    pub scale: String,
    pub created_at: String,
    #[serde(rename = "timeFormat")]
    pub time_format: String,
    #[serde(rename = "dateFormat")]
    pub date_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub district: Option<OrgRef>,
    #[serde(rename = "packages", skip_serializing_if = "Option::is_none")]
    pub packages_info: Option<PackagesInfo>,
}

/// Subscription package information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PackagesInfo {
    pub info: PackageBasicInfo,
    pub packages: BTreeMap<String, PackageItem>,
}

/// Basic package info.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageBasicInfo {
    pub start_package: String,
    pub end_package: String,
    pub users: i64,
    pub active_users: i64,
    pub is_paid: i64,
}

/// A single package subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageItem {
    pub id: String,
    pub shop_id: String,
    pub code: String,
    pub name: String,
    pub roles: Vec<String>,
    pub users: i64,
    pub start_package: String,
    pub end_package: String,
    pub is_available: i64,
}

/// Parameters for `POST /request/make-request` (`type=employee-shift`).
#[derive(Debug, Clone, Default)]
pub struct MakeShiftChangeRequestParams {
    pub user_id: String,
    pub employee_shift_id: String,
    /// `YYYY-MM-DD`.
    pub working_day: String,
    /// `YYYY-MM-DD HH:mm:ss`.
    pub checkin_time: String,
    /// `YYYY-MM-DD HH:mm:ss`.
    pub checkout_time: String,
    pub reason: String,
}

/// Response payload for an employee-shift time-change request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ShiftChangeRequest {
    pub id: String,
    pub user_id: String,
    pub employee_shift_id: String,
    /// `HH:mm`.
    pub checkin_time: String,
    /// `YYYY-MM-DD HH:mm:ss`.
    pub checkin_datetime: String,
    pub checkout_time: String,
    pub checkout_datetime: String,
    /// 0 = pending, 1 = approved, 2 = rejected.
    pub status: i64,
    pub reason: String,
    /// `employee-shift`.
    #[serde(rename = "type")]
    pub kind: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub updated_at: String,
    pub serial: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub identification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_obj: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_obj: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_obj: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_obj: Option<RequestUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_user_obj: Option<RequestUser>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub approver_obj: Vec<OrgRef>,
}

/// Compact user reference embedded in request responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RequestUser {
    pub id: String,
    pub name: String,
    pub username: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub identification: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub branch_id: String,
}

/// Filters for `/request/list-request`.
#[derive(Debug, Clone, Default)]
pub struct ListRequestsParams {
    /// `YYYY-MM-DD`.
    pub from_date: String,
    /// `YYYY-MM-DD`.
    pub to_date: String,
    /// Empty for all, e.g. `employee-shift`, `onleave`.
    pub kind: String,
    /// 0 = pending, 1 = approved, 2 = rejected; `None` = all.
    pub status: Option<i64>,
    /// Defaults to 1.
    pub page: i64,
}

/// A row in `/request/list-request`; fields vary by type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RequestSummary {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub status: i64,
    pub serial: String,
    pub reason: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub identification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_obj: Option<RequestUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_obj: Option<OrgRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_obj: Option<OrgRef>,

    // employee-shift specific
    #[serde(skip_serializing_if = "String::is_empty")]
    pub employee_shift_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub checkin_datetime: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub checkout_datetime: String,

    // onleave specific
    #[serde(skip_serializing_if = "String::is_empty")]
    pub from_date: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub to_date: String,
}

/// Datetime format used by the Tanca API (`YYYY-MM-DD HH:mm:ss`).
pub const TANCA_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
/// Date format used by the Tanca API (`YYYY-MM-DD`).
pub const TANCA_DATE_FORMAT: &str = "%Y-%m-%d";

/// Formats a datetime for the Tanca API.
pub fn format_time(t: &NaiveDateTime) -> String {
    t.format(TANCA_TIME_FORMAT).to_string()
}

/// Formats a date for the Tanca API.
pub fn format_date(t: &NaiveDateTime) -> String {
    t.format(TANCA_DATE_FORMAT).to_string()
}

/// Formats a date with zero time for the `working_day` field.
pub fn format_working_day(t: &NaiveDateTime) -> String {
    format!("{} 00:00:00", format_date(t))
}

/// Parses a `YYYY-MM-DD HH:mm:ss` wall-clock time (interpreted as Asia/Saigon
/// time by the API).
pub fn parse_time(s: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(s, TANCA_TIME_FORMAT)
        .map_err(|e| format!("parsing time {s:?} as \"2006-01-02 15:04:05\": {e}"))
}

/// Current local wall-clock time.
pub fn now() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}
