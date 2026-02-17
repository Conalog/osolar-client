export interface ApiResponse<T> {
  success: boolean;
  message?: string | null;
  data?: T | null;
}

export interface GeoPoint {
  type?: string;
  coordinates: number[];
}

export interface PlantOwner {
  business_number: string;
  firm_name: string;
  firm_address: string;
  representative_name: string;
}

export interface PlantFeatureProperties {
  plant_uuid: string;
  plant_name: string;
  plant_address: string;
  plant_capacity: number;
  plant_owner: PlantOwner;
}

export interface PlantFeature {
  type?: string;
  geometry: GeoPoint;
  properties: PlantFeatureProperties;
}

export interface PlantGeoJSONResponse {
  type?: string;
  features: PlantFeature[];
}

export interface PlantLinkRequest {
  plant_uuid: string;
  link_id?: string | null;
  remark: string;
}

export interface PlantLinkResponse {
  link_id: string;
  created_at: string;
}

export interface PlantLinkListResponse {
  link_id: string;
  plant_name: string;
  plant_address?: string | null;
  remark?: string | null;
  created_at?: string | null;
}

export interface PlantInfoResponse {
  link_id: string;
  plant_name: string;
  plant_address: string;
  plant_capacity: string;
  plant_certified: boolean;
  plant_geometry: GeoPoint;
  plant_owner: PlantOwner;
}

export interface RecFixedContractInfo {
  ess: boolean;
  target: string;
  price_type?: string | null;
  price?: number | null;
  start_date?: string | null;
  end_date?: string | null;
  contract_years?: number | null;
}

export interface PlantContractResponse {
  ppa_type: string;
  rec_trade_type: string;
  rec_contracts: RecFixedContractInfo[];
}

export interface DocumentResponse {
  document_id: string;
  document_code: string;
  document_name: string;
  file_name: string;
  download_url: string;
}

export interface BillingAmountResponse {
  billing_month: string;
  smp_billing_amount: number;
  rec_billing_amount: number;
}

export interface GenerationAmountResponse {
  generation_month: string;
  generation_amount: number;
  nearby_generation_amount: number;
}

export type TaskType = "REC_ISSUANCE" | "SMP_BILLING" | "REC_BILLING" | "REC_SPOT_TRADING";
export type TaskStatus = "완료" | "진행중" | "실패" | "대기";

export interface TaskDetail {
  date?: string | null;
  name: string;
  type: TaskType;
  status: TaskStatus;
  summary: string;
  value: string;
}

export interface PlantOverviewResponse {
  link_id: string;
  plant_name: string;
  billing_summary: BillingAmountResponse[];
  recent_tasks: TaskDetail[];
}

export interface SearchPlantsParams {
  q: string;
  field: string;
  distanceKm?: number;
}

export interface MonthlyGenerationParams {
  startYear?: number;
  endYear?: number;
}

export interface MonthlyBillingParams {
  startYear?: number;
  endYear?: number;
}
