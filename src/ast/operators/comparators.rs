use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use log::trace;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::operators::math_operators::{Operator, OperatorData};
use crate::ast::token::{ExpressionEnum};
use crate::ast::operators::comparators::ComparatorEnum::*;
use crate::ast::context::context_object::ContextObject;
use crate::ast::Link;
use crate::runtime::execution_context::ExecutionContext;
use crate::tokenizer::utils::CharStream;
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::errors::ParseErrorEnum::UnknownParseError;
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{BooleanValue, NumberValue, StringValue};

//----------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ComparatorEnum {
    Equals,
    NotEquals,
    Less,
    Greater,
    LessEquals,
    GreaterEquals,
}

impl TryFrom<&str> for ComparatorEnum {
    type Error = ParseErrorEnum;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "=" => Ok(Equals),
            "<>" => Ok(NotEquals),
            "<" => Ok(Less),
            ">" => Ok(Greater),
            "<=" => Ok(LessEquals),
            ">=" => Ok(GreaterEquals),
            _ => Err(UnknownParseError(format!("Unknown comparator: {}", value)))
        }
    }
}

impl ComparatorEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            Equals => "=",
            NotEquals => "<>",
            Less => "<",
            Greater => ">",
            LessEquals => "<=",
            GreaterEquals => ">=",
        }
    }

    pub fn parse(iter: &mut CharStream) -> Option<ComparatorEnum> {
        match (iter.next().unwrap(), iter.peek()) {
            ('<', Some('=')) => {
                iter.next();
                Some(LessEquals)
            }
            ('>', Some('=')) => {
                iter.next();
                Some(GreaterEquals)
            }
            ('<', Some('>')) => {
                iter.next();
                Some(NotEquals)
            }
            ('=', _) => Some(Equals),
            ('<', _) => Some(Less),
            ('>', _) => Some(Greater),
            _ => None
        }
    }
}

//----------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct ComparatorOperator {
    pub data: OperatorData<ComparatorEnum>,
}

impl TypedValue for ComparatorOperator {
    fn get_type(&self) -> ValueType {
        ValueType::BooleanType
    }
}

impl StaticLink for ComparatorOperator {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {

        trace!("Linking comparator operator: {:?}", self.data.left);

        LinkingError::expect_same_types("Comparator",
                                        self.data.left.link(Rc::clone(&ctx))?,
                                        self.data.right.link(ctx)?,
        )?;

        Ok(ValueType::BooleanType)
    }
}

impl Operator for ComparatorOperator {}

impl ComparatorOperator {
    pub fn build(operator: ComparatorEnum, left: ExpressionEnum, right: ExpressionEnum) -> Result<Self, ParseErrorEnum> {
        let comparator = ComparatorOperator {
            data: OperatorData {
                operator,
                left,
                right,
            }
        };

        Ok(comparator)
    }

    fn eval_operator(&self, left: &ValueEnum, right: &ValueEnum) -> Result<ValueEnum, RuntimeError> {
        match (left, &self.data.operator, right) {
            (NumberValue(left), Equals, NumberValue(right)) => Ok(BooleanValue(left == right)),
            (BooleanValue(left), Equals, BooleanValue(right)) => Ok(BooleanValue(left == right)),
            (StringValue(left), Equals, StringValue(right)) => Ok(BooleanValue(left == right)),

            (NumberValue(left), NotEquals, NumberValue(right)) => Ok(BooleanValue(left != right)),
            (BooleanValue(left), NotEquals, BooleanValue(right)) => Ok(BooleanValue(left != right)),
            (StringValue(left), NotEquals, StringValue(right)) => Ok(BooleanValue(left != right)),

            (NumberValue(left), LessEquals, NumberValue(right)) => Ok(BooleanValue(left <= right)),
            (NumberValue(left), GreaterEquals, NumberValue(right)) => Ok(BooleanValue(left >= right)),
            (NumberValue(left), Less, NumberValue(right)) => Ok(BooleanValue(left < right)),
            (NumberValue(left), Greater, NumberValue(right)) => Ok(BooleanValue(left > right)),

            (left, comparator, right) => RuntimeError::eval_error(format!("Not implemented {} {} {}", left, comparator, right)).into(),
        }
    }
}

impl EvaluatableExpression for ComparatorOperator {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let left_token = &self.data.left.eval(Rc::clone(&context))?;
        let right_token = &self.data.right.eval(context)?;

        self.eval_operator(left_token, right_token)
    }
}

//----------------------------------------------------------------------------------------------
// Display
//----------------------------------------------------------------------------------------------

impl Display for ComparatorOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.data, f)
    }
}

impl Display for OperatorData<ComparatorEnum> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

impl Display for ComparatorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
