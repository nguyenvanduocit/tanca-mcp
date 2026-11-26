package services

import (
	"context"
	"encoding/json"
	"fmt"
	"net/url"
	"strconv"
)

// GetEmployeeListParams for filtering employee list
type GetEmployeeListParams struct {
	Page                int
	Limit               int
	StartDate           string   // YYYY-MM-DD
	EndDate             string   // YYYY-MM-DD
	IsNoNeedTimekeeping *int     // 0 = needs timekeeping, 1 = exempt
	IsInactive          *int     // 0 = active only, 1 = include inactive
	DepartmentIDs       []string // Filter by department IDs
	TaskStatusFilter    *int     // 1 = filter by task status
	TransType           string   // e.g., "timekeeping"
}

// GetEmployeeList retrieves a paginated list of employees
func (c *Client) GetEmployeeList(ctx context.Context, params GetEmployeeListParams) (*PaginatedData[Employee], error) {
	shopID, err := c.GetShopID(ctx)
	if err != nil {
		return nil, err
	}

	query := url.Values{}
	query.Set("shop_id", shopID)

	// Defaults
	if params.Page <= 0 {
		params.Page = 1
	}
	if params.Limit <= 0 {
		params.Limit = 30
	}

	query.Set("page", strconv.Itoa(params.Page))
	query.Set("limit", strconv.Itoa(params.Limit))

	if params.StartDate != "" {
		query.Set("start_date", params.StartDate)
	}
	if params.EndDate != "" {
		query.Set("end_date", params.EndDate)
	}
	if params.IsNoNeedTimekeeping != nil {
		query.Set("is_no_need_timekeeping", strconv.Itoa(*params.IsNoNeedTimekeeping))
	}
	if params.IsInactive != nil {
		query.Set("is_inactive", strconv.Itoa(*params.IsInactive))
	}
	// Department IDs - PHP-style array notation
	for i, deptID := range params.DepartmentIDs {
		query.Set(fmt.Sprintf("department_ids[%d]", i), deptID)
	}
	if params.TaskStatusFilter != nil {
		query.Set("task_status_filter", strconv.Itoa(*params.TaskStatusFilter))
	}
	if params.TransType != "" {
		query.Set("trans_type", params.TransType)
	}

	body, err := c.Get(ctx, "/employee/list", query)
	if err != nil {
		return nil, fmt.Errorf("get employee list: %w", err)
	}

	var resp Response[PaginatedData[Employee]]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}

	return &resp.Data, nil
}

// GetAllEmployees retrieves all employees (auto-paginating)
func (c *Client) GetAllEmployees(ctx context.Context) ([]Employee, error) {
	var all []Employee
	page := 1
	limit := 100 // Use larger page size for efficiency

	for {
		data, err := c.GetEmployeeList(ctx, GetEmployeeListParams{
			Page:  page,
			Limit: limit,
		})
		if err != nil {
			return nil, err
		}

		all = append(all, data.Items...)

		// Check if we've fetched all pages
		if page >= data.Meta.TotalPages || len(data.Items) == 0 {
			break
		}
		page++
	}

	return all, nil
}

// GetTimekeepingEmployees retrieves only employees that need timekeeping
func (c *Client) GetTimekeepingEmployees(ctx context.Context) ([]Employee, error) {
	var all []Employee
	page := 1
	limit := 100
	noTimekeeping := 0 // Only those who need timekeeping

	for {
		data, err := c.GetEmployeeList(ctx, GetEmployeeListParams{
			Page:                page,
			Limit:               limit,
			IsNoNeedTimekeeping: &noTimekeeping,
		})
		if err != nil {
			return nil, err
		}

		all = append(all, data.Items...)

		if page >= data.Meta.TotalPages || len(data.Items) == 0 {
			break
		}
		page++
	}

	return all, nil
}

// GetEmployeesByDepartment retrieves employees from specific department
func (c *Client) GetEmployeesByDepartment(ctx context.Context, departmentID, startDate, endDate string) ([]Employee, error) {
	var all []Employee
	page := 1
	limit := 30
	noTimekeeping := 0
	inactive := 0
	taskStatusFilter := 1

	for {
		data, err := c.GetEmployeeList(ctx, GetEmployeeListParams{
			Page:                page,
			Limit:               limit,
			StartDate:           startDate,
			EndDate:             endDate,
			IsNoNeedTimekeeping: &noTimekeeping,
			IsInactive:          &inactive,
			DepartmentIDs:       []string{departmentID},
			TaskStatusFilter:    &taskStatusFilter,
			TransType:           "timekeeping",
		})
		if err != nil {
			return nil, err
		}

		all = append(all, data.Items...)

		if page >= data.Meta.TotalPages || len(data.Items) == 0 {
			break
		}
		page++
	}

	return all, nil
}
