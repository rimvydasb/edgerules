#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

mod wasm_convert;

#[cfg(feature = "mutable_decision_service")]
use crate::runtime::portable::{DecisionServiceController, PortableError};
#[cfg(feature = "mutable_decision_service")]
#[cfg(feature = "mutable_decision_service")]
use serde_json::Value as PortableJson;
#[cfg(feature = "mutable_decision_service")]
use serde_wasm_bindgen::{from_value as serde_from_value, to_value as serde_to_value};
#[cfg(feature = "mutable_decision_service")]
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

// Inline JS glue to leverage host RegExp for regexReplace/regexSplit on Web/Node
// without pulling in the Rust regex crate (keeps WASM small).
#[wasm_bindgen(inline_js = r#"
export function __er_regex_replace(s, pattern, flags, repl) {
  try {
    const re = new RegExp(pattern, flags || 'g');
    return String(s).replace(re, repl);
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_regex_split(s, pattern, flags) {
  try {
    const re = new RegExp(pattern, flags || 'g');
    const SEP = "\u001F"; // Unit Separator as rarely-used delimiter
    const parts = String(s).split(re).map(p => p.split(SEP).join(SEP + SEP));
    return parts.join(SEP);
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_to_base64(s) {
  try {
    if (typeof btoa === 'function') {
      return btoa(String(s));
    }
    // Node.js
    return Buffer.from(String(s), 'utf-8').toString('base64');
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_from_base64(s) {
  try {
    if (typeof atob === 'function') {
      return atob(String(s));
    }
    // Node.js
    return Buffer.from(String(s), 'base64').toString('utf-8');
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}
"#)]
extern "C" {
    fn __er_regex_replace(s: &str, pattern: &str, flags: &str, repl: &str) -> String;
    fn __er_regex_split(s: &str, pattern: &str, flags: &str) -> String;
    fn __er_to_base64(s: &str) -> String;
    fn __er_from_base64(s: &str) -> String;
}

#[cfg(feature = "mutable_decision_service")]
thread_local! {
    static DECISION_SERVICE: RefCell<Option<DecisionServiceController>> = RefCell::new(None);
}

// Internal helper used by string functions to call into JS RegExp replace.
// Returns Err with a human-readable message if the pattern or flags are invalid.
pub(crate) fn regex_replace_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
    repl: &str,
) -> Result<String, String> {
    let f = flags.unwrap_or("g");
    let out = __er_regex_replace(s, pattern, f, repl);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

// Calls into JS RegExp split; returns vector of parts.
pub(crate) fn regex_split_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
) -> Result<Vec<String>, String> {
    let f = flags.unwrap_or("g");
    let out = __er_regex_split(s, pattern, f);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        // Split on the Unit Separator and collapse escaped separators
        let sep = '\u{001F}';
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut chars = out.chars().peekable();
        while let Some(c) = chars.next() {
            if c == sep {
                if let Some(next) = chars.peek() {
                    if *next == sep {
                        // Escaped separator -> emit one and consume the duplicate
                        current.push(sep);
                        chars.next();
                        continue;
                    }
                }
                // Segment boundary
                parts.push(current);
                current = String::new();
            } else {
                current.push(c);
            }
        }
        parts.push(current);
        Ok(parts)
    }
}

pub(crate) fn to_base64_js(s: &str) -> Result<String, String> {
    let out = __er_to_base64(s);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

pub(crate) fn from_base64_js(s: &str) -> Result<String, String> {
    let out = __er_from_base64(s);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

#[cfg(feature = "mutable_decision_service")]
fn set_decision_service(controller: DecisionServiceController) {
    DECISION_SERVICE.with(|slot| {
        *slot.borrow_mut() = Some(controller);
    });
}

#[cfg(feature = "mutable_decision_service")]
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

#[cfg(feature = "mutable_decision_service")]
fn js_to_portable(value: &JsValue) -> Result<PortableJson, PortableError> {
    serde_from_value(value.clone()).map_err(|err| PortableError::new(err.to_string()))
}

#[cfg(feature = "mutable_decision_service")]
fn portable_to_js(value: &PortableJson) -> Result<JsValue, PortableError> {
    serde_to_value(value).map_err(|err| PortableError::new(err.to_string()))
}

#[cfg(feature = "mutable_decision_service")]
fn throw_portable_error(err: PortableError) -> ! {
    wasm_convert::throw_js_error(err.into_message())
}

#[cfg(feature = "mutable_decision_service")]
fn js_request_to_value(
    js: &JsValue,
) -> Result<crate::typesystem::values::ValueEnum, PortableError> {
    wasm_convert::js_to_value(js).map_err(PortableError::new)
}

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
#[allow(unused_variables)]
pub fn create_decision_service(model: &JsValue) -> JsValue {
    #[cfg(feature = "mutable_decision_service")]
    {
        let portable = match js_to_portable(model) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        let controller = match DecisionServiceController::from_portable(&portable) {
            Ok(ctrl) => ctrl,
            Err(err) => throw_portable_error(err),
        };
        set_decision_service(controller);
        let snapshot = match with_decision_service(|svc| svc.model_snapshot()) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        return match portable_to_js(&snapshot) {
            Ok(js) => js,
            Err(err) => throw_portable_error(err),
        };
    }
    #[cfg(not(feature = "mutable_decision_service"))]
    {
        wasm_convert::throw_js_error(
            "Decision service API requires the 'mutable_decision_service' feature".to_string(),
        );
    }
}

#[wasm_bindgen]
#[allow(unused_variables)]
pub fn execute_decision_service(service_method: &str, decision_request: &JsValue) -> JsValue {
    #[cfg(feature = "mutable_decision_service")]
    {
        let response = match with_decision_service(|svc| {
            let request = js_request_to_value(decision_request)?;
            svc.execute_value(service_method, request)
        }) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        return match wasm_convert::value_to_js(&response) {
            Ok(js) => js,
            Err(err) => wasm_convert::throw_js_error(err.to_string()),
        };
    }
    #[cfg(not(feature = "mutable_decision_service"))]
    {
        wasm_convert::throw_js_error(
            "Decision service API requires the 'mutable_decision_service' feature".to_string(),
        );
    }
}

#[wasm_bindgen]
pub fn get_decision_service_model() -> JsValue {
    #[cfg(feature = "mutable_decision_service")]
    {
        let snapshot = match with_decision_service(|svc| svc.model_snapshot()) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        return match portable_to_js(&snapshot) {
            Ok(js) => js,
            Err(err) => throw_portable_error(err),
        };
    }
    #[cfg(not(feature = "mutable_decision_service"))]
    {
        wasm_convert::throw_js_error(
            "Decision service API requires the 'mutable_decision_service' feature".to_string(),
        );
    }
}

#[wasm_bindgen]
#[allow(unused_variables)]
pub fn set_to_decision_service_model(path: &str, object: &JsValue) -> JsValue {
    #[cfg(feature = "mutable_decision_service")]
    {
        let portable_payload = match js_to_portable(object) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        let updated = match with_decision_service(|svc| svc.set_entry(path, &portable_payload)) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        return match portable_to_js(&updated) {
            Ok(js) => js,
            Err(err) => throw_portable_error(err),
        };
    }
    #[cfg(not(feature = "mutable_decision_service"))]
    {
        wasm_convert::throw_js_error(
            "Decision service API requires the 'mutable_decision_service' feature".to_string(),
        );
    }
}

#[wasm_bindgen]
#[allow(unused_variables)]
pub fn remove_from_decision_service_model(path: &str) -> JsValue {
    #[cfg(feature = "mutable_decision_service")]
    {
        match with_decision_service(|svc| svc.remove_entry(path)) {
            Ok(_) => JsValue::from_bool(true),
            Err(err) => throw_portable_error(err),
        }
    }
    #[cfg(not(feature = "mutable_decision_service"))]
    {
        wasm_convert::throw_js_error(
            "Decision service API requires the 'mutable_decision_service' feature".to_string(),
        );
    }
}

#[wasm_bindgen]
#[allow(unused_variables)]
pub fn get_from_decision_service_model(path: &str) -> JsValue {
    #[cfg(feature = "mutable_decision_service")]
    {
        let portable = match with_decision_service(|svc| svc.get_entry(path)) {
            Ok(value) => value,
            Err(err) => throw_portable_error(err),
        };
        return match portable_to_js(&portable) {
            Ok(js) => js,
            Err(err) => throw_portable_error(err),
        };
    }
    #[cfg(not(feature = "mutable_decision_service"))]
    {
        wasm_convert::throw_js_error(
            "Decision service API requires the 'mutable_decision_service' feature".to_string(),
        );
    }
}
