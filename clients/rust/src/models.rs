use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    #[serde(rename = "type")]
    pub point_type: Option<String>,
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantOwner {
    pub business_number: String,
    pub firm_name: String,
    pub firm_address: String,
    pub representative_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CapacityValue {
    // Live responses may encode this field as either a number or a numeric string.
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantFeatureProperties {
    pub plant_uuid: String,
    pub plant_name: String,
    pub plant_address: String,
    pub plant_capacity: CapacityValue,
    pub plant_owner: PlantOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantFeature {
    #[serde(rename = "type")]
    pub feature_type: Option<String>,
    pub geometry: GeoPoint,
    pub properties: PlantFeatureProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantGeoJsonResponse {
    #[serde(rename = "type")]
    pub collection_type: Option<String>,
    pub features: Vec<PlantFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantConnectionRequest {
    pub plant_uuid: String,
    #[serde(rename = "link_id")]
    pub connection_id: Option<String>,
    pub remark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantConnectionResponse {
    #[serde(rename = "link_id")]
    pub connection_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantConnectionListResponse {
    #[serde(rename = "link_id")]
    pub connection_id: String,
    pub plant_name: String,
    pub plant_address: Option<String>,
    pub remark: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantInfoResponse {
    #[serde(rename = "link_id")]
    pub connection_id: String,
    pub plant_name: String,
    pub plant_address: String,
    pub plant_capacity: String,
    pub plant_certified: bool,
    pub plant_geometry: GeoPoint,
    pub plant_owner: PlantOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecFixedContractInfo {
    pub ess: bool,
    pub target: String,
    pub price_type: Option<String>,
    pub price: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub contract_years: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantContractResponse {
    pub ppa_type: String,
    pub rec_trade_type: String,
    pub rec_contracts: Vec<RecFixedContractInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub document_id: String,
    pub document_code: String,
    pub document_name: String,
    pub file_name: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAmountResponse {
    pub billing_month: String,
    pub smp_billing_amount: i64,
    pub rec_billing_amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationAmountResponse {
    pub generation_month: String,
    pub generation_amount: i64,
    pub nearby_generation_amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetail {
    pub date: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub status: String,
    pub summary: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantOverviewResponse {
    #[serde(rename = "link_id")]
    pub connection_id: String,
    pub plant_name: String,
    pub billing_summary: Vec<BillingAmountResponse>,
    pub recent_tasks: Vec<TaskDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchPlantsParams {
    pub q: String,
    pub field: String,
    pub distance_km: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MonthlyGenerationParams {
    pub start_year: Option<i64>,
    pub end_year: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MonthlyBillingParams {
    pub start_year: Option<i64>,
    pub end_year: Option<i64>,
}
