#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen::prelude::*;

use crate::code_to_trace;
use crate::runtime::edge_rules::EdgeRules;

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Provide a stable no-op export when the panic hook feature is disabled,
// so JS/TS that calls `init_panic_hook()` does not break in release builds.
#[cfg(all(not(feature = "console_error_panic_hook")))]
#[wasm_bindgen]
pub fn init_panic_hook() {
    // no-op
}

#[wasm_bindgen]
pub fn to_trace(code: &str) -> String {
    code_to_trace(code)
}

#[wasm_bindgen]
pub fn evaluate_value(code: &str) -> String {
    evaluate_field(code, "value")
}

#[wasm_bindgen]
pub fn evaluate_field(code: &str, field: &str) -> String {
    let mut service = EdgeRules::new();
    match service.load_source(code) {
        Ok(()) => match service.to_runtime() {
            Ok(runtime) => match runtime.evaluate_field(field) {
                Ok(v) => v.to_string(),
                Err(e) => e.to_string(),
            },
            Err(e) => e.to_string(),
        },
        Err(e) => e.to_string(),
    }
}
