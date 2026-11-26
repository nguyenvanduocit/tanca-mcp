package services

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math/rand"
	"mime/multipart"
	"net/http"
	"net/url"
	"sync"
	"time"
)

// Config holds Tanca API configuration
type Config struct {
	BaseURL      string // https://api.tanca.io/api/v4
	Token        string // Bearer token (include "Bearer " prefix)
	ShopID       string // MongoDB ObjectId
	BranchID     string // MongoDB ObjectId
	DeviceHeader string // Base64 encoded device info
	Timezone     string // IANA timezone, default: Asia/Saigon
	Lang         string // Language code, default: vi
	IsAdmin      string // "1" for admin, "0" for regular
}

// Client is the Tanca API client
type Client struct {
	cfg        Config
	httpClient *http.Client
	maxRetries int
	baseDelay  time.Duration

	shopIDOnce   sync.Once
	cachedShopID string
	shopIDErr    error
}

// NewClient creates a new Tanca API client
func NewClient(cfg Config) *Client {
	// Set defaults
	if cfg.BaseURL == "" {
		cfg.BaseURL = "https://api.tanca.io/api/v4"
	}
	if cfg.Timezone == "" {
		cfg.Timezone = "Asia/Saigon"
	}
	if cfg.Lang == "" {
		cfg.Lang = "vi"
	}
	if cfg.IsAdmin == "" {
		cfg.IsAdmin = "1"
	}

	return &Client{
		cfg: cfg,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
		maxRetries: 3,
		baseDelay:  time.Second,
	}
}

// doRequest executes HTTP request with retry logic
func (c *Client) doRequest(ctx context.Context, method, path string, query url.Values, body interface{}) ([]byte, error) {
	var lastErr error

	for attempt := 0; attempt <= c.maxRetries; attempt++ {
		if attempt > 0 {
			delay := c.calculateBackoff(attempt)
			log.Printf("[tanca] retry %d/%d after %v", attempt, c.maxRetries, delay)

			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(delay):
			}
		}

		respBody, err := c.executeRequest(ctx, method, path, query, body)
		if err == nil {
			return respBody, nil
		}

		lastErr = err

		// Don't retry on context cancellation or client errors (4xx)
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}
		if isClientError(err) {
			return nil, err
		}

		log.Printf("[tanca] request failed (attempt %d): %v", attempt+1, err)
	}

	return nil, fmt.Errorf("max retries exceeded: %w", lastErr)
}

// executeRequest performs a single HTTP request
func (c *Client) executeRequest(ctx context.Context, method, path string, query url.Values, body interface{}) ([]byte, error) {
	// Build URL
	u := c.cfg.BaseURL + path
	if len(query) > 0 {
		u += "?" + query.Encode()
	}

	// Prepare body
	var bodyReader io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("marshal body: %w", err)
		}
		bodyReader = bytes.NewReader(data)
	}

	// Create request
	req, err := http.NewRequestWithContext(ctx, method, u, bodyReader)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	// Set headers
	c.setHeaders(req)
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	// Execute
	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("execute request: %w", err)
	}
	defer resp.Body.Close()

	// Read response
	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}

	// Check status
	if resp.StatusCode >= 400 {
		return nil, &APIError{
			StatusCode: resp.StatusCode,
			Body:       string(respBody),
		}
	}

	return respBody, nil
}

// setHeaders adds required Tanca headers
func (c *Client) setHeaders(req *http.Request) {
	req.Header.Set("authorization", c.cfg.Token)
	req.Header.Set("device", c.cfg.DeviceHeader)
	req.Header.Set("is-admin", c.cfg.IsAdmin)
	req.Header.Set("lang", c.cfg.Lang)
	req.Header.Set("timezone", c.cfg.Timezone)
	req.Header.Set("Accept", "application/json")
}

// calculateBackoff returns delay with exponential backoff and jitter
func (c *Client) calculateBackoff(attempt int) time.Duration {
	// Exponential: 1s, 2s, 4s, 8s...
	delay := c.baseDelay * (1 << (attempt - 1))

	// Cap at 1 minute
	if delay > time.Minute {
		delay = time.Minute
	}

	// Add jitter (0-25%)
	jitter := time.Duration(rand.Int63n(int64(delay / 4)))
	return delay + jitter
}

// APIError represents a Tanca API error response
type APIError struct {
	StatusCode int
	Body       string
}

func (e *APIError) Error() string {
	return fmt.Sprintf("tanca API error %d: %s", e.StatusCode, truncate(e.Body, 200))
}

// isClientError returns true for 4xx errors that shouldn't be retried
func isClientError(err error) bool {
	if apiErr, ok := err.(*APIError); ok {
		return apiErr.StatusCode >= 400 && apiErr.StatusCode < 500
	}
	return false
}

// truncate limits string length for logging
func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}

// Get performs a GET request
func (c *Client) Get(ctx context.Context, path string, query url.Values) ([]byte, error) {
	return c.doRequest(ctx, http.MethodGet, path, query, nil)
}

// Post performs a POST request
func (c *Client) Post(ctx context.Context, path string, body interface{}) ([]byte, error) {
	return c.doRequest(ctx, http.MethodPost, path, nil, body)
}

// PostForm performs a POST request with multipart/form-data body
func (c *Client) PostForm(ctx context.Context, path string, fields map[string]string) ([]byte, error) {
	var lastErr error
	for attempt := 0; attempt <= c.maxRetries; attempt++ {
		if attempt > 0 {
			delay := c.calculateBackoff(attempt)
			log.Printf("[tanca] retry %d/%d after %v", attempt, c.maxRetries, delay)
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(delay):
			}
		}

		respBody, err := c.executeFormRequest(ctx, path, fields)
		if err == nil {
			return respBody, nil
		}
		lastErr = err
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}
		if isClientError(err) {
			return nil, err
		}
		log.Printf("[tanca] form request failed (attempt %d): %v", attempt+1, err)
	}
	return nil, fmt.Errorf("max retries exceeded: %w", lastErr)
}

func (c *Client) executeFormRequest(ctx context.Context, path string, fields map[string]string) ([]byte, error) {
	var buf bytes.Buffer
	mw := multipart.NewWriter(&buf)
	for k, v := range fields {
		if err := mw.WriteField(k, v); err != nil {
			return nil, fmt.Errorf("write form field %s: %w", k, err)
		}
	}
	if err := mw.Close(); err != nil {
		return nil, fmt.Errorf("close multipart writer: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.cfg.BaseURL+path, &buf)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	c.setHeaders(req)
	req.Header.Set("Content-Type", mw.FormDataContentType())

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("execute request: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}
	if resp.StatusCode >= 400 {
		return nil, &APIError{StatusCode: resp.StatusCode, Body: string(respBody)}
	}
	return respBody, nil
}

// GetShopID returns the shop ID, fetching it from API if not configured
func (c *Client) GetShopID(ctx context.Context) (string, error) {
	// If configured, use it directly
	if c.cfg.ShopID != "" {
		return c.cfg.ShopID, nil
	}

	// Fetch once and cache
	c.shopIDOnce.Do(func() {
		log.Println("[tanca] fetching shop ID from API...")
		shop, err := c.GetShopDetail(ctx)
		if err != nil {
			c.shopIDErr = fmt.Errorf("auto-fetch shop ID: %w", err)
			return
		}
		c.cachedShopID = shop.ID
		log.Printf("[tanca] shop ID: %s (%s)", shop.ID, shop.Name)
	})

	if c.shopIDErr != nil {
		return "", c.shopIDErr
	}
	return c.cachedShopID, nil
}

// BranchID returns the configured branch ID
func (c *Client) BranchID() string {
	return c.cfg.BranchID
}
