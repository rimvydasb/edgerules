use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::ops::{Range};
use std::rc::Rc;
use log::{error, trace};
use crate::ast::token::*;
use crate::*;
use crate::ast::token::ExpressionEnum::*;
use crate::ast::context::context_object::ContextObject;
use crate::ast::Link;
use crate::ast::variable::VariableLink;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::{ErrorStack, RuntimeError};
use crate::typesystem::types::number::NumberEnum::Int;
use crate::typesystem::types::{ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{NumberValue, RangeValue, Reference};

pub trait StaticLink: Display + Debug {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType>;
}

pub trait EvaluatableExpression: StaticLink {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError>;
}

impl<T> From<T> for ExpressionEnum where T: EvaluatableExpression + Sized + 'static {
    fn from(expression: T) -> Self {
        FunctionCall(Box::new(expression))
    }
}

impl ExpressionEnum {
    pub fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let trace_context = Rc::clone(&context);
        let eval_result = match self {
            Variable(variable) => variable.eval(context),
            ContextVariable => {
                trace!(">>> evaluating context variable");
                context.borrow().get_context_variable()
            }
            Operator(operator) => operator.eval(context),
            FunctionCall(function) => function.eval(context),
            Selection(selection) => selection.eval(context),
            Filter(filter) => filter.eval(context),
            ObjectField(field_name, _obj) => {
                RuntimeError::eval_error(format!("ObjectField evaluation is deprecated. Still used by {:?}", field_name)).into()
            }
            Value(value) => Ok(value.clone()),
            Collection(elements) => elements.eval(context),
            RangeExpression(left, right) => {
                match (left.eval(Rc::clone(&context))?, right.eval(context)?) {
                    (NumberValue(Int(left_number)), NumberValue(Int(right_number))) => {
                        let range = Range {
                            start: left_number,
                            end: right_number + 1,
                        };

                        Ok(RangeValue(range))
                    }
                    _ => {
                        RuntimeError::eval_error(format!("Range is not a valid number")).into()
                    }
                }
            }
            StaticObject(object) => {
                let reference = ExecutionContext::create_temp_child_context(Rc::clone(&context), object.clone());
                Ok(Reference(reference))
            }
        };

        if let Err(error) = eval_result {
            //let error_str = error.get_error_type().to_string();
            error!(">                   `{:?}`", error.get_error_type());
            let with_context = error.with_context(|| format!("Error evaluating `{}.{}`", trace_context.borrow().object.borrow().node.node_type, self));
            return Err(with_context);
        }

        eval_result
    }

    pub fn variable(_literal: &str) -> ExpressionEnum {
        let path: Vec<&str> = _literal.split('.').collect();
        let path = VariableLink::new_unlinked_path(path.iter().map(|s| String::from(*s)).collect());
        Variable(path)
    }
}

// @Todo: would be good to understand where it is actually used. Try removing it.
impl PartialEq for ExpressionEnum {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StaticObject(ref a), StaticObject(ref b)) => { a == b }
            (Value(ref a), Value(ref b)) => a == b,
            (ObjectField(ref a, ref aa), ObjectField(ref b, ref bb)) => a == b && aa == bb,
            (Variable(ref a), Variable(ref b)) => a == b,
            // (&Array(ref a), &Array(ref b)) => {
            //     if a.len() == b.len() {
            //         a.iter().zip(b).filter(|&(a, b)| a == b).count() == b.len()
            //     } else {
            //         false
            //     }
            // }
            (FunctionCall(ref _a), FunctionCall(ref _b)) => {
                //@Todo: implement equals for function calls
                false
            }
            _ => false,
        }
    }
}
