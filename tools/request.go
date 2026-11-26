package tools

import (
	"context"
	"fmt"
	"strings"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
	"github.com/nguyenvanduocit/tanca-mcp/services"
)

type MakeShiftChangeRequestInput struct {
	UserID          string `json:"user_id" validate:"required"`
	EmployeeShiftID string `json:"employee_shift_id" validate:"required"`
	WorkingDay      string `json:"working_day" validate:"required"`
	CheckinTime     string `json:"checkin_time" validate:"required"`
	CheckoutTime    string `json:"checkout_time" validate:"required"`
	Reason          string `json:"reason" validate:"required"`
}

type ListRequestsInput struct {
	FromDate string `json:"from_date" validate:"required"`
	ToDate   string `json:"to_date" validate:"required"`
	Type     string `json:"type,omitempty"`
	Status   *int   `json:"status,omitempty"`
	Page     int    `json:"page,omitempty"`
}

type GetRequestDetailInput struct {
	RequestID string `json:"request_id" validate:"required"`
}

func RegisterRequestTool(s *server.MCPServer) {
	makeShiftChangeRequestTool := mcp.NewTool("tanca_make_shift_change_request",
		mcp.WithDescription("Submit a time-change request (đề xuất chỉnh giờ vào/ra) for an employee shift. Goes through the approval workflow — status=0 pending until approved. Use tanca_get_shift_summary to find employee_shift_id."),
		mcp.WithString("user_id", mcp.Required(), mcp.Description("Employee's user ID")),
		mcp.WithString("employee_shift_id", mcp.Required(), mcp.Description("The employee shift ID for the working day to adjust (from tanca_get_shift_summary)")),
		mcp.WithString("working_day", mcp.Required(), mcp.Description("Working day in YYYY-MM-DD format")),
		mcp.WithString("checkin_time", mcp.Required(), mcp.Description("Proposed check-in datetime (YYYY-MM-DD HH:mm:ss)")),
		mcp.WithString("checkout_time", mcp.Required(), mcp.Description("Proposed check-out datetime (YYYY-MM-DD HH:mm:ss)")),
		mcp.WithString("reason", mcp.Required(), mcp.Description("Reason for the time change (e.g. 'Quên chấm công')")),
	)
	s.AddTool(makeShiftChangeRequestTool, mcp.NewTypedToolHandler(makeShiftChangeRequestHandler))

	listRequestsTool := mcp.NewTool("tanca_list_requests",
		mcp.WithDescription("List submitted requests (yêu cầu) within a date range. Filter by type ('employee-shift', 'onleave', 'overtime', etc.) and status (0=pending, 1=approved, 2=rejected)."),
		mcp.WithString("from_date", mcp.Required(), mcp.Description("Start date (YYYY-MM-DD)")),
		mcp.WithString("to_date", mcp.Required(), mcp.Description("End date (YYYY-MM-DD)")),
		mcp.WithString("type", mcp.Description("Request type filter: 'employee-shift', 'onleave', 'overtime', 'business-trip', etc. Empty for all.")),
		mcp.WithNumber("status", mcp.Description("Status filter: 0=pending, 1=approved, 2=rejected. Omit for all.")),
		mcp.WithNumber("page", mcp.Description("Page number (default 1)")),
	)
	s.AddTool(listRequestsTool, mcp.NewTypedToolHandler(listRequestsHandler))

	getRequestDetailTool := mcp.NewTool("tanca_get_request_detail",
		mcp.WithDescription("Get full details of a single request by its ID."),
		mcp.WithString("request_id", mcp.Required(), mcp.Description("The request ID")),
	)
	s.AddTool(getRequestDetailTool, mcp.NewTypedToolHandler(getRequestDetailHandler))
}

func makeShiftChangeRequestHandler(ctx context.Context, _ mcp.CallToolRequest, input MakeShiftChangeRequestInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	req, err := client.MakeShiftChangeRequest(ctx, services.MakeShiftChangeRequestParams{
		UserID:          input.UserID,
		EmployeeShiftID: input.EmployeeShiftID,
		WorkingDay:      input.WorkingDay,
		CheckinTime:     input.CheckinTime,
		CheckoutTime:    input.CheckoutTime,
		Reason:          input.Reason,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to submit shift change request: %w", err)
	}

	var result strings.Builder
	result.WriteString("Time change request submitted successfully!\n\n")
	result.WriteString(fmt.Sprintf("Request ID: %s\n", req.ID))
	result.WriteString(fmt.Sprintf("Serial: %s\n", req.Serial))
	result.WriteString(fmt.Sprintf("Status: %s\n", requestStatusName(req.Status)))
	result.WriteString(fmt.Sprintf("Working Day: %s\n", req.CheckinDatetime[:10]))
	result.WriteString(fmt.Sprintf("Check-in: %s\n", req.CheckinDatetime))
	result.WriteString(fmt.Sprintf("Check-out: %s\n", req.CheckoutDatetime))
	result.WriteString(fmt.Sprintf("Reason: %s\n", req.Reason))
	if req.UserObj != nil {
		result.WriteString(fmt.Sprintf("Employee: %s (%s)\n", req.UserObj.Name, req.UserObj.Username))
	}
	if len(req.ApproverObj) > 0 {
		approvers := make([]string, 0, len(req.ApproverObj))
		for _, a := range req.ApproverObj {
			approvers = append(approvers, a.Name)
		}
		result.WriteString(fmt.Sprintf("Approver(s): %s\n", strings.Join(approvers, ", ")))
	}

	return mcp.NewToolResultText(result.String()), nil
}

func listRequestsHandler(ctx context.Context, _ mcp.CallToolRequest, input ListRequestsInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	data, err := client.ListRequests(ctx, services.ListRequestsParams{
		FromDate: input.FromDate,
		ToDate:   input.ToDate,
		Type:     input.Type,
		Status:   input.Status,
		Page:     input.Page,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to list requests: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Requests %s → %s (Page %d/%d, Total: %d)\n\n",
		input.FromDate, input.ToDate, data.Meta.CurrentPage, data.Meta.TotalPages, data.Meta.Total))

	for _, r := range data.Items {
		result.WriteString(fmt.Sprintf("- [%s] %s — %s\n", r.Serial, requestTypeLabel(r.Type), requestStatusName(r.Status)))
		result.WriteString(fmt.Sprintf("  ID: %s\n", r.ID))
		if r.UserObj != nil {
			result.WriteString(fmt.Sprintf("  Employee: %s\n", r.UserObj.Name))
		}
		if r.Reason != "" {
			result.WriteString(fmt.Sprintf("  Reason: %s\n", r.Reason))
		}
		switch r.Type {
		case "employee-shift":
			if r.CheckinDatetime != "" {
				result.WriteString(fmt.Sprintf("  Proposed: %s → %s\n", r.CheckinDatetime, r.CheckoutDatetime))
			}
		case "onleave":
			if r.FromDate != "" {
				result.WriteString(fmt.Sprintf("  Range: %s → %s\n", r.FromDate, r.ToDate))
			}
		}
		result.WriteString(fmt.Sprintf("  Created: %s\n\n", r.CreatedAt))
	}

	return mcp.NewToolResultText(result.String()), nil
}

func getRequestDetailHandler(ctx context.Context, _ mcp.CallToolRequest, input GetRequestDetailInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	r, err := client.GetRequestDetail(ctx, input.RequestID)
	if err != nil {
		return nil, fmt.Errorf("failed to get request detail: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Request %s — %s\n\n", r.Serial, requestTypeLabel(r.Type)))
	result.WriteString(fmt.Sprintf("ID: %s\n", r.ID))
	result.WriteString(fmt.Sprintf("Status: %s\n", requestStatusName(r.Status)))
	result.WriteString(fmt.Sprintf("Created: %s\n", r.CreatedAt))
	if r.UserObj != nil {
		result.WriteString(fmt.Sprintf("Employee: %s (%s)\n", r.UserObj.Name, r.UserObj.Username))
	}
	if r.BranchObj != nil {
		result.WriteString(fmt.Sprintf("Branch: %s\n", r.BranchObj.Name))
	}
	if r.DepartmentObj != nil {
		result.WriteString(fmt.Sprintf("Department: %s\n", r.DepartmentObj.Name))
	}
	if r.Reason != "" {
		result.WriteString(fmt.Sprintf("Reason: %s\n", r.Reason))
	}
	switch r.Type {
	case "employee-shift":
		if r.CheckinDatetime != "" {
			result.WriteString(fmt.Sprintf("Proposed Check-in: %s\n", r.CheckinDatetime))
			result.WriteString(fmt.Sprintf("Proposed Check-out: %s\n", r.CheckoutDatetime))
		}
	case "onleave":
		if r.FromDate != "" {
			result.WriteString(fmt.Sprintf("From: %s\n", r.FromDate))
			result.WriteString(fmt.Sprintf("To: %s\n", r.ToDate))
		}
	}

	return mcp.NewToolResultText(result.String()), nil
}

func requestStatusName(status int) string {
	switch status {
	case 0:
		return "Pending (chờ duyệt)"
	case 1:
		return "Approved (đã duyệt)"
	case 2:
		return "Rejected (từ chối)"
	default:
		return fmt.Sprintf("Unknown (%d)", status)
	}
}

func requestTypeLabel(t string) string {
	switch t {
	case "employee-shift":
		return "Thay đổi giờ vào/ra"
	case "onleave":
		return "Nghỉ phép"
	case "overtime":
		return "Làm thêm giờ"
	case "business-trip":
		return "Công tác/Ra ngoài"
	case "register-shift":
		return "Đăng ký ca làm"
	case "company-advance", "advance-reimbursement":
		return "Tạm ứng - Hoàn ứng"
	case "payment":
		return "Thanh toán"
	default:
		return t
	}
}
