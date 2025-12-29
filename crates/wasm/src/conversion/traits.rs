use edge_rules::typesystem::errors::RuntimeError;
use wasm_bindgen::JsValue;

pub trait ToJs {
    fn to_js(&self) -> Result<JsValue, RuntimeError>;
}

pub trait FromJs {
    fn from_js(js: &JsValue) -> Result<Self, String>
    where
        Self: Sized;
}
