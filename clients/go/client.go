package osolarlink

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
)

const defaultBaseURL = "https://openapi.osolar.io"

type Client struct {
	apiKey     string
	baseURL    string
	httpClient *http.Client
}

type APIError struct {
	StatusCode int
	Status     string
	Body       []byte
}

func (e *APIError) Error() string {
	return fmt.Sprintf("osolar api error %d %s: %s", e.StatusCode, e.Status, e.Body)
}

func NewClient(apiKey string, baseURL string, httpClient *http.Client) *Client {
	if baseURL == "" {
		baseURL = defaultBaseURL
	}
	if httpClient == nil {
		httpClient = http.DefaultClient
	}

	return &Client{
		apiKey:     apiKey,
		baseURL:    trimTrailingSlash(baseURL),
		httpClient: httpClient,
	}
}

func (c *Client) SearchPlants(ctx context.Context, params SearchPlantsParams) (*ApiResponse[PlantGeoJSONResponse], error) {
	query := url.Values{}
	query.Set("q", params.Q)
	query.Set("field", params.Field)
	if params.DistanceKM != nil {
		query.Set("distance_km", fmt.Sprintf("%v", *params.DistanceKM))
	}
	return doJSON[ApiResponse[PlantGeoJSONResponse]](ctx, c, http.MethodGet, "/v1/search", query, nil)
}

func (c *Client) LinkPlant(ctx context.Context, body PlantLinkRequest) (*ApiResponse[PlantLinkResponse], error) {
	return doJSON[ApiResponse[PlantLinkResponse]](ctx, c, http.MethodPost, "/v1/links", nil, body)
}

func (c *Client) ListLinkedPlants(ctx context.Context) (*ApiResponse[[]PlantLinkListResponse], error) {
	return doJSON[ApiResponse[[]PlantLinkListResponse]](ctx, c, http.MethodGet, "/v1/links", nil, nil)
}

func (c *Client) GetPlantInfo(ctx context.Context, linkID string) (*ApiResponse[PlantInfoResponse], error) {
	return doJSON[ApiResponse[PlantInfoResponse]](ctx, c, http.MethodGet, "/v1/links/"+url.PathEscape(linkID), nil, nil)
}

func (c *Client) GetPlantContract(ctx context.Context, linkID string) (*ApiResponse[PlantContractResponse], error) {
	path := "/v1/links/" + url.PathEscape(linkID) + "/contract"
	return doJSON[ApiResponse[PlantContractResponse]](ctx, c, http.MethodGet, path, nil, nil)
}

func (c *Client) GetPlantDocuments(ctx context.Context, linkID string) (*ApiResponse[[]DocumentResponse], error) {
	path := "/v1/links/" + url.PathEscape(linkID) + "/documents"
	return doJSON[ApiResponse[[]DocumentResponse]](ctx, c, http.MethodGet, path, nil, nil)
}

func (c *Client) GetPlantOverview(ctx context.Context, linkID string) (*ApiResponse[PlantOverviewResponse], error) {
	path := "/v1/links/" + url.PathEscape(linkID) + "/overview"
	return doJSON[ApiResponse[PlantOverviewResponse]](ctx, c, http.MethodGet, path, nil, nil)
}

func (c *Client) GetMonthlyGeneration(ctx context.Context, linkID string, params MonthlyGenerationParams) (*ApiResponse[[]GenerationAmountResponse], error) {
	query := url.Values{}
	if params.StartYear != nil {
		query.Set("start_year", fmt.Sprintf("%d", *params.StartYear))
	}
	if params.EndYear != nil {
		query.Set("end_year", fmt.Sprintf("%d", *params.EndYear))
	}
	path := "/v1/links/" + url.PathEscape(linkID) + "/generation/monthly"
	return doJSON[ApiResponse[[]GenerationAmountResponse]](ctx, c, http.MethodGet, path, query, nil)
}

func (c *Client) GetMonthlyBilling(ctx context.Context, linkID string, params MonthlyBillingParams) (*ApiResponse[[]BillingAmountResponse], error) {
	query := url.Values{}
	if params.StartYear != nil {
		query.Set("startYear", fmt.Sprintf("%d", *params.StartYear))
	}
	if params.EndYear != nil {
		query.Set("endYear", fmt.Sprintf("%d", *params.EndYear))
	}
	path := "/v1/links/" + url.PathEscape(linkID) + "/billing/monthly"
	return doJSON[ApiResponse[[]BillingAmountResponse]](ctx, c, http.MethodGet, path, query, nil)
}

func doJSON[T any](ctx context.Context, c *Client, method string, path string, query url.Values, body any) (*T, error) {
	fullURL := c.baseURL + path
	if len(query) > 0 {
		fullURL += "?" + query.Encode()
	}

	var bodyReader io.Reader
	if body != nil {
		payload, err := json.Marshal(body)
		if err != nil {
			return nil, err
		}
		bodyReader = bytes.NewReader(payload)
	}

	req, err := http.NewRequestWithContext(ctx, method, fullURL, bodyReader)
	if err != nil {
		return nil, err
	}
	req.Header.Set("x-api-key", c.apiKey)
	if body != nil {
		req.Header.Set("content-type", "application/json")
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, &APIError{StatusCode: resp.StatusCode, Status: resp.Status, Body: respBody}
	}

	out := new(T)
	if err := json.Unmarshal(respBody, out); err != nil {
		return nil, err
	}
	return out, nil
}

func trimTrailingSlash(baseURL string) string {
	for len(baseURL) > 0 && baseURL[len(baseURL)-1] == '/' {
		baseURL = baseURL[:len(baseURL)-1]
	}
	return baseURL
}
