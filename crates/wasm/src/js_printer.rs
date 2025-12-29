use crate::portable::PortableError;
use edge_js::{to_js_expression, to_js_model};
use edge_rules::runtime::edge_rules::EdgeRulesModel;

pub fn expression_to_js(code: &str) -> Result<String, PortableError> {
    let expr = EdgeRulesModel::parse_expression(code).map_err(PortableError::from)?;
    Ok(to_js_expression(&expr))
}

pub fn model_to_js(code: &str) -> Result<String, PortableError> {
    let mut model = EdgeRulesModel::new();
    model.append_source(code).map_err(PortableError::from)?;
    to_js_model(&mut model).map_err(PortableError::new)
}
