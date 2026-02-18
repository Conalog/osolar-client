from __future__ import annotations

from typing import Any, cast, overload
from urllib.parse import quote

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

class OsolarLinkClient:
    def __init__(
        self,
        api_key: str,
        base_url: str = "https://openapi.osolar.io",
        timeout: float = 30.0,
        http_client: httpx.Client | None = None,
    ):
        self._api_key = api_key
        self._base_url = base_url.rstrip("/")
        self._owns_client = http_client is None
        self._http_client = http_client or httpx.Client(timeout=timeout)

    def close(self) -> None:
        if self._owns_client:
            self._http_client.close()

    def __enter__(self) -> "OsolarLinkClient":
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        self.close()

    def search_plants(self, q: str, field: str, distance_km: float | None = None) -> SearchPlantsApiResponse:
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
        if body is not None:
            if plant_uuid is not None or remark is not None or link_id is not None:
                raise ValueError("Use either `body` or keyword arguments, not both.")
            payload = cast(dict[str, Any], body)
        else:
            if plant_uuid is None or remark is None:
                raise ValueError("`plant_uuid` and `remark` are required when `body` is not provided.")
            payload = {"plant_uuid": plant_uuid, "remark": remark}
            if link_id is not None:
                payload["link_id"] = link_id

        return self._request("POST", "/v1/links", json=payload)

    def list_linked_plants(self) -> ListLinkedPlantsApiResponse:
        return self._request("GET", "/v1/links")

    def get_plant_info(self, link_id: str) -> PlantInfoApiResponse:
        safe_link_id = quote(link_id, safe="")
        return self._request("GET", f"/v1/links/{safe_link_id}")

    def get_plant_contract(self, link_id: str) -> PlantContractApiResponse:
        safe_link_id = quote(link_id, safe="")
        return self._request("GET", f"/v1/links/{safe_link_id}/contract")

    def get_plant_documents(self, link_id: str) -> PlantDocumentsApiResponse:
        safe_link_id = quote(link_id, safe="")
        return self._request("GET", f"/v1/links/{safe_link_id}/documents")

    def get_plant_overview(self, link_id: str) -> PlantOverviewApiResponse:
        safe_link_id = quote(link_id, safe="")
        return self._request("GET", f"/v1/links/{safe_link_id}/overview")

    def get_monthly_generation(
        self,
        link_id: str,
        start_year: int | None = None,
        end_year: int | None = None,
    ) -> MonthlyGenerationApiResponse:
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
            raise ApiError(response.status_code, "Invalid JSON response", response.text) from exc

        if not isinstance(body, dict):
            raise ApiError(response.status_code, "Unexpected JSON response type", body)

        return body
