use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

use crate::error::ApiError;
use crate::models::{
    ApiResponse, BillingAmountResponse, DocumentResponse, GenerationAmountResponse,
    MonthlyBillingParams, MonthlyGenerationParams, PlantConnectionListResponse,
    PlantConnectionRequest, PlantConnectionResponse, PlantContractResponse, PlantGeoJsonResponse,
    PlantInfoResponse, PlantOverviewResponse, SearchPlantsParams,
};

const DEFAULT_BASE_URL: &str = "https://openapi.osolar.io";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct OsolarClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::blocking::Client,
}

impl OsolarClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            http_client: reqwest::blocking::Client::builder()
                .timeout(DEFAULT_TIMEOUT)
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new()),
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

    pub fn connect_plant(
        &self,
        body: &PlantConnectionRequest,
    ) -> Result<ApiResponse<PlantConnectionResponse>, ApiError> {
        self.request(
            Method::POST,
            "/v1/links",
            None::<&[(&str, String)]>,
            Some(body),
        )
    }

    pub fn list_connected_plants(
        &self,
    ) -> Result<ApiResponse<Vec<PlantConnectionListResponse>>, ApiError> {
        self.request(
            Method::GET,
            "/v1/links",
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_info(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantInfoResponse>, ApiError> {
        let path = format!("/v1/links/{}", urlencoding::encode(connection_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_contract(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantContractResponse>, ApiError> {
        let path = format!("/v1/links/{}/contract", urlencoding::encode(connection_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_documents(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<Vec<DocumentResponse>>, ApiError> {
        let path = format!("/v1/links/{}/documents", urlencoding::encode(connection_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_plant_overview(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantOverviewResponse>, ApiError> {
        let path = format!("/v1/links/{}/overview", urlencoding::encode(connection_id));
        self.request(
            Method::GET,
            &path,
            None::<&[(&str, String)]>,
            Option::<&()>::None,
        )
    }

    pub fn get_monthly_generation(
        &self,
        connection_id: &str,
        params: MonthlyGenerationParams,
    ) -> Result<ApiResponse<Vec<GenerationAmountResponse>>, ApiError> {
        let mut query: Vec<(&str, String)> = vec![];
        if let Some(start_year) = params.start_year {
            // OpenAPI contract uses snake_case for generation filters.
            query.push(("start_year", start_year.to_string()));
        }
        if let Some(end_year) = params.end_year {
            query.push(("end_year", end_year.to_string()));
        }

        let path = format!(
            "/v1/links/{}/generation/monthly",
            urlencoding::encode(connection_id)
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
        connection_id: &str,
        params: MonthlyBillingParams,
    ) -> Result<ApiResponse<Vec<BillingAmountResponse>>, ApiError> {
        let mut query: Vec<(&str, String)> = vec![];
        if let Some(start_year) = params.start_year {
            // OpenAPI contract uses camelCase for billing filters.
            query.push(("startYear", start_year.to_string()));
        }
        if let Some(end_year) = params.end_year {
            query.push(("endYear", end_year.to_string()));
        }

        let path = format!(
            "/v1/links/{}/billing/monthly",
            urlencoding::encode(connection_id)
        );
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
        let raw_body = response.bytes()?;

        if !status.is_success() {
            let body = match serde_json::from_slice::<Value>(&raw_body) {
                Ok(value) => value,
                Err(_) => Value::String(String::from_utf8_lossy(&raw_body).into_owned()),
            };
            return Err(ApiError::Http {
                status: status.as_u16(),
                body,
            });
        }

        if raw_body.is_empty() || status == reqwest::StatusCode::NO_CONTENT {
            // Keep behavior aligned with other SDKs for successful empty responses.
            return Ok(serde_json::from_slice::<T>(br#"{"success":true,"data":null}"#)?);
        }

        Ok(serde_json::from_slice::<T>(&raw_body)?)
    }
}
