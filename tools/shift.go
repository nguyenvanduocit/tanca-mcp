package tools

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
	"github.com/nguyenvanduocit/tanca-mcp/services"
)

// Input types for shift tools
type GetShiftSummaryInput struct {
	UserID    string `json:"user_id" validate:"required"`
	StartDate string `json:"start_date" validate:"required"`
	EndDate   string `json:"end_date" validate:"required"`
}

type GetTodayShiftInput struct {
	UserID string `json:"user_id" validate:"required"`
}

type GetShiftByDayInput struct {
	UserID     string `json:"user_id" validate:"required"`
	WorkingDay string `json:"working_day" validate:"required"`
}

type CheckInInput struct {
	ShiftAssignmentID string `json:"shift_assignment_id" validate:"required"`
	UserID            string `json:"user_id" validate:"required"`
	BranchID          string `json:"branch_id" validate:"required"`
	CheckinTime       string `json:"checkin_time,omitempty"` // YYYY-MM-DD HH:mm:ss, defaults to now
}

type CheckOutInput struct {
	ShiftAssignmentID string `json:"shift_assignment_id" validate:"required"`
	UserID            string `json:"user_id" validate:"required"`
	BranchID          string `json:"branch_id" validate:"required"`
	CheckoutTime      string `json:"checkout_time,omitempty"` // YYYY-MM-DD HH:mm:ss, defaults to now
}

type GetClockLogsInput struct {
	EmployeeShiftID string `json:"employee_shift_id" validate:"required"`
}

func RegisterShiftTool(s *server.MCPServer) {
	listShiftsTool := mcp.NewTool("tanca_list_shifts",
		mcp.WithDescription("List all available shift types/assignments configured in Tanca."),
	)
	s.AddTool(listShiftsTool, mcp.NewTypedToolHandler(listShiftsHandler))

	getShiftSummaryTool := mcp.NewTool("tanca_get_shift_summary",
		mcp.WithDescription("Get an employee's shift assignments for a date range. Returns shift details including check-in/out times and status."),
		mcp.WithString("user_id", mcp.Required(), mcp.Description("Employee's user ID")),
		mcp.WithString("start_date", mcp.Required(), mcp.Description("Start date (YYYY-MM-DD format)")),
		mcp.WithString("end_date", mcp.Required(), mcp.Description("End date (YYYY-MM-DD format)")),
	)
	s.AddTool(getShiftSummaryTool, mcp.NewTypedToolHandler(getShiftSummaryHandler))

	getTodayShiftTool := mcp.NewTool("tanca_get_today_shift",
		mcp.WithDescription("Get an employee's shift for today. Returns the shift assignment ID needed for check-in/out."),
		mcp.WithString("user_id", mcp.Required(), mcp.Description("Employee's user ID")),
	)
	s.AddTool(getTodayShiftTool, mcp.NewTypedToolHandler(getTodayShiftHandler))

	getShiftByDayTool := mcp.NewTool("tanca_get_shift_by_day",
		mcp.WithDescription("Get an employee's shifts for a specific working day. Returns the employee_shift_id needed by tanca_make_shift_change_request."),
		mcp.WithString("user_id", mcp.Required(), mcp.Description("Employee's user ID")),
		mcp.WithString("working_day", mcp.Required(), mcp.Description("Target working day in YYYY-MM-DD format")),
	)
	s.AddTool(getShiftByDayTool, mcp.NewTypedToolHandler(getShiftByDayHandler))

	checkInTool := mcp.NewTool("tanca_check_in",
		mcp.WithDescription("Record a check-in for an employee's shift. Requires the shift assignment ID from tanca_get_today_shift."),
		mcp.WithString("shift_assignment_id", mcp.Required(), mcp.Description("The shift assignment ID from tanca_get_shift_summary")),
		mcp.WithString("user_id", mcp.Required(), mcp.Description("Employee's user ID")),
		mcp.WithString("branch_id", mcp.Required(), mcp.Description("Branch ID where check-in occurs")),
		mcp.WithString("checkin_time", mcp.Description("Check-in time (YYYY-MM-DD HH:mm:ss). Defaults to current time.")),
	)
	s.AddTool(checkInTool, mcp.NewTypedToolHandler(checkInHandler))

	checkOutTool := mcp.NewTool("tanca_check_out",
		mcp.WithDescription("Record a check-out for an employee's shift. Requires the shift assignment ID from tanca_get_today_shift."),
		mcp.WithString("shift_assignment_id", mcp.Required(), mcp.Description("The shift assignment ID from tanca_get_shift_summary")),
		mcp.WithString("user_id", mcp.Required(), mcp.Description("Employee's user ID")),
		mcp.WithString("branch_id", mcp.Required(), mcp.Description("Branch ID where check-out occurs")),
		mcp.WithString("checkout_time", mcp.Description("Check-out time (YYYY-MM-DD HH:mm:ss). Defaults to current time.")),
	)
	s.AddTool(checkOutTool, mcp.NewTypedToolHandler(checkOutHandler))

	getClockLogsTool := mcp.NewTool("tanca_get_clock_logs",
		mcp.WithDescription("Get all clock logs (check-in/out entries) for a specific shift assignment."),
		mcp.WithString("employee_shift_id", mcp.Required(), mcp.Description("The employee shift ID to get logs for")),
	)
	s.AddTool(getClockLogsTool, mcp.NewTypedToolHandler(getClockLogsHandler))
}

func listShiftsHandler(ctx context.Context, request mcp.CallToolRequest, input EmptyInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	shifts, err := client.GetShiftAssignments(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to list shifts: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Available Shifts (Total: %d)\n\n", len(shifts)))

	for _, sa := range shifts {
		result.WriteString(fmt.Sprintf("- %s (ID: %s)\n", sa.Title, sa.ID))
		result.WriteString(fmt.Sprintf("  Shift: %s (%s)\n", sa.Shift.Name, sa.Shift.ShiftKey))
		result.WriteString(fmt.Sprintf("  Time: %s:%s - %s:%s\n",
			sa.Shift.StartHourObj.Value, sa.Shift.StartMinuteObj.Value,
			sa.Shift.EndHourObj.Value, sa.Shift.EndMinuteObj.Value))
		result.WriteString(fmt.Sprintf("  Working Hours: %d\n", sa.Shift.WorkingHour))
		if sa.Shift.IsOvertimeShift == 1 {
			result.WriteString("  Type: Overtime Shift\n")
		}
		result.WriteString("\n")
	}

	return mcp.NewToolResultText(result.String()), nil
}

func getShiftSummaryHandler(ctx context.Context, request mcp.CallToolRequest, input GetShiftSummaryInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	summary, err := client.GetEmployeeShiftSummary(ctx, services.GetEmployeeShiftSummaryParams{
		UserID:    input.UserID,
		StartDate: input.StartDate,
		EndDate:   input.EndDate,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to get shift summary: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Shift Summary for %s (%s)\n", summary.Name, summary.Identification))
	result.WriteString(fmt.Sprintf("Phone: %s\n", summary.Phone))
	result.WriteString(fmt.Sprintf("Total Working Hours: %d\n", summary.TotalWorkingHour))
	result.WriteString(fmt.Sprintf("Real Working Hours: %d\n", summary.RealWorkingHour))
	result.WriteString("\nShifts:\n")

	for date, shifts := range summary.Shifts {
		result.WriteString(fmt.Sprintf("\n%s:\n", date))
		for _, shift := range shifts {
			result.WriteString(fmt.Sprintf("  - %s (ID: %s)\n", shift.Name, shift.ID))
			result.WriteString(fmt.Sprintf("    Time: %s - %s\n", shift.StartTime, shift.EndTime))
			result.WriteString(fmt.Sprintf("    Working Hours: %d\n", shift.WorkingHour))
			if shift.CheckinTime != nil {
				result.WriteString(fmt.Sprintf("    Check-in: %s (%s)\n", *shift.CheckinTime, shift.CheckinType))
			} else {
				result.WriteString("    Check-in: Not recorded\n")
			}
			if shift.CheckoutTime != nil {
				result.WriteString(fmt.Sprintf("    Check-out: %s (%s)\n", *shift.CheckoutTime, shift.CheckoutType))
			} else {
				result.WriteString("    Check-out: Not recorded\n")
			}
			if shift.Status != nil {
				result.WriteString(fmt.Sprintf("    Status: %s\n", shift.Status.Name))
			}
		}
	}

	return mcp.NewToolResultText(result.String()), nil
}

func getTodayShiftHandler(ctx context.Context, request mcp.CallToolRequest, input GetTodayShiftInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	shift, err := client.GetTodayShift(ctx, input.UserID)
	if err != nil {
		return nil, fmt.Errorf("failed to get today's shift: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Today's Shift for User %s\n\n", input.UserID))
	result.WriteString(fmt.Sprintf("Shift Assignment ID: %s\n", shift.ID))
	result.WriteString(fmt.Sprintf("Shift Name: %s\n", shift.Name))
	result.WriteString(fmt.Sprintf("Shift Key: %s\n", shift.ShiftKey))
	result.WriteString(fmt.Sprintf("Time: %s - %s\n", shift.StartTime, shift.EndTime))
	result.WriteString(fmt.Sprintf("Working Day: %s\n", shift.WorkingDay))
	result.WriteString(fmt.Sprintf("Working Hours: %d\n", shift.WorkingHour))
	result.WriteString(fmt.Sprintf("Branch ID: %s\n", shift.BranchID))

	if shift.CheckinTime != nil {
		result.WriteString(fmt.Sprintf("\nCheck-in: %s (%s)\n", *shift.CheckinTime, shift.CheckinType))
	} else {
		result.WriteString("\nCheck-in: Not recorded\n")
	}
	if shift.CheckoutTime != nil {
		result.WriteString(fmt.Sprintf("Check-out: %s (%s)\n", *shift.CheckoutTime, shift.CheckoutType))
	} else {
		result.WriteString("Check-out: Not recorded\n")
	}

	if shift.Status != nil {
		result.WriteString(fmt.Sprintf("\nStatus: %s\n", shift.Status.Name))
	}

	return mcp.NewToolResultText(result.String()), nil
}

func getShiftByDayHandler(ctx context.Context, _ mcp.CallToolRequest, input GetShiftByDayInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	shifts, err := client.ListEmployeeShiftsForDay(ctx, input.UserID, input.WorkingDay)
	if err != nil {
		return nil, fmt.Errorf("failed to get shift by day: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Shifts for User %s on %s (Total: %d)\n\n", input.UserID, input.WorkingDay, len(shifts)))
	if len(shifts) == 0 {
		result.WriteString("No shift assigned for this day.\n")
	}
	for _, shift := range shifts {
		result.WriteString(fmt.Sprintf("- %s (%s)\n", shift.Name, shift.ShiftKey))
		result.WriteString(fmt.Sprintf("  Employee Shift ID: %s\n", shift.ID))
		result.WriteString(fmt.Sprintf("  Time: %s → %s\n", shift.StartTime, shift.EndTime))
		result.WriteString(fmt.Sprintf("  Working Hours: %d\n", shift.WorkingHour))
		result.WriteString(fmt.Sprintf("  Branch ID: %s\n", shift.BranchID))
		if shift.CheckinTime != nil {
			result.WriteString(fmt.Sprintf("  Check-in: %s\n", *shift.CheckinTime))
		} else {
			result.WriteString("  Check-in: Not recorded\n")
		}
		if shift.CheckoutTime != nil {
			result.WriteString(fmt.Sprintf("  Check-out: %s\n", *shift.CheckoutTime))
		} else {
			result.WriteString("  Check-out: Not recorded\n")
		}
		result.WriteString("\n")
	}
	return mcp.NewToolResultText(result.String()), nil
}

func checkInHandler(ctx context.Context, request mcp.CallToolRequest, input CheckInInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	// Parse check-in time or use current time
	var checkinTime time.Time
	if input.CheckinTime != "" {
		loc, _ := time.LoadLocation("Asia/Saigon")
		var err error
		checkinTime, err = time.ParseInLocation(services.TancaTimeFormat, input.CheckinTime, loc)
		if err != nil {
			return nil, fmt.Errorf("invalid checkin_time format (use YYYY-MM-DD HH:mm:ss): %w", err)
		}
	} else {
		checkinTime = time.Now()
	}

	resp, err := client.CheckIn(ctx, input.ShiftAssignmentID, input.UserID, input.BranchID, checkinTime)
	if err != nil {
		return nil, fmt.Errorf("failed to check in: %w", err)
	}

	var result strings.Builder
	result.WriteString("Check-in recorded successfully!\n\n")
	result.WriteString(fmt.Sprintf("Shift: %s\n", resp.Name))
	result.WriteString(fmt.Sprintf("Check-in Time: %s\n", resp.CheckinTime))
	result.WriteString(fmt.Sprintf("Check-in Type: %s\n", resp.CheckinType))
	if resp.Status != nil {
		result.WriteString(fmt.Sprintf("Status: %s\n", resp.Status.Name))
	}

	return mcp.NewToolResultText(result.String()), nil
}

func checkOutHandler(ctx context.Context, request mcp.CallToolRequest, input CheckOutInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	// Parse check-out time or use current time
	var checkoutTime time.Time
	if input.CheckoutTime != "" {
		loc, _ := time.LoadLocation("Asia/Saigon")
		var err error
		checkoutTime, err = time.ParseInLocation(services.TancaTimeFormat, input.CheckoutTime, loc)
		if err != nil {
			return nil, fmt.Errorf("invalid checkout_time format (use YYYY-MM-DD HH:mm:ss): %w", err)
		}
	} else {
		checkoutTime = time.Now()
	}

	resp, err := client.CheckOut(ctx, input.ShiftAssignmentID, input.UserID, input.BranchID, checkoutTime)
	if err != nil {
		return nil, fmt.Errorf("failed to check out: %w", err)
	}

	var result strings.Builder
	result.WriteString("Check-out recorded successfully!\n\n")
	result.WriteString(fmt.Sprintf("Shift: %s\n", resp.Name))
	result.WriteString(fmt.Sprintf("Check-out Time: %s\n", resp.CheckoutTime))
	result.WriteString(fmt.Sprintf("Check-out Type: %s\n", resp.CheckoutType))
	if resp.Status != nil {
		result.WriteString(fmt.Sprintf("Status: %s\n", resp.Status.Name))
	}

	return mcp.NewToolResultText(result.String()), nil
}

func getClockLogsHandler(ctx context.Context, request mcp.CallToolRequest, input GetClockLogsInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	logs, err := client.GetCheckedTime(ctx, input.EmployeeShiftID)
	if err != nil {
		return nil, fmt.Errorf("failed to get clock logs: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Clock Logs for Shift %s (Total: %d)\n\n", input.EmployeeShiftID, len(logs)))

	for i, log := range logs {
		trashed := ""
		if log.IsTrashed == 1 {
			trashed = " [DELETED]"
		}

		result.WriteString(fmt.Sprintf("%d. %s%s\n", i+1, log.Time, trashed))
		result.WriteString(fmt.Sprintf("   Type: %s\n", log.ClockType))
		result.WriteString(fmt.Sprintf("   Created: %s\n", log.CreatedAt))
		if log.CreatedUser != nil {
			result.WriteString(fmt.Sprintf("   Created By: %s\n", log.CreatedUser.Name))
		}
		if log.Reason != "" {
			result.WriteString(fmt.Sprintf("   Reason: %s\n", log.Reason))
		}
		if log.Location != nil {
			result.WriteString(fmt.Sprintf("   Location: %s, %s\n", log.Location.Latitude, log.Location.Longitude))
		}
		result.WriteString("\n")
	}

	return mcp.NewToolResultText(result.String()), nil
}
