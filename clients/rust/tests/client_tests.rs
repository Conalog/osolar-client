use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use osolar_client::client::OsolarClient;
use osolar_client::error::ApiError;
use osolar_client::models::{MonthlyBillingParams, MonthlyGenerationParams, SearchPlantsParams};
use serde_json::json;

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

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
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

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let err = client
        .list_connected_plants()
        .expect_err("list_connected_plants should fail");

    match err {
        ApiError::Http { status, .. } => assert_eq!(status, 403),
        other => panic!("expected ApiError::Http, got {other:?}"),
    }
}

#[test]
fn debug_redacts_api_key() {
    let client = OsolarClient::new("secret-key");
    let s = format!("{client:?}");
    assert!(!s.contains("secret-key"), "Debug output must not contain api key");
    assert!(s.contains("<redacted>"));
}

#[test]
fn does_not_follow_redirects() {
    let redirect_to = MockServer::start();
    let redirect_from = MockServer::start();

    let dest = redirect_to.mock(|when, then| {
        when.method(GET).path("/v1/links");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"success":true,"data":[]}"#);
    });

    let from = redirect_from.mock(|when, then| {
        when.method(GET).path("/v1/links");
        then.status(302)
            .header("location", format!("{}/v1/links", redirect_to.base_url()));
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(redirect_from.base_url())
        .allow_insecure_http();
    let err = client
        .list_connected_plants()
        .expect_err("redirects should not be followed automatically");

    from.assert();
    dest.assert_calls(0);
    match err {
        ApiError::Http { status, .. } => assert_eq!(status, 302),
        other => panic!("expected ApiError::Http, got {other:?}"),
    }
}

#[test]
fn returns_response_too_large_error() {
    let server = MockServer::start();
    let oversized = "a".repeat((10 * 1024 * 1024 + 1) as usize);
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/v1/links");
        then.status(200)
            .header("content-type", "application/json")
            .body(oversized.clone());
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let err = client
        .list_connected_plants()
        .expect_err("oversized response should be rejected");

    match err {
        ApiError::ResponseTooLarge { .. } => {}
        other => panic!("expected ApiError::ResponseTooLarge, got {other:?}"),
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

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
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

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
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

#[test]
fn connect_plant_serializes_link_id_and_parses_response() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/links")
            .header("x-api-key", "test-key")
            .header("content-type", "application/json")
            .json_body(json!({
                "plant_uuid": "plant-uuid",
                "link_id": "conn-777",
                "remark": "note"
            }));
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"success":true,"data":{"link_id":"conn-777","created_at":"2024-05-16T14:12:00"}}"#);
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let response = client
        .connect_plant(&osolar_client::models::PlantConnectionRequest {
            plant_uuid: "plant-uuid".to_string(),
            connection_id: Some("conn-777".to_string()),
            remark: "note".to_string(),
        })
        .expect("connect_plant should succeed");

    mock.assert();
    let data = response
        .data
        .expect("connect_plant response should contain data payload");
    assert_eq!(data.connection_id, "conn-777");
}

#[test]
fn get_plant_contract_accepts_single_contract_object_shape() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/links/conn-1/contract")
            .header("x-api-key", "test-key");
        then.status(200).header("content-type", "application/json").body(
            r#"{
                "success": true,
                "data": {
                    "ppa_type": "한국전력공사",
                    "rec_trade_type": "고정가격계약",
                    "rec_fixed_contract": {
                        "ess": true,
                        "target": "동서발전",
                        "price_type": "SMP+1REC*가중치",
                        "price": 165000
                    }
                }
            }"#,
        );
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let response = client
        .get_plant_contract("conn-1")
        .expect("get_plant_contract should succeed");

    mock.assert();
    let data = response
        .data
        .expect("contract response should contain data payload");
    assert_eq!(data.rec_contracts.len(), 1);
    assert_eq!(data.rec_contracts[0].target, "동서발전");
}

#[test]
fn get_plant_documents_parses_document_list() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/links/conn-1/documents")
            .header("x-api-key", "test-key");
        then.status(200).header("content-type", "application/json").body(
            r#"{
                "success": true,
                "data": [{
                    "document_id": "doc-1",
                    "document_code": "OSR-0001",
                    "document_name": "준공도면",
                    "file_name": "completion.pdf",
                    "download_url": "https://example.com/completion.pdf"
                }]
            }"#,
        );
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let response = client
        .get_plant_documents("conn-1")
        .expect("get_plant_documents should succeed");

    mock.assert();
    let data = response
        .data
        .expect("documents response should contain data payload");
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].document_id, "doc-1");
}

#[test]
fn get_plant_overview_parses_nested_payloads() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/links/conn-1/overview")
            .header("x-api-key", "test-key");
        then.status(200).header("content-type", "application/json").body(
            r#"{
                "success": true,
                "data": {
                    "link_id": "conn-1",
                    "plant_name": "오솔라 발전소 A",
                    "billing_summary": [{
                        "billing_month": "2025-03",
                        "smp_billing_amount": 3000000,
                        "rec_billing_amount": 2800000
                    }],
                    "recent_tasks": [{
                        "date": "2025-03-01",
                        "name": "SMP 청구",
                        "type": "SMP_BILLING",
                        "status": "완료",
                        "summary": "요약",
                        "value": "세부값"
                    }]
                }
            }"#,
        );
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let response = client
        .get_plant_overview("conn-1")
        .expect("get_plant_overview should succeed");

    mock.assert();
    let data = response
        .data
        .expect("overview response should contain data payload");
    assert_eq!(data.connection_id, "conn-1");
    assert_eq!(data.billing_summary.len(), 1);
    assert_eq!(data.recent_tasks.len(), 1);
}

#[test]
fn monthly_billing_allows_null_rec_billing_amount() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/links/conn-1/billing/monthly")
            .header("x-api-key", "test-key");
        then.status(200).header("content-type", "application/json").body(
            r#"{
                "success": true,
                "data": [{
                    "billing_month": "2025-03",
                    "smp_billing_amount": 3000000,
                    "rec_billing_amount": null
                }]
            }"#,
        );
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let response = client
        .get_monthly_billing("conn-1", MonthlyBillingParams::default())
        .expect("get_monthly_billing should succeed");

    mock.assert();
    let data = response
        .data
        .expect("billing response should contain data payload");
    assert_eq!(data.len(), 1);
    assert!(data[0].rec_billing_amount.is_none());
}

#[test]
fn list_connected_plants_treats_empty_body_as_success() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/links")
            .header("x-api-key", "test-key");
        then.status(204);
    });

    let client = OsolarClient::new("test-key")
        .with_base_url(server.base_url())
        .allow_insecure_http();
    let response = client
        .list_connected_plants()
        .expect("list_connected_plants should succeed on empty body");

    mock.assert();
    assert!(response.success);
    assert!(response.data.is_none());
}
