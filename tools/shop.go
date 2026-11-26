package tools

import (
	"context"
	"fmt"
	"strings"

	"github.com/mark3labs/mcp-go/mcp"
	"github.com/mark3labs/mcp-go/server"
	"github.com/nguyenvanduocit/tanca-mcp/services"
)

func RegisterShopTool(s *server.MCPServer) {
	getShopInfoTool := mcp.NewTool("tanca_get_shop_info",
		mcp.WithDescription("Get company/shop information from Tanca. Returns shop ID, name, address, contact info, subscription packages, and other company details. Useful for getting shop_id without needing to configure it manually."),
	)
	s.AddTool(getShopInfoTool, mcp.NewTypedToolHandler(getShopInfoHandler))
}

func getShopInfoHandler(ctx context.Context, request mcp.CallToolRequest, input EmptyInput) (*mcp.CallToolResult, error) {
	client := services.TancaClient()

	shop, err := client.GetShopDetail(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to get shop info: %w", err)
	}

	var result strings.Builder
	result.WriteString("=== Company Information ===\n\n")
	result.WriteString(fmt.Sprintf("Shop ID: %s\n", shop.ID))
	result.WriteString(fmt.Sprintf("Name: %s\n", shop.Name))
	result.WriteString(fmt.Sprintf("Username: %s\n", shop.Username))

	if shop.Email != "" {
		result.WriteString(fmt.Sprintf("Email: %s\n", shop.Email))
	}
	if shop.Phone != "" {
		result.WriteString(fmt.Sprintf("Phone: %s\n", shop.Phone))
	}
	if shop.Website != "" {
		result.WriteString(fmt.Sprintf("Website: %s\n", shop.Website))
	}
	if shop.Address != "" {
		result.WriteString(fmt.Sprintf("Address: %s\n", shop.Address))
	}

	if shop.Country != nil {
		result.WriteString(fmt.Sprintf("Country: %s\n", shop.Country.Name))
	}
	if shop.Province != nil {
		result.WriteString(fmt.Sprintf("Province: %s\n", shop.Province.Name))
	}
	if shop.District != nil {
		result.WriteString(fmt.Sprintf("District: %s\n", shop.District.Name))
	}

	if shop.TaxCode != "" {
		result.WriteString(fmt.Sprintf("Tax Code: %s\n", shop.TaxCode))
	}
	if shop.FoundedDate != "" {
		result.WriteString(fmt.Sprintf("Founded: %s\n", shop.FoundedDate))
	}
	result.WriteString(fmt.Sprintf("Created At: %s\n", shop.CreatedAt))

	if shop.PackagesInfo != nil {
		result.WriteString("\n=== Subscription ===\n")
		info := shop.PackagesInfo.Info
		result.WriteString(fmt.Sprintf("Period: %s to %s\n", info.StartPackage, info.EndPackage))
		result.WriteString(fmt.Sprintf("Users: %d active / %d total\n", info.ActiveUsers, info.Users))

		if len(shop.PackagesInfo.Packages) > 0 {
			result.WriteString("\nActive Packages:\n")
			for _, pkg := range shop.PackagesInfo.Packages {
				if pkg.IsAvailable == 1 {
					result.WriteString(fmt.Sprintf("  - %s (%s)\n", pkg.Name, pkg.Code))
				}
			}
		}
	}

	return mcp.NewToolResultText(result.String()), nil
}
