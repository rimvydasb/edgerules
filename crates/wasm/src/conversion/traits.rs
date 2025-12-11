use wasm_bindgen::JsValue;
use edge_rules::typesystem::errors::RuntimeError;

pub trait ToJs {
    fn to_js(&self) -> Result<JsValue, RuntimeError>;
}

pub trait FromJs {
    fn from_js(js: &JsValue) -> Result<Self, String>
    where
        Self: Sized;
}
