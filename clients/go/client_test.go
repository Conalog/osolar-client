package osolar

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
)

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

		w.Header().Set("content-type", "application/json")
		_, _ = w.Write([]byte(`{"success":true,"data":{"features":[]}}`))
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

func TestReturnsAPIErrorOnNon2xx(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusForbidden)
		_, _ = w.Write([]byte(`{"success":false}`))
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
