use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::operators::logical_operators::LogicalOperatorEnum::*;
use crate::ast::operators::math_operators::{Operator, OperatorData};
use crate::ast::token::{EPriorities, ExpressionEnum};
use crate::ast::Link;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::ParseErrorEnum::UnexpectedLiteral;
use crate::typesystem::errors::{ParseErrorEnum, RuntimeError};
use crate::typesystem::types::ValueType::BooleanType;
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::BooleanValue;
use std::cell::RefCell;
use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::fmt::Debug;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum LogicalOperatorEnum {
    Not = EPriorities::GateNot as isize,
    And = EPriorities::GateAnd as isize,
    Or = EPriorities::GatesOr as isize,
    Xor = EPriorities::GatesXor as isize,
}

impl LogicalOperatorEnum {
    fn as_str(&self) -> &'static str {
        match self {
            Not => "not",
            And => "and",
            Or => "or",
            Xor => "xor",
        }
    }
}

impl TryFrom<&str> for LogicalOperatorEnum {
    type Error = ParseErrorEnum;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not" => Ok(Not),
            "and" => Ok(And),
            "or" => Ok(Or),
            "xor" => Ok(Xor),
            _ => Err(UnexpectedLiteral(value.to_string(), Some("not, and, or, xor".to_string()))),
        }
    }
}

//----------------------------------------------------------------------------------------------

pub struct LogicalOperator {
    pub data: OperatorData<LogicalOperatorEnum>,
    pub function: fn(a: &bool, b: &bool) -> bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Debug for LogicalOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogicalOperator").field("data", &self.data).finish()
    }
}

impl Operator for LogicalOperator {}

impl TypedValue for LogicalOperator {
    fn get_type(&self) -> ValueType {
        BooleanType
    }
}

impl StaticLink for LogicalOperator {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        self.data.link(ctx, BooleanType)
    }
}

impl LogicalOperator {
    pub fn build(
        operator: LogicalOperatorEnum,
        left: ExpressionEnum,
        right: ExpressionEnum,
    ) -> Result<Self, ParseErrorEnum> {
        let function = match operator {
            And => |left: &bool, right: &bool| *left && *right,
            Or => |left: &bool, right: &bool| *left || *right,
            Xor => |left: &bool, right: &bool| *left ^ *right,
            Not => |left: &bool, _right: &bool| !*left,
        };

        Ok(LogicalOperator { data: OperatorData { operator, left, right }, function })
    }
}

impl EvaluatableExpression for LogicalOperator {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let left_token = &self.data.left.eval(Rc::clone(&context))?;
        let right_token = &self.data.right.eval(context)?;

        match (left_token, right_token) {
            (BooleanValue(_left), BooleanValue(_right)) => Ok(BooleanValue((self.function)(_left, _right))),
            _ => RuntimeError::internal_integrity_error(160).into(),
        }
    }
}

//----------------------------------------------------------------------------------------------
// Display
//----------------------------------------------------------------------------------------------

impl Display for LogicalOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.data, f)
    }
}

impl Display for OperatorData<LogicalOperatorEnum> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

impl Display for LogicalOperatorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
