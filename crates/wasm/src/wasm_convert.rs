use crate::conversion::ToJs;
use crate::portable::model::model_from_portable;
use crate::portable::PortableError;
use edge_rules::ast::metaphors::metaphor::UserFunction;
use edge_rules::ast::token::ExpressionEnum;
use edge_rules::ast::user_function_call::UserFunctionCall;
use edge_rules::runtime::edge_rules::EdgeRulesModel;
use edge_rules::typesystem::types::TypedValue;
use std::rc::Rc;
use wasm_bindgen::JsValue;

fn execute_or_evaluate(
    service: EdgeRulesModel,
    field: Option<String>,
) -> Result<JsValue, PortableError> {
    if let Some(f) = &field {
        if let Ok(method) = service.get_user_function(f) {
            if !method
                .borrow()
                .function_definition
                .get_parameters()
                .is_empty()
            {
                return Err(PortableError::new(format!(
                    "Function '{}' requires arguments and cannot be evaluated via evaluate. Use DecisionService instead.",
                    f
                )));
            }

            let definition_result = method
                .borrow()
                .function_definition
                .create_context(vec![], None);

            let runtime = service.to_runtime().map_err(PortableError::from)?;

            let mut call = UserFunctionCall::new(f.clone(), vec![]);
            match definition_result {
                Ok(def) => {
                    call.return_type = Ok(def.get_type());
                    call.definition = Ok(def);
                }
                Err(err) => return Err(PortableError::from(err)),
            }

            let value = runtime
                .evaluate_expression(ExpressionEnum::from(call))
                .map_err(PortableError::from)?;
            return value.to_js().map_err(PortableError::from);
        }
    }

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

pub fn evaluate_inner(input: &JsValue, field: Option<String>) -> Result<JsValue, PortableError> {
    if let Some(code) = input.as_string() {
        let trimmed = code.trim();
        let is_model = trimmed.starts_with('{') && trimmed.ends_with('}');

        if is_model {
            let mut service = EdgeRulesModel::new();
            service.append_source(&code).map_err(PortableError::from)?;
            execute_or_evaluate(service, field)
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
        let service = model_from_portable(input)?;
        execute_or_evaluate(service, field)
    }
}
