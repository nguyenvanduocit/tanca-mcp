package tools

import (
	"context"
	"fmt"
	"strings"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
	"github.com/nguyenvanduocit/tanca-mcp/services"
)

// Input types for employee tools
type ListEmployeesInput struct {
	Page         int    `json:"page,omitempty"`
	Limit        int    `json:"limit,omitempty"`
	DepartmentID string `json:"department_id,omitempty"`
	StartDate    string `json:"start_date,omitempty"`
	EndDate      string `json:"end_date,omitempty"`
}

type GetEmployeeInput struct {
	UserID string `json:"user_id" validate:"required"`
}

func RegisterEmployeeTool(s *server.MCPServer) {
	listEmployeesTool := mcp.NewTool("tanca_list_employees",
		mcp.WithDescription("List all employees from Tanca. Returns employee name, ID, phone, department, position, and timekeeping status."),
		mcp.WithNumber("page", mcp.Description("Page number for pagination (default: 1)")),
		mcp.WithNumber("limit", mcp.Description("Number of employees per page (default: 30, max: 100)")),
		mcp.WithString("department_id", mcp.Description("Filter by department ID (MongoDB ObjectId)")),
		mcp.WithString("start_date", mcp.Description("Filter start date (YYYY-MM-DD format)")),
		mcp.WithString("end_date", mcp.Description("Filter end date (YYYY-MM-DD format)")),
	)
	s.AddTool(listEmployeesTool, mcp.NewTypedToolHandler(listEmployeesHandler))

	getTimekeepingEmployeesTool := mcp.NewTool("tanca_list_timekeeping_employees",
		mcp.WithDescription("List all employees that require timekeeping (excludes exempt employees). Auto-paginates to get all employees."),
	)
	s.AddTool(getTimekeepingEmployeesTool, mcp.NewTypedToolHandler(listTimekeepingEmployeesHandler))
}

func listEmployeesHandler(ctx context.Context, request mcp.CallToolRequest, input ListEmployeesInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	params := services.GetEmployeeListParams{
		Page:      input.Page,
		Limit:     input.Limit,
		StartDate: input.StartDate,
		EndDate:   input.EndDate,
	}

	if input.DepartmentID != "" {
		params.DepartmentIDs = []string{input.DepartmentID}
	}

	data, err := client.GetEmployeeList(ctx, params)
	if err != nil {
		return nil, fmt.Errorf("failed to list employees: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Employees (Page %d/%d, Total: %d)\n\n",
		data.Meta.CurrentPage, data.Meta.TotalPages, data.Meta.Total))

	for _, emp := range data.Items {
		timekeeping := "Yes"
		if emp.IsNoNeedTimekeeping == 1 {
			timekeeping = "No"
		}

		result.WriteString(fmt.Sprintf("- %s (%s)\n", emp.Name, emp.Identification))
		result.WriteString(fmt.Sprintf("  User ID: %s\n", emp.UserID))
		result.WriteString(fmt.Sprintf("  Phone: %s\n", emp.Username))
		result.WriteString(fmt.Sprintf("  Branch ID: %s\n", emp.BranchID))
		result.WriteString(fmt.Sprintf("  Department ID: %s\n", emp.DepartmentID))
		result.WriteString(fmt.Sprintf("  Position ID: %s\n", emp.PositionID))
		result.WriteString(fmt.Sprintf("  Needs Timekeeping: %s\n", timekeeping))
		if emp.WorkingDate != "" {
			result.WriteString(fmt.Sprintf("  Working Date: %s\n", emp.WorkingDate))
		}
		result.WriteString("\n")
	}

	return mcp.NewToolResultText(result.String()), nil
}

type EmptyInput struct{}

func listTimekeepingEmployeesHandler(ctx context.Context, request mcp.CallToolRequest, input EmptyInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	employees, err := client.GetTimekeepingEmployees(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to list timekeeping employees: %w", err)
	}

	var result strings.Builder
	result.WriteString(fmt.Sprintf("Employees Requiring Timekeeping (Total: %d)\n\n", len(employees)))

	for _, emp := range employees {
		result.WriteString(fmt.Sprintf("- %s (%s)\n", emp.Name, emp.Identification))
		result.WriteString(fmt.Sprintf("  User ID: %s\n", emp.UserID))
		result.WriteString(fmt.Sprintf("  Phone: %s\n", emp.Username))
		result.WriteString(fmt.Sprintf("  Branch ID: %s\n", emp.BranchID))
		result.WriteString(fmt.Sprintf("  Department ID: %s\n", emp.DepartmentID))
		result.WriteString("\n")
	}

	return mcp.NewToolResultText(result.String()), nil
}
