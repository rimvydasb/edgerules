use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::metaphors::functions::FunctionDefinition;
use crate::ast::metaphors::metaphor::Metaphor;
use crate::ast::Link;
use crate::typesystem::types::{TypedValue, ValueType};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum BuiltinMetaphor {
    Function(FunctionDefinition),
}

impl Display for BuiltinMetaphor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinMetaphor::Function(def) => Display::fmt(def, f),
        }
    }
}

impl TypedValue for BuiltinMetaphor {
    fn get_type(&self) -> ValueType {
        match self {
            BuiltinMetaphor::Function(def) => def.get_type(),
        }
    }
}

impl Metaphor for BuiltinMetaphor {
    fn get_name(&self) -> String {
        match self {
            BuiltinMetaphor::Function(def) => def.get_name(),
        }
    }

    fn get_parameters(&self) -> &Vec<FormalParameter> {
        match self {
            BuiltinMetaphor::Function(def) => def.get_parameters(),
        }
    }

    fn create_context(&self, parameters: Vec<FormalParameter>) -> Link<FunctionContext> {
        match self {
            BuiltinMetaphor::Function(def) => def.create_context(parameters),
        }
    }
}

impl From<FunctionDefinition> for BuiltinMetaphor {
    fn from(val: FunctionDefinition) -> Self {
        BuiltinMetaphor::Function(val)
    }
}
