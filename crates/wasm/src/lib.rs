#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

mod conversion;
#[cfg(feature = "to_js")]
mod js_printer;
mod portable;
mod utils;
mod wasm_convert;

use conversion::{FromJs, ToJs};
use edge_rules::typesystem::values::ValueEnum;
use portable::{DecisionServiceController, PortableError};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

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
    utils::throw_js_value(err.to_js())
}

fn js_request_to_value(js: &JsValue) -> Result<ValueEnum, PortableError> {
    ValueEnum::from_js(js).map_err(|e| PortableError::new(e))
}

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
#[cfg(not(feature = "console_error_panic_hook"))]
#[wasm_bindgen]
pub fn init_panic_hook() {}

#[wasm_bindgen]
pub struct DecisionEngine;

#[wasm_bindgen]
impl DecisionEngine {
    #[wasm_bindgen]
    pub fn evaluate(input: &JsValue, field: Option<String>) -> JsValue {
        match wasm_convert::evaluate_inner(input, field) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    #[cfg(feature = "to_js")]
    #[wasm_bindgen(js_name = "printExpressionJs")]
    pub fn print_expression_js(code: &str) -> String {
        match js_printer::expression_to_js(code) {
            Ok(js) => js,
            Err(err) => throw_portable_error(err),
        }
    }

    #[cfg(feature = "to_js")]
    #[wasm_bindgen(js_name = "printModelJs")]
    pub fn print_model_js(code: &str) -> String {
        match js_printer::model_to_js(code) {
            Ok(js) => js,
            Err(err) => throw_portable_error(err),
        }
    }
}

#[wasm_bindgen]
pub struct DecisionService;

#[wasm_bindgen]
impl DecisionService {
    #[wasm_bindgen(constructor)]
    pub fn new(model: &JsValue) -> DecisionService {
        let controller = if let Some(source) = model.as_string() {
            match DecisionServiceController::from_source(&source) {
                Ok(ctrl) => ctrl,
                Err(err) => throw_portable_error(err),
            }
        } else {
            match DecisionServiceController::from_portable(model) {
                Ok(ctrl) => ctrl,
                Err(err) => throw_portable_error(err),
            }
        };

        set_decision_service(controller);
        DecisionService
    }

    pub fn execute(&self, method: &str, request: &JsValue) -> JsValue {
        let response = match with_decision_service(|svc| {
            let req_val = js_request_to_value(request)?;
            svc.execute_value(method, req_val)
        }) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        match response.to_js() {
            Ok(js) => js,
            Err(err) => utils::throw_js_error(err.to_string()),
        }
    }

    pub fn get(&self, path: &str) -> JsValue {
        match with_decision_service(|svc| svc.get_entry(path)) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    pub fn set(&self, path: &str, object: &JsValue) -> JsValue {
        match with_decision_service(|svc| svc.set_entry(path, object)) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }

    pub fn remove(&self, path: &str) -> bool {
        match with_decision_service(|svc| svc.remove_entry(path)) {
            Ok(_) => true,
            Err(err) => throw_portable_error(err),
        }
    }

    pub fn rename(&self, old_path: &str, new_path: &str) -> bool {
        match with_decision_service(|svc| svc.rename_entry(old_path, new_path)) {
            Ok(_) => true,
            Err(err) => throw_portable_error(err),
        }
    }

    #[wasm_bindgen(js_name = "getType")]
    pub fn get_type(&self, path: &str) -> JsValue {
        match with_decision_service(|svc| {
            let vt = svc.get_entry_type(path)?;
            vt.to_js().map_err(PortableError::from)
        }) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        }
    }
}
