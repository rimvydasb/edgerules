#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen::prelude::*;

use crate::runtime::edge_rules::EdgeRules;

// Inline JS glue to leverage host RegExp for regexReplace on Web/Node
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
    fn __er_to_base64(s: &str) -> String;
    fn __er_from_base64(s: &str) -> String;
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
