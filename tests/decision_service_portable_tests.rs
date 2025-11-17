#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use edge_rules::runtime::edge_rules::EdgeRulesModel;
use edge_rules::runtime::portable::DecisionServiceController;
use edge_rules::test_support::ValueEnum;
use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;

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

fn obj() -> Object {
    Object::new()
}
fn set(obj: &Object, k: &str, v: &JsValue) {
    let _ = Reflect::set(obj, &JsValue::from_str(k), v);
}

#[test]
fn portable_controller_executes_requests() {
    let portable = {
        let root = obj();
        // Request type
        let request = obj();
        set(&request, "@type", &JsValue::from_str("type"));
        set(&request, "amount", &JsValue::from_str("<number>"));
        set(&root, "Request", &JsValue::from(request.clone()));
        // decide function
        let decide = obj();
        set(&decide, "@type", &JsValue::from_str("function"));
        let params = obj();
        set(&params, "request", &JsValue::from_str("Request"));
        set(&decide, "@parameters", &JsValue::from(params.clone()));
        set(
            &decide,
            "decision",
            &JsValue::from_str("request.amount * 2"),
        );
        set(&root, "decide", &JsValue::from(decide.clone()));
        JsValue::from(root)
    };

    let mut controller =
        DecisionServiceController::from_portable(&portable).expect("controller from portable");
    let request = build_request_value("{ amount: 10 }");
    let response = controller
        .execute_value("decide", request)
        .expect("execute portable controller");
    assert!(value_to_string(&response).contains("decision:20"));
}

#[test]
fn portable_controller_updates_entries() {
    let portable = {
        let root = obj();
        let config = obj();
        set(&config, "initial", &JsValue::from_str("1"));
        set(&root, "config", &JsValue::from(config));
        let decide = obj();
        set(&decide, "@type", &JsValue::from_str("function"));
        let params = obj();
        set(&params, "request", &JsValue::from_str("Request"));
        set(&decide, "@parameters", &JsValue::from(params));
        set(&decide, "result", &JsValue::from_str("request.amount"));
        set(&root, "decide", &JsValue::from(decide));
        let request = obj();
        set(&request, "@type", &JsValue::from_str("type"));
        set(&request, "amount", &JsValue::from_str("<number>"));
        set(&root, "Request", &JsValue::from(request));
        JsValue::from(root)
    };

    let mut controller =
        DecisionServiceController::from_portable(&portable).expect("controller from portable");

    let updated = controller
        .set_entry("config.threshold", &JsValue::from_f64(5.0))
        .expect("set new config value");
    assert_eq!(updated.as_f64(), Some(5.0));

    let entry = controller.get_entry("config.threshold").expect("get entry");
    assert_eq!(entry.as_f64(), Some(5.0));

    controller
        .remove_entry("config.threshold")
        .expect("remove entry");
    let err = controller.get_entry("config.threshold").unwrap_err();
    assert!(err.to_string().to_lowercase().contains("not found"));
}

#[test]
fn portable_controller_serializes_snapshot() {
    let portable = {
        let root = obj();
        let request = obj();
        set(&request, "@type", &JsValue::from_str("type"));
        set(&request, "amount", &JsValue::from_str("<number>"));
        set(&root, "Request", &JsValue::from(request));
        let config = obj();
        set(&config, "featureEnabled", &JsValue::from_str("false"));
        set(&root, "config", &JsValue::from(config));
        let decide = obj();
        set(&decide, "@type", &JsValue::from_str("function"));
        let params = obj();
        set(&params, "request", &JsValue::from_str("Request"));
        set(&decide, "@parameters", &JsValue::from(params));
        set(&decide, "decision", &JsValue::from_str("request.amount"));
        set(&root, "decide", &JsValue::from(decide));
        JsValue::from(root)
    };

    let mut controller =
        DecisionServiceController::from_portable(&portable).expect("controller from portable");

    let _ = controller
        .set_entry("config.featureEnabled", &JsValue::from_bool(true))
        .expect("set config value");

    let snapshot = controller
        .model_snapshot()
        .expect("snapshot portable model");

    // Check that Request has @type == "type"
    let request = js_sys::Reflect::get(&snapshot, &JsValue::from_str("Request")).unwrap();
    let kind = js_sys::Reflect::get(&request, &JsValue::from_str("@type")).unwrap();
    assert_eq!(kind.as_string(), Some("type".to_string()));

    // Check that config.featureEnabled == true
    let config = js_sys::Reflect::get(&snapshot, &JsValue::from_str("config")).unwrap();
    let feature = js_sys::Reflect::get(&config, &JsValue::from_str("featureEnabled")).unwrap();
    assert_eq!(feature.as_bool(), Some(true));
}
