package main

import (
	"context"
	"fmt"
	"os"

	"github.com/conalog/osolar-client/clients/go"
)

func main() {
	apiKey := os.Getenv("OSOLAR_API_KEY")
	if apiKey == "" {
		fmt.Fprintln(os.Stderr, "OSOLAR_API_KEY is required")
		os.Exit(1)
	}

	client := osolar.NewClient(apiKey, "", nil)
	resp, err := client.ListLinkedPlants(context.Background())
	if err != nil {
		fmt.Fprintf(os.Stderr, "Live smoke test failed: %v\n", err)
		os.Exit(1)
	}

	count := 0
	if resp.Data != nil {
		count = len(*resp.Data)
	}

	fmt.Printf("{\"success\":%t,\"linkedPlantCount\":%d}\n", resp.Success, count)
}
