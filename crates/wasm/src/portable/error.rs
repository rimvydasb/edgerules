use edge_rules::ast::context::duplicate_name_error::DuplicateNameError;
use edge_rules::runtime::edge_rules::{ContextUpdateErrorEnum, EvalError, ParseErrors};
use edge_rules::typesystem::errors::{ParseErrorEnum, RuntimeError};
use std::fmt::{Display, Formatter};

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct PortableError {
    message: String,
}
impl PortableError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
    pub fn into_message(self) -> String {
        self.message
    }
}
impl Display for PortableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl From<PortableError> for String {
    fn from(v: PortableError) -> Self {
        v.message
    }
}
impl From<ContextUpdateErrorEnum> for PortableError {
    fn from(err: ContextUpdateErrorEnum) -> Self {
        PortableError::from(ParseErrorEnum::from(err))
    }
}
impl From<DuplicateNameError> for PortableError {
    fn from(err: DuplicateNameError) -> Self {
        Self::new(err.to_string())
    }
}
impl From<ParseErrorEnum> for PortableError {
    fn from(err: ParseErrorEnum) -> Self {
        Self::new(err.to_string())
    }
}
impl From<ParseErrors> for PortableError {
    fn from(err: ParseErrors) -> Self {
        Self::new(err.to_string())
    }
}
impl From<EvalError> for PortableError {
    fn from(err: EvalError) -> Self {
        Self::new(err.to_string())
    }
}
impl From<RuntimeError> for PortableError {
    fn from(err: RuntimeError) -> Self {
        Self::new(err.to_string())
    }
}
