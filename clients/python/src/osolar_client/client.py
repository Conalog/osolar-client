from __future__ import annotations

from types import TracebackType
from typing import Any, Literal, overload
from urllib.parse import quote, urlparse

import httpx

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

SearchField = Literal["business_number", "address"]
_ALLOWED_SEARCH_FIELDS = {"business_number", "address"}


class OsolarLinkClient:
    def __init__(
        self,
        api_key: str,
        base_url: str = "https://openapi.osolar.io",
        timeout: float = 30.0,
        http_client: httpx.Client | None = None,
    ):
        """Create a synchronous client for the OSOLAR-LINK Open API."""
        if not isinstance(api_key, str) or not api_key.strip():
            raise ValueError("`api_key` must be a non-empty string.")
        self._api_key = api_key
        self._base_url = base_url.rstrip("/")
        parsed = urlparse(self._base_url)
        if parsed.scheme not in ("http", "https"):
            raise ValueError("`base_url` must start with http:// or https://.")
        if parsed.scheme == "http":
            host = (parsed.hostname or "").lower()
            if host not in ("127.0.0.1", "localhost", "::1"):
                raise ValueError(
                    "`base_url` must use https:// (http:// is allowed only for localhost)."
                )
        self._owns_client = http_client is None
        self._http_client = http_client or httpx.Client(timeout=timeout)

    def close(self) -> None:
        """Close the underlying http client if this client created it."""
        if self._owns_client:
            self._http_client.close()

    def __enter__(self) -> "OsolarLinkClient":
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        self.close()

    def search_plants(
        self,
        q: str,
        field: SearchField,
        distance_km: float | None = None,
    ) -> SearchPlantsApiResponse:
        """Search plants by a text query and constrained search field."""
        if field not in _ALLOWED_SEARCH_FIELDS:
            raise ValueError("`field` must be one of: business_number, address.")
        query = {"q": q, "field": field, "distance_km": distance_km}
        return self._request("GET", "/v1/search", params=query)

    @overload
    def link_plant(self, body: PlantLinkRequest) -> LinkPlantApiResponse: ...

    @overload
    def link_plant(
        self,
        body: None = None,
        *,
        plant_uuid: str,
        remark: str,
        link_id: str | None = None,
    ) -> LinkPlantApiResponse: ...

    def link_plant(
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

        return self._request("POST", "/v1/links", json=payload)

    def list_linked_plants(self) -> ListLinkedPlantsApiResponse:
        """List plants linked to the current API key."""
        return self._request("GET", "/v1/links")

    def get_plant_info(self, link_id: str) -> PlantInfoApiResponse:
        """Fetch the base information for a linked plant."""
        safe_link_id = quote(link_id, safe="")
        return self._request("GET", f"/v1/links/{safe_link_id}")

    def get_plant_contract(self, link_id: str) -> PlantContractApiResponse:
        """Fetch contract information for a linked plant."""
        safe_link_id = quote(link_id, safe="")
        response: Any = self._request("GET", f"/v1/links/{safe_link_id}/contract")
        return self._normalize_plant_contract_response(response)

    def get_plant_documents(self, link_id: str) -> PlantDocumentsApiResponse:
        """Fetch available documents for a linked plant."""
        safe_link_id = quote(link_id, safe="")
        return self._request("GET", f"/v1/links/{safe_link_id}/documents")

    def get_plant_overview(self, link_id: str) -> PlantOverviewApiResponse:
        """Fetch overview data for a linked plant."""
        safe_link_id = quote(link_id, safe="")
        return self._request("GET", f"/v1/links/{safe_link_id}/overview")

    def get_monthly_generation(
        self,
        link_id: str,
        start_year: int | None = None,
        end_year: int | None = None,
    ) -> MonthlyGenerationApiResponse:
        """Fetch monthly generation data for a linked plant."""
        safe_link_id = quote(link_id, safe="")
        query = {"start_year": start_year, "end_year": end_year}
        return self._request(
            "GET",
            f"/v1/links/{safe_link_id}/generation/monthly",
            params=query,
        )

    def get_monthly_billing(
        self,
        link_id: str,
        start_year: int | None = None,
        end_year: int | None = None,
    ) -> MonthlyBillingApiResponse:
        """Fetch monthly billing data for a linked plant."""
        safe_link_id = quote(link_id, safe="")
        # OpenAPI spec uses camelCase for this endpoint.
        query = {"startYear": start_year, "endYear": end_year}
        return self._request(
            "GET",
            f"/v1/links/{safe_link_id}/billing/monthly",
            params=query,
        )

    def _request(
        self,
        method: str,
        path: str,
        params: dict[str, Any] | None = None,
        json: dict[str, Any] | None = None,
    ) -> Any:
        filtered_params = {k: v for k, v in (params or {}).items() if v is not None}
        response = self._http_client.request(
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
