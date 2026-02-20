from __future__ import annotations

from types import TracebackType
from typing import Any, overload
from urllib.parse import quote, urlparse

import httpx

from .client import SearchField, _ALLOWED_SEARCH_FIELDS
from .exceptions import ApiError
from .models import (
    LinkPlantApiResponse,
    ListLinkedPlantsApiResponse,
    MonthlyBillingApiResponse,
    MonthlyGenerationApiResponse,
    PlantContractApiResponse,
    PlantDocumentsApiResponse,
    PlantInfoApiResponse,
    PlantLinkRequest,
    PlantOverviewApiResponse,
    SearchPlantsApiResponse,
)


class AsyncOsolarLinkClient:
    def __init__(
        self,
        api_key: str,
        base_url: str = "https://openapi.osolar.io",
        timeout: float = 30.0,
        http_client: httpx.AsyncClient | None = None,
    ):
        """Create an asynchronous client for the OSOLAR-LINK Open API."""
        if not isinstance(api_key, str) or not api_key.strip():
            raise ValueError("`api_key` must be a non-empty string.")
        self._api_key = api_key
        self._base_url = base_url.rstrip("/")
        parsed = urlparse(self._base_url)
        if parsed.scheme not in ("http", "https"):
            raise ValueError("`base_url` must start with http:// or https://.")
        host = parsed.hostname
        if host is None:
            raise ValueError("`base_url` must include a hostname.")
        if parsed.scheme == "http" and host not in ("127.0.0.1", "localhost", "::1"):
            raise ValueError(
                "`base_url` must use https:// (http:// is allowed only for localhost)."
            )
        self._owns_client = http_client is None
        self._http_client = http_client or httpx.AsyncClient(timeout=timeout)

    async def aclose(self) -> None:
        """Close the underlying async http client if this client created it."""
        if self._owns_client:
            await self._http_client.aclose()

    async def __aenter__(self) -> "AsyncOsolarLinkClient":
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        await self.aclose()

    async def search_plants(
        self,
        q: str,
        field: SearchField,
        distance_km: float | None = None,
    ) -> SearchPlantsApiResponse:
        """Search plants by a text query and constrained search field."""
        if field not in _ALLOWED_SEARCH_FIELDS:
            raise ValueError("`field` must be one of: business_number, address.")
        query = {"q": q, "field": field, "distance_km": distance_km}
        return await self._request("GET", "/v1/search", params=query)

    @overload
    async def link_plant(self, body: PlantLinkRequest) -> LinkPlantApiResponse: ...

    @overload
    async def link_plant(
        self,
        body: None = None,
        *,
        plant_uuid: str,
        remark: str,
        link_id: str | None = None,
    ) -> LinkPlantApiResponse: ...

    async def link_plant(
        self,
        body: PlantLinkRequest | None = None,
        *,
        plant_uuid: str | None = None,
        remark: str | None = None,
        link_id: str | None = None,
    ) -> LinkPlantApiResponse:
        """Create a plant link request."""
        if body is not None:
            if plant_uuid is not None or remark is not None or link_id is not None:
                raise ValueError("Use either `body` or keyword arguments, not both.")
            if (
                not isinstance(body, dict)
                or "plant_uuid" not in body
                or "remark" not in body
            ):
                raise ValueError("`plant_uuid` and `remark` are required in `body`.")
            payload = dict(body)
        else:
            if plant_uuid is None or remark is None:
                raise ValueError(
                    "`plant_uuid` and `remark` are required when `body` is not provided."
                )
            payload = {"plant_uuid": plant_uuid, "remark": remark}
            if link_id is not None:
                payload["link_id"] = link_id

        return await self._request("POST", "/v1/links", json=payload)

    async def list_linked_plants(self) -> ListLinkedPlantsApiResponse:
        """List plants linked to the current API key."""
        return await self._request("GET", "/v1/links")

    async def get_plant_info(self, link_id: str) -> PlantInfoApiResponse:
        """Fetch the base information for a linked plant."""
        safe_link_id = self._quote_path_segment(link_id)
        return await self._request("GET", f"/v1/links/{safe_link_id}")

    async def get_plant_contract(self, link_id: str) -> PlantContractApiResponse:
        """Fetch contract information for a linked plant."""
        safe_link_id = self._quote_path_segment(link_id)
        response: Any = await self._request("GET", f"/v1/links/{safe_link_id}/contract")
        return self._normalize_plant_contract_response(response)

    async def get_plant_documents(self, link_id: str) -> PlantDocumentsApiResponse:
        """Fetch available documents for a linked plant."""
        safe_link_id = self._quote_path_segment(link_id)
        return await self._request("GET", f"/v1/links/{safe_link_id}/documents")

    async def get_plant_overview(self, link_id: str) -> PlantOverviewApiResponse:
        """Fetch overview data for a linked plant."""
        safe_link_id = self._quote_path_segment(link_id)
        return await self._request("GET", f"/v1/links/{safe_link_id}/overview")

    async def get_monthly_generation(
        self,
        link_id: str,
        start_year: int | None = None,
        end_year: int | None = None,
    ) -> MonthlyGenerationApiResponse:
        """Fetch monthly generation data for a linked plant."""
        safe_link_id = self._quote_path_segment(link_id)
        query = {"start_year": start_year, "end_year": end_year}
        return await self._request(
            "GET",
            f"/v1/links/{safe_link_id}/generation/monthly",
            params=query,
        )

    async def get_monthly_billing(
        self,
        link_id: str,
        start_year: int | None = None,
        end_year: int | None = None,
    ) -> MonthlyBillingApiResponse:
        """Fetch monthly billing data for a linked plant."""
        safe_link_id = self._quote_path_segment(link_id)
        # OpenAPI spec uses camelCase for this endpoint.
        query = {"startYear": start_year, "endYear": end_year}
        return await self._request(
            "GET",
            f"/v1/links/{safe_link_id}/billing/monthly",
            params=query,
        )

    async def _request(
        self,
        method: str,
        path: str,
        params: dict[str, Any] | None = None,
        json: dict[str, Any] | None = None,
    ) -> Any:
        filtered_params = {k: v for k, v in (params or {}).items() if v is not None}
        response = await self._http_client.request(
            method,
            f"{self._base_url}{path}",
            params=filtered_params,
            json=json,
            headers={"x-api-key": self._api_key},
        )

        if response.status_code < 200 or response.status_code >= 300:
            body: Any
            try:
                body = response.json()
            except ValueError:
                body = response.text
            raise ApiError(response.status_code, response.reason_phrase, body)

        if not response.content:
            return {"success": True, "data": None}

        try:
            body = response.json()
        except ValueError as exc:
            raise ApiError(
                response.status_code, "Invalid JSON response", response.text
            ) from exc

        if not isinstance(body, dict):
            raise ApiError(response.status_code, "Unexpected JSON response type", body)

        return body

    @staticmethod
    def _quote_path_segment(value: str) -> str:
        # urllib.parse.quote defaults safe to "/", which is appropriate for full
        # paths but not for path segments (we need to encode "/").
        #
        # Use quote(...) + replace to avoid equivalent-string mutants in mutmut
        # like safe="" -> safe="XXXX" (which cannot be killed by tests).
        return quote(value).replace("/", "%2F")

    @staticmethod
    def _normalize_plant_contract_response(response: Any) -> Any:
        if not isinstance(response, dict):
            return response
        data = response.get("data")
        if not isinstance(data, dict):
            return response

        def with_default_ess(item: Any) -> Any:
            if not isinstance(item, dict):
                return item
            out = dict(item)
            if "ess" not in out:
                out["ess"] = False
            return out

        contracts: list[Any] | None = None
        if isinstance(data.get("rec_contracts"), list):
            contracts = [with_default_ess(it) for it in data["rec_contracts"]]
        elif isinstance(data.get("rec_fixed_contract"), list):
            contracts = [with_default_ess(it) for it in data["rec_fixed_contract"]]
        elif isinstance(data.get("rec_fixed_contract"), dict):
            contracts = [with_default_ess(data["rec_fixed_contract"])]

        if contracts is None:
            contracts = []

        normalized = dict(response)
        normalized_data = dict(data)
        normalized_data["rec_contracts"] = contracts
        normalized_data.pop("rec_fixed_contract", None)
        normalized["data"] = normalized_data
        return normalized
