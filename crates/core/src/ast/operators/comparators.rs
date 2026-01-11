use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::operators::comparators::ComparatorEnum::*;
use crate::ast::operators::math_operators::{Operator, OperatorData};
use crate::ast::token::ExpressionEnum;
use crate::ast::Link;
use crate::runtime::execution_context::ExecutionContext;
use crate::tokenizer::utils::CharStream;
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{
    BooleanValue, DateTimeValue, DateValue, DurationValue as DurationVariant, NumberValue,
    PeriodValue as PeriodVariant, StringValue, TimeValue,
};
use crate::typesystem::values::ValueOrSv;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

//----------------------------------------------------------------------------------------------

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum ComparatorEnum {
    Equals,
    NotEquals,
    Less,
    Greater,
    LessEquals,
    GreaterEquals,
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
        match (iter.next_char().unwrap(), iter.peek()) {
            ('<', Some('=')) => {
                iter.next_char();
                Some(LessEquals)
            }
            ('>', Some('=')) => {
                iter.next_char();
                Some(GreaterEquals)
            }
            ('<', Some('>')) => {
                iter.next_char();
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

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq)]
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

            // periods support only equality / inequality
            (ValueType::PeriodType, Equals) | (ValueType::PeriodType, NotEquals) => {}
            (ValueType::PeriodType, operator) => {
                return Err(LinkingError::operation_not_supported(
                    operator.as_str(),
                    same_type.clone(),
                    same_type,
                ));
            }

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
    ) -> Option<Ordering> {
        left.partial_cmp(right)
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
        use crate::typesystem::values::ValueOrSv::Value;
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

            (TimeValue(Value(a)), LessEquals, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a <= b))
            }
            (TimeValue(Value(a)), GreaterEquals, TimeValue(ValueOrSv::Value(b))) => {
                Ok(BooleanValue(a >= b))
            }
            (TimeValue(Value(a)), Less, TimeValue(Value(b))) => Ok(BooleanValue(a < b)),
            (TimeValue(Value(a)), Greater, TimeValue(Value(b))) => Ok(BooleanValue(a > b)),

            (DateTimeValue(ValueOrSv::Value(a)), LessEquals, DateTimeValue(Value(b))) => {
                Ok(BooleanValue(a <= b))
            }
            (DateTimeValue(Value(a)), GreaterEquals, DateTimeValue(Value(b))) => {
                Ok(BooleanValue(a >= b))
            }
            (DateTimeValue(Value(a)), Less, DateTimeValue(Value(b))) => Ok(BooleanValue(a < b)),
            (DateTimeValue(Value(a)), Greater, DateTimeValue(Value(b))) => Ok(BooleanValue(a > b)),

            (DateValue(Value(date)), Equals, DateTimeValue(Value(datetime))) => Ok(BooleanValue(
                Self::date_datetime_ordering(date, datetime) == Ordering::Equal,
            )),
            (DateValue(Value(date)), NotEquals, DateTimeValue(Value(datetime))) => Ok(
                BooleanValue(Self::date_datetime_ordering(date, datetime) != Ordering::Equal),
            ),
            (DateValue(Value(date)), Less, DateTimeValue(Value(datetime))) => Ok(BooleanValue(
                Self::date_datetime_ordering(date, datetime) == Ordering::Less,
            )),
            (DateValue(Value(date)), Greater, DateTimeValue(Value(datetime))) => Ok(BooleanValue(
                Self::date_datetime_ordering(date, datetime) == Ordering::Greater,
            )),
            (DateValue(Value(date)), LessEquals, DateTimeValue(Value(datetime))) => {
                Ok(BooleanValue({
                    let ordering = Self::date_datetime_ordering(date, datetime);
                    ordering == Ordering::Less || ordering == Ordering::Equal
                }))
            }
            (DateValue(Value(date)), GreaterEquals, DateTimeValue(Value(datetime))) => {
                Ok(BooleanValue({
                    let ordering = Self::date_datetime_ordering(date, datetime);
                    ordering == Ordering::Greater || ordering == Ordering::Equal
                }))
            }

            (DateTimeValue(Value(datetime)), Equals, DateValue(Value(date))) => Ok(BooleanValue(
                Self::datetime_date_ordering(datetime, date) == Ordering::Equal,
            )),
            (DateTimeValue(Value(datetime)), NotEquals, DateValue(Value(date))) => Ok(
                BooleanValue(Self::datetime_date_ordering(datetime, date) != Ordering::Equal),
            ),
            (DateTimeValue(Value(datetime)), Less, DateValue(Value(date))) => Ok(BooleanValue(
                Self::datetime_date_ordering(datetime, date) == Ordering::Less,
            )),
            (DateTimeValue(Value(datetime)), Greater, DateValue(Value(date))) => Ok(BooleanValue(
                Self::datetime_date_ordering(datetime, date) == Ordering::Greater,
            )),
            (DateTimeValue(Value(datetime)), LessEquals, DateValue(Value(date))) => {
                Ok(BooleanValue({
                    let ordering = Self::datetime_date_ordering(datetime, date);
                    ordering == Ordering::Less || ordering == Ordering::Equal
                }))
            }
            (DateTimeValue(Value(datetime)), GreaterEquals, DateValue(Value(date))) => {
                Ok(BooleanValue({
                    let ordering = Self::datetime_date_ordering(datetime, date);
                    ordering == Ordering::Greater || ordering == Ordering::Equal
                }))
            }

            (DurationVariant(Value(a)), Equals, DurationVariant(Value(b))) => {
                Ok(BooleanValue(a == b))
            }
            (DurationVariant(Value(a)), NotEquals, DurationVariant(Value(b))) => {
                Ok(BooleanValue(a != b))
            }
            (DurationVariant(Value(a)), Less, DurationVariant(Value(b))) => match self
                .duration_ordering(a, b)
            {
                Some(ordering) => Ok(BooleanValue(ordering == Ordering::Less)),
                None => RuntimeError::internal_integrity_error(150).into(),
            },
            (DurationVariant(Value(a)), Greater, DurationVariant(Value(b))) => match self
                .duration_ordering(a, b)
            {
                Some(ordering) => Ok(BooleanValue(ordering == Ordering::Greater)),
                None => RuntimeError::internal_integrity_error(151).into(),
            },
            (DurationVariant(Value(a)), LessEquals, DurationVariant(Value(b))) => match self
                .duration_ordering(a, b)
            {
                Some(ordering) => Ok(BooleanValue(
                    ordering == Ordering::Less || ordering == Ordering::Equal,
                )),
                None => RuntimeError::internal_integrity_error(152).into(),
            },
            (DurationVariant(Value(a)), GreaterEquals, DurationVariant(Value(b))) => {
                match self.duration_ordering(a, b) {
                    Some(ordering) => Ok(BooleanValue(
                        ordering == Ordering::Greater || ordering == Ordering::Equal,
                    )),
                    None => {
                        trace!("Durations: a: {:?}, b: {:?}", a, b);
                        RuntimeError::internal_integrity_error(153).into()
                    }
                }
            }
            (PeriodVariant(Value(a)), Equals, PeriodVariant(Value(b))) => Ok(BooleanValue(a == b)),
            (PeriodVariant(Value(a)), NotEquals, PeriodVariant(Value(b))) => {
                Ok(BooleanValue(a != b))
            }
            (PeriodVariant(Value(_)), _comparator, PeriodVariant(Value(_))) => {
                RuntimeError::internal_integrity_error(154).into()
            }

            (_left, _comparator, _right) => RuntimeError::internal_integrity_error(155).into(),
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
