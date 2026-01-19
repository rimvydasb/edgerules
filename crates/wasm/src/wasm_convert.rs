use crate::conversion::ToJs;
use crate::portable::model::model_from_portable;
use crate::portable::PortableError;
use edge_rules::runtime::edge_rules::EdgeRulesModel;
use std::rc::Rc;
use wasm_bindgen::JsValue;

pub fn evaluate_inner(input: &JsValue, field: Option<String>) -> Result<JsValue, PortableError> {
    if let Some(code) = input.as_string() {
        let trimmed = code.trim();
        let is_model = trimmed.starts_with('{') && trimmed.ends_with('}');

        if is_model {
            let mut service = EdgeRulesModel::new();
            service.append_source(&code).map_err(PortableError::from)?;
            let runtime = service.to_runtime().map_err(PortableError::from)?;

            if let Some(f) = field {
                let value = runtime.evaluate_field(&f).map_err(PortableError::from)?;
                value.to_js().map_err(PortableError::from)
            } else {
                runtime.eval_all().map_err(PortableError::from)?;
                let context = Rc::clone(&runtime.context);
                context.to_js().map_err(PortableError::from)
            }
        } else {
            if field.is_some() {
                return Err(PortableError::new(
                    "Field path is not applicable for single expression",
                ));
            }
            let mut service = EdgeRulesModel::new();
            let runtime = service.to_runtime_snapshot().map_err(PortableError::from)?;
            let value = runtime
                .evaluate_expression_str(&code)
                .map_err(PortableError::from)?;
            value.to_js().map_err(PortableError::from)
        }
    } else {
        let mut service = model_from_portable(input)?;
        let runtime = service.to_runtime().map_err(PortableError::from)?;

        if let Some(f) = field {
            let value = runtime.evaluate_field(&f).map_err(PortableError::from)?;
            value.to_js().map_err(PortableError::from)
        } else {
            runtime.eval_all().map_err(PortableError::from)?;
            let context = Rc::clone(&runtime.context);
            context.to_js().map_err(PortableError::from)
        }
    }
}
