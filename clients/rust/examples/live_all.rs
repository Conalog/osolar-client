use osolar_link_client::ApiError;
use osolar_link_client::OsolarLinkClient;
use osolar_link_client::models::{
    MonthlyBillingParams, MonthlyGenerationParams, PlantLinkRequest, SearchPlantsParams,
};
use serde_json::{Map, Value, json};

fn main() {
    let api_key = std::env::var("OSOLAR_API_KEY").unwrap_or_else(|_| {
        eprintln!("OSOLAR_API_KEY is required");
        std::process::exit(1);
    });

    let client = OsolarLinkClient::new(api_key);
    let mut results: Map<String, Value> = Map::new();

    let mut link_id: Option<String> = None;
    let mut search_keyword = "서울".to_string();
    let mut plant_uuid_for_link: Option<String> = None;

    match client.list_linked_plants() {
        Ok(resp) => {
            let mut linked_count = 0usize;
            if let Some(data) = &resp.data {
                linked_count = data.len();
                if let Some(first) = data.first() {
                    link_id = Some(first.link_id.clone());
                    if is_uuid(&first.link_id) {
                        plant_uuid_for_link = Some(first.link_id.clone());
                    }
                    if let Some(addr) = &first.plant_address {
                        if !addr.is_empty() {
                            search_keyword = addr.chars().take(12).collect();
                        }
                    }
                }
            }
            results.insert(
                "GET /v1/links".to_string(),
                json!({"ok": true, "linkedPlantCount": linked_count, "sampleLinkId": link_id}),
            );
        }
        Err(err) => {
            results.insert(
                "GET /v1/links".to_string(),
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
            if plant_uuid_for_link.is_none() {
                if let Some(data) = resp.data.as_ref() {
                    if let Some(first) = data.features.first() {
                        plant_uuid_for_link = Some(first.properties.plant_uuid.clone());
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

    match client.link_plant(&PlantLinkRequest {
        plant_uuid: plant_uuid_for_link.unwrap_or_else(|| "not-a-valid-uuid".to_string()),
        link_id: None,
        remark: "sdk live-all route smoke test".to_string(),
    }) {
        Ok(_) => {
            results.insert(
                "POST /v1/links".to_string(),
                json!({"ok": true, "note": "unexpectedly succeeded"}),
            );
        }
        Err(ApiError::Http { status, .. }) => {
            results.insert(
                "POST /v1/links".to_string(),
                json!({"ok": status >= 400, "status": status, "note": "non-2xx is acceptable for live route smoke"}),
            );
        }
        Err(err) => {
            results.insert(
                "POST /v1/links".to_string(),
                json!({"ok": false, "error": err.to_string()}),
            );
        }
    }

    let routes = vec![
        "GET /v1/links/{link_id}",
        "GET /v1/links/{link_id}/contract",
        "GET /v1/links/{link_id}/documents",
        "GET /v1/links/{link_id}/overview",
        "GET /v1/links/{link_id}/generation/monthly",
        "GET /v1/links/{link_id}/billing/monthly",
    ];

    for route in routes {
        let Some(id) = &link_id else {
            results.insert(
                route.to_string(),
                json!({"ok": false, "skipped": true, "reason": "no linked plant available"}),
            );
            continue;
        };

        let result = match route {
            "GET /v1/links/{link_id}" => client.get_plant_info(id).map(|_| ()),
            "GET /v1/links/{link_id}/contract" => client.get_plant_contract(id).map(|_| ()),
            "GET /v1/links/{link_id}/documents" => client.get_plant_documents(id).map(|_| ()),
            "GET /v1/links/{link_id}/overview" => client.get_plant_overview(id).map(|_| ()),
            "GET /v1/links/{link_id}/generation/monthly" => client
                .get_monthly_generation(id, MonthlyGenerationParams::default())
                .map(|_| ()),
            "GET /v1/links/{link_id}/billing/monthly" => client
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

fn is_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (idx, b) in bytes.iter().enumerate() {
        match idx {
            8 | 13 | 18 | 23 => {
                if *b != b'-' {
                    return false;
                }
            }
            _ => {
                if !(*b as char).is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}
