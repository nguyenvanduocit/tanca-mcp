package services

import (
	"log"
	"os"
	"sync"
)

// TancaClient returns a singleton Tanca client instance
var TancaClient = sync.OnceValue[*Client](func() *Client {
	cfg := loadTancaConfig()
	return NewClient(cfg)
})

func loadTancaConfig() Config {
	token := os.Getenv("TANCA_TOKEN")
	shopID := os.Getenv("TANCA_SHOP_ID") // optional, auto-fetched if not set
	branchID := os.Getenv("TANCA_BRANCH_ID")
	device := os.Getenv("TANCA_DEVICE")

	if token == "" {
		log.Fatal("TANCA_TOKEN is required")
	}

	return Config{
		BaseURL:      "https://api.tanca.io/api/v4",
		Token:        token,
		ShopID:       shopID,
		BranchID:     branchID,
		DeviceHeader: device,
		Timezone:     "Asia/Saigon",
		Lang:         "vi",
		IsAdmin:      "1",
	}
}
