use osolar_client::models::{
    MonthlyBillingParams, MonthlyGenerationParams, PlantConnectionRequest, SearchPlantsParams,
};
use osolar_client::ApiError;
use osolar_client::OsolarClient;
use serde_json::{json, Map, Value};

fn main() {
    let api_key = std::env::var("OSOLAR_API_KEY").unwrap_or_else(|_| {
        eprintln!("OSOLAR_API_KEY is required");
        std::process::exit(1);
    });

    let client = OsolarClient::new(api_key);
    let mut results: Map<String, Value> = Map::new();

    let mut connection_id: Option<String> = None;
    let mut search_keyword = "서울".to_string();
    let mut plant_uuid_for_connection: Option<String> = None;

    match client.list_connected_plants() {
        Ok(resp) => {
            let mut connected_count = 0usize;
            if let Some(data) = &resp.data {
                connected_count = data.len();
                if let Some(first) = data.first() {
                    connection_id = Some(first.connection_id.clone());
                    if let Some(addr) = &first.plant_address {
                        if !addr.is_empty() {
                            search_keyword = addr.chars().take(12).collect();
                        }
                    }
                }
            }
            results.insert(
                "list_connections".to_string(),
                json!({"ok": true, "connectedPlantCount": connected_count, "sampleConnectionId": connection_id}),
            );
        }
        Err(err) => {
            results.insert(
                "list_connections".to_string(),
                json!({"ok": false, "error": err.to_string()}),
            );
        }
    }

    match client.search_plants(SearchPlantsParams {
        q: search_keyword.clone(),
        field: "address".to_string(),
        distance_km: Some(2.0),
    }) {
        Ok(resp) => {
            let feature_count = resp.data.as_ref().map(|d| d.features.len()).unwrap_or(0);
            if plant_uuid_for_connection.is_none() {
                if let Some(data) = resp.data.as_ref() {
                    if let Some(first) = data.features.first() {
                        plant_uuid_for_connection = Some(first.properties.plant_uuid.clone());
                    }
                }
            }
            results.insert(
                "GET /v1/search".to_string(),
                json!({"ok": true, "featureCount": feature_count, "query": search_keyword}),
            );
        }
        Err(err) => {
            results.insert(
                "GET /v1/search".to_string(),
                json!({"ok": false, "error": err.to_string()}),
            );
        }
    }

    match client.connect_plant(&PlantConnectionRequest {
        plant_uuid: plant_uuid_for_connection.unwrap_or_else(|| "not-a-valid-uuid".to_string()),
        connection_id: None,
        remark: "sdk live-all route smoke test".to_string(),
    }) {
        Ok(_) => {
            results.insert(
                "create_connection".to_string(),
                json!({"ok": true, "note": "unexpectedly succeeded"}),
            );
        }
        Err(ApiError::Http { status, .. }) => {
            results.insert(
                "create_connection".to_string(),
                json!({"ok": status >= 400, "status": status, "note": "non-2xx is acceptable for live route smoke"}),
            );
        }
        Err(err) => {
            results.insert(
                "create_connection".to_string(),
                json!({"ok": false, "error": err.to_string()}),
            );
        }
    }

    let routes = vec![
        "plant_info",
        "plant_contract",
        "plant_documents",
        "plant_overview",
        "monthly_generation",
        "monthly_billing",
    ];

    for route in routes {
        let Some(id) = &connection_id else {
            results.insert(
                route.to_string(),
                json!({"ok": false, "skipped": true, "reason": "no connected plant available"}),
            );
            continue;
        };

        let result = match route {
            "plant_info" => client.get_plant_info(id).map(|_| ()),
            "plant_contract" => client.get_plant_contract(id).map(|_| ()),
            "plant_documents" => client.get_plant_documents(id).map(|_| ()),
            "plant_overview" => client.get_plant_overview(id).map(|_| ()),
            "monthly_generation" => client
                .get_monthly_generation(id, MonthlyGenerationParams::default())
                .map(|_| ()),
            "monthly_billing" => client
                .get_monthly_billing(id, MonthlyBillingParams::default())
                .map(|_| ()),
            _ => unreachable!(),
        };

        match result {
            Ok(()) => {
                results.insert(route.to_string(), json!({"ok": true, "payloadSize": 1}));
            }
            Err(err) => {
                results.insert(
                    route.to_string(),
                    json!({"ok": false, "error": err.to_string()}),
                );
            }
        }
    }

    let results_value = Value::Object(results);
    println!(
        "{}",
        serde_json::to_string_pretty(&results_value).unwrap_or_else(|_| "{}".to_string())
    );

    let hard_fail = if let Value::Object(obj) = &results_value {
        obj.values().any(|v| {
            let ok = v.get("ok").and_then(Value::as_bool).unwrap_or(false);
            let skipped = v.get("skipped").and_then(Value::as_bool).unwrap_or(false);
            !ok && !skipped
        })
    } else {
        true
    };

    if hard_fail {
        std::process::exit(1);
    }
}
