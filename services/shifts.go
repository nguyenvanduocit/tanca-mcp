package services

import (
	"context"
	"encoding/json"
	"fmt"
	"net/url"
	"time"
)

// GetShiftAssignments retrieves all shift assignment types
func (c *Client) GetShiftAssignments(ctx context.Context) ([]ShiftAssignment, error) {
	body, err := c.Get(ctx, "/shift/list-shift-assignment-with-shift-v2", nil)
	if err != nil {
		return nil, fmt.Errorf("get shift assignments: %w", err)
	}

	var resp Response[PaginatedData[ShiftAssignment]]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}

	return resp.Data.Items, nil
}

// GetEmployeeShiftSummaryParams for shift summary query
type GetEmployeeShiftSummaryParams struct {
	UserID    string
	StartDate string // YYYY-MM-DD
	EndDate   string // YYYY-MM-DD
}

// GetEmployeeShiftSummary retrieves employee's shift assignments for a date range
func (c *Client) GetEmployeeShiftSummary(ctx context.Context, params GetEmployeeShiftSummaryParams) (*EmployeeShiftSummary, error) {
	shopID, err := c.GetShopID(ctx)
	if err != nil {
		return nil, err
	}

	query := url.Values{}
	query.Set("shop_id", shopID)
	query.Set("user_id", params.UserID)
	query.Set("start_date", params.StartDate)
	query.Set("end_date", params.EndDate)
	query.Set("is_shift", "1")
	query.Set("is_shift_only", "1")
	query.Set("is_status_shift", "1")

	body, err := c.Get(ctx, "/shift/summary-employee-shift", query)
	if err != nil {
		return nil, fmt.Errorf("get employee shift summary: %w", err)
	}

	var resp Response[PaginatedData[EmployeeShiftSummary]]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}

	if len(resp.Data.Items) == 0 {
		return nil, fmt.Errorf("no shift data found for user %s", params.UserID)
	}

	return &resp.Data.Items[0], nil
}

// GetTodayShift is a convenience method to get today's shift for an employee
func (c *Client) GetTodayShift(ctx context.Context, userID string) (*EmployeeShift, error) {
	today := FormatDate(time.Now())

	summary, err := c.GetEmployeeShiftSummary(ctx, GetEmployeeShiftSummaryParams{
		UserID:    userID,
		StartDate: today,
		EndDate:   today,
	})
	if err != nil {
		return nil, err
	}

	// Look for today's shift in the map
	// Key format: "YYYY-MM-DD 00:00:00"
	key := today + " 00:00:00"
	shifts, ok := summary.Shifts[key]
	if !ok || len(shifts) == 0 {
		return nil, fmt.Errorf("no shift assigned for %s on %s", userID, today)
	}

	// Return first shift (employees typically have one shift per day)
	return &shifts[0], nil
}

// ListEmployeeShiftsForDay returns the shifts assigned to a user on a specific working day.
// Use this to discover the employee_shift_id needed by MakeShiftChangeRequest.
func (c *Client) ListEmployeeShiftsForDay(ctx context.Context, userID, workingDay string) ([]EmployeeShift, error) {
	query := url.Values{}
	query.Set("is_unique_shift", "1")
	query.Set("is_available", "1")
	query.Set("user_id", userID)
	query.Set("working_day", workingDay)

	body, err := c.Get(ctx, "/shift/list-employee-shift", query)
	if err != nil {
		return nil, fmt.Errorf("list employee shifts for day: %w", err)
	}

	var resp Response[[]EmployeeShift]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}
	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}
	return resp.Data, nil
}

// CheckIn records a check-in for an employee's shift
func (c *Client) CheckIn(ctx context.Context, shiftAssignmentID, userID, branchID string, checkinTime time.Time) (*CheckInOutResponse, error) {
	workingDay := FormatWorkingDay(checkinTime)

	req := CheckInOutRequest{
		ID:           shiftAssignmentID,
		UserID:       userID,
		BranchID:     branchID,
		WorkingDay:   workingDay,
		CheckinTime:  FormatTime(checkinTime),
		CheckoutTime: FormatTime(checkinTime), // Required but ignored when is_checkout=0
		IsCheckin:    1,
		IsCheckout:   0,
		Reason:       "MCP Admin",
	}

	return c.checkInOut(ctx, req)
}

// CheckOut records a check-out for an employee's shift
func (c *Client) CheckOut(ctx context.Context, shiftAssignmentID, userID, branchID string, checkoutTime time.Time) (*CheckInOutResponse, error) {
	workingDay := FormatWorkingDay(checkoutTime)

	req := CheckInOutRequest{
		ID:           shiftAssignmentID,
		UserID:       userID,
		BranchID:     branchID,
		WorkingDay:   workingDay,
		CheckinTime:  FormatTime(checkoutTime), // Required but ignored when is_checkin=0
		CheckoutTime: FormatTime(checkoutTime),
		IsCheckin:    0,
		IsCheckout:   1,
		Reason:       "MCP Admin",
	}

	return c.checkInOut(ctx, req)
}

// CheckInOut records both check-in and check-out in a single call
func (c *Client) CheckInOut(ctx context.Context, shiftAssignmentID, userID, branchID string, checkinTime, checkoutTime time.Time) (*CheckInOutResponse, error) {
	workingDay := FormatWorkingDay(checkinTime)

	req := CheckInOutRequest{
		ID:           shiftAssignmentID,
		UserID:       userID,
		BranchID:     branchID,
		WorkingDay:   workingDay,
		CheckinTime:  FormatTime(checkinTime),
		CheckoutTime: FormatTime(checkoutTime),
		IsCheckin:    1,
		IsCheckout:   1,
		Reason:       "MCP Admin",
	}

	return c.checkInOut(ctx, req)
}

// checkInOut is the internal implementation for all check-in/out operations
func (c *Client) checkInOut(ctx context.Context, req CheckInOutRequest) (*CheckInOutResponse, error) {
	body, err := c.Post(ctx, "/shift/check-in-out-shift", req)
	if err != nil {
		return nil, fmt.Errorf("check-in/out: %w", err)
	}

	var resp Response[CheckInOutResponse]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}

	return &resp.Data, nil
}

// GetCheckedTime retrieves clock logs for a shift assignment
func (c *Client) GetCheckedTime(ctx context.Context, employeeShiftID string) ([]ClockLog, error) {
	query := url.Values{}
	query.Set("employee_shift_id", employeeShiftID)

	body, err := c.Get(ctx, "/shift/get-checked-time-employee-shift", query)
	if err != nil {
		return nil, fmt.Errorf("get checked time: %w", err)
	}

	var resp Response[[]ClockLog]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}

	return resp.Data, nil
}

// HasCheckedIn checks if an employee has already checked in for a shift
func (c *Client) HasCheckedIn(ctx context.Context, employeeShiftID string) (bool, error) {
	logs, err := c.GetCheckedTime(ctx, employeeShiftID)
	if err != nil {
		return false, err
	}

	// Check for any non-trashed clock entries
	for _, log := range logs {
		if log.IsTrashed == 0 {
			return true, nil
		}
	}

	return false, nil
}
