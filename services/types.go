package services

import "time"

// Response wraps all Tanca API responses
type Response[T any] struct {
	ErrorCode int      `json:"error_code"`
	Message   []string `json:"message"`
	FromCache *int     `json:"from_cache"`
	Data      T        `json:"data"`
}

// PaginatedData wraps paginated list responses
type PaginatedData[T any] struct {
	Meta      Meta `json:"meta"`
	Items     []T  `json:"items"`
	FromCache int  `json:"from_cache"`
}

// Meta contains pagination info
type Meta struct {
	Total       int `json:"total"`
	Count       int `json:"count"`
	PerPage     int `json:"per_page"`
	CurrentPage int `json:"current_page"`
	TotalPages  int `json:"total_pages"`
}

// Employee from /employee/list
type Employee struct {
	Name                string `json:"name"`
	UserID              string `json:"user_id"`
	Username            string `json:"username"` // phone number
	RegionID            string `json:"region_id"`
	BranchID            string `json:"branch_id"`
	DepartmentID        string `json:"department_id"`
	PositionID          string `json:"position_id"`
	Identification      string `json:"identification"`
	IsNoNeedTimekeeping int    `json:"is_no_need_timekeeping"`
	WorkingDate         string `json:"working_date,omitempty"`
	LabourEndDate       string `json:"labour_end_date,omitempty"`
}

// ShiftAssignment from /shift/list-shift-assignment-with-shift-v2
type ShiftAssignment struct {
	ID        string `json:"id"`
	Title     string `json:"title"`
	SortIndex int    `json:"sort_index"`
	Shift     Shift  `json:"shift"`
}

// Shift details embedded in ShiftAssignment
type Shift struct {
	ID              string     `json:"id"`
	Name            string     `json:"name"`
	ShiftKey        string     `json:"shift_key"`
	Symbol          *string    `json:"symbol"`
	Color           *string    `json:"color"`
	SortIndex       int        `json:"sort_index"`
	WorkingHour     int        `json:"working_hour"`
	StartHourObj    TimeOption `json:"start_hour_obj"`
	StartMinuteObj  TimeOption `json:"start_minute_obj"`
	EndHourObj      TimeOption `json:"end_hour_obj"`
	EndMinuteObj    TimeOption `json:"end_minute_obj"`
	IsOvertimeShift int        `json:"is_overtime_shift"`
	Timezone        string     `json:"timezone"`
}

// TimeOption for hour/minute selection
type TimeOption struct {
	ID    string `json:"id"`
	Name  string `json:"name"`
	Value string `json:"value"`
	Type  string `json:"type"`
}

// EmployeeShiftSummary from /shift/summary-employee-shift
type EmployeeShiftSummary struct {
	UserID           string                     `json:"user_id"`
	Name             string                     `json:"name"`
	Phone            string                     `json:"phone"`
	Identification   string                     `json:"identification"`
	BranchID         string                     `json:"branch_id"`
	BranchObj        *OrgRef                    `json:"branch_obj,omitempty"`
	DepartmentObj    *OrgRef                    `json:"department_obj,omitempty"`
	PositionObj      *OrgRef                    `json:"position_obj,omitempty"`
	Shifts           map[string][]EmployeeShift `json:"shifts"` // date -> shifts
	TotalWorkingHour int                        `json:"total_working_hour"`
	RealWorkingHour  int                        `json:"real_working_hour"`
}

// OrgRef is a reference to org unit (branch, department, position)
type OrgRef struct {
	ID   string `json:"id"`
	Name string `json:"name"`
}

// EmployeeShift is a single shift assignment for an employee
type EmployeeShift struct {
	ID             string       `json:"id"` // shift assignment ID - use for check-in/out
	Name           string       `json:"name"`
	ShiftKey       string       `json:"shift_key"`
	ShiftID        string       `json:"shift_id"`
	ShiftType      string       `json:"shift_type"` // hard or soft
	StartTime      string       `json:"start_time"`
	EndTime        string       `json:"end_time"`
	WorkingHour    int          `json:"working_hour"`
	WorkingDay     string       `json:"working_day"`
	UserID         string       `json:"user_id"`
	BranchID       string       `json:"branch_id"`
	CheckinTime    *string      `json:"checkin_time"`
	CheckoutTime   *string      `json:"checkout_time"`
	CheckinType    string       `json:"checkin_type"`
	CheckoutType   string       `json:"checkout_type"`
	Status         *ShiftStatus `json:"status,omitempty"`
}

// ShiftStatus describes current attendance status
type ShiftStatus struct {
	Color       string   `json:"color"`
	StatusColor []string `json:"status_color,omitempty"`
	Name        string   `json:"name"`
	Detail      []string `json:"detail,omitempty"`
}

// CheckInOutRequest for POST /shift/check-in-out-shift
type CheckInOutRequest struct {
	ID           string `json:"id"`            // shift assignment ID
	UserID       string `json:"user_id"`
	BranchID     string `json:"branch_id"`
	WorkingDay   string `json:"working_day"`   // YYYY-MM-DD HH:mm:ss
	CheckinTime  string `json:"checkin_time"`  // YYYY-MM-DD HH:mm:ss
	CheckoutTime string `json:"checkout_time"` // YYYY-MM-DD HH:mm:ss
	IsCheckin    int    `json:"is_checkin"`    // 1 to record, 0 to skip
	IsCheckout   int    `json:"is_checkout"`   // 1 to record, 0 to skip
	Reason       string `json:"reason,omitempty"`
}

// CheckInOutResponse from POST /shift/check-in-out-shift
type CheckInOutResponse struct {
	ID              string       `json:"id"`
	Name            string       `json:"name"`
	ShiftKey        string       `json:"shift_key"`
	ShiftID         string       `json:"shift_id"`
	ShiftType       string       `json:"shift_type"`
	StartTime       string       `json:"start_time"`
	EndTime         string       `json:"end_time"`
	WorkingHour     int          `json:"working_hour"`
	WorkingDay      string       `json:"working_day"`
	WeekOfYear      int          `json:"week_of_year"`
	BranchID        string       `json:"branch_id"`
	UserID          string       `json:"user_id"`
	CheckinTime     string       `json:"checkin_time"`
	CheckoutTime    string       `json:"checkout_time"`
	IsConfirm       int          `json:"is_confirm"`
	IsOvertimeShift int          `json:"is_overtime_shift"`
	ShopID          string       `json:"shop_id"`
	CheckinType     string       `json:"checkin_type"`
	CheckoutType    string       `json:"checkout_type"`
	CheckinLogID    string       `json:"checkin_log_id,omitempty"`
	CheckoutLogID   string       `json:"checkout_log_id,omitempty"`
	Status          *ShiftStatus `json:"status,omitempty"`
}

// ClockLog from /shift/get-checked-time-employee-shift
type ClockLog struct {
	Time            string       `json:"time"`
	TimekeeperLogID string       `json:"timekeeper_log_id"`
	IsTrashed       int          `json:"is_trashed"`
	CreatedAt       string       `json:"created_at"`
	CreatedUser     *ClockUser   `json:"created_user,omitempty"`
	ClockType       string       `json:"clock_type"` // mobile, admin, gps, wifi, beacon
	Location        *GeoLocation `json:"location,omitempty"`
	Option          *ClockOption `json:"option,omitempty"`
	Reason          string       `json:"reason,omitempty"`
	Note            string       `json:"note,omitempty"`
}

// ClockUser who created the clock entry
type ClockUser struct {
	Name     string `json:"name"`
	Username string `json:"username"`
	UserID   string `json:"user_id"`
}

// GeoLocation for GPS-based clock
type GeoLocation struct {
	Latitude  string `json:"latitude"`
	Longitude string `json:"longitude"`
	Accuracy  string `json:"accuracy,omitempty"`
}

// ClockOption describes the check-in method
type ClockOption struct {
	Type     string  `json:"type"` // gps, wifi, beacon
	Name     string  `json:"name"`
	Radius   int     `json:"radius,omitempty"`
	Lat      float64 `json:"lat,omitempty"`
	Long     float64 `json:"long,omitempty"`
	TypeName string  `json:"type_name,omitempty"`
}

// Shop represents company/shop information from /shop/detail
type Shop struct {
	ID           string  `json:"id"`
	Name         string  `json:"name"`
	Alias        string  `json:"alias"`
	Username     string  `json:"username"`
	Address      string  `json:"address"`
	AddressLat   *string `json:"address_lat"`
	AddressLng   *string `json:"address_lng"`
	Tel          string  `json:"tel"`
	Phone        string  `json:"phone"`
	Email        string  `json:"email"`
	Website      string  `json:"website"`
	Description  string  `json:"description"`
	Logo         string  `json:"logo"`
	BrandLogo    string  `json:"brand_logo"`
	BrandColor   string  `json:"brand_color"`
	Code         string  `json:"code"`
	TaxCode      string  `json:"tax_code"`
	Bank         string  `json:"bank"`
	BankAccount  string  `json:"bank_account"`
	FoundedDate  string  `json:"founded_date"`
	Scale        string  `json:"scale"`
	CreatedAt    string  `json:"created_at"`
	TimeFormat   string  `json:"timeFormat"`
	DateFormat   string  `json:"dateFormat"`
	Country      *OrgRef `json:"country,omitempty"`
	Province     *OrgRef `json:"province,omitempty"`
	District     *OrgRef `json:"district,omitempty"`
	PackagesInfo *PackagesInfo `json:"packages,omitempty"`
}

// PackagesInfo contains subscription package information
type PackagesInfo struct {
	Info     PackageBasicInfo        `json:"info"`
	Packages map[string]PackageItem  `json:"packages"`
}

// PackageBasicInfo contains basic package info
type PackageBasicInfo struct {
	StartPackage string `json:"start_package"`
	EndPackage   string `json:"end_package"`
	Users        int    `json:"users"`
	ActiveUsers  int    `json:"active_users"`
	IsPaid       int    `json:"is_paid"`
}

// PackageItem represents a single package subscription
type PackageItem struct {
	ID           string   `json:"id"`
	ShopID       string   `json:"shop_id"`
	Code         string   `json:"code"`
	Name         string   `json:"name"`
	Roles        []string `json:"roles"`
	Users        int      `json:"users"`
	StartPackage string   `json:"start_package"`
	EndPackage   string   `json:"end_package"`
	IsAvailable  int      `json:"is_available"`
}

// MakeShiftChangeRequestParams for POST /request/make-request (type=employee-shift)
type MakeShiftChangeRequestParams struct {
	UserID          string
	EmployeeShiftID string
	WorkingDay      string // YYYY-MM-DD
	CheckinTime     string // YYYY-MM-DD HH:mm:ss (full datetime)
	CheckoutTime    string // YYYY-MM-DD HH:mm:ss (full datetime)
	Reason          string
}

// ShiftChangeRequest is the response payload for an employee-shift time change request
type ShiftChangeRequest struct {
	ID               string       `json:"id"`
	UserID           string       `json:"user_id"`
	EmployeeShiftID  string       `json:"employee_shift_id"`
	CheckinTime      string       `json:"checkin_time"`      // HH:mm
	CheckinDatetime  string       `json:"checkin_datetime"`  // YYYY-MM-DD HH:mm:ss
	CheckoutTime     string       `json:"checkout_time"`
	CheckoutDatetime string       `json:"checkout_datetime"`
	Status           int          `json:"status"` // 0=pending, 1=approved, 2=rejected
	Reason           string       `json:"reason"`
	Type             string       `json:"type"` // "employee-shift"
	CreatedAt        string       `json:"created_at"`
	UpdatedAt        string       `json:"updated_at,omitempty"`
	Serial           string       `json:"serial"`
	Identification   string       `json:"identification,omitempty"`
	BranchObj        *OrgRef      `json:"branch_obj,omitempty"`
	DepartmentObj    *OrgRef      `json:"department_obj,omitempty"`
	PositionObj      *OrgRef      `json:"position_obj,omitempty"`
	UserObj          *RequestUser `json:"user_obj,omitempty"`
	CreatedUserObj   *RequestUser `json:"created_user_obj,omitempty"`
	ApproverObj      []OrgRef     `json:"approver_obj,omitempty"`
}

// RequestUser is a compact user reference embedded in request responses
type RequestUser struct {
	ID             string `json:"id"`
	Name           string `json:"name"`
	Username       string `json:"username"`
	Identification string `json:"identification,omitempty"`
	BranchID       string `json:"branch_id,omitempty"`
}

// ListRequestsParams filters /request/list-request
type ListRequestsParams struct {
	FromDate string // YYYY-MM-DD
	ToDate   string // YYYY-MM-DD
	Type     string // empty for all, e.g. "employee-shift", "onleave"
	Status   *int   // 0=pending, 1=approved, 2=rejected; nil=all
	Page     int    // default 1
}

// RequestSummary is a generic row in /request/list-request — fields vary by type
type RequestSummary struct {
	ID             string       `json:"id"`
	Type           string       `json:"type"`
	Status         int          `json:"status"`
	Serial         string       `json:"serial"`
	Reason         string       `json:"reason"`
	CreatedAt      string       `json:"created_at"`
	Identification string       `json:"identification,omitempty"`
	UserObj        *RequestUser `json:"user_obj,omitempty"`
	BranchObj      *OrgRef      `json:"branch_obj,omitempty"`
	DepartmentObj  *OrgRef      `json:"department_obj,omitempty"`

	// employee-shift specific
	EmployeeShiftID  string `json:"employee_shift_id,omitempty"`
	CheckinDatetime  string `json:"checkin_datetime,omitempty"`
	CheckoutDatetime string `json:"checkout_datetime,omitempty"`

	// onleave specific
	FromDate string `json:"from_date,omitempty"`
	ToDate   string `json:"to_date,omitempty"`
}

// Datetime format used by Tanca API
const TancaTimeFormat = "2006-01-02 15:04:05"
const TancaDateFormat = "2006-01-02"

// FormatTime formats time for Tanca API
func FormatTime(t time.Time) string {
	return t.Format(TancaTimeFormat)
}

// FormatDate formats date for Tanca API
func FormatDate(t time.Time) string {
	return t.Format(TancaDateFormat)
}

// FormatWorkingDay formats date with zero time for working_day field
func FormatWorkingDay(t time.Time) string {
	return t.Format(TancaDateFormat) + " 00:00:00"
}
