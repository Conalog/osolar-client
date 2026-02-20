package osolar

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"strings"
	"time"
)

const defaultBaseURL = "https://openapi.osolar.io"
const defaultHTTPTimeout = 30 * time.Second
const maxResponseBodyBytes = 2 << 20

type Client struct {
	apiKey     string
	baseURL    string
	httpClient *http.Client
	initErr    error
}

type APIError struct {
	StatusCode int
	Status     string
	Body       []byte
}

func (e *APIError) Error() string {
	if len(e.Body) == 0 {
		return fmt.Sprintf("osolar api error %d %s", e.StatusCode, e.Status)
	}
	return fmt.Sprintf("osolar api error %d %s (body %d bytes)", e.StatusCode, e.Status, len(e.Body))
}

func NewClient(apiKey string, baseURL string, httpClient *http.Client) *Client {
	if baseURL == "" {
		baseURL = defaultBaseURL
	}
	baseURL = trimTrailingSlash(baseURL)
	initErr := validateBaseURL(baseURL)
	if httpClient == nil {
		httpClient = &http.Client{Timeout: defaultHTTPTimeout}
	} else {
		httpClient = cloneHTTPClient(httpClient)
	}
	httpClient.CheckRedirect = composeRedirectPolicy(httpClient.CheckRedirect)

	return &Client{
		apiKey:     apiKey,
		baseURL:    baseURL,
		httpClient: httpClient,
		initErr:    initErr,
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
	if c.initErr != nil {
		return nil, c.initErr
	}
	if ctx == nil {
		return nil, errors.New("context cannot be nil")
	}

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
	resp.Body = http.MaxBytesReader(nil, resp.Body, maxResponseBodyBytes)
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		respBody, _ := io.ReadAll(resp.Body)
		return nil, &APIError{StatusCode: resp.StatusCode, Status: resp.Status, Body: respBody}
	}

	if resp.StatusCode == http.StatusNoContent {
		out := new(T)
		markSuccessOnEmptyBody(out)
		return out, nil
	}

	out := new(T)
	if err := json.NewDecoder(resp.Body).Decode(out); err != nil {
		if err == io.EOF {
			markSuccessOnEmptyBody(out)
			return out, nil
		}
		var maxBytesErr *http.MaxBytesError
		if errors.As(err, &maxBytesErr) {
			return nil, fmt.Errorf("osolar response body exceeds %d bytes", maxResponseBodyBytes)
		}
		return nil, err
	}
	return out, nil
}

func trimTrailingSlash(baseURL string) string {
	return strings.TrimRight(baseURL, "/")
}

func validateBaseURL(baseURL string) error {
	parsed, err := url.Parse(baseURL)
	if err != nil {
		return fmt.Errorf("invalid base URL: %w", err)
	}
	if parsed.Scheme == "" || parsed.Host == "" {
		return errors.New("invalid base URL: scheme and host are required")
	}
	if parsed.RawQuery != "" || parsed.Fragment != "" {
		return errors.New("invalid base URL: query and fragment are not allowed")
	}
	if parsed.Scheme == "https" {
		return nil
	}
	if parsed.Scheme == "http" && isLoopbackHost(parsed.Hostname()) {
		return nil
	}
	return fmt.Errorf("insecure base URL scheme %q: use https (http allowed only for localhost)", parsed.Scheme)
}

func isLoopbackHost(host string) bool {
	if strings.EqualFold(host, "localhost") {
		return true
	}
	ip := net.ParseIP(host)
	return ip != nil && ip.IsLoopback()
}

func cloneHTTPClient(httpClient *http.Client) *http.Client {
	cloned := *httpClient
	return &cloned
}

func composeRedirectPolicy(userPolicy func(req *http.Request, via []*http.Request) error) func(req *http.Request, via []*http.Request) error {
	return func(req *http.Request, via []*http.Request) error {
		if err := rejectUnsafeRedirect(req, via); err != nil {
			return err
		}
		if userPolicy != nil {
			return userPolicy(req, via)
		}
		return nil
	}
}

func rejectUnsafeRedirect(req *http.Request, via []*http.Request) error {
	if len(via) == 0 {
		return nil
	}

	firstReqURL := via[0].URL
	if !sameHostPort(req.URL, firstReqURL) {
		return errors.New("refusing cross-host redirect for authenticated request")
	}
	if !strings.EqualFold(req.URL.Scheme, firstReqURL.Scheme) {
		return errors.New("refusing redirect with scheme change for authenticated request")
	}
	if strings.EqualFold(firstReqURL.Scheme, "https") && !strings.EqualFold(req.URL.Scheme, "https") {
		return errors.New("refusing https downgrade redirect for authenticated request")
	}

	return nil
}

func sameHostPort(a *url.URL, b *url.URL) bool {
	return strings.EqualFold(a.Hostname(), b.Hostname()) && normalizedPort(a) == normalizedPort(b)
}

func normalizedPort(u *url.URL) string {
	if port := u.Port(); port != "" {
		return port
	}
	if strings.EqualFold(u.Scheme, "https") {
		return "443"
	}
	if strings.EqualFold(u.Scheme, "http") {
		return "80"
	}
	return ""
}

type successMarker interface {
	markSuccess()
}

func markSuccessOnEmptyBody[T any](out *T) {
	if sm, ok := any(out).(successMarker); ok {
		sm.markSuccess()
	}
}
