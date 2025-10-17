use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::operators::comparators::ComparatorEnum::*;
use crate::ast::operators::math_operators::{Operator, OperatorData};
use crate::ast::token::ExpressionEnum;
use crate::ast::Link;
use crate::runtime::execution_context::ExecutionContext;
use crate::tokenizer::utils::CharStream;
use crate::typesystem::errors::ParseErrorEnum::UnknownParseError;
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{
    BooleanValue, DateTimeValue, DateValue, NumberValue, StringValue, TimeValue,
};
use crate::typesystem::values::ValueOrSv;
use log::trace;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

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
            _ => Err(UnknownParseError(format!("Unknown comparator: {}", value))),
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
            _ => None,
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

        let left_type = self.data.left.link(Rc::clone(&ctx))?;
        let right_type = self.data.right.link(ctx)?;

        let type_pair = (left_type.clone(), right_type.clone());
        let same_type = if type_pair.0 == type_pair.1 {
            LinkingError::expect_same_types("Comparator", type_pair.0, type_pair.1)?
        } else {
            match type_pair {
                (ValueType::DateType, ValueType::DateTimeType)
                | (ValueType::DateTimeType, ValueType::DateType) => ValueType::DateTimeType,
                (left, right) => {
                    return LinkingError::expect_same_types("Comparator", left, right);
                }
            }
        };

        match (&same_type, &self.data.operator) {
            (ValueType::BooleanType, Equals) => {}
            (ValueType::BooleanType, NotEquals) => {}
            (ValueType::BooleanType, operator) => {
                return Err(LinkingError::operation_not_supported(
                    operator.as_str(),
                    same_type.clone(),
                    same_type,
                ));
            }

            // if both are strings, only = and <> are allowed
            (ValueType::StringType, Equals) => {}
            (ValueType::StringType, NotEquals) => {}
            (ValueType::StringType, operator) => {
                return Err(LinkingError::operation_not_supported(
                    operator.as_str(),
                    same_type.clone(),
                    same_type,
                ));
            }

            // if both are dates, only =, <>, <, <=, >, >= are allowed
            (ValueType::DateType, Equals)
            | (ValueType::DateType, NotEquals)
            | (ValueType::DateType, Less)
            | (ValueType::DateType, LessEquals)
            | (ValueType::DateType, Greater)
            | (ValueType::DateType, GreaterEquals) => {}

            // if both are times, only =, <>, <, <=, >, >= are allowed
            (ValueType::TimeType, Equals)
            | (ValueType::TimeType, NotEquals)
            | (ValueType::TimeType, Less)
            | (ValueType::TimeType, LessEquals)
            | (ValueType::TimeType, Greater)
            | (ValueType::TimeType, GreaterEquals) => {}

            // if both are datetimes, only =, <>, <, <=, >, >= are allowed
            (ValueType::DateTimeType, Equals)
            | (ValueType::DateTimeType, NotEquals)
            | (ValueType::DateTimeType, Less)
            | (ValueType::DateTimeType, LessEquals)
            | (ValueType::DateTimeType, Greater)
            | (ValueType::DateTimeType, GreaterEquals) => {}

            // if both are durations, allow =, <>, <, <=, >, >=
            (ValueType::DurationType, Equals)
            | (ValueType::DurationType, NotEquals)
            | (ValueType::DurationType, Less)
            | (ValueType::DurationType, LessEquals)
            | (ValueType::DurationType, Greater)
            | (ValueType::DurationType, GreaterEquals) => {}

            // if both are numbers all comparators are allowed
            (ValueType::NumberType, _) => {}

            // other types are not supported
            (other_type, operator) => {
                return Err(LinkingError::operation_not_supported(
                    operator.as_str(),
                    other_type.clone(),
                    other_type.clone(),
                ));
            }
        }

        Ok(ValueType::BooleanType)
    }
}

impl Operator for ComparatorOperator {}

impl ComparatorOperator {
    pub fn build(
        operator: ComparatorEnum,
        left: ExpressionEnum,
        right: ExpressionEnum,
    ) -> Result<Self, ParseErrorEnum> {
        let comparator = ComparatorOperator {
            data: OperatorData {
                operator,
                left,
                right,
            },
        };

        Ok(comparator)
    }

    fn duration_ordering(
        &self,
        left: &crate::typesystem::values::DurationValue,
        right: &crate::typesystem::values::DurationValue,
    ) -> Result<Ordering, RuntimeError> {
        left.partial_cmp(right).ok_or_else(|| {
            RuntimeError::eval_error("Cannot compare durations of different kinds".to_string())
        })
    }

    fn date_datetime_ordering(date: &time::Date, datetime: &time::PrimitiveDateTime) -> Ordering {
        time::PrimitiveDateTime::new(*date, time::Time::MIDNIGHT).cmp(datetime)
    }

    fn datetime_date_ordering(datetime: &time::PrimitiveDateTime, date: &time::Date) -> Ordering {
        Self::date_datetime_ordering(date, datetime).reverse()
    }

    fn eval_operator(
        &self,
        left: &ValueEnum,
        right: &ValueEnum,
    ) -> Result<ValueEnum, RuntimeError> {
        match (left, &self.data.operator, right) {
            (NumberValue(left), Equals, NumberValue(right)) => Ok(BooleanValue(left == right)),
            (BooleanValue(left), Equals, BooleanValue(right)) => Ok(BooleanValue(left == right)),
            (StringValue(left), Equals, StringValue(right)) => Ok(BooleanValue(left == right)),
            (DateValue(ValueOrSv::Value(a)), Equals, DateValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a == b))
            }
            (TimeValue(ValueOrSv::Value(a)), Equals, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a == b))
            }
            (DateTimeValue(ValueOrSv::Value(a)), Equals, DateTimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a == b))
            }

            (NumberValue(left), NotEquals, NumberValue(right)) => Ok(BooleanValue(left != right)),
            (BooleanValue(left), NotEquals, BooleanValue(right)) => Ok(BooleanValue(left != right)),
            (StringValue(left), NotEquals, StringValue(right)) => Ok(BooleanValue(left != right)),
            (DateValue(ValueOrSv::Value(a)), NotEquals, DateValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a != b))
            }
            (TimeValue(ValueOrSv::Value(a)), NotEquals, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a != b))
            }
            (DateTimeValue(ValueOrSv::Value(a)), NotEquals, DateTimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a != b))
            }

            (NumberValue(left), LessEquals, NumberValue(right)) => Ok(BooleanValue(left <= right)),
            (NumberValue(left), GreaterEquals, NumberValue(right)) => {
                Ok(BooleanValue(left >= right))
            }
            (NumberValue(left), Less, NumberValue(right)) => Ok(BooleanValue(left < right)),
            (NumberValue(left), Greater, NumberValue(right)) => Ok(BooleanValue(left > right)),

            (DateValue(ValueOrSv::Value(a)), LessEquals, DateValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a <= b))
            }
            (DateValue(ValueOrSv::Value(a)), GreaterEquals, DateValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a >= b))
            }
            (DateValue(ValueOrSv::Value(a)), Less, DateValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a < b))
            }
            (DateValue(ValueOrSv::Value(a)), Greater, DateValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a > b))
            }

            (TimeValue(ValueOrSv::Value(a)), LessEquals, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a <= b))
            }
            (TimeValue(ValueOrSv::Value(a)), GreaterEquals, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a >= b))
            }
            (TimeValue(ValueOrSv::Value(a)), Less, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a < b))
            }
            (TimeValue(ValueOrSv::Value(a)), Greater, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a > b))
            }

            (
                DateTimeValue(ValueOrSv::Value(a)),
                LessEquals,
                DateTimeValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue(a <= b)),
            (
                DateTimeValue(ValueOrSv::Value(a)),
                GreaterEquals,
                DateTimeValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue(a >= b)),
            (DateTimeValue(ValueOrSv::Value(a)), Less, DateTimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a < b))
            }
            (DateTimeValue(ValueOrSv::Value(a)), Greater, DateTimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a > b))
            }

            (
                DateValue(ValueOrSv::Value(date)),
                Equals,
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) => Ok(BooleanValue(
                Self::date_datetime_ordering(date, datetime) == Ordering::Equal,
            )),
            (
                DateValue(ValueOrSv::Value(date)),
                NotEquals,
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) => Ok(BooleanValue(
                Self::date_datetime_ordering(date, datetime) != Ordering::Equal,
            )),
            (
                DateValue(ValueOrSv::Value(date)),
                Less,
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) => Ok(BooleanValue(
                Self::date_datetime_ordering(date, datetime) == Ordering::Less,
            )),
            (
                DateValue(ValueOrSv::Value(date)),
                Greater,
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) => Ok(BooleanValue(
                Self::date_datetime_ordering(date, datetime) == Ordering::Greater,
            )),
            (
                DateValue(ValueOrSv::Value(date)),
                LessEquals,
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) => Ok(BooleanValue({
                let ordering = Self::date_datetime_ordering(date, datetime);
                ordering == Ordering::Less || ordering == Ordering::Equal
            })),
            (
                DateValue(ValueOrSv::Value(date)),
                GreaterEquals,
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) => Ok(BooleanValue({
                let ordering = Self::date_datetime_ordering(date, datetime);
                ordering == Ordering::Greater || ordering == Ordering::Equal
            })),

            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                Equals,
                DateValue(ValueOrSv::Value(date)),
            ) => Ok(BooleanValue(
                Self::datetime_date_ordering(datetime, date) == Ordering::Equal,
            )),
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                NotEquals,
                DateValue(ValueOrSv::Value(date)),
            ) => Ok(BooleanValue(
                Self::datetime_date_ordering(datetime, date) != Ordering::Equal,
            )),
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                Less,
                DateValue(ValueOrSv::Value(date)),
            ) => Ok(BooleanValue(
                Self::datetime_date_ordering(datetime, date) == Ordering::Less,
            )),
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                Greater,
                DateValue(ValueOrSv::Value(date)),
            ) => Ok(BooleanValue(
                Self::datetime_date_ordering(datetime, date) == Ordering::Greater,
            )),
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                LessEquals,
                DateValue(ValueOrSv::Value(date)),
            ) => Ok(BooleanValue({
                let ordering = Self::datetime_date_ordering(datetime, date);
                ordering == Ordering::Less || ordering == Ordering::Equal
            })),
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                GreaterEquals,
                DateValue(ValueOrSv::Value(date)),
            ) => Ok(BooleanValue({
                let ordering = Self::datetime_date_ordering(datetime, date);
                ordering == Ordering::Greater || ordering == Ordering::Equal
            })),

            (
                ValueEnum::DurationValue(ValueOrSv::Value(a)),
                Equals,
                ValueEnum::DurationValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue(a == b)),
            (
                ValueEnum::DurationValue(ValueOrSv::Value(a)),
                NotEquals,
                ValueEnum::DurationValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue(a != b)),
            (
                ValueEnum::DurationValue(ValueOrSv::Value(a)),
                Less,
                ValueEnum::DurationValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue(
                self.duration_ordering(a, b)? == Ordering::Less,
            )),
            (
                ValueEnum::DurationValue(ValueOrSv::Value(a)),
                Greater,
                ValueEnum::DurationValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue(
                self.duration_ordering(a, b)? == Ordering::Greater,
            )),
            (
                ValueEnum::DurationValue(ValueOrSv::Value(a)),
                LessEquals,
                ValueEnum::DurationValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue({
                let ordering = self.duration_ordering(a, b)?;
                ordering == Ordering::Less || ordering == Ordering::Equal
            })),
            (
                ValueEnum::DurationValue(ValueOrSv::Value(a)),
                GreaterEquals,
                ValueEnum::DurationValue(ValueOrSv::Value(b)),
            ) => Ok(BooleanValue({
                let ordering = self.duration_ordering(a, b)?;
                ordering == Ordering::Greater || ordering == Ordering::Equal
            })),

            (left, comparator, right) => RuntimeError::eval_error(format!(
                "Not possible to compare {} {} {}",
                left, comparator, right
            ))
            .into(),
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
