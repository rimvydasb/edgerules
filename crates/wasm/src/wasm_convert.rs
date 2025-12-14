use crate::conversion::{FromJs, ToJs};
use crate::portable::PortableError;
use edge_rules::ast::token::ExpressionEnum;
use edge_rules::runtime::edge_rules::EdgeRulesModel;
use edge_rules::typesystem::values::ValueEnum;
use js_sys::Array;
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
    let runtime = service
        .to_runtime_snapshot()
        .map_err(PortableError::from)?;
    let value = runtime
        .evaluate_expression_str(code)
        .map_err(PortableError::from)?;
    value.to_js().map_err(PortableError::from)
}

pub fn evaluate_field_inner(code: &str, field: &str) -> Result<JsValue, PortableError> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(PortableError::from)?;
    let runtime = service.to_runtime().map_err(PortableError::from)?;
    let value = runtime
        .evaluate_field(field)
        .map_err(PortableError::from)?;
    value.to_js().map_err(PortableError::from)
}

pub fn evaluate_method_inner(code: &str, method: &str, args: &JsValue) -> Result<JsValue, PortableError> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(PortableError::from)?;
    let runtime = service.to_runtime().map_err(PortableError::from)?;
    let expr_args = js_args_to_expressions(args).map_err(PortableError::new)?;
    let value = runtime
        .call_method(method, expr_args)
        .map_err(PortableError::from)?;
    value.to_js().map_err(PortableError::from)
}

fn js_args_to_expressions(args: &JsValue) -> Result<Vec<ExpressionEnum>, String> {
    if args.is_undefined() || args.is_null() {
        return Ok(Vec::new());
    }

    if Array::is_array(args) {
        let array = Array::from(args);
        let mut expressions = Vec::with_capacity(array.length() as usize);
        for item in array.iter() {
            let value = ValueEnum::from_js(&item)?;
            expressions.push(ExpressionEnum::from(value));
        }
        Ok(expressions)
    } else {
        let value = ValueEnum::from_js(args)?;
        Ok(vec![ExpressionEnum::from(value)])
    }
}