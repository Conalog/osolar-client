use httpmock::Method::GET;
use httpmock::MockServer;
use osolar_link_client::client::OsolarLinkClient;
use osolar_link_client::error::ApiError;
use osolar_link_client::models::SearchPlantsParams;

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

    let client = OsolarLinkClient::new("test-key").with_base_url(server.base_url());
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

    let client = OsolarLinkClient::new("test-key").with_base_url(server.base_url());
    let err = client
        .list_linked_plants()
        .expect_err("list_linked_plants should fail");

    match err {
        ApiError::Http { status, .. } => assert_eq!(status, 403),
        other => panic!("expected ApiError::Http, got {other:?}"),
    }
}
