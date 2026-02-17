import { OsolarLinkClient } from "../dist/index.js";

const apiKey = process.env.OSOLAR_API_KEY;
if (!apiKey) {
  console.error("OSOLAR_API_KEY is required");
  process.exit(1);
}

const client = new OsolarLinkClient({ apiKey });

try {
  const response = await client.listLinkedPlants();
  const count = Array.isArray(response.data) ? response.data.length : 0;
  console.log(
    JSON.stringify(
      {
        success: response.success,
        linkedPlantCount: count,
      },
      null,
      2,
    ),
  );
} catch (error) {
  console.error("Live smoke test failed:", error);
  process.exit(1);
}
