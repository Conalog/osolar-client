from __future__ import annotations

import json
from typing import Any, Callable, cast

import httpx
import pytest

from osolar_client import ApiError, OsolarLinkClient, PlantLinkRequest


class RecordingHttpClient:
    def __init__(self, response: httpx.Response | None = None) -> None:
        self._response = response or httpx.Response(200, json={"success": True, "data": {}})
        self.calls: list[tuple[str, str, dict[str, Any]]] = []
        self.is_closed = False

    def request(self, method: str, url: str, **kwargs: Any) -> httpx.Response:
        self.calls.append((method, url, kwargs))
        return self._response

    def close(self) -> None:
        self.is_closed = True


class RecordingRequestClient(OsolarLinkClient):
    def __init__(self) -> None:
        dummy_http_client = cast(httpx.Client, RecordingHttpClient())
        super().__init__(api_key="test-key", base_url="https://example.com", http_client=dummy_http_client)
        self.calls: list[tuple[str, str, dict[str, Any] | None, dict[str, Any] | None]] = []

    def _request(
        self,
        method: str,
        path: str,
        params: dict[str, Any] | None = None,
        json: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        self.calls.append((method, path, params, json))
        return {"success": True, "data": {}}


def make_client(handler: Callable[[httpx.Request], httpx.Response]) -> tuple[OsolarLinkClient, httpx.Client]:
    transport = httpx.MockTransport(handler)
    http_client = httpx.Client(transport=transport)
    client = OsolarLinkClient(api_key="test-key", base_url="https://example.com", http_client=http_client)
    return client, http_client


def test_search_plants_sends_expected_query_and_header() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.method == "GET"
        assert request.url.path == "/v1/search"
        assert request.url.params["q"] == "foo"
        assert request.url.params["field"] == "address"
        assert request.url.params["distance_km"] == "2.0"
        assert request.headers["x-api-key"] == "test-key"
        return httpx.Response(200, json={"success": True, "data": {"features": []}})

    client, http_client = make_client(handler)
    try:
        response = client.search_plants(q="foo", field="address", distance_km=2.0)
        assert response["success"] is True
    finally:
        http_client.close()


def test_search_plants_omits_none_distance_km() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == "/v1/search"
        assert "distance_km" not in request.url.params
        return httpx.Response(200, json={"success": True, "data": {"features": []}})

    client, http_client = make_client(handler)
    try:
        response = client.search_plants(q="foo", field="address")
        assert response["success"] is True
    finally:
        http_client.close()


def test_search_plants_keeps_zero_distance_km() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == "/v1/search"
        assert request.url.params["distance_km"] == "0.0"
        return httpx.Response(200, json={"success": True, "data": {"features": []}})

    client, http_client = make_client(handler)
    try:
        response = client.search_plants(q="foo", field="address", distance_km=0.0)
        assert response["success"] is True
    finally:
        http_client.close()


def test_search_plants_rejects_unsupported_field() -> None:
    client = OsolarLinkClient(api_key="test-key")
    unsafe_client = cast(Any, client)
    try:
        with pytest.raises(ValueError, match=r"^`field` must be one of: business_number, address\.$"):
            unsafe_client.search_plants(q="foo", field="name")
    finally:
        client.close()


def test_raises_api_error_on_non_2xx() -> None:
    def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(403, json={"success": False, "message": "forbidden"})

    client, http_client = make_client(handler)
    try:
        with pytest.raises(ApiError) as exc_info:
            client.list_linked_plants()
        error = exc_info.value
        assert error.status_code == 403
        assert error.response_body == {"success": False, "message": "forbidden"}
        assert str(error) == "OSOLAR API error 403: Forbidden"
    finally:
        http_client.close()


def test_link_plant_supports_keyword_arguments() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == "/v1/links"
        assert request.method == "POST"
        assert json.loads(request.content.decode("utf-8")) == {
            "plant_uuid": "plant-1",
            "remark": "memo",
            "link_id": "link-1",
        }
        return httpx.Response(200, json={"success": True, "data": {"link_id": "link-1", "created_at": "now"}})

    client, http_client = make_client(handler)
    try:
        response = client.link_plant(plant_uuid="plant-1", remark="memo", link_id="link-1")
        assert response["success"] is True
    finally:
        http_client.close()


def test_link_plant_rejects_mixed_payload_styles() -> None:
    client = OsolarLinkClient(api_key="test-key")
    unsafe_client = cast(Any, client)
    with pytest.raises(ValueError, match=r"^Use either `body` or keyword arguments, not both\.$"):
        unsafe_client.link_plant(
            {"plant_uuid": "plant-1", "remark": "memo"},
            plant_uuid="plant-2",
            remark="memo",
        )
    client.close()


def test_link_plant_accepts_typed_dict_body() -> None:
    body: PlantLinkRequest = {"plant_uuid": "plant-1", "remark": "memo"}

    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == "/v1/links"
        assert request.method == "POST"
        assert json.loads(request.content.decode("utf-8")) == body
        return httpx.Response(200, json={"success": True, "data": {"link_id": "plant-1", "created_at": "now"}})

    client, http_client = make_client(handler)
    try:
        response = client.link_plant(body)
        assert response["success"] is True
    finally:
        http_client.close()


@pytest.mark.parametrize(
    "body",
    [
        {"plant_uuid": "plant-1"},
        {"remark": "memo"},
        {},
        1,
    ],
)
def test_link_plant_body_requires_plant_uuid_and_remark(body: Any) -> None:
    client = OsolarLinkClient(api_key="test-key")
    unsafe_client = cast(Any, client)
    try:
        with pytest.raises(ValueError, match=r"^`plant_uuid` and `remark` are required in `body`\.$"):
            unsafe_client.link_plant(body)
    finally:
        client.close()


@pytest.mark.parametrize(
    "kwargs",
    [
        {"plant_uuid": "plant-2"},
        {"remark": "memo"},
        {"link_id": "link-2"},
    ],
)
def test_link_plant_rejects_any_keyword_when_body_is_given(kwargs: dict[str, str]) -> None:
    client = OsolarLinkClient(api_key="test-key")
    unsafe_client = cast(Any, client)
    try:
        with pytest.raises(ValueError, match=r"^Use either `body` or keyword arguments, not both\.$"):
            unsafe_client.link_plant({"plant_uuid": "plant-1", "remark": "memo"}, **kwargs)
    finally:
        client.close()


@pytest.mark.parametrize(
    "kwargs",
    [
        {"plant_uuid": "plant-1"},
        {"remark": "memo"},
    ],
)
def test_link_plant_requires_plant_uuid_and_remark_when_body_missing(kwargs: dict[str, str]) -> None:
    client = OsolarLinkClient(api_key="test-key")
    unsafe_client = cast(Any, client)
    try:
        with pytest.raises(
            ValueError,
            match=r"^`plant_uuid` and `remark` are required when `body` is not provided\.$",
        ):
            unsafe_client.link_plant(**kwargs)
    finally:
        client.close()


def test_get_plant_info_url_encodes_link_id() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.raw_path == b"/v1/links/abc%2Fdef%20ghi"
        return httpx.Response(200, json={"success": True, "data": {"link_id": "abc/def ghi"}})

    client, http_client = make_client(handler)
    try:
        response = client.get_plant_info("abc/def ghi")
        assert response["success"] is True
    finally:
        http_client.close()


def test_monthly_methods_use_endpoint_specific_query_keys() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        if request.url.path.endswith("/generation/monthly"):
            assert request.method == "GET"
            assert request.url.path == "/v1/links/id/generation/monthly"
            assert request.url.params["start_year"] == "2020"
            assert request.url.params["end_year"] == "2021"
            assert "startYear" not in request.url.params
            assert "endYear" not in request.url.params
            return httpx.Response(200, json={"success": True, "data": []})

        if request.url.path.endswith("/billing/monthly"):
            assert request.method == "GET"
            assert request.url.path == "/v1/links/id/billing/monthly"
            assert request.url.params["startYear"] == "2020"
            assert request.url.params["endYear"] == "2021"
            assert "start_year" not in request.url.params
            assert "end_year" not in request.url.params
            return httpx.Response(200, json={"success": True, "data": []})

        raise AssertionError(f"unexpected path: {request.url.path}")

    client, http_client = make_client(handler)
    try:
        generation = client.get_monthly_generation("id", start_year=2020, end_year=2021)
        billing = client.get_monthly_billing("id", start_year=2020, end_year=2021)
        assert generation["success"] is True
        assert billing["success"] is True
    finally:
        http_client.close()


@pytest.mark.parametrize(
    ("method_name", "expected_suffix"),
    [
        ("get_plant_contract", "/contract"),
        ("get_plant_documents", "/documents"),
        ("get_plant_overview", "/overview"),
    ],
)
def test_link_detail_methods_build_expected_paths(method_name: str, expected_suffix: str) -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.url.path == f"/v1/links/abc-123{expected_suffix}"
        return httpx.Response(200, json={"success": True, "data": {}})

    client, http_client = make_client(handler)
    try:
        method = getattr(client, method_name)
        response = method("abc-123")
        assert response["success"] is True
    finally:
        http_client.close()


@pytest.mark.parametrize(
    ("method_name", "expected_raw_path"),
    [
        ("get_plant_contract", b"/v1/links/a%2Fb/contract"),
        ("get_plant_documents", b"/v1/links/a%2Fb/documents"),
        ("get_plant_overview", b"/v1/links/a%2Fb/overview"),
    ],
)
def test_link_detail_methods_encode_slashes_in_link_id(method_name: str, expected_raw_path: bytes) -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        assert request.method == "GET"
        assert request.url.raw_path == expected_raw_path
        return httpx.Response(200, json={"success": True, "data": {}})

    client, http_client = make_client(handler)
    try:
        method = getattr(client, method_name)
        method("a/b")
    finally:
        http_client.close()


def test_request_handles_empty_success_response() -> None:
    def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(204)

    client, http_client = make_client(handler)
    try:
        response = client.list_linked_plants()
        assert response == {"success": True, "data": None}
    finally:
        http_client.close()


def test_request_raises_api_error_on_invalid_json_success_response() -> None:
    def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(200, text="<html>ok</html>")

    client, http_client = make_client(handler)
    try:
        with pytest.raises(ApiError) as exc_info:
            client.list_linked_plants()
        error = exc_info.value
        assert error.status_code == 200
        assert error.response_body == "<html>ok</html>"
        assert str(error) == "OSOLAR API error 200: Invalid JSON response"
    finally:
        http_client.close()


def test_request_raises_api_error_on_non_object_json_success_response() -> None:
    def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(200, json=[1, 2, 3])

    client, http_client = make_client(handler)
    try:
        with pytest.raises(ApiError) as exc_info:
            client.list_linked_plants()
        error = exc_info.value
        assert error.status_code == 200
        assert error.response_body == [1, 2, 3]
        assert str(error) == "OSOLAR API error 200: Unexpected JSON response type"
    finally:
        http_client.close()


def test_request_treats_300_as_error() -> None:
    def handler(_: httpx.Request) -> httpx.Response:
        return httpx.Response(300, json={"success": False, "message": "redirect"})

    client, http_client = make_client(handler)
    try:
        with pytest.raises(ApiError) as exc_info:
            client.list_linked_plants()
        error = exc_info.value
        assert error.status_code == 300
        assert error.response_body == {"success": False, "message": "redirect"}
        assert str(error) == "OSOLAR API error 300: Multiple Choices"
    finally:
        http_client.close()


def test_request_sends_expected_header_key_exactly() -> None:
    recording_http_client = RecordingHttpClient()
    client = OsolarLinkClient(api_key="test-key", http_client=cast(httpx.Client, recording_http_client))
    try:
        client.list_linked_plants()
        method, _, kwargs = recording_http_client.calls[0]
        assert method == "GET"
        assert kwargs["headers"] == {"x-api-key": "test-key"}
    finally:
        client.close()


def test_init_uses_default_base_url() -> None:
    recording_http_client = RecordingHttpClient()
    client = OsolarLinkClient(api_key="test-key", http_client=cast(httpx.Client, recording_http_client))
    try:
        client.list_linked_plants()
        _, url, _ = recording_http_client.calls[0]
        assert url == "https://openapi.osolar.io/v1/links"
    finally:
        client.close()


@pytest.mark.parametrize("api_key", ["", " ", "\t"])
def test_init_rejects_empty_api_key(api_key: str) -> None:
    with pytest.raises(ValueError, match=r"^`api_key` must be a non-empty string\.$"):
        OsolarLinkClient(api_key=api_key)


def test_init_trims_trailing_slash_from_base_url() -> None:
    recording_http_client = RecordingHttpClient()
    client = OsolarLinkClient(
        api_key="test-key",
        base_url="https://example.com/",
        http_client=cast(httpx.Client, recording_http_client),
    )
    try:
        client.list_linked_plants()
        _, url, _ = recording_http_client.calls[0]
        assert url == "https://example.com/v1/links"
    finally:
        client.close()


def test_init_does_not_strip_non_slash_suffix_from_base_url() -> None:
    recording_http_client = RecordingHttpClient()
    client = OsolarLinkClient(
        api_key="test-key",
        base_url="https://example.com/X",
        http_client=cast(httpx.Client, recording_http_client),
    )
    try:
        client.list_linked_plants()
        _, url, _ = recording_http_client.calls[0]
        assert url == "https://example.com/X/v1/links"
    finally:
        client.close()


def test_init_uses_default_timeout_for_owned_http_client() -> None:
    client = OsolarLinkClient(api_key="test-key")
    try:
        timeout = client._http_client.timeout
        assert timeout.connect == 30.0
        assert timeout.read == 30.0
        assert timeout.write == 30.0
        assert timeout.pool == 30.0
    finally:
        client.close()


def test_list_linked_plants_passes_expected_request_signature() -> None:
    client = RecordingRequestClient()
    result = client.list_linked_plants()
    assert result["success"] is True
    assert client.calls == [("GET", "/v1/links", None, None)]


def test_search_plants_passes_expected_request_signature() -> None:
    client = RecordingRequestClient()
    result = client.search_plants(q="foo", field="address", distance_km=2.0)
    assert result["success"] is True
    assert client.calls == [("GET", "/v1/search", {"q": "foo", "field": "address", "distance_km": 2.0}, None)]


def test_link_plant_passes_expected_request_signature() -> None:
    client = RecordingRequestClient()
    result = client.link_plant(plant_uuid="plant-1", remark="memo", link_id="link-1")
    assert result["success"] is True
    assert client.calls == [
        (
            "POST",
            "/v1/links",
            None,
            {"plant_uuid": "plant-1", "remark": "memo", "link_id": "link-1"},
        )
    ]


def test_get_plant_info_passes_expected_request_signature() -> None:
    client = RecordingRequestClient()
    result = client.get_plant_info("abc/def")
    assert result["success"] is True
    assert client.calls == [("GET", "/v1/links/abc%2Fdef", None, None)]


@pytest.mark.parametrize(
    ("method_name", "expected_path"),
    [
        ("get_plant_contract", "/v1/links/abc%2Fdef/contract"),
        ("get_plant_documents", "/v1/links/abc%2Fdef/documents"),
        ("get_plant_overview", "/v1/links/abc%2Fdef/overview"),
    ],
)
def test_link_detail_methods_pass_expected_request_signature(method_name: str, expected_path: str) -> None:
    client = RecordingRequestClient()
    method = getattr(client, method_name)
    result = method("abc/def")
    assert result["success"] is True
    assert client.calls == [("GET", expected_path, None, None)]


def test_get_monthly_generation_passes_expected_request_signature() -> None:
    client = RecordingRequestClient()
    result = client.get_monthly_generation("a/b", start_year=2020, end_year=2021)
    assert result["success"] is True
    assert client.calls == [
        (
            "GET",
            "/v1/links/a%2Fb/generation/monthly",
            {"start_year": 2020, "end_year": 2021},
            None,
        )
    ]


def test_get_monthly_billing_passes_expected_request_signature() -> None:
    client = RecordingRequestClient()
    result = client.get_monthly_billing("a/b", start_year=2020, end_year=2021)
    assert result["success"] is True
    assert client.calls == [
        (
            "GET",
            "/v1/links/a%2Fb/billing/monthly",
            {"startYear": 2020, "endYear": 2021},
            None,
        )
    ]


def test_context_manager_closes_owned_http_client() -> None:
    with OsolarLinkClient(api_key="test-key") as client:
        assert client._http_client.is_closed is False
        internal = client._http_client

    assert internal.is_closed is True


def test_close_does_not_close_injected_http_client() -> None:
    transport = httpx.MockTransport(lambda _: httpx.Response(200, json={"success": True}))
    shared_http_client = httpx.Client(transport=transport)
    client = OsolarLinkClient(
        api_key="test-key",
        base_url="https://example.com",
        http_client=shared_http_client,
    )

    try:
        client.close()
        assert shared_http_client.is_closed is False
    finally:
        shared_http_client.close()
