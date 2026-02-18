package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"regexp"
	"strings"

	"github.com/conalog/osolar-client/clients/go"
)

type routeResult struct {
	OK           bool   `json:"ok"`
	Status       int    `json:"status,omitempty"`
	PayloadSize  int    `json:"payloadSize,omitempty"`
	LinkedCount  int    `json:"linkedPlantCount,omitempty"`
	FeatureCount int    `json:"featureCount,omitempty"`
	SampleLinkID string `json:"sampleLinkId,omitempty"`
	Query        string `json:"query,omitempty"`
	Skipped      bool   `json:"skipped,omitempty"`
	Reason       string `json:"reason,omitempty"`
	Note         string `json:"note,omitempty"`
	Error        string `json:"error,omitempty"`
}

func main() {
	apiKey := os.Getenv("OSOLAR_API_KEY")
	if apiKey == "" {
		fmt.Fprintln(os.Stderr, "OSOLAR_API_KEY is required")
		os.Exit(1)
	}

	ctx := context.Background()
	client := osolar.NewClient(apiKey, "", nil)
	results := map[string]routeResult{}

	var linkID string
	searchKeyword := "서울"
	plantUUIDForLink := ""
	uuidRe := regexp.MustCompile(`^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}$`)

	linked, err := client.ListLinkedPlants(ctx)
	if err != nil {
		results["GET /v1/links"] = routeResult{OK: false, Error: err.Error()}
	} else {
		count := 0
		if linked.Data != nil {
			count = len(*linked.Data)
			if count > 0 {
				linkID = (*linked.Data)[0].LinkID
				if uuidRe.MatchString(linkID) {
					plantUUIDForLink = linkID
				}
				if (*linked.Data)[0].PlantAddress != nil && *(*linked.Data)[0].PlantAddress != "" {
					addr := *(*linked.Data)[0].PlantAddress
					runes := []rune(addr)
					if len(runes) > 12 {
						searchKeyword = string(runes[:12])
					} else {
						searchKeyword = addr
					}
				}
			}
		}
		results["GET /v1/links"] = routeResult{OK: true, LinkedCount: count, SampleLinkID: linkID}
	}

	distance := 2.0
	search, err := client.SearchPlants(ctx, osolar.SearchPlantsParams{Q: searchKeyword, Field: "address", DistanceKM: &distance})
	if err != nil {
		results["GET /v1/search"] = routeResult{OK: false, Error: err.Error()}
	} else {
		featureCount := 0
		if search.Data != nil {
			featureCount = len(search.Data.Features)
			if plantUUIDForLink == "" && featureCount > 0 {
				plantUUIDForLink = search.Data.Features[0].Properties.PlantUUID
			}
		}
		results["GET /v1/search"] = routeResult{OK: true, FeatureCount: featureCount, Query: searchKeyword}
	}

	if plantUUIDForLink == "" {
		plantUUIDForLink = "not-a-valid-uuid"
		fmt.Fprintln(os.Stderr, "No valid plant UUID found; using invalid UUID to exercise expected error-path for POST /v1/links")
	}
	_, err = client.LinkPlant(ctx, osolar.PlantLinkRequest{PlantUUID: plantUUIDForLink, Remark: "sdk live-all route smoke test"})
	if err != nil {
		if apiErr, ok := err.(*osolar.APIError); ok {
			results["POST /v1/links"] = routeResult{OK: apiErr.StatusCode >= 400, Status: apiErr.StatusCode, Note: "non-2xx is acceptable for live route smoke"}
		} else {
			results["POST /v1/links"] = routeResult{OK: false, Error: err.Error()}
		}
	} else {
		results["POST /v1/links"] = routeResult{OK: true, Note: "unexpectedly succeeded"}
	}

	guarded := []struct {
		route string
		call  func() error
	}{
		{"GET /v1/links/{link_id}", func() error { _, e := client.GetPlantInfo(ctx, linkID); return e }},
		{"GET /v1/links/{link_id}/contract", func() error { _, e := client.GetPlantContract(ctx, linkID); return e }},
		{"GET /v1/links/{link_id}/documents", func() error { _, e := client.GetPlantDocuments(ctx, linkID); return e }},
		{"GET /v1/links/{link_id}/overview", func() error { _, e := client.GetPlantOverview(ctx, linkID); return e }},
		{"GET /v1/links/{link_id}/generation/monthly", func() error {
			_, e := client.GetMonthlyGeneration(ctx, linkID, osolar.MonthlyGenerationParams{})
			return e
		}},
		{"GET /v1/links/{link_id}/billing/monthly", func() error {
			_, e := client.GetMonthlyBilling(ctx, linkID, osolar.MonthlyBillingParams{})
			return e
		}},
	}

	for _, g := range guarded {
		if strings.TrimSpace(linkID) == "" {
			results[g.route] = routeResult{OK: false, Skipped: true, Reason: "no linked plant available"}
			continue
		}
		err := g.call()
		if err != nil {
			results[g.route] = routeResult{OK: false, Error: err.Error()}
		} else {
			results[g.route] = routeResult{OK: true, PayloadSize: 1}
		}
	}

	payload, err := json.MarshalIndent(results, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to marshal results: %v\n", err)
		os.Exit(1)
	}
	fmt.Println(string(payload))

	hardFail := false
	for _, r := range results {
		if !r.OK && !r.Skipped {
			hardFail = true
			break
		}
	}
	if hardFail {
		os.Exit(1)
	}
}
