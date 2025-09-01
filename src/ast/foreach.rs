use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;
use std::rc::Rc;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::token::ExpressionEnum::Value;
use crate::ast::token::{ExpressionEnum};
use crate::runtime::execution_context::*;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::{Integer, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{Array, RangeValue};
use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::function_context::{FunctionContext, RETURN_EXPRESSION};
use crate::ast::{is_linked, Link};
use crate::ast::context::context_object_type::FormalParameter;
use crate::link::linker::link_parts;
use crate::link::node_data::{NodeData, NodeDataEnum};
use crate::tokenizer::utils::Either;
use crate::utils::{context_unwrap};

/// for in_loop_variable in in_expression return return_expression
/// in_expression.map(in_loop_variable -> return_expression)
/// map(in_expression,(in_loop_variable) return_expression)
#[derive(Debug)]
pub struct ForFunction {
    pub in_loop_variable: String,
    pub in_expression: ExpressionEnum,
    /// In definition return_expression is wrapped in InlineFunctionContext
    pub return_expression: Rc<RefCell<ContextObject>>,
    pub return_type: Link<ValueType>,
}

impl Display for ForFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let return_expression = context_unwrap(self.return_expression.borrow().to_string());
        write!(f, "for {} in {} return {}", self.in_loop_variable, self.in_expression, return_expression)
    }
}

impl Display for Either<ExpressionEnum, FunctionContext> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(expression) => write!(f, "{}", expression),
            Either::Right(inline_function) => write!(f, "{}", inline_function),
        }
    }
}

impl ForFunction {
    pub fn new(in_loop_variable: String, in_expression: ExpressionEnum, return_expression: ExpressionEnum) -> Self {
        let mut builder = ContextObjectBuilder::new();
        builder.add_expression(RETURN_EXPRESSION, return_expression);

        ForFunction {
            in_loop_variable,
            in_expression,
            return_expression: builder.build(),
            return_type: LinkingError::not_linked().into(),
        }
    }

    fn create_in_loop_context(&self, parent: &Rc<RefCell<ExecutionContext>>, value: ExpressionEnum) -> Rc<RefCell<ExecutionContext>> {
        let mut obj = ContextObjectBuilder::new();
        obj.add_expression(self.in_loop_variable.as_str(), value);

        ExecutionContext::create_temp_child_context(Rc::clone(parent), obj.build())
    }

    fn iterate_values(&self, values: Vec<Result<ValueEnum, RuntimeError>>, _list_type: ValueType, parent: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let mut result: Vec<Result<ValueEnum, RuntimeError>> = Vec::new();

        for value in values {
            let ctx = self.create_in_loop_context(&parent, Value(value?));
            //@Todo way too complex
            let map_value = self.return_expression.borrow().expressions.get(RETURN_EXPRESSION).unwrap().borrow().expression.eval(ctx);
            //@Todo return values only, not tokens
            result.push(map_value);
        }

        match self.return_type.clone()? {
            ValueType::ListType(item_type) => {
                Ok(Array(result, *item_type))
            }
            err => {
                RuntimeError::eval_error(format!("Cannot iterate through non list type `{}`", err)).into()
            }
        }
    }

    fn iterate_range(&self, values: Range<Integer>, parent: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let mut result: Vec<Result<ValueEnum, RuntimeError>> = Vec::new();

        for value in values {
            let ctx = self.create_in_loop_context(&parent, Value(ValueEnum::from(value)));
            //@Todo way too complex
            let map_value = self.return_expression.borrow().expressions.get(RETURN_EXPRESSION).unwrap().borrow().expression.eval(ctx);
            //@Todo return values only, not tokens
            result.push(map_value);
        }

        Ok(Array(result, ValueType::NumberType))
    }
}

impl EvaluatableExpression for ForFunction {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        match self.in_expression.eval(Rc::clone(&context))? {
            Array(values, list_type) => self.iterate_values(values, list_type, Rc::clone(&context)),
            RangeValue(range) => self.iterate_range(range, Rc::clone(&context)),
            other => RuntimeError::eval_error(format!("Cannot iterate {}", other.get_type())).into(),
        }
    }
}

impl StaticLink for ForFunction {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            let list_type = self.in_expression.link(Rc::clone(&ctx))?;

            let item_type = match list_type {
                ValueType::ListType(list_item_type) => {
                    *list_item_type
                }
                ValueType::RangeType => {
                    ValueType::NumberType
                }
                _ => {
                    return LinkingError::other_error(format!("Cannot iterate through non list type `{}`", list_type)).into();
                }
            };

            let for_parameter = FormalParameter::new(self.in_loop_variable.clone(), item_type);

            self.return_expression.borrow_mut().parameters.push(for_parameter);
            self.return_expression.borrow_mut().node = NodeData::new(NodeDataEnum::Internal(Rc::downgrade(&ctx)));

            link_parts(Rc::clone(&self.return_expression))?;

            let field_type = self.return_expression.borrow().expressions.get(RETURN_EXPRESSION).unwrap().borrow().field_type.clone()?;

            self.return_type = Ok(ValueType::ListType(Box::new(field_type)));
        }

        self.return_type.clone()
    }
}
