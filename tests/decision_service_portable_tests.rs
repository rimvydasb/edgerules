use edge_rules::runtime::edge_rules::EdgeRulesModel;
use edge_rules::runtime::portable::DecisionServiceController;
use edge_rules::test_support::ValueEnum;
use serde_json::json;

fn build_request_value(source: &str) -> ValueEnum {
    let mut model = EdgeRulesModel::new();
    let payload = format!("{{ requestData: {} }}", source.trim());
    model
        .append_source(&payload)
        .expect("request object should parse");
    let runtime = model.to_runtime().expect("request object should link");
    runtime
        .evaluate_field("requestData")
        .expect("request field should evaluate")
}

fn value_to_string(value: &ValueEnum) -> String {
    value.to_string().replace('\n', "").replace(' ', "")
}

#[test]
fn portable_controller_executes_requests() {
    let portable = json!({
        "Request": {
            "@type": "type",
            "amount": "<number>"
        },
        "decide": {
            "@type": "function",
            "@parameters": { "request": "Request" },
            "decision": "request.amount * 2"
        }
    });

    let mut controller =
        DecisionServiceController::from_portable(&portable).expect("controller from portable json");
    let request = build_request_value("{ amount: 10 }");
    let response = controller
        .execute_value("decide", request)
        .expect("execute portable controller");
    assert!(value_to_string(&response).contains("decision:20"));
}

#[test]
fn portable_controller_updates_entries() {
    let portable = json!({
        "config": {
            "initial": "1"
        },
        "decide": {
            "@type": "function",
            "@parameters": { "request": "Request" },
            "result": "request.amount"
        },
        "Request": {
            "@type": "type",
            "amount": "<number>"
        }
    });

    let mut controller =
        DecisionServiceController::from_portable(&portable).expect("controller from portable json");

    let updated = controller
        .set_entry("config.threshold", &json!(5))
        .expect("set new config value");
    assert_eq!(updated, json!(5));

    let entry = controller
        .get_entry("config.threshold")
        .expect("get previously stored entry");
    assert_eq!(entry, json!(5));

    controller
        .remove_entry("config.threshold")
        .expect("remove entry");
    let err = controller.get_entry("config.threshold").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("not found"));
}

#[test]
fn portable_controller_serializes_snapshot() {
    let portable = json!({
        "Request": {
            "@type": "type",
            "amount": "<number>"
        },
        "config": {
            "featureEnabled": "false"
        },
        "decide": {
            "@type": "function",
            "@parameters": { "request": "Request" },
            "decision": "request.amount"
        }
    });

    let mut controller =
        DecisionServiceController::from_portable(&portable).expect("controller from portable json");

    controller
        .set_entry("config.featureEnabled", &json!(true))
        .expect("set config value");

    let snapshot = controller
        .model_snapshot()
        .expect("snapshot portable model");

    assert_eq!(
        snapshot.get("Request").and_then(|v| v.get("@type")),
        Some(&json!("type"))
    );
    assert_eq!(
        snapshot.get("config").and_then(|v| v.get("featureEnabled")),
        Some(&json!(true))
    );
}
