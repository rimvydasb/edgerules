use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::token::ExpressionEnum;
use crate::ast::{is_linked, Link};
use crate::runtime::execution_context::*;
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::utils::bracket_unwrap;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub struct IfThenElseFunction {
    pub condition: ExpressionEnum,
    pub then_expression: ExpressionEnum,
    pub else_expression: ExpressionEnum,
    pub result_type: Link<ValueType>,
}

impl Display for IfThenElseFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "if {} then {} else {}",
            self.condition,
            bracket_unwrap(format!("{}", self.then_expression)),
            bracket_unwrap(format!("{}", self.else_expression))
        )
    }
}

impl IfThenElseFunction {
    pub fn build(
        condition: ExpressionEnum,
        then_expression: ExpressionEnum,
        else_expression: ExpressionEnum,
    ) -> Result<Self, ParseErrorEnum> {
        Ok(IfThenElseFunction {
            condition,
            then_expression,
            else_expression,
            result_type: LinkingError::not_linked().into(),
        })
    }
}

impl StaticLink for IfThenElseFunction {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.result_type) {
            let condition_type = self.condition.link(Rc::clone(&ctx))?;
            let then_expression = self.then_expression.link(Rc::clone(&ctx))?;
            self.result_type = self.else_expression.link(Rc::clone(&ctx));

            LinkingError::expect_single_type(
                "if condition",
                condition_type,
                &ValueType::BooleanType,
            )?;

            if let Ok(else_expression) = &self.result_type {
                LinkingError::expect_same_types(
                    "`then` and `else` expressions",
                    then_expression,
                    else_expression.clone(),
                )?;
            }
        }

        self.result_type.clone()
    }
}

impl EvaluatableExpression for IfThenElseFunction {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let result = self.condition.eval(Rc::clone(&context))?;

        match result {
            ValueEnum::BooleanValue(true) => Ok(self.then_expression.eval(context)?),
            ValueEnum::BooleanValue(false) => Ok(self.else_expression.eval(context)?),
            _ => RuntimeError::type_not_supported(result.get_type().clone()).into(),
        }
    }
}
