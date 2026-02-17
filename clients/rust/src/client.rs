use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::models::{
    ApiResponse, BillingAmountResponse, DocumentResponse, GenerationAmountResponse,
    MonthlyBillingParams, MonthlyGenerationParams, PlantContractResponse, PlantGeoJsonResponse,
    PlantInfoResponse, PlantLinkListResponse, PlantLinkRequest, PlantLinkResponse,
    PlantOverviewResponse, SearchPlantsParams,
};

const DEFAULT_BASE_URL: &str = "https://openapi.osolar.io";

#[derive(Debug, Clone)]
pub struct OsolarLinkClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::blocking::Client,
}

impl OsolarLinkClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn with_http_client(mut self, client: reqwest::blocking::Client) -> Self {
        self.http_client = client;
        self
    }

    pub fn search_plants(
        &self,
        params: SearchPlantsParams,
    ) -> Result<ApiResponse<PlantGeoJsonResponse>, ApiError> {
        let mut query = vec![("q", params.q), ("field", params.field)];
        if let Some(distance_km) = params.distance_km {
            query.push(("distance_km", distance_km.to_string()));
        }
        self.request(Method::GET, "/v1/search", Some(&query), Option::<&()>::None)
    }

    pub fn link_plant(
        &self,
        body: &PlantLinkRequest,
    ) -> Result<ApiResponse<PlantLinkResponse>, ApiError> {
        self.request(
            Method::POST,
            "/v1/links",
            None::<&[(&str, String)]>,
            Some(body),
        )
    }

    pub fn list_linked_plants(&self) -> Result<ApiResponse<Vec<PlantLinkListResponse>>, ApiError> {
        self.request(
            Method::GET,
            "/v1/links",
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_info(
        &self,
        link_id: &str,
    ) -> Result<ApiResponse<PlantInfoResponse>, ApiError> {
        let path = format!("/v1/links/{}", urlencoding::encode(link_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_contract(
        &self,
        link_id: &str,
    ) -> Result<ApiResponse<PlantContractResponse>, ApiError> {
        let path = format!("/v1/links/{}/contract", urlencoding::encode(link_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_documents(
        &self,
        link_id: &str,
    ) -> Result<ApiResponse<Vec<DocumentResponse>>, ApiError> {
        let path = format!("/v1/links/{}/documents", urlencoding::encode(link_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_overview(
        &self,
        link_id: &str,
    ) -> Result<ApiResponse<PlantOverviewResponse>, ApiError> {
        let path = format!("/v1/links/{}/overview", urlencoding::encode(link_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_monthly_generation(
        &self,
        link_id: &str,
        params: MonthlyGenerationParams,
    ) -> Result<ApiResponse<Vec<GenerationAmountResponse>>, ApiError> {
        let mut query: Vec<(&str, String)> = vec![];
        if let Some(start_year) = params.start_year {
            query.push(("start_year", start_year.to_string()));
        }
        if let Some(end_year) = params.end_year {
            query.push(("end_year", end_year.to_string()));
        }

        let path = format!(
            "/v1/links/{}/generation/monthly",
            urlencoding::encode(link_id)
        );
        let query_ref = if query.is_empty() {
            None
        } else {
            Some(query.as_slice())
        };
        self.request(Method::GET, &path, query_ref, Option::<&()>::None)
    }

    pub fn get_monthly_billing(
        &self,
        link_id: &str,
        params: MonthlyBillingParams,
    ) -> Result<ApiResponse<Vec<BillingAmountResponse>>, ApiError> {
        let mut query: Vec<(&str, String)> = vec![];
        if let Some(start_year) = params.start_year {
            query.push(("startYear", start_year.to_string()));
        }
        if let Some(end_year) = params.end_year {
            query.push(("endYear", end_year.to_string()));
        }

        let path = format!("/v1/links/{}/billing/monthly", urlencoding::encode(link_id));
        let query_ref = if query.is_empty() {
            None
        } else {
            Some(query.as_slice())
        };
        self.request(Method::GET, &path, query_ref, Option::<&()>::None)
    }

    fn request<Q, B, T>(
        &self,
        method: Method,
        path: &str,
        query: Option<Q>,
        body: Option<B>,
    ) -> Result<T, ApiError>
    where
        Q: Serialize,
        B: Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self
            .http_client
            .request(method, &url)
            .header("x-api-key", &self.api_key);

        if let Some(query) = query {
            request = request.query(&query);
        }

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send()?;
        let status = response.status();
        let raw_body = response.text()?;

        if !status.is_success() {
            let body = serde_json::from_str::<Value>(&raw_body)
                .unwrap_or_else(|_| Value::String(raw_body));
            return Err(ApiError::Http {
                status: status.as_u16(),
                body,
            });
        }

        Ok(serde_json::from_str::<T>(&raw_body)?)
    }
}
