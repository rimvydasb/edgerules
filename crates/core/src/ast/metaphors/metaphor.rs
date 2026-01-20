use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::fmt::Debug;
use std::fmt::Display;
use std::rc::Rc;

use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::Link;
use crate::typesystem::types::TypedValue;

#[cfg(not(target_arch = "wasm32"))]
pub trait UserFunction: Display + Debug + TypedValue {
    fn get_name(&self) -> String;

    /// user function interface
    fn get_parameters(&self) -> &Vec<FormalParameter>;

    /// user functions are not usual functions and do not have simple eval
    /// instead they return a context object that can be used to evaluate the function later on
    /// this is done for various flexibility and optimisation reasons
    fn create_context(
        &self,
        parameters: Vec<FormalParameter>,
        parent: Option<Rc<RefCell<ContextObject>>>,
    ) -> Link<FunctionContext>;
}

#[cfg(target_arch = "wasm32")]
pub trait UserFunction: Display + TypedValue {
    fn get_name(&self) -> String;

    /// user function interface
    fn get_parameters(&self) -> &Vec<FormalParameter>;

    /// user functions are not usual functions and do not have simple eval
    /// instead they return a context object that can be used to evaluate the function later on
    /// this is done for various flexibility and optimisation reasons
    fn create_context(
        &self,
        parameters: Vec<FormalParameter>,
        parent: Option<Rc<RefCell<ContextObject>>>,
    ) -> Link<FunctionContext>;
}
