use crate::utils;
use edge_rules::ast::context::duplicate_name_error::DuplicateNameError;
use edge_rules::runtime::edge_rules::{ContextQueryErrorEnum, EvalError, ParseErrors};
use edge_rules::typesystem::errors::{LinkingError, LinkingErrorEnum, ParseErrorEnum, RuntimeError, RuntimeErrorEnum};
use js_sys::{Array, Object};
use std::fmt::{Display, Formatter};
use wasm_bindgen::JsValue;

/// Lightweight builder for creating JS objects to reduce verbosity and size
struct JsBuilder(Object);

impl JsBuilder {
    fn new() -> Self {
        Self(Object::new())
    }

    fn add_str(self, key: &str, value: &str) -> Self {
        let _ = utils::set_prop(&self.0, key, &JsValue::from_str(value));
        self
    }

    fn add_f64(self, key: &str, value: f64) -> Self {
        let _ = utils::set_prop(&self.0, key, &JsValue::from_f64(value));
        self
    }

    fn add_val(self, key: &str, value: &JsValue) -> Self {
        let _ = utils::set_prop(&self.0, key, value);
        self
    }

    // Helper for common "type" property
    fn add_type(self, type_name: &str) -> Self {
        self.add_str("type", type_name)
    }

    // Helper for "fields" array [object, field]
    fn add_fields(self, object: &str, field: &str) -> Self {
        let fields = Array::new();
        fields.push(&JsValue::from_str(object));
        fields.push(&JsValue::from_str(field));
        self.add_val("fields", &fields)
    }

    // Helper for optional location array
    fn add_location(self, location: &[String]) -> Self {
        if !location.is_empty() {
            self.add_str("location", &location.join("."))
        } else {
            self
        }
    }

    // Helper for optional expression
    fn add_expression(self, expression: Option<&String>) -> Self {
        if let Some(expr) = expression {
            self.add_str("expression", expr)
        } else {
            self
        }
    }

    fn build(self) -> Object {
        self.0
    }

    fn into_js(self) -> JsValue {
        self.0.into()
    }
}

pub enum PortableObjectKey {
    Method,
    Arguments,
    Ref,
    Parameters,
    Version,
    ModelName,
    Type,
    Root,
    Invocation,
    Function,
    Value,
    Custom(String),
}

impl PortableObjectKey {
    pub fn as_str(&self) -> &str {
        match self {
            PortableObjectKey::Method => "@method",
            PortableObjectKey::Arguments => "@arguments",
            PortableObjectKey::Ref => "@ref",
            PortableObjectKey::Parameters => "@parameters",
            PortableObjectKey::Version => "@version",
            PortableObjectKey::ModelName => "@model_name",
            PortableObjectKey::Type => "@type",
            PortableObjectKey::Root => "(root)",
            PortableObjectKey::Invocation => "invocation",
            PortableObjectKey::Function => "function",
            PortableObjectKey::Value => "value",
            PortableObjectKey::Custom(s) => s.as_str(),
        }
    }
}

impl AsRef<str> for PortableObjectKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for PortableObjectKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub enum SchemaViolationType {
    MissingRequiredField,
    InvalidFieldType,
    InvalidFormat,
    Empty,
    NotSupported,
}

impl SchemaViolationType {
    pub fn as_str(&self) -> &str {
        match self {
            SchemaViolationType::MissingRequiredField => "Missing required field",
            SchemaViolationType::InvalidFieldType => "Invalid field type",
            SchemaViolationType::InvalidFormat => "Invalid format",
            SchemaViolationType::Empty => "Empty",
            SchemaViolationType::NotSupported => "Not supported",
        }
    }
}

impl Display for SchemaViolationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum PortableError {
    DecisionServiceError(String),
    EdgeRulesAPIError(ContextQueryErrorEnum),
    LinkingStage(LinkingError),
    ParsingStage(ParseErrors),
    RuntimeStage(RuntimeError),
    SerializationError(PortableObjectKey, SchemaViolationType),
    SchemaViolation(PortableObjectKey, SchemaViolationType),
}

impl PortableError {
    pub fn to_js(&self) -> JsValue {
        match self {
            PortableError::DecisionServiceError(msg) => {
                JsBuilder::new().add_type("DecisionServiceError").add_str("message", msg).into_js()
            }
            PortableError::EdgeRulesAPIError(err) => {
                JsBuilder::new().add_str("message", &err.to_string()).into_js()
            },
            PortableError::ParsingStage(err) => {
                JsBuilder::new().add_str("stage", "parse").add_str("message", &err.to_string()).into_js()
            }
            PortableError::RuntimeStage(err) => {
                let builder = JsBuilder::new()
                    .add_str("stage", "runtime")
                    .add_str("message", &err.to_string())
                    .add_location(&err.location())
                    .add_expression(err.expression());

                let error_obj = match err.kind() {
                    RuntimeErrorEnum::RuntimeFieldNotFound(object, field) => {
                        JsBuilder::new().add_type("FieldNotFound").add_fields(object, field).build()
                    }
                    RuntimeErrorEnum::RuntimeCyclicReference(object, field) => {
                        JsBuilder::new().add_type("CyclicReference").add_fields(object, field).build()
                    }
                    RuntimeErrorEnum::EvalError(msg) => {
                        JsBuilder::new().add_type("EvalError").add_str("message", msg).build()
                    }
                    RuntimeErrorEnum::ValueParsingError(from, to, code) => {
                        let msg = if *code > 0 {
                            format!("Failed to parse '{}' from '{}'. (Error code: {})", to, from, code)
                        } else {
                            format!("Failed to parse '{}' from '{}'", to, from)
                        };
                        JsBuilder::new()
                            .add_type("ValueParsingError")
                            .add_str("from", &from.to_string())
                            .add_str("to", &to.to_string())
                            .add_f64("code", *code as f64)
                            .add_str("message", &msg)
                            .build()
                    }
                    RuntimeErrorEnum::InternalIntegrityError(code) => JsBuilder::new()
                        .add_type("InternalIntegrityError")
                        .add_f64("code", *code as f64)
                        .add_str("message", &format!("Internal integrity error: code {}", code))
                        .build(),
                    _ => JsBuilder::new()
                        .add_type("OtherRuntimeError")
                        .add_str("message", &err.kind().to_string())
                        .build(),
                };
                builder.add_val("error", &error_obj.into()).into_js()
            }
            PortableError::LinkingStage(err) => {
                let builder = JsBuilder::new()
                    .add_str("stage", "linking")
                    .add_str("message", &err.to_string())
                    .add_location(&err.location())
                    .add_expression(err.expression());

                let error_obj = match err.kind() {
                    LinkingErrorEnum::FieldNotFound(object, field) => {
                        JsBuilder::new().add_type("FieldNotFound").add_fields(object, field).build()
                    }
                    LinkingErrorEnum::TypesNotCompatible(subject, unexpected, expected) => {
                        let mut b = JsBuilder::new().add_type("TypesNotCompatible");
                        if let Some(sub) = subject {
                            b = b.add_str("subject", sub);
                        }
                        b = b.add_str("unexpected", &unexpected.to_string());
                        if let Some(exp) = expected {
                            let exp_arr = Array::new();
                            for ex in exp {
                                exp_arr.push(&JsValue::from_str(&ex.to_string()));
                            }
                            b = b.add_val("expected", &exp_arr);
                        }
                        b.build()
                    }
                    LinkingErrorEnum::DifferentTypesDetected(subject, t1, t2) => {
                        let mut b = JsBuilder::new().add_type("DifferentTypesDetected");
                        if let Some(sub) = subject {
                            b = b.add_str("subject", sub);
                        }
                        b.add_str("type1", &t1.to_string()).add_str("type2", &t2.to_string()).build()
                    }
                    LinkingErrorEnum::FunctionNotFound { name, .. } => {
                        JsBuilder::new().add_type("FunctionNotFound").add_str("name", name).build()
                    }
                    LinkingErrorEnum::CyclicReference(object, field) => {
                        JsBuilder::new().add_type("CyclicReference").add_fields(object, field).build()
                    }
                    _ => JsBuilder::new()
                        .add_type("OtherLinkingError")
                        .add_str("message", &err.kind().to_string())
                        .build(),
                };
                builder.add_val("error", &error_obj.into()).into_js()
            }
            PortableError::SerializationError(key, violation) => JsBuilder::new()
                .add_type("SerializationError")
                .add_str("key", key.as_str())
                .add_str("violation", violation.as_str())
                .into_js(),
            PortableError::SchemaViolation(key, violation) => JsBuilder::new()
                .add_type("SchemaViolation")
                .add_str("key", key.as_str())
                .add_str("violation", violation.as_str())
                .into_js(),
        }
    }
}

impl From<ContextQueryErrorEnum> for PortableError {
    fn from(err: ContextQueryErrorEnum) -> Self {
        PortableError::EdgeRulesAPIError(err)
    }
}

impl From<DuplicateNameError> for PortableError {
    fn from(err: DuplicateNameError) -> Self {
        PortableError::EdgeRulesAPIError(ContextQueryErrorEnum::DuplicateNameError(err))
    }
}

impl From<ParseErrorEnum> for PortableError {
    fn from(err: ParseErrorEnum) -> Self {
        PortableError::ParsingStage(ParseErrors::from(err))
    }
}

impl From<ParseErrors> for PortableError {
    fn from(err: ParseErrors) -> Self {
        PortableError::ParsingStage(err)
    }
}

impl From<EvalError> for PortableError {
    fn from(err: EvalError) -> Self {
        match err {
            EvalError::FailedParsing(errors) => PortableError::ParsingStage(errors),
            EvalError::FailedExecution(error) => PortableError::RuntimeStage(error),
        }
    }
}

impl From<RuntimeError> for PortableError {
    fn from(err: RuntimeError) -> Self {
        PortableError::RuntimeStage(err)
    }
}

impl From<LinkingError> for PortableError {
    fn from(err: LinkingError) -> Self {
        PortableError::LinkingStage(err)
    }
}
