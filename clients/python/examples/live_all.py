from __future__ import annotations

import json
import os
import re
import sys
from typing import Any, Callable

from osolar_client import ApiError, OsolarLinkClient


def main() -> int:
    api_key = os.environ.get("OSOLAR_API_KEY")
    if not api_key:
        print("OSOLAR_API_KEY is required", file=sys.stderr)
        return 1

    client = OsolarLinkClient(api_key=api_key)
    results: dict[str, dict[str, Any]] = {}

    def ok(route: str, **detail: Any) -> None:
        results[route] = {"ok": True, **detail}

    def fail(route: str, err: Exception) -> None:
        if isinstance(err, ApiError):
            results[route] = {
                "ok": False,
                "status": err.status_code,
                "error": str(err),
                "error_body_redacted": True,
            }
        else:
            results[route] = {"ok": False, "error": str(err)}

    link_id: str | None = None
    search_keyword = "서울"
    plant_uuid_for_link: str | None = None

    try:
        linked = client.list_linked_plants()
        linked_data = linked.get("data") if isinstance(linked, dict) else None
        linked_count = len(linked_data) if isinstance(linked_data, list) else 0
        if (
            isinstance(linked_data, list)
            and linked_data
            and isinstance(linked_data[0], dict)
        ):
            first_link = linked_data[0]
            link_id = first_link.get("link_id")
            if isinstance(link_id, str):
                if re.fullmatch(
                    r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[1-5][0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}",
                    link_id,
                ):
                    plant_uuid_for_link = link_id
            else:
                link_id = None
            address = first_link.get("plant_address")
            if isinstance(address, str) and address:
                search_keyword = address[:12]
        ok("GET /v1/links", linkedPlantCount=linked_count, sampleLinkId=link_id)
    except Exception as err:  # noqa: BLE001
        fail("GET /v1/links", err)

    try:
        search = client.search_plants(q=search_keyword, field="address", distance_km=2)
        data = search.get("data") if isinstance(search, dict) else None
        features = data.get("features") if isinstance(data, dict) else None
        feature_count = len(features) if isinstance(features, list) else 0
        if (
            not plant_uuid_for_link
            and isinstance(features, list)
            and features
            and isinstance(features[0], dict)
        ):
            props = features[0].get("properties")
            if isinstance(props, dict) and isinstance(props.get("plant_uuid"), str):
                plant_uuid_for_link = props["plant_uuid"]
        ok("GET /v1/search", featureCount=feature_count, query=search_keyword)
    except Exception as err:  # noqa: BLE001
        fail("GET /v1/search", err)

    try:
        client.link_plant(
            {
                "plant_uuid": plant_uuid_for_link or "not-a-valid-uuid",
                "remark": "sdk live-all route smoke test",
            }
        )
        ok("POST /v1/links", note="unexpectedly succeeded")
    except ApiError as err:
        results["POST /v1/links"] = {
            "ok": err.status_code >= 400,
            "status": err.status_code,
            "note": "non-2xx is acceptable for live route smoke",
        }
    except Exception as err:  # noqa: BLE001
        fail("POST /v1/links", err)

    guarded_routes: list[tuple[str, Callable[[], Any]]] = [
        ("GET /v1/links/{link_id}", lambda: client.get_plant_info(link_id or "")),
        (
            "GET /v1/links/{link_id}/contract",
            lambda: client.get_plant_contract(link_id or ""),
        ),
        (
            "GET /v1/links/{link_id}/documents",
            lambda: client.get_plant_documents(link_id or ""),
        ),
        (
            "GET /v1/links/{link_id}/overview",
            lambda: client.get_plant_overview(link_id or ""),
        ),
        (
            "GET /v1/links/{link_id}/generation/monthly",
            lambda: client.get_monthly_generation(link_id or ""),
        ),
        (
            "GET /v1/links/{link_id}/billing/monthly",
            lambda: client.get_monthly_billing(link_id or ""),
        ),
    ]

    for route, fn in guarded_routes:
        if not link_id:
            results[route] = {
                "ok": False,
                "skipped": True,
                "reason": "no linked plant available",
            }
            continue
        try:
            response = fn()
            data = response.get("data") if isinstance(response, dict) else None
            payload_size = (
                len(data) if isinstance(data, list) else (1 if data is not None else 0)
            )
            ok(route, payloadSize=payload_size)
        except Exception as err:  # noqa: BLE001
            fail(route, err)

    print(json.dumps(results, ensure_ascii=False, indent=2))
    hard_fail = any(
        (not v.get("ok", False)) and (not v.get("skipped", False))
        for v in results.values()
    )
    client.close()
    return 1 if hard_fail else 0


if __name__ == "__main__":
    raise SystemExit(main())
