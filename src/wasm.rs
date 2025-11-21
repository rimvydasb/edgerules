#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

mod wasm_convert;
mod wasm_portable;

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_portable::{DecisionServiceController, PortableError};

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

thread_local! {
    static DECISION_SERVICE: RefCell<Option<DecisionServiceController>> = RefCell::new(None);
}

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
        let sep = '\u{001F}';
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut chars = out.chars().peekable();
        while let Some(c) = chars.next() {
            if c == sep {
                if let Some(next) = chars.peek() {
                    if *next == sep {
                        current.push(sep);
                        chars.next();
                        continue;
                    }
                }
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
