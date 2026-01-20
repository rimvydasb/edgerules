#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = RegExp)]
    pub type HostRegExp;
    #[wasm_bindgen(constructor, js_class = "RegExp", catch)]
    fn new(pattern: &str, flags: &str) -> Result<HostRegExp, JsValue>;

    #[wasm_bindgen(js_name = Object)]
    pub type HostObject;
    #[wasm_bindgen(method, structural, catch)]
    fn replace(this: &HostObject, re: &HostRegExp, repl: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, structural, catch)]
    fn split(this: &HostObject, re: &HostRegExp) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(method, structural, js_name = toString, catch)]
    fn to_string_with_enc(this: &HostObject, enc: &str) -> Result<String, JsValue>;

    #[wasm_bindgen(catch)]
    fn btoa(s: &str) -> Result<String, JsValue>;
    #[wasm_bindgen(catch)]
    fn atob(s: &str) -> Result<String, JsValue>;

    #[wasm_bindgen(js_namespace = Buffer, js_name = from, catch)]
    fn buffer_from(s: &str, enc: &str) -> Result<HostObject, JsValue>;
}

pub(crate) fn regex_replace_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
    repl: &str,
) -> Result<String, String> {
    let f = flags.unwrap_or("g");
    let re = HostRegExp::new(pattern, f).map_err(|e| format!("{:?}", e))?;
    let s_js = JsValue::from_str(s);
    let host_s: &HostObject = s_js.unchecked_ref();
    let out = host_s.replace(&re, repl).map_err(|e| format!("{:?}", e))?;
    out.as_string()
        .ok_or_else(|| "replace did not return a string".to_string())
}

pub(crate) fn regex_split_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
) -> Result<Vec<String>, String> {
    let f = flags.unwrap_or("g");
    let re = HostRegExp::new(pattern, f).map_err(|e| format!("{:?}", e))?;
    let s_js = JsValue::from_str(s);
    let host_s: &HostObject = s_js.unchecked_ref();
    let array_val = host_s.split(&re).map_err(|e| format!("{:?}", e))?;

    let len_val = js_sys::Reflect::get(&array_val, &JsValue::from_str("length"))
        .map_err(|e| format!("{:?}", e))?;
    let len = len_val.as_f64().unwrap_or(0.0) as u32;

    let mut parts = Vec::with_capacity(len as usize);
    for i in 0..len {
        let p_val = js_sys::Reflect::get(&array_val, &JsValue::from_f64(i as f64))
            .map_err(|e| format!("{:?}", e))?;
        if let Some(p) = p_val.as_string() {
            parts.push(p);
        }
    }
    Ok(parts)
}

pub(crate) fn to_base64_js(s: &str) -> Result<String, String> {
    if let Ok(out) = btoa(s) {
        return Ok(out);
    }
    if let Ok(buf) = buffer_from(s, "utf-8") {
        if let Ok(out) = buf.to_string_with_enc("base64") {
            return Ok(out);
        }
    }
    Err("No base64 implementation found (btoa or Buffer)".to_string())
}

pub(crate) fn from_base64_js(s: &str) -> Result<String, String> {
    if let Ok(out) = atob(s) {
        return Ok(out);
    }
    if let Ok(buf) = buffer_from(s, "base64") {
        if let Ok(out) = buf.to_string_with_enc("utf-8") {
            return Ok(out);
        }
    }
    Err("No base64 implementation found (atob or Buffer)".to_string())
}
