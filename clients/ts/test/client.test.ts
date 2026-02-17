import { describe, expect, it, vi } from "vitest";
import { ApiError, OsolarLinkClient } from "../src/client.js";

describe("OsolarLinkClient", () => {
  it("sends x-api-key header and serializes query parameters", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      const url = String(input);
      expect(url).toContain("/v1/search");
      expect(url).toContain("q=foo");
      expect(url).toContain("field=address");
      expect(url).toContain("distance_km=2");
      expect((init?.headers as Record<string, string>)["x-api-key"]).toBe("test-key");

      return new Response(JSON.stringify({ success: true, data: { features: [] } }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    const response = await client.searchPlants({ q: "foo", field: "address", distanceKm: 2 });
    expect(response.success).toBe(true);
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("throws ApiError on non-2xx", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(JSON.stringify({ success: false, message: "forbidden" }), {
        status: 403,
        statusText: "Forbidden",
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.listLinkedPlants()).rejects.toBeInstanceOf(ApiError);
  });

  it("keeps non-JSON error body in ApiError.responseBody", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response("rate limited", {
        status: 429,
        statusText: "Too Many Requests",
        headers: { "content-type": "text/plain" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.listLinkedPlants()).rejects.toMatchObject({
      name: "ApiError",
      status: 429,
      responseBody: "rate limited",
    });
  });

  it("serializes linkPlant body with JSON content type", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      expect(String(input)).toBe("https://example.com/v1/links");
      expect(init?.method).toBe("POST");
      expect((init?.headers as Record<string, string>)["content-type"]).toBe("application/json");
      expect(init?.body).toBe(
        JSON.stringify({
          plant_uuid: "plant-1",
          link_id: "external-1",
          remark: "RTU link",
        }),
      );

      return new Response(JSON.stringify({ success: true, data: { link_id: "external-1", created_at: "now" } }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await client.linkPlant({
      plant_uuid: "plant-1",
      link_id: "external-1",
      remark: "RTU link",
    });
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("encodes linkId path segment for getPlantOverview", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo) => {
      expect(String(input)).toContain("/v1/links/id%2Fwith%20space/overview");
      return new Response(JSON.stringify({ success: true, data: { link_id: "id/with space", plant_name: "A", billing_summary: [], recent_tasks: [] } }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await client.getPlantOverview("id/with space");
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("encodes linkId path segment for getPlantInfo", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo) => {
      expect(String(input)).toContain("/v1/links/id%2Fwith%20space");
      expect(String(input)).not.toContain("/overview");
      return new Response(JSON.stringify({ success: true, data: { link_id: "id/with space" } }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await client.getPlantInfo("id/with space");
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("calls contract endpoint for getPlantContract", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo) => {
      expect(String(input)).toContain("/v1/links/link-1/contract");
      return new Response(JSON.stringify({ success: true, data: { ppa_type: "PPA", rec_trade_type: "fixed", rec_contracts: [] } }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await client.getPlantContract("link-1");
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("calls documents endpoint for getPlantDocuments", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo) => {
      expect(String(input)).toContain("/v1/links/link-1/documents");
      return new Response(JSON.stringify({ success: true, data: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await client.getPlantDocuments("link-1");
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("serializes getMonthlyGeneration query as snake_case", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo) => {
      const url = String(input);
      expect(url).toContain("/generation/monthly");
      expect(url).toContain("start_year=2023");
      expect(url).toContain("end_year=2024");
      expect(url).not.toContain("startYear");
      expect(url).not.toContain("endYear");
      return new Response(JSON.stringify({ success: true, data: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await client.getMonthlyGeneration("link-1", { startYear: 2023, endYear: 2024 });
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("serializes getMonthlyBilling query as camelCase", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo) => {
      const url = String(input);
      expect(url).toContain("/billing/monthly");
      expect(url).toContain("startYear=2023");
      expect(url).toContain("endYear=2024");
      expect(url).not.toContain("start_year");
      expect(url).not.toContain("end_year");
      return new Response(JSON.stringify({ success: true, data: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await client.getMonthlyBilling("link-1", { startYear: 2023, endYear: 2024 });
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("throws before request when generation startYear is greater than endYear", async () => {
    const fetchMock = vi.fn();
    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.getMonthlyGeneration("link-1", { startYear: 2025, endYear: 2024 })).rejects.toBeInstanceOf(
      RangeError,
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("throws before request when billing startYear is greater than endYear", async () => {
    const fetchMock = vi.fn();
    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.getMonthlyBilling("link-1", { startYear: 2025, endYear: 2024 })).rejects.toBeInstanceOf(
      RangeError,
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("throws when successful response body is not valid JSON", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response("not-json", {
        status: 200,
        headers: { "content-type": "text/plain" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.listLinkedPlants()).rejects.toThrow("Expected JSON response");
  });

  it("throws when successful response body is empty", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response("", {
        status: 200,
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.listLinkedPlants()).rejects.toThrow("Expected JSON response body");
  });

  it("throws when successful response JSON root is null", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response("null", {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.listLinkedPlants()).rejects.toThrow("Expected JSON object response");
  });
});
