use std::fmt::{Debug, Display, Formatter};
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::metaphors::decision_tables::DecisionTable;
use crate::ast::metaphors::functions::FunctionDefinition;
use crate::ast::metaphors::metaphor::Metaphor;
use crate::ast::Link;
use crate::typesystem::types::{TypedValue, ValueType};

#[derive(Debug)]
pub enum BuiltinMetaphor {
    Function(FunctionDefinition),
    DecisionTable(DecisionTable),
}

impl Display for BuiltinMetaphor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinMetaphor::Function(def) => Display::fmt(def, f),
            BuiltinMetaphor::DecisionTable(dt) => Display::fmt(dt, f),
        }
    }
}

impl TypedValue for BuiltinMetaphor {
    fn get_type(&self) -> ValueType {
        match self {
            BuiltinMetaphor::Function(def) => def.get_type(),
            BuiltinMetaphor::DecisionTable(dt) => dt.get_type(),
        }
    }
}

impl Metaphor for BuiltinMetaphor {
    fn get_name(&self) -> String {
        match self {
            BuiltinMetaphor::Function(def) => def.get_name(),
            BuiltinMetaphor::DecisionTable(dt) => dt.get_name(),
        }
    }

    fn get_parameters(&self) -> &Vec<FormalParameter> {
        match self {
            BuiltinMetaphor::Function(def) => def.get_parameters(),
            BuiltinMetaphor::DecisionTable(dt) => dt.get_parameters(),
        }
    }

    fn create_context(&self, parameters: Vec<FormalParameter>) -> Link<FunctionContext> {
        match self {
            BuiltinMetaphor::Function(def) => def.create_context(parameters),
            BuiltinMetaphor::DecisionTable(dt) => dt.create_context(parameters),
        }
    }
}

impl From<FunctionDefinition> for BuiltinMetaphor {
    fn from(val: FunctionDefinition) -> Self {
        BuiltinMetaphor::Function(val)
    }
}

impl From<DecisionTable> for BuiltinMetaphor {
    fn from(val: DecisionTable) -> Self {
        BuiltinMetaphor::DecisionTable(val)
    }
}
