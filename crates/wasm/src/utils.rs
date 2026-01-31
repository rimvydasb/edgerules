use js_sys::{Array, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

pub fn set_prop(target: &JsValue, key: &str, value: &JsValue) -> Result<bool, String> {
    Reflect::set(target, &JsValue::from_str(key), value).map_err(|_| format!("Failed to set property '{}'", key))
}

pub fn get_prop(target: &JsValue, key: &str) -> Option<JsValue> {
    Reflect::get(target, &JsValue::from_str(key)).ok().filter(|v| !v.is_undefined())
}

pub fn is_object(v: &JsValue) -> bool {
    v.is_object() && !Array::is_array(v)
}

pub fn throw_js_error(message: impl Into<String>) -> ! {
    wasm_bindgen::throw_str(&message.into());
}

pub fn throw_js_value(value: JsValue) -> ! {
    wasm_bindgen::throw_val(value);
}

pub fn js_to_object(v: &JsValue) -> Result<Object, String> {
    v.clone().dyn_into().map_err(|_| "Value is not an object".to_string())
}

pub fn js_to_array(v: &JsValue) -> Result<Array, String> {
    if Array::is_array(v) {
        Ok(Array::from(v))
    } else {
        Err("Value is not an array".to_string())
    }
}
