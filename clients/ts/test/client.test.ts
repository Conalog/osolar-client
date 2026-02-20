import { describe, expect, it, vi } from "vitest";
import { ApiError, OsolarLinkClient } from "../src/client.js";

describe("OsolarLinkClient", () => {
  it("throws when apiKey is empty", () => {
    expect(
      () =>
        new OsolarLinkClient({
          apiKey: "   ",
        }),
    ).toThrow("apiKey must be a non-empty string");
  });

  it("sends x-api-key header and serializes query parameters", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      const url = String(input);
      expect(url).toContain("/v1/search");
      expect(url).toContain("q=foo");
      expect(url).toContain("field=address");
      expect(url).toContain("distance_km=2");
      expect((init?.headers as Record<string, string>)["x-api-key"]).toBe("test-key");
      expect(init?.redirect).toBe("manual");

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

  it("rejects non-localhost http baseUrl to avoid plaintext x-api-key exposure", () => {
    expect(() => new OsolarLinkClient({ apiKey: "test-key", baseUrl: "http://example.com" })).toThrow(
      "baseUrl must use https (http is only allowed for localhost) to avoid sending x-api-key over plaintext HTTP",
    );
  });

  it("rejects baseUrl with query params or fragment to avoid malformed request URLs", () => {
    expect(() => new OsolarLinkClient({ apiKey: "test-key", baseUrl: "https://example.com?env=prod" })).toThrow(
      "baseUrl must not include query parameters or a fragment",
    );
    expect(() => new OsolarLinkClient({ apiKey: "test-key", baseUrl: "https://example.com#frag" })).toThrow(
      "baseUrl must not include query parameters or a fragment",
    );
  });

  it("does not throw at construction time when global fetch is missing", async () => {
    const originalFetch = globalThis.fetch;
    // @ts-expect-error test-only: simulate runtime without fetch
    delete (globalThis as unknown as { fetch?: unknown }).fetch;

    try {
      const client = new OsolarLinkClient({ apiKey: "test-key", baseUrl: "https://example.com" });
      await expect(client.listLinkedPlants()).rejects.toThrow(
        "fetch is not defined in this runtime. Pass config.fetchFn (for example, undici's fetch) when constructing OsolarLinkClient.",
      );
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("throws when searchPlants q is empty", async () => {
    const fetchMock = vi.fn();
    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.searchPlants({ q: " ", field: "address" })).rejects.toThrow(
      "q must be a non-empty string",
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("throws when searchPlants field is empty", async () => {
    const fetchMock = vi.fn();
    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.searchPlants({ q: "foo", field: " " })).rejects.toThrow(
      "field must be a non-empty string",
    );
    expect(fetchMock).not.toHaveBeenCalled();
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

  it("throws when linkPlant plant_uuid is empty", async () => {
    const fetchMock = vi.fn();
    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.linkPlant({ plant_uuid: " ", remark: "RTU link" })).rejects.toThrow(
      "plant_uuid must be a non-empty string",
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("throws when linkPlant remark is empty", async () => {
    const fetchMock = vi.fn();
    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.linkPlant({ plant_uuid: "plant-1", remark: " " })).rejects.toThrow(
      "remark must be a non-empty string",
    );
    expect(fetchMock).not.toHaveBeenCalled();
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

  it("throws when linkId is empty", async () => {
    const fetchMock = vi.fn();
    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.getPlantInfo("  ")).rejects.toThrow("linkId must be a non-empty string");
    expect(fetchMock).not.toHaveBeenCalled();
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

    const response = await client.getPlantContract("link-1");
    expect(response.data?.rec_contracts).toEqual([]);
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("normalizes legacy rec_fixed_contract object to rec_contracts array", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          success: true,
          data: {
            ppa_type: "PPA",
            rec_trade_type: "fixed",
            rec_fixed_contract: {
              target: "한수원",
            },
          },
        }),
        {
          status: 200,
          headers: { "content-type": "application/json" },
        },
      );
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    const response = await client.getPlantContract("link-1");
    expect(response.data?.rec_contracts).toEqual([{ ess: false, target: "한수원" }]);
  });

  it("normalizes missing contract list to empty array", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          success: true,
          data: {
            ppa_type: "PPA",
            rec_trade_type: "spot",
          },
        }),
        {
          status: 200,
          headers: { "content-type": "application/json" },
        },
      );
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    const response = await client.getPlantContract("link-1");
    expect(response.data?.rec_contracts).toEqual([]);
  });

  it("normalizes legacy rec_fixed_contract array to rec_contracts", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          success: true,
          data: {
            ppa_type: "PPA",
            rec_trade_type: "fixed",
            rec_fixed_contract: [
              { target: "동서발전" },
              { ess: true, target: "한수원" },
            ],
          },
        }),
        {
          status: 200,
          headers: { "content-type": "application/json" },
        },
      );
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    const response = await client.getPlantContract("link-1");
    expect(response.data?.rec_contracts).toEqual([
      { ess: false, target: "동서발전" },
      { ess: true, target: "한수원" },
    ]);
  });

  it("returns null contract data as-is without throwing", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          success: true,
          data: null,
        }),
        {
          status: 200,
          headers: { "content-type": "application/json" },
        },
      );
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    const response = await client.getPlantContract("link-1");
    expect(response.data).toBeNull();
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

  it("preserves null rec_billing_amount values from billing response", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          success: true,
          data: [
            {
              billing_month: "2025-03",
              smp_billing_amount: 100,
              rec_billing_amount: null,
            },
          ],
        }),
        {
          status: 200,
          headers: { "content-type": "application/json" },
        },
      );
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    const response = await client.getMonthlyBilling("link-1");
    expect(response.data?.[0]?.rec_billing_amount).toBeNull();
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

  it("throws when successful response JSON root is an array", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response("[]", {
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

  it("throws when ApiResponse envelope is missing success flag", async () => {
    const fetchMock = vi.fn(async () => {
      return new Response(JSON.stringify({ data: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
    });

    await expect(client.listLinkedPlants()).rejects.toThrow("Expected ApiResponse envelope with boolean success");
  });

  it("supports AbortSignal from options", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      expect(init?.signal).toBeInstanceOf(AbortSignal);
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

    const controller = new AbortController();
    await client.listLinkedPlants({ signal: controller.signal });
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("applies timeout from config", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      expect(init?.signal).toBeInstanceOf(AbortSignal);
      return new Response(JSON.stringify({ success: true, data: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
      timeout: 1000,
    });

    await client.listLinkedPlants();
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it("applies timeout from options overriding config", async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo, init?: RequestInit) => {
      expect(init?.signal).toBeInstanceOf(AbortSignal);
      return new Response(JSON.stringify({ success: true, data: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    });

    const client = new OsolarLinkClient({
      apiKey: "test-key",
      baseUrl: "https://example.com",
      fetchFn: fetchMock as unknown as typeof fetch,
      timeout: 5000,
    });

    await client.listLinkedPlants({ timeout: 1000 });
    expect(fetchMock).toHaveBeenCalledOnce();
  });
});
