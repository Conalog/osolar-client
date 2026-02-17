from __future__ import annotations

from typing import Literal, NotRequired, TypedDict


class ApiResponseBase(TypedDict):
    success: bool
    message: NotRequired[str | None]


class GeoPoint(TypedDict):
    coordinates: list[float]
    type: NotRequired[str]


class PlantOwner(TypedDict):
    business_number: str
    firm_name: str
    firm_address: str
    representative_name: str


class PlantFeatureProperties(TypedDict):
    plant_uuid: str
    plant_name: str
    plant_address: str
    plant_capacity: float
    plant_owner: PlantOwner


class PlantFeature(TypedDict):
    geometry: GeoPoint
    properties: PlantFeatureProperties
    type: NotRequired[str]


class PlantGeoJSONResponse(TypedDict):
    features: list[PlantFeature]
    type: NotRequired[str]


class PlantLinkRequest(TypedDict):
    plant_uuid: str
    remark: str
    link_id: NotRequired[str | None]


class PlantLinkResponse(TypedDict):
    link_id: str
    created_at: str


class PlantLinkListResponse(TypedDict):
    link_id: str
    plant_name: str
    plant_address: NotRequired[str | None]
    remark: NotRequired[str | None]
    created_at: NotRequired[str | None]


class PlantInfoResponse(TypedDict):
    link_id: str
    plant_name: str
    plant_address: str
    plant_capacity: str
    plant_certified: bool
    plant_geometry: GeoPoint
    plant_owner: PlantOwner


class RecFixedContractInfo(TypedDict):
    ess: bool
    target: str
    price_type: NotRequired[str | None]
    price: NotRequired[int | None]
    start_date: NotRequired[str | None]
    end_date: NotRequired[str | None]
    contract_years: NotRequired[int | None]


class PlantContractResponse(TypedDict):
    ppa_type: str
    rec_trade_type: str
    rec_contracts: list[RecFixedContractInfo]


class DocumentResponse(TypedDict):
    document_id: str
    document_code: str
    document_name: str
    file_name: str
    download_url: str


class BillingAmountResponse(TypedDict):
    billing_month: str
    smp_billing_amount: int
    rec_billing_amount: int


class GenerationAmountResponse(TypedDict):
    generation_month: str
    generation_amount: int
    nearby_generation_amount: int


TaskType = Literal["REC_ISSUANCE", "SMP_BILLING", "REC_BILLING", "REC_SPOT_TRADING"]
TaskStatus = Literal["완료", "진행중", "실패", "대기"]


class TaskDetail(TypedDict):
    name: str
    type: TaskType
    status: TaskStatus
    summary: str
    value: str
    date: NotRequired[str | None]


class PlantOverviewResponse(TypedDict):
    link_id: str
    plant_name: str
    billing_summary: list[BillingAmountResponse]
    recent_tasks: list[TaskDetail]
