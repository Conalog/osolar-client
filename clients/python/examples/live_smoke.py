from __future__ import annotations

import json
import os
import sys

from osolar_link_client import ApiError, OsolarLinkClient


def main() -> int:
    api_key = os.environ.get("OSOLAR_API_KEY")
    if not api_key:
        print("OSOLAR_API_KEY is required", file=sys.stderr)
        return 1

    client = OsolarLinkClient(api_key=api_key)
    try:
        response = client.list_linked_plants()
        data = response.get("data")
        count = len(data) if isinstance(data, list) else 0
        print(json.dumps({"success": response.get("success"), "linkedPlantCount": count}, ensure_ascii=False, indent=2))
        return 0
    except ApiError as err:
        print(f"Live smoke test failed: {err}", file=sys.stderr)
        return 1
    finally:
        client.close()


if __name__ == "__main__":
    raise SystemExit(main())
