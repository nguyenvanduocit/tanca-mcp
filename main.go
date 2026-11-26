package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"log"
	"os"
	"strings"

	"github.com/joho/godotenv"
	"github.com/mark3labs/mcp-go/server"
	"github.com/nguyenvanduocit/tanca-mcp/tools"
)

func main() {
	envFile := flag.String("env", "", "Path to environment file (optional when environment variables are set directly)")
	httpPort := flag.String("http_port", "", "Port for HTTP server. If not provided, will use stdio")
	flag.Parse()

	// Load environment file if specified
	if *envFile != "" {
		if err := godotenv.Load(*envFile); err != nil {
			fmt.Printf("Warning: Error loading env file %s: %v\n", *envFile, err)
		} else {
			fmt.Printf("Loaded environment variables from %s\n", *envFile)
		}
	}

	// Check required environment variables
	if os.Getenv("TANCA_TOKEN") == "" {
		fmt.Println("Configuration Error: Missing TANCA_TOKEN")
		fmt.Println()
		fmt.Println("Setup Instructions:")
		fmt.Println("1. Get your Tanca API token from Tanca admin panel")
		fmt.Println("2. Set the environment variable:")
		fmt.Println()
		fmt.Println("   Option A - Using .env file:")
		fmt.Println("   TANCA_TOKEN=Bearer your-token-here")
		fmt.Println("   TANCA_SHOP_ID=your-shop-id (optional, auto-fetched)")
		fmt.Println("   TANCA_BRANCH_ID=your-branch-id (optional)")
		fmt.Println()
		fmt.Println("   Option B - Using environment variables:")
		fmt.Println("   export TANCA_TOKEN=\"Bearer your-token-here\"")
		fmt.Println()
		os.Exit(1)
	}

	fmt.Println("TANCA_TOKEN is set")
	if shopID := os.Getenv("TANCA_SHOP_ID"); shopID != "" {
		fmt.Printf("Shop ID: %s\n", shopID)
	} else {
		fmt.Println("Shop ID: (will be auto-fetched)")
	}

	mcpServer := server.NewMCPServer(
		"Tanca MCP",
		"1.0.0",
		server.WithLogging(),
		server.WithRecovery(),
	)

	// Register all Tanca tools
	tools.RegisterShopTool(mcpServer)
	tools.RegisterEmployeeTool(mcpServer)
	tools.RegisterShiftTool(mcpServer)
	tools.RegisterRequestTool(mcpServer)

	if *httpPort != "" {
		fmt.Println()
		fmt.Println("Starting Tanca MCP Server in HTTP mode...")
		fmt.Printf("Server will be available at: http://localhost:%s/mcp\n", *httpPort)
		fmt.Println()
		fmt.Println("Claude Code Configuration:")
		fmt.Println("Add the following to your Claude Code MCP settings:")
		fmt.Println()
		fmt.Println("```json")
		fmt.Println("{")
		fmt.Println("  \"mcpServers\": {")
		fmt.Println("    \"tanca\": {")
		fmt.Printf("      \"url\": \"http://localhost:%s/mcp\"\n", *httpPort)
		fmt.Println("    }")
		fmt.Println("  }")
		fmt.Println("}")
		fmt.Println("```")
		fmt.Println()
		fmt.Println("Server starting...")

		httpServer := server.NewStreamableHTTPServer(mcpServer, server.WithEndpointPath("/mcp"))
		if err := httpServer.Start(fmt.Sprintf(":%s", *httpPort)); err != nil && !isContextCanceled(err) {
			log.Fatalf("Server error: %v", err)
		}
	} else {
		if err := server.ServeStdio(mcpServer); err != nil && !isContextCanceled(err) {
			log.Fatalf("Server error: %v", err)
		}
	}
}

// isContextCanceled checks if the error is related to context cancellation
func isContextCanceled(err error) bool {
	if err == nil {
		return false
	}

	// Check if it's directly context.Canceled
	if errors.Is(err, context.Canceled) {
		return true
	}

	// Check if the error message contains context canceled
	errMsg := strings.ToLower(err.Error())
	return strings.Contains(errMsg, "context canceled") ||
		strings.Contains(errMsg, "operation was canceled") ||
		strings.Contains(errMsg, "context deadline exceeded")
}
