use osolar_client::models::{
    GeoPoint, PlantConnectionListResponse, PlantConnectionRequest, PlantConnectionResponse,
    PlantInfoResponse, PlantOwner, PlantOverviewResponse,
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
fn serialize_info_uses_link_id_wire_key() {
    let info = PlantInfoResponse {
        connection_id: "conn-200".to_string(),
        plant_name: "Plant".to_string(),
        plant_address: "Addr".to_string(),
        plant_capacity: "50".to_string(),
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
