//! Plain-text renderings shared by the CLI and the MCP tools.

use std::fmt::Write;

use crate::services::{
    CheckInOutResponse, ClockLog, Employee, EmployeeShift, EmployeeShiftSummary, PaginatedData,
    RequestSummary, ShiftAssignment, ShiftChangeRequest, Shop,
};

/// Output flavour. The CLI and the MCP tools label requests slightly
/// differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flavor {
    Cli,
    Mcp,
}

// Writing to a String cannot fail.
macro_rules! w {
    ($out:expr, $($arg:tt)*) => {{ let _ = writeln!($out, $($arg)*); }};
}

pub fn employees(data: &PaginatedData<Employee>) -> String {
    let mut s = String::new();
    w!(
        s,
        "Employees (Page {}/{}, Total: {})\n",
        data.meta.current_page,
        data.meta.total_pages,
        data.meta.total
    );
    for emp in &data.items {
        let timekeeping = if emp.is_no_need_timekeeping == 1 {
            "No"
        } else {
            "Yes"
        };
        w!(s, "- {} ({})", emp.name, emp.identification);
        w!(s, "  User ID: {}", emp.user_id);
        w!(s, "  Phone: {}", emp.username);
        w!(s, "  Branch ID: {}", emp.branch_id);
        w!(s, "  Department ID: {}", emp.department_id);
        w!(s, "  Position ID: {}", emp.position_id);
        w!(s, "  Needs Timekeeping: {timekeeping}");
        if !emp.working_date.is_empty() {
            w!(s, "  Working Date: {}", emp.working_date);
        }
        w!(s, "");
    }
    s
}

pub fn timekeeping_employees(employees: &[Employee]) -> String {
    let mut s = String::new();
    w!(
        s,
        "Employees Requiring Timekeeping (Total: {})\n",
        employees.len()
    );
    for emp in employees {
        w!(s, "- {} ({})", emp.name, emp.identification);
        w!(s, "  User ID: {}", emp.user_id);
        w!(s, "  Phone: {}", emp.username);
        w!(s, "  Branch ID: {}", emp.branch_id);
        w!(s, "  Department ID: {}", emp.department_id);
        w!(s, "");
    }
    s
}

pub fn shifts(shifts: &[ShiftAssignment]) -> String {
    let mut s = String::new();
    w!(s, "Available Shifts (Total: {})\n", shifts.len());
    for sa in shifts {
        let sh = &sa.shift;
        w!(s, "- {} (ID: {})", sa.title, sa.id);
        w!(s, "  Shift: {} ({})", sh.name, sh.shift_key);
        w!(
            s,
            "  Time: {}:{} - {}:{}",
            sh.start_hour_obj.value,
            sh.start_minute_obj.value,
            sh.end_hour_obj.value,
            sh.end_minute_obj.value
        );
        w!(s, "  Working Hours: {}", sh.working_hour);
        if sh.is_overtime_shift == 1 {
            w!(s, "  Type: Overtime Shift");
        }
        w!(s, "");
    }
    s
}

pub fn shift_summary(summary: &EmployeeShiftSummary) -> String {
    let mut s = String::new();
    w!(
        s,
        "Shift Summary for {} ({})",
        summary.name,
        summary.identification
    );
    w!(s, "Phone: {}", summary.phone);
    w!(s, "Total Working Hours: {}", summary.total_working_hour);
    w!(s, "Real Working Hours: {}", summary.real_working_hour);
    w!(s, "\nShifts:");
    for (date, shifts) in &summary.shifts {
        w!(s, "\n{date}:");
        for shift in shifts {
            w!(s, "  - {} (ID: {})", shift.name, shift.id);
            w!(s, "    Time: {} - {}", shift.start_time, shift.end_time);
            w!(s, "    Working Hours: {}", shift.working_hour);
            match &shift.checkin_time {
                Some(t) => w!(s, "    Check-in: {t} ({})", shift.checkin_type),
                None => w!(s, "    Check-in: Not recorded"),
            }
            match &shift.checkout_time {
                Some(t) => w!(s, "    Check-out: {t} ({})", shift.checkout_type),
                None => w!(s, "    Check-out: Not recorded"),
            }
            if let Some(st) = &shift.status {
                w!(s, "    Status: {}", st.name);
            }
        }
    }
    s
}

pub fn today_shift(user_id: &str, shift: &EmployeeShift) -> String {
    let mut s = String::new();
    w!(s, "Today's Shift for User {user_id}\n");
    w!(s, "Shift Assignment ID: {}", shift.id);
    w!(s, "Shift Name: {}", shift.name);
    w!(s, "Shift Key: {}", shift.shift_key);
    w!(s, "Time: {} - {}", shift.start_time, shift.end_time);
    w!(s, "Working Day: {}", shift.working_day);
    w!(s, "Working Hours: {}", shift.working_hour);
    w!(s, "Branch ID: {}", shift.branch_id);
    match &shift.checkin_time {
        Some(t) => w!(s, "\nCheck-in: {t} ({})", shift.checkin_type),
        None => w!(s, "\nCheck-in: Not recorded"),
    }
    match &shift.checkout_time {
        Some(t) => w!(s, "Check-out: {t} ({})", shift.checkout_type),
        None => w!(s, "Check-out: Not recorded"),
    }
    if let Some(st) = &shift.status {
        w!(s, "\nStatus: {}", st.name);
    }
    s
}

pub fn shifts_by_day(user_id: &str, working_day: &str, shifts: &[EmployeeShift]) -> String {
    let mut s = String::new();
    w!(
        s,
        "Shifts for User {user_id} on {working_day} (Total: {})\n",
        shifts.len()
    );
    if shifts.is_empty() {
        w!(s, "No shift assigned for this day.");
    }
    for shift in shifts {
        w!(s, "- {} ({})", shift.name, shift.shift_key);
        w!(s, "  Employee Shift ID: {}", shift.id);
        w!(s, "  Time: {} → {}", shift.start_time, shift.end_time);
        w!(s, "  Working Hours: {}", shift.working_hour);
        w!(s, "  Branch ID: {}", shift.branch_id);
        match &shift.checkin_time {
            Some(t) => w!(s, "  Check-in: {t}"),
            None => w!(s, "  Check-in: Not recorded"),
        }
        match &shift.checkout_time {
            Some(t) => w!(s, "  Check-out: {t}"),
            None => w!(s, "  Check-out: Not recorded"),
        }
        w!(s, "");
    }
    s
}

pub fn check_in(resp: &CheckInOutResponse) -> String {
    let mut s = String::new();
    w!(s, "Check-in recorded successfully!\n");
    w!(s, "Shift: {}", resp.name);
    w!(s, "Check-in Time: {}", resp.checkin_time);
    w!(s, "Check-in Type: {}", resp.checkin_type);
    if let Some(st) = &resp.status {
        w!(s, "Status: {}", st.name);
    }
    s
}

pub fn check_out(resp: &CheckInOutResponse) -> String {
    let mut s = String::new();
    w!(s, "Check-out recorded successfully!\n");
    w!(s, "Shift: {}", resp.name);
    w!(s, "Check-out Time: {}", resp.checkout_time);
    w!(s, "Check-out Type: {}", resp.checkout_type);
    if let Some(st) = &resp.status {
        w!(s, "Status: {}", st.name);
    }
    s
}

pub fn clock_logs(employee_shift_id: &str, logs: &[ClockLog]) -> String {
    let mut s = String::new();
    w!(
        s,
        "Clock Logs for Shift {employee_shift_id} (Total: {})\n",
        logs.len()
    );
    for (i, log) in logs.iter().enumerate() {
        let trashed = if log.is_trashed == 1 {
            " [DELETED]"
        } else {
            ""
        };
        w!(s, "{}. {}{trashed}", i + 1, log.time);
        w!(s, "   Type: {}", log.clock_type);
        w!(s, "   Created: {}", log.created_at);
        if let Some(u) = &log.created_user {
            w!(s, "   Created By: {}", u.name);
        }
        if !log.reason.is_empty() {
            w!(s, "   Reason: {}", log.reason);
        }
        if let Some(loc) = &log.location {
            w!(s, "   Location: {}, {}", loc.latitude, loc.longitude);
        }
        w!(s, "");
    }
    s
}

pub fn shop(shop: &Shop) -> String {
    let mut s = String::new();
    w!(s, "=== Company Information ===\n");
    w!(s, "Shop ID: {}", shop.id);
    w!(s, "Name: {}", shop.name);
    w!(s, "Username: {}", shop.username);
    for (label, value) in [
        ("Email", &shop.email),
        ("Phone", &shop.phone),
        ("Website", &shop.website),
        ("Address", &shop.address),
    ] {
        if !value.is_empty() {
            w!(s, "{label}: {value}");
        }
    }
    for (label, value) in [
        ("Country", &shop.country),
        ("Province", &shop.province),
        ("District", &shop.district),
    ] {
        if let Some(r) = value {
            w!(s, "{label}: {}", r.name);
        }
    }
    if !shop.tax_code.is_empty() {
        w!(s, "Tax Code: {}", shop.tax_code);
    }
    if !shop.founded_date.is_empty() {
        w!(s, "Founded: {}", shop.founded_date);
    }
    w!(s, "Created At: {}", shop.created_at);
    if let Some(p) = &shop.packages_info {
        w!(s, "\n=== Subscription ===");
        let info = &p.info;
        w!(s, "Period: {} to {}", info.start_package, info.end_package);
        w!(
            s,
            "Users: {} active / {} total",
            info.active_users,
            info.users
        );
        if !p.packages.is_empty() {
            w!(s, "\nActive Packages:");
            for pkg in p.packages.values() {
                if pkg.is_available == 1 {
                    w!(s, "  - {} ({})", pkg.name, pkg.code);
                }
            }
        }
    }
    s
}

pub fn shift_change_request(req: &ShiftChangeRequest, flavor: Flavor) -> String {
    let mut s = String::new();
    w!(s, "Time change request submitted successfully!\n");
    w!(s, "Request ID: {}", req.id);
    w!(s, "Serial: {}", req.serial);
    w!(s, "Status: {}", request_status_name(req.status, flavor));
    if flavor == Flavor::Mcp {
        let dt = &req.checkin_datetime;
        w!(s, "Working Day: {}", dt.get(..10).unwrap_or(dt));
    }
    w!(s, "Check-in: {}", req.checkin_datetime);
    w!(s, "Check-out: {}", req.checkout_datetime);
    w!(s, "Reason: {}", req.reason);
    if let Some(u) = &req.user_obj {
        w!(s, "Employee: {} ({})", u.name, u.username);
    }
    if !req.approver_obj.is_empty() {
        let names: Vec<&str> = req.approver_obj.iter().map(|a| a.name.as_str()).collect();
        w!(s, "Approver(s): {}", names.join(", "));
    }
    s
}

pub fn requests(
    from_date: &str,
    to_date: &str,
    data: &PaginatedData<RequestSummary>,
    flavor: Flavor,
) -> String {
    let mut s = String::new();
    w!(
        s,
        "Requests {from_date} → {to_date} (Page {}/{}, Total: {})\n",
        data.meta.current_page,
        data.meta.total_pages,
        data.meta.total
    );
    for r in &data.items {
        w!(
            s,
            "- [{}] {} — {}",
            r.serial,
            request_type_label(&r.kind, flavor),
            request_status_name(r.status, flavor)
        );
        w!(s, "  ID: {}", r.id);
        if let Some(u) = &r.user_obj {
            w!(s, "  Employee: {}", u.name);
        }
        if !r.reason.is_empty() {
            w!(s, "  Reason: {}", r.reason);
        }
        match r.kind.as_str() {
            "employee-shift" if !r.checkin_datetime.is_empty() => {
                w!(
                    s,
                    "  Proposed: {} → {}",
                    r.checkin_datetime,
                    r.checkout_datetime
                );
            }
            "onleave" if !r.from_date.is_empty() => {
                w!(s, "  Range: {} → {}", r.from_date, r.to_date);
            }
            _ => {}
        }
        w!(s, "  Created: {}\n", r.created_at);
    }
    s
}

pub fn request_detail(r: &RequestSummary, flavor: Flavor) -> String {
    let mut s = String::new();
    w!(
        s,
        "Request {} — {}\n",
        r.serial,
        request_type_label(&r.kind, flavor)
    );
    w!(s, "ID: {}", r.id);
    w!(s, "Status: {}", request_status_name(r.status, flavor));
    w!(s, "Created: {}", r.created_at);
    if let Some(u) = &r.user_obj {
        w!(s, "Employee: {} ({})", u.name, u.username);
    }
    if let Some(b) = &r.branch_obj {
        w!(s, "Branch: {}", b.name);
    }
    if let Some(d) = &r.department_obj {
        w!(s, "Department: {}", d.name);
    }
    if !r.reason.is_empty() {
        w!(s, "Reason: {}", r.reason);
    }
    match r.kind.as_str() {
        "employee-shift" if !r.checkin_datetime.is_empty() => {
            w!(s, "Proposed Check-in: {}", r.checkin_datetime);
            w!(s, "Proposed Check-out: {}", r.checkout_datetime);
        }
        "onleave" if !r.from_date.is_empty() => {
            w!(s, "From: {}", r.from_date);
            w!(s, "To: {}", r.to_date);
        }
        _ => {}
    }
    s
}

pub fn request_status_name(status: i64, flavor: Flavor) -> String {
    let (en, vi) = match status {
        0 => ("Pending", "chờ duyệt"),
        1 => ("Approved", "đã duyệt"),
        2 => ("Rejected", "từ chối"),
        _ => return format!("Unknown ({status})"),
    };
    match flavor {
        Flavor::Cli => en.to_string(),
        Flavor::Mcp => format!("{en} ({vi})"),
    }
}

pub fn request_type_label(kind: &str, flavor: Flavor) -> String {
    match kind {
        "employee-shift" => "Thay đổi giờ vào/ra",
        "onleave" => "Nghỉ phép",
        "overtime" => "Làm thêm giờ",
        "business-trip" => "Công tác/Ra ngoài",
        "register-shift" => "Đăng ký ca làm",
        "company-advance" | "advance-reimbursement" => "Tạm ứng - Hoàn ứng",
        "reimbursement" if flavor == Flavor::Cli => "Tạm ứng - Hoàn ứng",
        "payment" => "Thanh toán",
        other => other,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_type_labels_differ_by_flavor() {
        assert_eq!(request_status_name(0, Flavor::Cli), "Pending");
        assert_eq!(request_status_name(0, Flavor::Mcp), "Pending (chờ duyệt)");
        assert_eq!(request_status_name(7, Flavor::Mcp), "Unknown (7)");
        assert_eq!(
            request_type_label("reimbursement", Flavor::Cli),
            "Tạm ứng - Hoàn ứng"
        );
        assert_eq!(
            request_type_label("reimbursement", Flavor::Mcp),
            "reimbursement"
        );
    }

    #[test]
    fn empty_day_message() {
        assert_eq!(
            shifts_by_day("u", "2026-01-02", &[]),
            "Shifts for User u on 2026-01-02 (Total: 0)\n\nNo shift assigned for this day.\n"
        );
    }
}
