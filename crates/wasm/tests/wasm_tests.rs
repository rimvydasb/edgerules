#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_smoke_evaluates_expression() {
    let result = edge_rules_wasi::evaluate_expression("2 + 3");
    assert_eq!(result.as_f64(), Some(5.0));
}

// Example JsValue assertion on the exported API.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn wasm_evaluate_all_returns_context() {
    let ctx = edge_rules_wasi::evaluate_all("{ x: 2 + 3; y: 7 }");
    // Use JS reflection to read fields from the returned object.
    let x = js_sys::Reflect::get(&ctx, &wasm_bindgen::JsValue::from_str("x"))
        .expect("get x")
        .as_f64();
    let y = js_sys::Reflect::get(&ctx, &wasm_bindgen::JsValue::from_str("y"))
        .expect("get y")
        .as_f64();
    assert_eq!(x, Some(5.0));
    assert_eq!(y, Some(7.0));
}
