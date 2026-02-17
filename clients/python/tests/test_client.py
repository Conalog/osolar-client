from __future__ import annotations

import httpx
import pytest

from osolar_link_client import ApiError, OsolarLinkClient


def test_search_plants_sends_expected_query_and_header() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == "/v1/search"
        assert request.url.params["q"] == "foo"
        assert request.url.params["field"] == "address"
        assert request.url.params["distance_km"] == "2.0"
        assert request.headers["x-api-key"] == "test-key"
        return httpx.Response(200, json={"success": True, "data": {"features": []}})

    transport = httpx.MockTransport(handler)
    http_client = httpx.Client(transport=transport)
    client = OsolarLinkClient(api_key="test-key", base_url="https://example.com", http_client=http_client)

    response = client.search_plants(q="foo", field="address", distance_km=2.0)
    assert response["success"] is True


def test_raises_api_error_on_non_2xx() -> None:
    def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(403, json={"success": False, "message": "forbidden"})

    transport = httpx.MockTransport(handler)
    http_client = httpx.Client(transport=transport)
    client = OsolarLinkClient(api_key="test-key", base_url="https://example.com", http_client=http_client)

    with pytest.raises(ApiError):
        client.list_linked_plants()
