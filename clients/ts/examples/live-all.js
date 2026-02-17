import { ApiError, OsolarLinkClient } from "../dist/index.js";

const apiKey = process.env.OSOLAR_API_KEY;
if (!apiKey) {
  console.error("OSOLAR_API_KEY is required");
  process.exit(1);
}

const client = new OsolarLinkClient({ apiKey });
const results = {};

function ok(route, detail) {
  results[route] = { ok: true, ...detail };
}

function fail(route, error) {
  if (error instanceof ApiError) {
    results[route] = { ok: false, status: error.status, error: String(error.responseBody) };
    return;
  }
  results[route] = { ok: false, error: String(error) };
}

let linkId = null;
let searchKeyword = "서울";
let plantUuidForLink = null;

try {
  const linked = await client.listLinkedPlants();
  const linkedCount = Array.isArray(linked.data) ? linked.data.length : 0;
  if (linkedCount > 0) {
    linkId = linked.data[0].link_id;
    if (/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(linkId)) {
      plantUuidForLink = linkId;
    }
    const address = linked.data[0].plant_address;
    if (address && address.length > 0) {
      searchKeyword = address.slice(0, 12);
    }
  }
  ok("GET /v1/links", { linkedCount, sampleLinkId: linkId });
} catch (error) {
  fail("GET /v1/links", error);
}

try {
  const search = await client.searchPlants({ q: searchKeyword, field: "address", distanceKm: 2 });
  const featureCount = search.data?.features?.length ?? 0;
  if (!plantUuidForLink && featureCount > 0) {
    plantUuidForLink = search.data.features[0].properties.plant_uuid;
  }
  ok("GET /v1/search", { featureCount, query: searchKeyword });
} catch (error) {
  fail("GET /v1/search", error);
}

try {
  await client.linkPlant({
    plant_uuid: plantUuidForLink ?? "not-a-valid-uuid",
    remark: "sdk live-all route smoke test",
  });
  ok("POST /v1/links", { note: "unexpectedly succeeded" });
} catch (error) {
  if (error instanceof ApiError) {
    results["POST /v1/links"] = {
      ok: error.status >= 400,
      status: error.status,
      note: "non-2xx is acceptable for live route smoke",
    };
  } else {
    fail("POST /v1/links", error);
  }
}

const guardedRoutes = [
  ["GET /v1/links/{link_id}", () => client.getPlantInfo(linkId)],
  ["GET /v1/links/{link_id}/contract", () => client.getPlantContract(linkId)],
  ["GET /v1/links/{link_id}/documents", () => client.getPlantDocuments(linkId)],
  ["GET /v1/links/{link_id}/overview", () => client.getPlantOverview(linkId)],
  ["GET /v1/links/{link_id}/generation/monthly", () => client.getMonthlyGeneration(linkId)],
  ["GET /v1/links/{link_id}/billing/monthly", () => client.getMonthlyBilling(linkId)],
];

for (const [route, fn] of guardedRoutes) {
  if (!linkId) {
    results[route] = { ok: false, skipped: true, reason: "no linked plant available" };
    continue;
  }

  try {
    const response = await fn();
    const size = Array.isArray(response.data) ? response.data.length : response.data ? 1 : 0;
    ok(route, { payloadSize: size });
  } catch (error) {
    fail(route, error);
  }
}

console.log(JSON.stringify(results, null, 2));
const hasHardFailure = Object.values(results).some((r) => !r.ok && !r.skipped);
process.exit(hasHardFailure ? 1 : 0);
