use std::fmt::{Debug, Display};

use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::Link;
use crate::typesystem::types::TypedValue;

pub trait UserFunction: Display + Debug + TypedValue {
    fn get_name(&self) -> String;

    /// user function interface
    fn get_parameters(&self) -> &Vec<FormalParameter>;

    /// user functions are not usual functions and do not have simple eval
    /// instead they return a context object that can be used to evaluate the function later on
    /// this is done for various flexibility and optimisation reasons
    fn create_context(&self, parameters: Vec<FormalParameter>) -> Link<FunctionContext>;
}
