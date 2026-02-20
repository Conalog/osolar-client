use reqwest::{redirect, Method};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt;

use crate::client::{
    link_path, monthly_year_query, query_slice, search_query, CAMEL_CASE_YEAR_KEYS,
    CONTRACT_SUFFIX, DEFAULT_BASE_URL, DEFAULT_TIMEOUT, DOCUMENTS_SUFFIX, EMPTY_SUCCESS_RESPONSE,
    LINKS_PATH, MAX_RESPONSE_BYTES, MONTHLY_BILLING_SUFFIX, MONTHLY_GENERATION_SUFFIX,
    OVERVIEW_SUFFIX, SEARCH_PATH, SNAKE_CASE_YEAR_KEYS,
};
use crate::error::ApiError;
use crate::models::{
    ApiResponse, BillingAmountResponse, DocumentResponse, GenerationAmountResponse,
    MonthlyBillingParams, MonthlyGenerationParams, PlantConnectionListResponse,
    PlantConnectionRequest, PlantConnectionResponse, PlantContractResponse, PlantGeoJsonResponse,
    PlantInfoResponse, PlantOverviewResponse, SearchPlantsParams,
};

#[derive(Clone)]
pub struct AsyncOsolarClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
    allow_insecure_http: bool,
}

impl fmt::Debug for AsyncOsolarClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncOsolarClient")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("allow_insecure_http", &self.allow_insecure_http)
            .finish_non_exhaustive()
    }
}

impl AsyncOsolarClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            http_client: reqwest::Client::builder()
                .redirect(redirect::Policy::none())
                .timeout(DEFAULT_TIMEOUT)
                .build()
                .expect("failed to build hardened http client"),
            allow_insecure_http: false,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn allow_insecure_http(mut self) -> Self {
        self.allow_insecure_http = true;
        self
    }

    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    pub async fn search_plants(
        &self,
        params: SearchPlantsParams,
    ) -> Result<ApiResponse<PlantGeoJsonResponse>, ApiError> {
        let query = search_query(params);
        self.get_json(SEARCH_PATH, query_slice(&query)).await
    }

    pub async fn connect_plant(
        &self,
        body: &PlantConnectionRequest,
    ) -> Result<ApiResponse<PlantConnectionResponse>, ApiError> {
        self.post_json(LINKS_PATH, body).await
    }

    pub async fn list_connected_plants(
        &self,
    ) -> Result<ApiResponse<Vec<PlantConnectionListResponse>>, ApiError> {
        self.get_json(LINKS_PATH, None::<&[(&'static str, String)]>)
            .await
    }

    pub async fn get_plant_info(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantInfoResponse>, ApiError> {
        let path = link_path(connection_id, "");
        self.get_json(&path, None::<&[(&'static str, String)]>)
            .await
    }

    pub async fn get_plant_contract(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantContractResponse>, ApiError> {
        let path = link_path(connection_id, CONTRACT_SUFFIX);
        self.get_json(&path, None::<&[(&'static str, String)]>)
            .await
    }

    pub async fn get_plant_documents(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<Vec<DocumentResponse>>, ApiError> {
        let path = link_path(connection_id, DOCUMENTS_SUFFIX);
        self.get_json(&path, None::<&[(&'static str, String)]>)
            .await
    }

    pub async fn get_plant_overview(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantOverviewResponse>, ApiError> {
        let path = link_path(connection_id, OVERVIEW_SUFFIX);
        self.get_json(&path, None::<&[(&'static str, String)]>)
            .await
    }

    pub async fn get_monthly_generation(
        &self,
        connection_id: &str,
        params: MonthlyGenerationParams,
    ) -> Result<ApiResponse<Vec<GenerationAmountResponse>>, ApiError> {
        let path = link_path(connection_id, MONTHLY_GENERATION_SUFFIX);
        let query = monthly_year_query(params.start_year, params.end_year, SNAKE_CASE_YEAR_KEYS);
        self.get_json(&path, query_slice(&query)).await
    }

    pub async fn get_monthly_billing(
        &self,
        connection_id: &str,
        params: MonthlyBillingParams,
    ) -> Result<ApiResponse<Vec<BillingAmountResponse>>, ApiError> {
        let path = link_path(connection_id, MONTHLY_BILLING_SUFFIX);
        let query = monthly_year_query(params.start_year, params.end_year, CAMEL_CASE_YEAR_KEYS);
        self.get_json(&path, query_slice(&query)).await
    }

    async fn get_json<T, Q>(&self, path: &str, query: Option<Q>) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        Q: Serialize,
    {
        self.request(Method::GET, path, query, None::<Value>).await
    }

    async fn post_json<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request(
            Method::POST,
            path,
            None::<&[(&'static str, String)]>,
            Some(body),
        )
        .await
    }

    async fn request<Q, B, T>(
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
        if !self.allow_insecure_http {
            let parsed =
                reqwest::Url::parse(&self.base_url).map_err(|_| ApiError::InvalidBaseUrl {
                    base_url: self.base_url.clone(),
                })?;
            if parsed.scheme() != "https" {
                return Err(ApiError::InsecureBaseUrl {
                    base_url: self.base_url.clone(),
                });
            }
        }

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

        let mut response = request.send().await?;
        let status = response.status();
        let content_length = response.content_length();
        if let Some(len) = content_length {
            if len > MAX_RESPONSE_BYTES {
                return Err(ApiError::ResponseTooLarge {
                    content_length,
                    limit_bytes: MAX_RESPONSE_BYTES,
                });
            }
        }

        let mut raw_body = Vec::with_capacity(
            content_length
                .and_then(|len| usize::try_from(len).ok())
                .unwrap_or(0),
        );
        while let Some(chunk) = response.chunk().await? {
            if raw_body.len() + chunk.len() > MAX_RESPONSE_BYTES as usize {
                return Err(ApiError::ResponseTooLarge {
                    content_length,
                    limit_bytes: MAX_RESPONSE_BYTES,
                });
            }
            raw_body.extend_from_slice(&chunk);
        }

        if !status.is_success() {
            let body = match serde_json::from_slice::<crate::error::OsolarApiErrorBody>(&raw_body) {
                Ok(value) => crate::error::OsolarErrorPayload::Json(value),
                Err(_) => crate::error::OsolarErrorPayload::Text(
                    String::from_utf8_lossy(&raw_body).into_owned(),
                ),
            };
            return Err(ApiError::Http {
                status: status.as_u16(),
                body,
            });
        }

        if raw_body.is_empty() || status == reqwest::StatusCode::NO_CONTENT {
            return Ok(serde_json::from_slice::<T>(EMPTY_SUCCESS_RESPONSE)?);
        }

        Ok(serde_json::from_slice::<T>(&raw_body)?)
    }
}
