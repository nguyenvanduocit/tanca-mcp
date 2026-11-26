package services

import (
	"context"
	"encoding/json"
	"fmt"
	"net/url"
	"strconv"
)

// MakeShiftChangeRequest creates an employee-shift time-change request
// (POST /request/make-request, type=employee-shift). The request enters the
// approval flow with status=0 (pending).
func (c *Client) MakeShiftChangeRequest(ctx context.Context, params MakeShiftChangeRequestParams) (*ShiftChangeRequest, error) {
	fields := map[string]string{
		"type":              "employee-shift",
		"employee_shift_id": params.EmployeeShiftID,
		"working_day":       params.WorkingDay,
		"user_id":           params.UserID,
		"reason":            params.Reason,
		"checkin_time":      params.CheckinTime,
		"checkout_time":     params.CheckoutTime,
	}

	body, err := c.PostForm(ctx, "/request/make-request", fields)
	if err != nil {
		return nil, fmt.Errorf("make shift change request: %w", err)
	}

	var resp Response[ShiftChangeRequest]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}
	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}
	return &resp.Data, nil
}

// ListRequests retrieves submitted requests filtered by date range, type, and status.
func (c *Client) ListRequests(ctx context.Context, params ListRequestsParams) (*PaginatedData[RequestSummary], error) {
	if params.Page <= 0 {
		params.Page = 1
	}

	query := url.Values{}
	query.Set("from_date", params.FromDate)
	query.Set("to_date", params.ToDate)
	query.Set("type", params.Type)
	query.Set("page", strconv.Itoa(params.Page))
	if params.Status != nil {
		query.Set("status", strconv.Itoa(*params.Status))
	}

	body, err := c.Get(ctx, "/request/list-request", query)
	if err != nil {
		return nil, fmt.Errorf("list requests: %w", err)
	}

	var resp Response[PaginatedData[RequestSummary]]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}
	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}
	return &resp.Data, nil
}

// GetRequestDetail retrieves a single request by ID.
func (c *Client) GetRequestDetail(ctx context.Context, requestID string) (*RequestSummary, error) {
	query := url.Values{}
	query.Set("id", requestID)

	body, err := c.Get(ctx, "/request/detail", query)
	if err != nil {
		return nil, fmt.Errorf("get request detail: %w", err)
	}

	var resp Response[RequestSummary]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}
	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}
	return &resp.Data, nil
}
