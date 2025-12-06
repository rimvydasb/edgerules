#![allow(dead_code)]

pub use crate::ast::context::context_object_builder::ContextObjectBuilder;
pub use crate::ast::expression::StaticLink;
pub use crate::ast::metaphors::functions::FunctionDefinition;
pub use crate::ast::token::{ComplexTypeRef, EToken, EUnparsedToken, ExpressionEnum, UserTypeBody};
pub use crate::runtime::edge_rules::{
    ContextUpdateErrorEnum, EdgeRulesModel, EdgeRulesRuntime, EvalError, ParseErrors,
};
pub use crate::typesystem::errors::{LinkingError, LinkingErrorEnum, ParseErrorEnum};
pub use crate::typesystem::types::number::NumberEnum;
pub use crate::typesystem::types::SpecialValueEnum;
pub use crate::typesystem::types::ValueType;
pub use crate::typesystem::values::ValueEnum;

pub fn expr(code: &str) -> Result<ExpressionEnum, EvalError> {
    EdgeRulesModel::parse_expression(code).map_err(EvalError::from)
}
