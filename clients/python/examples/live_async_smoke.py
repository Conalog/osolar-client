from __future__ import annotations

import asyncio
import json
import os
import sys

from osolar_client import ApiError, AsyncOsolarLinkClient


async def amain() -> int:
    api_key = os.environ.get("OSOLAR_API_KEY")
    if not api_key:
        print("OSOLAR_API_KEY is required", file=sys.stderr)
        return 1

    async with AsyncOsolarLinkClient(api_key=api_key) as client:
        try:
            response = await client.list_linked_plants()
            data = response.get("data")
            count = len(data) if isinstance(data, list) else 0
            print(json.dumps({"success": response.get("success"), "linkedPlantCount": count}, ensure_ascii=False, indent=2))
            return 0
        except ApiError as err:
            print(f"Live async smoke test failed: {err}", file=sys.stderr)
            return 1


def main() -> int:
    return asyncio.run(amain())


if __name__ == "__main__":
    raise SystemExit(main())
