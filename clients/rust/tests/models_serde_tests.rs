use osolar_client::models::{
    CapacityValue, GeoPoint, PlantConnectionListResponse, PlantConnectionRequest,
    PlantConnectionResponse, PlantContractResponse, PlantInfoResponse, PlantOverviewResponse,
    PlantOwner,
};
use serde_json::{json, Value};

#[test]
fn deserialize_response_maps_link_id_to_connection_id() {
    let raw = r#"{"link_id":"conn-123","created_at":"2024-05-16T14:12:00"}"#;
    let parsed: PlantConnectionResponse =
        serde_json::from_str(raw).expect("response should deserialize");
    assert_eq!(parsed.connection_id, "conn-123");
}

#[test]
fn serialize_request_maps_connection_id_to_link_id() {
    let request = PlantConnectionRequest {
        plant_uuid: "plant-uuid".to_string(),
        connection_id: Some("conn-777".to_string()),
        remark: "note".to_string(),
    };

    let json = serde_json::to_value(&request).expect("request should serialize");
    assert_eq!(json.get("link_id"), Some(&Value::String("conn-777".to_string())));
    assert!(json.get("connection_id").is_none());
}

#[test]
fn serialize_request_omits_link_id_when_connection_id_none() {
    let request = PlantConnectionRequest {
        plant_uuid: "plant-uuid".to_string(),
        connection_id: None,
        remark: "note".to_string(),
    };

    let json = serde_json::to_value(&request).expect("request should serialize");
    assert!(json.get("link_id").is_none());
}

#[test]
fn deserialize_list_response_maps_link_id_to_connection_id() {
    let raw = r#"{"link_id":"conn-456","plant_name":"Plant A"}"#;
    let parsed: PlantConnectionListResponse =
        serde_json::from_str(raw).expect("list response should deserialize");
    assert_eq!(parsed.connection_id, "conn-456");
    assert_eq!(parsed.plant_name, "Plant A");
}

#[test]
fn deserialize_info_and_overview_map_link_id_to_connection_id() {
    let info = json!({
        "link_id": "conn-100",
        "plant_name": "Plant X",
        "plant_address": "Addr",
        "plant_capacity": "99.9",
        "plant_certified": true,
        "plant_geometry": {"type":"Point","coordinates":[127.0,37.0]},
        "plant_owner": {
            "business_number":"123",
            "firm_name":"Firm",
            "firm_address":"Firm Addr",
            "representative_name":"Rep"
        }
    });
    let overview = json!({
        "link_id":"conn-100",
        "plant_name":"Plant X",
        "billing_summary":[],
        "recent_tasks":[]
    });

    let parsed_info: PlantInfoResponse =
        serde_json::from_value(info).expect("info should deserialize");
    let parsed_overview: PlantOverviewResponse =
        serde_json::from_value(overview).expect("overview should deserialize");

    assert_eq!(parsed_info.connection_id, "conn-100");
    assert_eq!(parsed_overview.connection_id, "conn-100");
}

#[test]
fn deserialize_info_accepts_numeric_plant_capacity() {
    let info = json!({
        "link_id": "conn-300",
        "plant_name": "Plant Y",
        "plant_address": "Addr",
        "plant_capacity": 100.5,
        "plant_certified": true,
        "plant_geometry": {"type":"Point","coordinates":[127.0,37.0]},
        "plant_owner": {
            "business_number":"123",
            "firm_name":"Firm",
            "firm_address":"Firm Addr",
            "representative_name":"Rep"
        }
    });

    let parsed_info: PlantInfoResponse =
        serde_json::from_value(info).expect("numeric capacity should deserialize");
    match parsed_info.plant_capacity {
        CapacityValue::Number(value) => assert_eq!(value, 100.5),
        CapacityValue::Text(value) => panic!("expected numeric capacity, got {value}"),
    }
}

#[test]
fn serialize_info_uses_link_id_wire_key() {
    let info = PlantInfoResponse {
        connection_id: "conn-200".to_string(),
        plant_name: "Plant".to_string(),
        plant_address: "Addr".to_string(),
        plant_capacity: CapacityValue::Text("50".to_string()),
        plant_certified: false,
        plant_geometry: GeoPoint {
            point_type: Some("Point".to_string()),
            coordinates: vec![127.0, 37.0],
        },
        plant_owner: PlantOwner {
            business_number: "1".to_string(),
            firm_name: "Firm".to_string(),
            firm_address: "Addr".to_string(),
            representative_name: "Rep".to_string(),
        },
    };

    let json = serde_json::to_value(info).expect("info should serialize");
    assert_eq!(json.get("link_id"), Some(&Value::String("conn-200".to_string())));
    assert!(json.get("connection_id").is_none());
}

#[test]
fn deserialize_contract_response_accepts_single_rec_fixed_contract() {
    let raw = json!({
        "ppa_type": "한국전력공사",
        "rec_trade_type": "고정가격계약",
        "rec_fixed_contract": {
            "ess": true,
            "target": "동서발전",
            "price_type": "SMP+1REC*가중치",
            "price": 165000
        }
    });

    let parsed: PlantContractResponse =
        serde_json::from_value(raw).expect("contract should deserialize");
    assert_eq!(parsed.rec_contracts.len(), 1);
    assert_eq!(parsed.rec_contracts[0].target, "동서발전");
}

#[test]
fn deserialize_contract_response_accepts_rec_contracts_array() {
    let raw = json!({
        "ppa_type": "한국전력공사",
        "rec_trade_type": "고정가격계약",
        "rec_contracts": [{
            "ess": false,
            "target": "한수원"
        }]
    });

    let parsed: PlantContractResponse =
        serde_json::from_value(raw).expect("contract should deserialize");
    assert_eq!(parsed.rec_contracts.len(), 1);
    assert_eq!(parsed.rec_contracts[0].target, "한수원");
}

#[test]
fn deserialize_contract_response_allows_missing_contract_details() {
    let raw = json!({
        "ppa_type": "한국전력공사",
        "rec_trade_type": "현물시장"
    });

    let parsed: PlantContractResponse =
        serde_json::from_value(raw).expect("spot-market contract should deserialize");
    assert!(parsed.rec_contracts.is_empty());
}

#[test]
fn deserialize_contract_response_defaults_missing_ess_to_false() {
    let raw = json!({
        "ppa_type": "한국전력공사",
        "rec_trade_type": "고정가격계약",
        "rec_fixed_contract": {
            "target": "동서발전"
        }
    });

    let parsed: PlantContractResponse =
        serde_json::from_value(raw).expect("contract without ess should deserialize");
    assert_eq!(parsed.rec_contracts.len(), 1);
    assert!(!parsed.rec_contracts[0].ess);
}
