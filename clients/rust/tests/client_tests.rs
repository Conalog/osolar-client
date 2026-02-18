use httpmock::Method::GET;
use httpmock::MockServer;
use osolar_client::client::OsolarClient;
use osolar_client::error::ApiError;
use osolar_client::models::{MonthlyBillingParams, MonthlyGenerationParams, SearchPlantsParams};

#[test]
fn search_plants_sends_query_and_header() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/search")
            .query_param("q", "foo")
            .query_param("field", "address")
            .query_param("distance_km", "2")
            .header("x-api-key", "test-key");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"success":true,"data":{"features":[]}}"#);
    });

    let client = OsolarClient::new("test-key").with_base_url(server.base_url());
    let response = client
        .search_plants(SearchPlantsParams {
            q: "foo".to_string(),
            field: "address".to_string(),
            distance_km: Some(2.0),
        })
        .expect("search_plants should succeed");

    mock.assert();
    assert!(response.success);
}

#[test]
fn returns_http_error_on_non_2xx() {
    let server = MockServer::start();
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/v1/links");
        then.status(403)
            .header("content-type", "application/json")
            .body(r#"{"success":false,"message":"forbidden"}"#);
    });

    let client = OsolarClient::new("test-key").with_base_url(server.base_url());
    let err = client
        .list_connected_plants()
        .expect_err("list_connected_plants should fail");

    match err {
        ApiError::Http { status, .. } => assert_eq!(status, 403),
        other => panic!("expected ApiError::Http, got {other:?}"),
    }
}

#[test]
fn monthly_generation_uses_snake_case_query_params() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/links/conn-1/generation/monthly")
            .query_param("start_year", "2023")
            .query_param("end_year", "2024")
            .header("x-api-key", "test-key");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"success":true,"data":[]}"#);
    });

    let client = OsolarClient::new("test-key").with_base_url(server.base_url());
    let response = client
        .get_monthly_generation(
            "conn-1",
            MonthlyGenerationParams {
                start_year: Some(2023),
                end_year: Some(2024),
            },
        )
        .expect("get_monthly_generation should succeed");

    mock.assert();
    assert!(response.success);
}

#[test]
fn monthly_billing_uses_camel_case_query_params() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/links/conn-1/billing/monthly")
            .query_param("startYear", "2023")
            .query_param("endYear", "2024")
            .header("x-api-key", "test-key");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"success":true,"data":[]}"#);
    });

    let client = OsolarClient::new("test-key").with_base_url(server.base_url());
    let response = client
        .get_monthly_billing(
            "conn-1",
            MonthlyBillingParams {
                start_year: Some(2023),
                end_year: Some(2024),
            },
        )
        .expect("get_monthly_billing should succeed");

    mock.assert();
    assert!(response.success);
}
