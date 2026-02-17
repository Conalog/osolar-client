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
});
