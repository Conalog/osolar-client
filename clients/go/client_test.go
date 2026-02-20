package osolar

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
	"time"
)

func TestNewClientDefaults(t *testing.T) {
	client := NewClient("test-key", "", nil)

	if got, want := client.baseURL, defaultBaseURL; got != want {
		t.Fatalf("baseURL mismatch: got %s want %s", got, want)
	}
	if client.httpClient == nil {
		t.Fatal("expected non-nil httpClient")
	}
	if got, want := client.httpClient.Timeout, defaultHTTPTimeout; got != want {
		t.Fatalf("timeout mismatch: got %s want %s", got, want)
	}
	if client.initErr != nil {
		t.Fatalf("unexpected initErr: %v", client.initErr)
	}
}

func TestNewClientRejectsInsecureNonLoopbackBaseURL(t *testing.T) {
	client := NewClient("test-key", "http://example.com", &http.Client{Timeout: time.Second})
	_, err := client.ListLinkedPlants(context.Background())
	if err == nil || !strings.Contains(err.Error(), "insecure base URL scheme") {
		t.Fatalf("expected insecure base URL error, got %v", err)
	}
}

func TestNewClientRejectsBaseURLWithQueryOrFragment(t *testing.T) {
	tests := []string{
		"https://example.com?x=1",
		"https://example.com#frag",
		"https://example.com/api?x=1",
		"https://example.com/api#frag",
	}

	for _, baseURL := range tests {
		t.Run(baseURL, func(t *testing.T) {
			client := NewClient("test-key", baseURL, &http.Client{Timeout: time.Second})
			_, err := client.ListLinkedPlants(context.Background())
			if err == nil || !strings.Contains(err.Error(), "query and fragment are not allowed") {
				t.Fatalf("expected base URL query/fragment error, got %v", err)
			}
		})
	}
}

func TestNewClientAllowsLoopbackHTTPBaseURL(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		writeJSON(t, w, map[string]any{"success": true, "data": []any{}})
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	_, err := client.ListLinkedPlants(context.Background())
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
}

func TestNilContextDoesNotPanic(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Context() == nil {
			t.Fatal("expected non-nil request context")
		}
		writeJSON(t, w, map[string]any{"success": true, "data": []any{}})
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	defer func() {
		if r := recover(); r != nil {
			t.Fatalf("expected no panic, got %v", r)
		}
	}()

	if _, err := client.ListLinkedPlants(nil); err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
}

func TestSearchPlantsSendsExpectedQueryAndHeader(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.URL.Path, "/v1/search"; got != want {
			t.Fatalf("path mismatch: got %s want %s", got, want)
		}
		if got := r.URL.Query().Get("q"); got != "foo" {
			t.Fatalf("q mismatch: got %s", got)
		}
		if got := r.URL.Query().Get("field"); got != "address" {
			t.Fatalf("field mismatch: got %s", got)
		}
		if got := r.URL.Query().Get("distance_km"); got == "" {
			t.Fatal("distance_km missing")
		}
		if got := r.Header.Get("x-api-key"); got != "test-key" {
			t.Fatalf("header mismatch: got %s", got)
		}

		writeJSON(t, w, map[string]any{"success": true, "data": map[string]any{"features": []any{}}})
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	distance := 2.0
	resp, err := client.SearchPlants(context.Background(), SearchPlantsParams{
		Q:          "foo",
		Field:      "address",
		DistanceKM: &distance,
	})
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if !resp.Success {
		t.Fatal("expected success")
	}
}

func TestLinkPlantSendsExpectedBodyAndHeader(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.URL.Path, "/v1/links"; got != want {
			t.Fatalf("path mismatch: got %s want %s", got, want)
		}
		if got, want := r.Method, http.MethodPost; got != want {
			t.Fatalf("method mismatch: got %s want %s", got, want)
		}
		if got := r.Header.Get("x-api-key"); got != "test-key" {
			t.Fatalf("header mismatch: got %s", got)
		}
		if got := r.Header.Get("content-type"); got != "application/json" {
			t.Fatalf("content-type mismatch: got %s", got)
		}

		var body PlantLinkRequest
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatalf("failed to decode request body: %v", err)
		}
		if body.PlantUUID != "plant-uuid-1" || body.Remark != "remark" {
			t.Fatalf("unexpected body: %+v", body)
		}
		if body.LinkID == nil || *body.LinkID != "custom-link-id" {
			t.Fatalf("unexpected link_id: %+v", body.LinkID)
		}

		writeJSON(t, w, map[string]any{
			"success": true,
			"data": map[string]any{
				"link_id":    "custom-link-id",
				"created_at": "2026-02-18T00:00:00Z",
			},
		})
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	linkID := "custom-link-id"
	_, err := client.LinkPlant(context.Background(), PlantLinkRequest{
		PlantUUID: "plant-uuid-1",
		LinkID:    &linkID,
		Remark:    "remark",
	})
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
}

func TestMethodsEscapePathParameter(t *testing.T) {
	linkID := "link id/with/slash"
	wantEscapedPath := "/v1/links/" + url.PathEscape(linkID)
	tests := []struct {
		name     string
		wantPath string
		call     func(*Client) error
	}{
		{
			name:     "GetPlantInfo",
			wantPath: wantEscapedPath,
			call: func(c *Client) error {
				_, err := c.GetPlantInfo(context.Background(), linkID)
				return err
			},
		},
		{
			name:     "GetPlantContract",
			wantPath: wantEscapedPath + "/contract",
			call: func(c *Client) error {
				_, err := c.GetPlantContract(context.Background(), linkID)
				return err
			},
		},
		{
			name:     "GetPlantDocuments",
			wantPath: wantEscapedPath + "/documents",
			call: func(c *Client) error {
				_, err := c.GetPlantDocuments(context.Background(), linkID)
				return err
			},
		},
		{
			name:     "GetPlantOverview",
			wantPath: wantEscapedPath + "/overview",
			call: func(c *Client) error {
				_, err := c.GetPlantOverview(context.Background(), linkID)
				return err
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if got, want := r.URL.EscapedPath(), tc.wantPath; got != want {
					t.Fatalf("path mismatch: got %s want %s", got, want)
				}
				writeJSON(t, w, map[string]any{"success": true})
			}))
			defer ts.Close()

			client := NewClient("test-key", ts.URL, ts.Client())
			if err := tc.call(client); err != nil {
				t.Fatalf("unexpected err: %v", err)
			}
		})
	}
}

func TestMonthlyEndpointsUseExpectedQueryKeys(t *testing.T) {
	linkID := "link-1"
	startYear := 2023
	endYear := 2024
	tests := []struct {
		name      string
		wantPath  string
		wantQuery map[string]string
		call      func(*Client) error
	}{
		{
			name:     "GetMonthlyGeneration",
			wantPath: "/v1/links/link-1/generation/monthly",
			wantQuery: map[string]string{
				"start_year": "2023",
				"end_year":   "2024",
			},
			call: func(c *Client) error {
				_, err := c.GetMonthlyGeneration(context.Background(), linkID, MonthlyGenerationParams{
					StartYear: &startYear,
					EndYear:   &endYear,
				})
				return err
			},
		},
		{
			name:     "GetMonthlyBilling",
			wantPath: "/v1/links/link-1/billing/monthly",
			wantQuery: map[string]string{
				"startYear": "2023",
				"endYear":   "2024",
			},
			call: func(c *Client) error {
				_, err := c.GetMonthlyBilling(context.Background(), linkID, MonthlyBillingParams{
					StartYear: &startYear,
					EndYear:   &endYear,
				})
				return err
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if got, want := r.URL.Path, tc.wantPath; got != want {
					t.Fatalf("path mismatch: got %s want %s", got, want)
				}
				for k, v := range tc.wantQuery {
					if got := r.URL.Query().Get(k); got != v {
						t.Fatalf("query mismatch for %s: got %s want %s", k, got, v)
					}
				}
				writeJSON(t, w, map[string]any{"success": true})
			}))
			defer ts.Close()

			client := NewClient("test-key", ts.URL, ts.Client())
			if err := tc.call(client); err != nil {
				t.Fatalf("unexpected err: %v", err)
			}
		})
	}
}

func TestReturnsAPIErrorOnNon2xx(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusForbidden)
		_, _ = io.WriteString(w, `{"success":false}`)
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	_, err := client.ListLinkedPlants(context.Background())
	if err == nil {
		t.Fatal("expected error")
	}
	if _, ok := err.(*APIError); !ok {
		t.Fatalf("expected APIError, got %T", err)
	}
}

func TestRejectsOverlyLargeResponseBody(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, strings.Repeat("a", maxResponseBodyBytes+1))
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	_, err := client.ListLinkedPlants(context.Background())
	if err == nil || !strings.Contains(err.Error(), "response body exceeds") {
		t.Fatalf("expected response size error, got %v", err)
	}
}

func TestAllowsResponseBodyAtExactLimit(t *testing.T) {
	body := fixedSizeSuccessBody(maxResponseBodyBytes)
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, body)
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	resp, err := client.ListLinkedPlants(context.Background())
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if !resp.Success {
		t.Fatal("expected success to be true")
	}
}

func TestReturnsErrorOnMalformedJSON2xx(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, "{")
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	_, err := client.ListLinkedPlants(context.Background())
	if err == nil {
		t.Fatal("expected malformed json error")
	}
}

func TestReturnsRequestErrorFromHTTPClient(t *testing.T) {
	httpClient := &http.Client{
		Timeout:   time.Second,
		Transport: roundTripFunc(func(*http.Request) (*http.Response, error) { return nil, errors.New("network boom") }),
	}
	client := NewClient("test-key", "https://api.example.com", httpClient)

	_, err := client.ListLinkedPlants(context.Background())
	if err == nil || !strings.Contains(err.Error(), "network boom") {
		t.Fatalf("expected network error, got %v", err)
	}
}

func TestRejectsCrossHostRedirect(t *testing.T) {
	target := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		writeJSON(t, w, map[string]any{"success": true, "data": []any{}})
	}))
	defer target.Close()

	source := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, target.URL+"/v1/links", http.StatusFound)
	}))
	defer source.Close()

	client := NewClient("test-key", source.URL, nil)
	_, err := client.ListLinkedPlants(context.Background())
	if err == nil || !strings.Contains(err.Error(), "refusing cross-host redirect") {
		t.Fatalf("expected redirect rejection error, got %v", err)
	}
}

func TestTreatsNoContentAsSuccess(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	}))
	defer ts.Close()

	client := NewClient("test-key", ts.URL, ts.Client())
	resp, err := client.ListLinkedPlants(context.Background())
	if err != nil {
		t.Fatalf("unexpected err for no-content response: %v", err)
	}
	if !resp.Success {
		t.Fatal("expected success=true for no-content response")
	}
}

func TestAPIErrorStringRedactsBody(t *testing.T) {
	apiErr := &APIError{
		StatusCode: http.StatusBadRequest,
		Status:     "400 Bad Request",
		Body:       []byte("secret\nwith-newline"),
	}
	msg := apiErr.Error()
	if strings.Contains(msg, "secret") {
		t.Fatalf("expected error string to redact body, got %s", msg)
	}
	if !strings.Contains(msg, "body 19 bytes") {
		t.Fatalf("expected body length marker, got %s", msg)
	}
}

func fixedSizeSuccessBody(totalBytes int) string {
	const prefix = `{"success":true,"message":"`
	const suffix = `"}`
	minBytes := len(prefix) + len(suffix)
	if totalBytes < minBytes {
		panic("totalBytes too small for fixed-size success body")
	}
	return prefix + strings.Repeat("a", totalBytes-minBytes) + suffix
}

type roundTripFunc func(*http.Request) (*http.Response, error)

func (f roundTripFunc) RoundTrip(req *http.Request) (*http.Response, error) {
	return f(req)
}

func TestRejectUnsafeRedirectRejectsSchemeDowngrade(t *testing.T) {
	err := rejectUnsafeRedirect(
		&http.Request{URL: &url.URL{Scheme: "http", Host: "example.com:8443"}},
		[]*http.Request{{URL: &url.URL{Scheme: "https", Host: "example.com:8443"}}},
	)
	if err == nil || !strings.Contains(err.Error(), "scheme change") {
		t.Fatalf("expected scheme-change rejection, got %v", err)
	}
}

func TestRejectUnsafeRedirectNormalizesDefaultPort(t *testing.T) {
	err := rejectUnsafeRedirect(
		&http.Request{URL: &url.URL{Scheme: "https", Host: "example.com:443"}},
		[]*http.Request{{URL: &url.URL{Scheme: "https", Host: "example.com"}}},
	)
	if err != nil {
		t.Fatalf("expected redirect to pass with normalized default port, got %v", err)
	}
}

func TestNewClientAlwaysWrapsCustomRedirectPolicy(t *testing.T) {
	customPolicyCalled := 0
	customClient := &http.Client{
		Timeout: time.Second,
		CheckRedirect: func(req *http.Request, via []*http.Request) error {
			customPolicyCalled++
			return nil
		},
	}
	client := NewClient("test-key", "https://example.com", customClient)

	err := client.httpClient.CheckRedirect(
		&http.Request{URL: &url.URL{Scheme: "http", Host: "example.com:8443"}},
		[]*http.Request{{URL: &url.URL{Scheme: "https", Host: "example.com:8443"}}},
	)
	if err == nil || !strings.Contains(err.Error(), "scheme change") {
		t.Fatalf("expected internal redirect guard to block downgrade, got %v", err)
	}
	if customPolicyCalled != 0 {
		t.Fatalf("expected custom policy not to run on unsafe redirect, called=%d", customPolicyCalled)
	}

	err = client.httpClient.CheckRedirect(
		&http.Request{URL: &url.URL{Scheme: "https", Host: "example.com"}},
		[]*http.Request{{URL: &url.URL{Scheme: "https", Host: "example.com"}}},
	)
	if err != nil {
		t.Fatalf("expected safe redirect path to allow custom policy, got %v", err)
	}
	if customPolicyCalled != 1 {
		t.Fatalf("expected custom policy to be called once on safe redirect, called=%d", customPolicyCalled)
	}
}

func writeJSON(t *testing.T, w http.ResponseWriter, body any) {
	t.Helper()
	w.Header().Set("content-type", "application/json")
	if err := json.NewEncoder(w).Encode(body); err != nil {
		t.Fatalf("failed to encode json: %v", err)
	}
}
