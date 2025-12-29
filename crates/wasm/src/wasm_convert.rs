use crate::conversion::ToJs;
use crate::portable::PortableError;
use edge_rules::runtime::edge_rules::EdgeRulesModel;
use std::rc::Rc;
use wasm_bindgen::JsValue;

pub fn evaluate_all_inner(code: &str) -> Result<JsValue, PortableError> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(PortableError::from)?;
    let runtime = service.to_runtime().map_err(PortableError::from)?;
    runtime.eval_all().map_err(PortableError::from)?;
    let context = Rc::clone(&runtime.context);
    context.to_js().map_err(PortableError::from)
}

pub fn evaluate_expression_inner(code: &str) -> Result<JsValue, PortableError> {
    let mut service = EdgeRulesModel::new();
    let runtime = service.to_runtime_snapshot().map_err(PortableError::from)?;
    let value = runtime
        .evaluate_expression_str(code)
        .map_err(PortableError::from)?;
    value.to_js().map_err(PortableError::from)
}

pub fn evaluate_field_inner(code: &str, field: &str) -> Result<JsValue, PortableError> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(PortableError::from)?;
    let runtime = service.to_runtime().map_err(PortableError::from)?;
    let value = runtime.evaluate_field(field).map_err(PortableError::from)?;
    value.to_js().map_err(PortableError::from)
}
