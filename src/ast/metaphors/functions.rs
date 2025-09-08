use std::cell::RefCell;

use crate::ast::annotations::AnnotationEnum;
use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::metaphors::metaphor::Metaphor;
use crate::ast::utils::array_to_code_sep;
use crate::ast::Link;
use crate::link::linker;
use crate::tokenizer::C_ASSIGN;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::typesystem::types::{TypedValue, ValueType};

/// Non executable function definition holder. For an executable function definition see FunctionContext.
#[derive(Debug)]
pub struct FunctionDefinition {
    pub annotations: Vec<AnnotationEnum>,
    pub name: String,
    pub arguments: Vec<FormalParameter>,
    /// Function body later is used as a context object for execution context so it is Rc.
    /// RefCell is added only for linking purposes. Better to remove later.
    pub body: Rc<RefCell<ContextObject>>,
}

impl FunctionDefinition {
    pub fn build(
        annotations: Vec<AnnotationEnum>,
        name: String,
        arguments: Vec<FormalParameter>,
        body: Rc<RefCell<ContextObject>>,
    ) -> Self {
        FunctionDefinition {
            annotations,
            name,
            arguments,
            body,
        }
    }
}

// Conversions for BuiltinMetaphor are defined in builtin.rs

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}({}) {} {}",
            array_to_code_sep(self.annotations.iter(), "\n"),
            self.name,
            array_to_code_sep(self.arguments.iter(), ", "),
            C_ASSIGN,
            self.body.borrow()
        )
    }
}

impl TypedValue for FunctionDefinition {
    fn get_type(&self) -> ValueType {
        ValueType::ObjectType(Rc::clone(&self.body))
    }
}

impl Metaphor for FunctionDefinition {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_parameters(&self) -> &Vec<FormalParameter> {
        &self.arguments
    }

    fn create_context(&self, parameters: Vec<FormalParameter>) -> Link<FunctionContext> {
        parameters.iter().for_each(|arg| {
            self.body.borrow_mut().parameters.push(arg.clone());
        });

        linker::link_parts(Rc::clone(&self.body))?;

        let ctx = FunctionContext::create_for(Rc::clone(&self.body), parameters);

        Ok(ctx)
    }
}
