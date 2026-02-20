use reqwest::redirect;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::io::Read;
use std::time::Duration;

use crate::error::ApiError;
use crate::models::{
    ApiResponse, BillingAmountResponse, DocumentResponse, GenerationAmountResponse,
    MonthlyBillingParams, MonthlyGenerationParams, PlantConnectionListResponse,
    PlantConnectionRequest, PlantConnectionResponse, PlantContractResponse, PlantGeoJsonResponse,
    PlantInfoResponse, PlantOverviewResponse, SearchPlantsParams,
};

pub(crate) const DEFAULT_BASE_URL: &str = "https://openapi.osolar.io";
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const MAX_RESPONSE_BYTES: u64 = 10 * 1024 * 1024;
pub(crate) const SEARCH_PATH: &str = "/v1/search";
pub(crate) const LINKS_PATH: &str = "/v1/links";
pub(crate) const CONTRACT_SUFFIX: &str = "/contract";
pub(crate) const DOCUMENTS_SUFFIX: &str = "/documents";
pub(crate) const OVERVIEW_SUFFIX: &str = "/overview";
pub(crate) const MONTHLY_GENERATION_SUFFIX: &str = "/generation/monthly";
pub(crate) const MONTHLY_BILLING_SUFFIX: &str = "/billing/monthly";
pub(crate) const EMPTY_SUCCESS_RESPONSE: &[u8] = br#"{"success":true,"data":null}"#;

#[derive(Debug, Clone, Copy)]
pub(crate) struct YearQueryKeys {
    pub(crate) start: &'static str,
    pub(crate) end: &'static str,
}

pub(crate) const SNAKE_CASE_YEAR_KEYS: YearQueryKeys = YearQueryKeys {
    start: "start_year",
    end: "end_year",
};
pub(crate) const CAMEL_CASE_YEAR_KEYS: YearQueryKeys = YearQueryKeys {
    start: "startYear",
    end: "endYear",
};

pub(crate) type QueryPairs = Vec<(&'static str, String)>;

#[derive(Clone)]
pub struct OsolarClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::blocking::Client,
    allow_insecure_http: bool,
}

impl fmt::Debug for OsolarClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OsolarClient")
            .field("api_key", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("allow_insecure_http", &self.allow_insecure_http)
            .finish_non_exhaustive()
    }
}

impl OsolarClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            http_client: reqwest::blocking::Client::builder()
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

    pub fn with_http_client(mut self, client: reqwest::blocking::Client) -> Self {
        self.http_client = client;
        self
    }

    pub fn search_plants(
        &self,
        params: SearchPlantsParams,
    ) -> Result<ApiResponse<PlantGeoJsonResponse>, ApiError> {
        let query = search_query(params);
        self.get_json(SEARCH_PATH, query_slice(&query))
    }

    pub fn connect_plant(
        &self,
        body: &PlantConnectionRequest,
    ) -> Result<ApiResponse<PlantConnectionResponse>, ApiError> {
        self.post_json(LINKS_PATH, body)
    }

    pub fn list_connected_plants(
        &self,
    ) -> Result<ApiResponse<Vec<PlantConnectionListResponse>>, ApiError> {
        self.get_json(LINKS_PATH, None::<&[(&'static str, String)]>)
    }

    pub fn get_plant_info(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantInfoResponse>, ApiError> {
        let path = link_path(connection_id, "");
        self.get_json(&path, None::<&[(&'static str, String)]>)
    }

    pub fn get_plant_contract(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantContractResponse>, ApiError> {
        let path = link_path(connection_id, CONTRACT_SUFFIX);
        self.get_json(&path, None::<&[(&'static str, String)]>)
    }

    pub fn get_plant_documents(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<Vec<DocumentResponse>>, ApiError> {
        let path = link_path(connection_id, DOCUMENTS_SUFFIX);
        self.get_json(&path, None::<&[(&'static str, String)]>)
    }

    pub fn get_plant_overview(
        &self,
        connection_id: &str,
    ) -> Result<ApiResponse<PlantOverviewResponse>, ApiError> {
        let path = link_path(connection_id, OVERVIEW_SUFFIX);
        self.get_json(&path, None::<&[(&'static str, String)]>)
    }

    pub fn get_monthly_generation(
        &self,
        connection_id: &str,
        params: MonthlyGenerationParams,
    ) -> Result<ApiResponse<Vec<GenerationAmountResponse>>, ApiError> {
        let path = link_path(connection_id, MONTHLY_GENERATION_SUFFIX);
        let query = monthly_year_query(params.start_year, params.end_year, SNAKE_CASE_YEAR_KEYS);
        self.get_json(&path, query_slice(&query))
    }

    pub fn get_monthly_billing(
        &self,
        connection_id: &str,
        params: MonthlyBillingParams,
    ) -> Result<ApiResponse<Vec<BillingAmountResponse>>, ApiError> {
        let path = link_path(connection_id, MONTHLY_BILLING_SUFFIX);
        let query = monthly_year_query(params.start_year, params.end_year, CAMEL_CASE_YEAR_KEYS);
        self.get_json(&path, query_slice(&query))
    }

    fn get_json<T, Q>(&self, path: &str, query: Option<Q>) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        Q: Serialize,
    {
        self.request(Method::GET, path, query, None::<Value>)
    }

    fn post_json<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>
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

        let response = request.send()?;
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

        let mut raw_body = Vec::new();
        response
            .take(MAX_RESPONSE_BYTES + 1)
            .read_to_end(&mut raw_body)?;
        if raw_body.len() as u64 > MAX_RESPONSE_BYTES {
            return Err(ApiError::ResponseTooLarge {
                content_length,
                limit_bytes: MAX_RESPONSE_BYTES,
            });
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
            // Keep behavior aligned with other SDKs for successful empty responses.
            return Ok(serde_json::from_slice::<T>(EMPTY_SUCCESS_RESPONSE)?);
        }

        Ok(serde_json::from_slice::<T>(&raw_body)?)
    }
}

pub(crate) fn search_query(params: SearchPlantsParams) -> QueryPairs {
    let mut query = vec![("q", params.q), ("field", params.field)];
    if let Some(distance_km) = params.distance_km {
        query.push(("distance_km", distance_km.to_string()));
    }
    query
}

pub(crate) fn monthly_year_query(
    start_year: Option<i64>,
    end_year: Option<i64>,
    keys: YearQueryKeys,
) -> QueryPairs {
    let mut query = Vec::new();
    if let Some(start_year) = start_year {
        query.push((keys.start, start_year.to_string()));
    }
    if let Some(end_year) = end_year {
        query.push((keys.end, end_year.to_string()));
    }
    query
}

pub(crate) fn query_slice(query: &QueryPairs) -> Option<&[(&'static str, String)]> {
    if query.is_empty() {
        None
    } else {
        Some(query.as_slice())
    }
}

pub(crate) fn link_path(connection_id: &str, suffix: &str) -> String {
    format!(
        "{}/{encoded_connection_id}{suffix}",
        LINKS_PATH,
        encoded_connection_id = urlencoding::encode(connection_id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_path_encodes_connection_id() {
        assert_eq!(
            link_path("conn id/with/slash", ""),
            "/v1/links/conn%20id%2Fwith%2Fslash"
        );
        assert_eq!(
            link_path("conn id/with/slash", MONTHLY_BILLING_SUFFIX),
            "/v1/links/conn%20id%2Fwith%2Fslash/billing/monthly"
        );
    }

    #[test]
    fn monthly_year_query_uses_selected_key_style() {
        let snake_case = monthly_year_query(Some(2023), Some(2024), SNAKE_CASE_YEAR_KEYS);
        assert_eq!(
            snake_case,
            vec![
                ("start_year", "2023".to_string()),
                ("end_year", "2024".to_string())
            ]
        );

        let camel_case = monthly_year_query(Some(2023), Some(2024), CAMEL_CASE_YEAR_KEYS);
        assert_eq!(
            camel_case,
            vec![
                ("startYear", "2023".to_string()),
                ("endYear", "2024".to_string())
            ]
        );
    }

    #[test]
    fn monthly_year_query_omits_missing_values() {
        assert!(monthly_year_query(None, None, SNAKE_CASE_YEAR_KEYS).is_empty());
        assert_eq!(
            monthly_year_query(Some(2023), None, CAMEL_CASE_YEAR_KEYS),
            vec![("startYear", "2023".to_string())]
        );
    }
}
