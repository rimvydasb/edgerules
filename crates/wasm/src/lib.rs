#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

mod wasm_convert;
mod wasm_portable;

use edge_rules::typesystem::values::ValueEnum;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_portable::{DecisionServiceController, PortableError};

thread_local! {
    static DECISION_SERVICE: RefCell<Option<DecisionServiceController>> = RefCell::new(None);
}

fn set_decision_service(controller: DecisionServiceController) {
    DECISION_SERVICE.with(|slot| {
        *slot.borrow_mut() = Some(controller);
    });
}

fn with_decision_service<R>(
    f: impl FnOnce(&mut DecisionServiceController) -> Result<R, PortableError>,
) -> Result<R, PortableError> {
    DECISION_SERVICE.with(|slot| {
        let mut guard = slot.borrow_mut();
        let Some(controller) = guard.as_mut() else {
            return Err(PortableError::new(
                "Decision service is not initialized. Call create_decision_service first.",
            ));
        };
        f(controller)
    })
}

fn throw_portable_error(err: PortableError) -> ! {
    wasm_convert::throw_js_error(err.into_message())
}

fn js_request_to_value(js: &JsValue) -> Result<ValueEnum, PortableError> {
    wasm_convert::js_to_value(js).map_err(PortableError::new)
}

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
#[cfg(all(not(feature = "console_error_panic_hook")))]
#[wasm_bindgen]
pub fn init_panic_hook() {}

#[wasm_bindgen]
pub fn evaluate_all(code: &str) -> JsValue {
    match wasm_convert::evaluate_all_inner(code) {
        Ok(value) => value,
        Err(err) => wasm_convert::throw_js_error(err),
    }
}

#[wasm_bindgen]
pub fn evaluate_expression(code: &str) -> JsValue {
    match wasm_convert::evaluate_expression_inner(code) {
        Ok(value) => value,
        Err(err) => wasm_convert::throw_js_error(err),
    }
}

#[wasm_bindgen]
pub fn evaluate_field(code: &str, field: &str) -> JsValue {
    match wasm_convert::evaluate_field_inner(code, field) {
        Ok(value) => value,
        Err(err) => wasm_convert::throw_js_error(err),
    }
}

#[wasm_bindgen]
pub fn evaluate_method(code: &str, method: &str, args: &JsValue) -> JsValue {
    match wasm_convert::evaluate_method_inner(code, method, args) {
        Ok(value) => value,
        Err(err) => wasm_convert::throw_js_error(err),
    }
}

#[wasm_bindgen]
pub fn create_decision_service(model: &JsValue) -> JsValue {
    let controller = match DecisionServiceController::from_portable(model) {
        Ok(ctrl) => ctrl,
        Err(err) => throw_portable_error(err),
    };
    set_decision_service(controller);
    let snapshot = match with_decision_service(|svc| svc.model_snapshot()) {
        Ok(value) => value,
        Err(err) => throw_portable_error(err),
    };
    snapshot
}

#[wasm_bindgen]
pub fn execute_decision_service(service_method: &str, decision_request: &JsValue) -> JsValue {
    let response = match with_decision_service(|svc| {
        let request = js_request_to_value(decision_request)?;
        svc.execute_value(service_method, request)
    }) {
        Ok(value) => value,
        Err(err) => throw_portable_error(err),
    };
    match wasm_convert::value_to_js(&response) {
        Ok(js) => js,
        Err(err) => wasm_convert::throw_js_error(err.to_string()),
    }
}

#[wasm_bindgen]
pub fn get_decision_service_model() -> JsValue {
    let snapshot = match with_decision_service(|svc| svc.model_snapshot()) {
        Ok(value) => value,
        Err(err) => throw_portable_error(err),
    };
    snapshot
}

#[wasm_bindgen]
pub fn set_to_decision_service_model(path: &str, object: &JsValue) -> JsValue {
    let updated = match with_decision_service(|svc| svc.set_entry(path, object)) {
        Ok(value) => value,
        Err(err) => throw_portable_error(err),
    };
    updated
}

#[wasm_bindgen]
pub fn set_invocation(path: &str, invocation: &JsValue) -> JsValue {
    let updated = match with_decision_service(|svc| svc.set_invocation(path, invocation)) {
        Ok(value) => value,
        Err(err) => throw_portable_error(err),
    };
    updated
}

#[wasm_bindgen]
pub fn remove_from_decision_service_model(path: &str) -> JsValue {
    match with_decision_service(|svc| svc.remove_entry(path)) {
        Ok(_) => JsValue::from_bool(true),
        Err(err) => throw_portable_error(err),
    }
}

#[wasm_bindgen]
pub fn get_from_decision_service_model(path: &str) -> JsValue {
    let portable = match with_decision_service(|svc| svc.get_entry(path)) {
        Ok(value) => value,
        Err(err) => throw_portable_error(err),
    };
    portable
}
