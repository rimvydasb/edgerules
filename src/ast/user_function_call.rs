use crate::ast::context::context_object::ContextObject;
use crate::ast::metaphors::metaphor::Metaphor;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::context::function_context::FunctionContext;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::token::ExpressionEnum;
use crate::ast::utils::array_to_code_sep;
use crate::ast::{is_linked, Link};
use crate::link::linker;
use crate::runtime::execution_context::*;
use crate::typesystem::errors::{ErrorStack, LinkingError, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::Reference;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

/// User function is a function that is defined in the code by user with a custom name. This is kind of non-built-in function
#[derive(Debug)]
pub struct UserFunctionCall {
    pub name: String,
    pub args: Vec<ExpressionEnum>,
    pub definition: Link<FunctionContext>,
    #[allow(dead_code)]
    pub return_type: Link<ValueType>,
}

impl UserFunctionCall {
    pub fn new(name: String, args: Vec<ExpressionEnum>) -> UserFunctionCall {
        UserFunctionCall {
            name,
            args,
            definition: LinkingError::not_linked().into(),
            return_type: LinkingError::not_linked().into(),
        }
    }
}

// eval context is not immediately evaluated for output values, but passed to the caller
impl EvaluatableExpression for UserFunctionCall {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let values = self
            .args
            .iter()
            .map(|expr| expr.eval(Rc::clone(&context)))
            .collect();

        match &self.definition {
            Ok(definition) => {
                let eval_context = definition.create_eval_context(values)?;
                Ok(Reference(eval_context))
            }
            Err(error) => {
                let error = error
                    .clone()
                    .with_context(|| format!("Evaluating function `{}`", self.name));
                Err(RuntimeError::from(error))
            }
        }
    }
}

impl StaticLink for UserFunctionCall {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        // so the next time it is called, it will not be linked, but for each user function call link will happen. For example:
        // process(a) + process(b) - linking will happen. This may be good, because different types could be used theoretically - need to test it
        if !is_linked(&self.definition) {
            // 1. Make sure definition is acquired before doing anything else
            let definition = linker::find_implementation(Rc::clone(&ctx), self.name.clone())?;

            // 2. Next step is to check if all used arguments are valid
            if self.args.len() != definition.borrow().metaphor.get_parameters().len() {
                return LinkingError::other_error(format!(
                    "Function {} expects {} arguments, but {} were provided",
                    self.name,
                    definition.borrow().metaphor.get_parameters().len(),
                    self.args.len()
                ))
                .into();
            }

            // 3. Creating a mid context where all parameter values are set
            let mut parameters_list = Vec::new();
            let ctx_name = ctx.borrow().node.node_type.to_code();
            let function_name = self.name.clone();

            for (parameter, input_argument) in definition
                .borrow()
                .metaphor
                .get_parameters()
                .iter()
                .zip(self.args.iter_mut())
            {
                let arg_type = if let ExpressionEnum::Variable(var) = input_argument {
                    if var.path.len() == 1 && var.path[0] == ctx_name {
                        LinkingError::other_error(format!(
                            "Cannot pass context `{}` as argument to function `{}` defined in the same context",
                            ctx_name, function_name
                        ))
                        .into()
                    } else {
                        input_argument.link(Rc::clone(&ctx))
                    }
                } else {
                    input_argument.link(Rc::clone(&ctx))
                };
                parameters_list.push((parameter.name.clone(), arg_type));
            }

            let mut parameters = Vec::new();
            for (name, linked_type) in parameters_list {
                parameters.push(FormalParameter::new(name, linked_type?));
            }

            self.definition = Ok(definition.borrow().metaphor.create_context(parameters)?);
        }

        match &self.definition {
            Ok(ok) => Ok(ok.get_type()),
            Err(err) => Err(err.clone()),
        }
    }
}

impl Display for UserFunctionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            array_to_code_sep(self.args.iter(), ", ")
        )
    }
}
