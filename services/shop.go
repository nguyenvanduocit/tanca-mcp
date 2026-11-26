package services

import (
	"context"
	"encoding/json"
	"fmt"
)

// GetShopDetail retrieves the current shop/company information
func (c *Client) GetShopDetail(ctx context.Context) (*Shop, error) {
	body, err := c.Get(ctx, "/shop/detail", nil)
	if err != nil {
		return nil, fmt.Errorf("get shop detail: %w", err)
	}

	var resp Response[Shop]
	if err := json.Unmarshal(body, &resp); err != nil {
		return nil, fmt.Errorf("unmarshal response: %w", err)
	}

	if resp.ErrorCode != 0 {
		return nil, fmt.Errorf("API error %d: %v", resp.ErrorCode, resp.Message)
	}

	return &resp.Data, nil
}
