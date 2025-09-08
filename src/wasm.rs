#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen::prelude::*;

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
pub fn evaluate_all(code: &str) -> String {
    let service = EdgeRules::new();
    service.evaluate_all(code)
}

#[wasm_bindgen]
pub fn evaluate_expression(code: &str) -> String {
    let mut service = EdgeRules::new();
    match service.evaluate_expression(code) {
        Ok(v) => v.to_string(),
        Err(e) => e.to_string(),
    }
}

#[wasm_bindgen]
pub fn evaluate_field(code: &str, field: &str) -> String {
    let mut service = EdgeRules::new();
    match service.load_source(code) {
        Ok(()) => service.evaluate_field(field),
        Err(e) => e.to_string(),
    }
}
