use crate::utils;
use edge_rules::ast::context::duplicate_name_error::DuplicateNameError;
use edge_rules::runtime::edge_rules::{ContextQueryErrorEnum, EvalError, ParseErrors};
use edge_rules::typesystem::errors::{
    LinkingError, LinkingErrorEnum, ParseErrorEnum, RuntimeError, RuntimeErrorEnum,
};
use js_sys::{Array, Object};
use std::fmt::{Display, Formatter};
use wasm_bindgen::JsValue;

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum PortableError {
    FromContextQuery(ContextQueryErrorEnum),
    General { js_value: JsValue, message: String },
}

impl PortableError {
    pub fn new(message: impl Into<String>) -> Self {
        let msg = message.into();
        let obj = Object::new();
        let _ = utils::set_prop(&obj, "message", &JsValue::from_str(&msg));
        Self::General {
            js_value: obj.into(),
            message: msg,
        }
    }

    pub fn to_js(&self) -> JsValue {
        match self {
            PortableError::FromContextQuery(err) => Self::new(err.to_string()).to_js(),
            PortableError::General { js_value, .. } => js_value.clone(),
        }
    }
}

impl Display for PortableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PortableError::FromContextQuery(err) => write!(f, "{}", err),
            PortableError::General { message, .. } => write!(f, "{}", message),
        }
    }
}

impl From<PortableError> for String {
    fn from(v: PortableError) -> Self {
        v.to_string()
    }
}

impl From<ContextQueryErrorEnum> for PortableError {
    fn from(err: ContextQueryErrorEnum) -> Self {
        PortableError::FromContextQuery(err)
    }
}

impl From<DuplicateNameError> for PortableError {
    fn from(err: DuplicateNameError) -> Self {
        Self::new(err.to_string())
    }
}

impl From<ParseErrorEnum> for PortableError {
    fn from(err: ParseErrorEnum) -> Self {
        // @Todo: Implement structured parsing error
        let obj = Object::new();
        let _ = utils::set_prop(&obj, "stage", &JsValue::from_str("parse"));
        let _ = utils::set_prop(&obj, "message", &JsValue::from_str(&err.to_string()));

        PortableError::General {
            js_value: obj.into(),
            message: err.to_string(),
        }
    }
}

impl From<ParseErrors> for PortableError {
    fn from(err: ParseErrors) -> Self {
        let obj = Object::new();
        let _ = utils::set_prop(&obj, "stage", &JsValue::from_str("parse"));
        let _ = utils::set_prop(&obj, "message", &JsValue::from_str(&err.to_string()));

        PortableError::General {
            js_value: obj.into(),
            message: err.to_string(),
        }
    }
}

impl From<EvalError> for PortableError {
    fn from(err: EvalError) -> Self {
        Self::new(err.to_string())
    }
}

impl From<RuntimeError> for PortableError {
    fn from(err: RuntimeError) -> Self {
        let obj = Object::new();
        let _ = utils::set_prop(&obj, "stage", &JsValue::from_str("runtime"));
        let _ = utils::set_prop(&obj, "message", &JsValue::from_str(&err.to_string()));

        if !err.location().is_empty() {
            let loc_str = err.location().join(".");
            let _ = utils::set_prop(&obj, "location", &JsValue::from_str(&loc_str));
        }

        if let Some(expr) = err.expression() {
            let _ = utils::set_prop(&obj, "expression", &JsValue::from_str(expr));
        }

        let error_obj = Object::new();
        match err.kind() {
            RuntimeErrorEnum::RuntimeFieldNotFound(object, field) => {
                let _ = utils::set_prop(&error_obj, "type", &JsValue::from_str("FieldNotFound"));
                let fields = Array::new();
                fields.push(&JsValue::from_str(&object));
                fields.push(&JsValue::from_str(&field));
                let _ = utils::set_prop(&error_obj, "fields", &fields);
            }
            RuntimeErrorEnum::RuntimeCyclicReference(object, field) => {
                let _ = utils::set_prop(&error_obj, "type", &JsValue::from_str("CyclicReference"));
                let fields = Array::new();
                fields.push(&JsValue::from_str(&object));
                fields.push(&JsValue::from_str(&field));
                let _ = utils::set_prop(&error_obj, "fields", &fields);
            }
            RuntimeErrorEnum::EvalError(msg) => {
                let _ = utils::set_prop(&error_obj, "type", &JsValue::from_str("EvalError"));
                let _ = utils::set_prop(&error_obj, "message", &JsValue::from_str(&msg));
            }
            RuntimeErrorEnum::ValueParsingError(from, to, code) => {
                let _ = utils::set_prop(&error_obj, "type", &JsValue::from_str("ValueParsingError"));
                let _ = utils::set_prop(&error_obj, "from", &JsValue::from_str(&from.to_string()));
                let _ = utils::set_prop(&error_obj, "to", &JsValue::from_str(&to.to_string()));
                let _ = utils::set_prop(&error_obj, "code", &JsValue::from_f64(*code as f64));
                let msg = if *code > 0 {
                    format!("Failed to parse '{}' from '{}'. (Error code: {})", to, from, code)
                } else {
                    format!("Failed to parse '{}' from '{}'", to, from)
                };
                let _ = utils::set_prop(&error_obj, "message", &JsValue::from_str(&msg));
            }
            _ => {
                let _ =
                    utils::set_prop(&error_obj, "type", &JsValue::from_str("OtherRuntimeError"));
                let _ = utils::set_prop(
                    &error_obj,
                    "message",
                    &JsValue::from_str(&err.kind().to_string()),
                );
            }
        }
        let _ = utils::set_prop(&obj, "error", &error_obj);

        PortableError::General {
            js_value: obj.into(),
            message: err.to_string(),
        }
    }
}

impl From<LinkingError> for PortableError {
    fn from(err: LinkingError) -> Self {
        let obj = Object::new();
        let _ = utils::set_prop(&obj, "stage", &JsValue::from_str("linking"));
        let _ = utils::set_prop(&obj, "message", &JsValue::from_str(&err.to_string()));

        if !err.location().is_empty() {
            let loc_str = err.location().join(".");
            let _ = utils::set_prop(&obj, "location", &JsValue::from_str(&loc_str));
        }

        if let Some(expr) = err.expression() {
            let _ = utils::set_prop(&obj, "expression", &JsValue::from_str(expr));
        }

        let error_obj = Object::new();
        match err.kind() {
            LinkingErrorEnum::FieldNotFound(object, field) => {
                let _ = utils::set_prop(&error_obj, "type", &JsValue::from_str("FieldNotFound"));
                let fields = Array::new();
                fields.push(&JsValue::from_str(&object));
                fields.push(&JsValue::from_str(&field));
                let _ = utils::set_prop(&error_obj, "fields", &fields);
            }
            LinkingErrorEnum::TypesNotCompatible(subject, unexpected, expected) => {
                let _ =
                    utils::set_prop(&error_obj, "type", &JsValue::from_str("TypesNotCompatible"));
                if let Some(sub) = subject {
                    let _ = utils::set_prop(&error_obj, "subject", &JsValue::from_str(&sub));
                }
                let _ = utils::set_prop(
                    &error_obj,
                    "unexpected",
                    &JsValue::from_str(&unexpected.to_string()),
                );
                if let Some(exp) = expected {
                    let exp_arr = Array::new();
                    for ex in exp {
                        exp_arr.push(&JsValue::from_str(&ex.to_string()));
                    }
                    let _ = utils::set_prop(&error_obj, "expected", &exp_arr);
                }
            }
            LinkingErrorEnum::DifferentTypesDetected(subject, t1, t2) => {
                let _ = utils::set_prop(
                    &error_obj,
                    "type",
                    &JsValue::from_str("DifferentTypesDetected"),
                );
                if let Some(sub) = subject {
                    let _ = utils::set_prop(&error_obj, "subject", &JsValue::from_str(&sub));
                }
                let _ = utils::set_prop(&error_obj, "type1", &JsValue::from_str(&t1.to_string()));
                let _ = utils::set_prop(&error_obj, "type2", &JsValue::from_str(&t2.to_string()));
            }
            LinkingErrorEnum::FunctionNotFound { name, .. } => {
                let _ = utils::set_prop(&error_obj, "type", &JsValue::from_str("FunctionNotFound"));
                let _ = utils::set_prop(&error_obj, "name", &JsValue::from_str(&name));
            }
            LinkingErrorEnum::CyclicReference(object, field) => {
                let _ = utils::set_prop(&error_obj, "type", &JsValue::from_str("CyclicReference"));
                let fields = Array::new();
                fields.push(&JsValue::from_str(&object));
                fields.push(&JsValue::from_str(&field));
                let _ = utils::set_prop(&error_obj, "fields", &fields);
            }
            _ => {
                let _ =
                    utils::set_prop(&error_obj, "type", &JsValue::from_str("OtherLinkingError"));
                let _ = utils::set_prop(
                    &error_obj,
                    "message",
                    &JsValue::from_str(&err.kind().to_string()),
                );
            }
        }
        let _ = utils::set_prop(&obj, "error", &error_obj);

        PortableError::General {
            js_value: obj.into(),
            message: err.to_string(),
        }
    }
}
