use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::operators::math_operators::MathOperatorEnum::*;
use crate::ast::token::EToken::Unparsed;
use crate::ast::token::ExpressionEnum::{Value, Variable};
use crate::ast::token::{EToken, EUnparsedToken, ExpressionEnum};
use crate::ast::Link;
use crate::runtime::execution_context::*;
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::number::NumberEnum::{Int, Real};
use crate::typesystem::types::ValueType::{NumberType, DateType, TimeType, DateTimeType, DurationType};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{NumberValue, DateValue, TimeValue, DateTimeValue, DurationValue};
use crate::typesystem::values::{DurationValue as ErDurationValue, DurationKind, ValueOrSv};
use time::{Duration as TDuration};
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

//----------------------------------------------------------------------------------------------
// Operator

pub type BinaryNumberFunction =
    fn(a: NumberEnum, b: NumberEnum) -> Result<NumberEnum, RuntimeError>;

pub trait Operator: Display + Debug + EvaluatableExpression {}

#[derive(Debug, PartialEq)]
pub struct OperatorData<T: Display> {
    pub operator: T,
    pub left: ExpressionEnum,
    pub right: ExpressionEnum,
}

impl<T: Display> OperatorData<T> {
    pub fn link(
        &mut self,
        ctx: Rc<RefCell<ContextObject>>,
        expected_type: ValueType,
    ) -> Link<ValueType> {
        let left_type = self.left.link(ctx.clone())?;
        let right_type = self.right.link(ctx)?;

        // If either side is undefined (deferred linking), accept the operator type
        // without strict checking and let runtime enforce actual types.
        if left_type == ValueType::UndefinedType || right_type == ValueType::UndefinedType {
            return Ok(expected_type);
        }

        let expected = LinkingError::expect_same_types(
            self.operator.to_string().as_str(),
            left_type,
            right_type.clone(),
        )?;
        LinkingError::expect_single_type("Left side of operator", expected, &expected_type)?;
        LinkingError::expect_single_type("Right side of operator", right_type, &expected_type)
    }
}

//----------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum MathOperatorEnum {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Power,
    Modulus,
}

impl MathOperatorEnum {
    pub fn as_str(&self) -> &'static str {
        match self {
            Addition => "+",
            Subtraction => "-",
            Multiplication => "*",
            Division => "/",
            Power => "^",
            Modulus => "%",
        }
    }

    pub fn build(operator: &str) -> EToken {
        match MathOperatorEnum::try_from(operator) {
            Ok(operator) => operator.into(),
            Err(error) => EToken::ParseError(error),
        }
    }
}

impl TryFrom<&str> for MathOperatorEnum {
    type Error = ParseErrorEnum;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Addition),
            "-" => Ok(Subtraction),
            "*" | "×" => Ok(Multiplication),
            "/" | "÷" => Ok(Division),
            "^" => Ok(Power),
            "%" => Ok(Modulus),
            _ => Err(ParseErrorEnum::UnknownParseError(format!(
                "Unknown operator: {}",
                value
            ))),
        }
    }
}

impl TryFrom<EToken> for MathOperatorEnum {
    type Error = ParseErrorEnum;

    fn try_from(value: EToken) -> Result<Self, Self::Error> {
        if let Unparsed(EUnparsedToken::MathOperatorToken(operator)) = value {
            Ok(operator)
        } else {
            Err(ParseErrorEnum::UnknownParseError(format!(
                "Unknown operator: {}",
                value
            )))
        }
    }
}

impl From<MathOperatorEnum> for EToken {
    fn from(val: MathOperatorEnum) -> Self {
        Unparsed(EUnparsedToken::MathOperatorToken(val))
    }
}

//----------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct MathOperator {
    pub data: OperatorData<MathOperatorEnum>,
    pub function: BinaryNumberFunction,
}

impl TypedValue for MathOperator {
    fn get_type(&self) -> ValueType {
        NumberType
    }
}

impl StaticLink for MathOperator {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        // Custom linking to support date/time/datetime with duration
        let left_type = self.data.left.link(Rc::clone(&ctx))?;
        let right_type = self.data.right.link(ctx)?;

        // If either side is unresolved, defer with a sensible default (number)
        if matches!(left_type, ValueType::UndefinedType) || matches!(right_type, ValueType::UndefinedType) {
            return Ok(NumberType);
        }

        // Numeric operations default
        if matches!(left_type, NumberType) && matches!(right_type, NumberType) {
            return Ok(NumberType);
        }

        match self.data.operator {
            Addition => {
                // date + duration => date; datetime + duration => datetime; time + duration => time (optional)
                return match (left_type.clone(), right_type) {
                    (DateType, DurationType) => Ok(DateType),
                    (DateTimeType, DurationType) => Ok(DateTimeType),
                    (TimeType, DurationType) => Ok(TimeType),
                    _ => LinkingError::types_not_compatible(
                        Some("Operator '+'".to_string()),
                        left_type,
                        Some(vec![NumberType, DateType, DateTimeType, TimeType])
                    ).into(),
                };
            }
            Subtraction => {
                // date - date => duration; time - time => duration; datetime - datetime => duration
                // date - duration => date; datetime - duration => datetime; time - duration => time
                return match (left_type.clone(), right_type.clone()) {
                    (DateType, DateType) | (TimeType, TimeType) | (DateTimeType, DateTimeType) => Ok(DurationType),
                    (DateType, DurationType) => Ok(DateType),
                    (DateTimeType, DurationType) => Ok(DateTimeType),
                    (TimeType, DurationType) => Ok(TimeType),
                    _ => LinkingError::types_not_compatible(
                        Some("Operator '-'".to_string()),
                        left_type,
                        Some(vec![NumberType, DateType, DateTimeType, TimeType])
                    ).into(),
                };
            }
            Multiplication | Division | Power | Modulus => {
                // Only numbers supported
                LinkingError::expect_same_types("Operator", left_type, right_type.clone())?;
                LinkingError::expect_single_type("Operator", right_type, &NumberType)
            }
        }
    }
}

impl MathOperator {
    pub fn build(
        operator: MathOperatorEnum,
        left: ExpressionEnum,
        right: ExpressionEnum,
    ) -> Result<Self, ParseErrorEnum> {
        let function = match operator {
            Addition => |left: NumberEnum, right: NumberEnum| -> Result<NumberEnum, RuntimeError> {
                Ok(left + right)
            },
            Subtraction => |left: NumberEnum,
                            right: NumberEnum|
             -> Result<NumberEnum, RuntimeError> { Ok(left - right) },
            Multiplication => |left: NumberEnum,
                               right: NumberEnum|
             -> Result<NumberEnum, RuntimeError> { Ok(left * right) },
            Division => |left: NumberEnum, right: NumberEnum| -> Result<NumberEnum, RuntimeError> {
                Ok(left / right)
            },
            Power => |left: NumberEnum, right: NumberEnum| -> Result<NumberEnum, RuntimeError> {
                match (left, right) {
                    (Int(left), Int(right)) => Ok(NumberEnum::from(left.pow(right as u32))),
                    (Real(left), Int(right)) => Ok(NumberEnum::from(left.powi(right as i32))),
                    (Int(left), Real(right)) => Ok(NumberEnum::from(left.pow(right as u32))),
                    (Real(left), Real(right)) => Ok(NumberEnum::from(left.powf(right))),
                    (left, right) => RuntimeError::eval_error(format!(
                        "Operator '^' is not implemented for '{} ^ {}'",
                        left, right
                    ))
                    .into(),
                }
            },
            Modulus => |left: NumberEnum, right: NumberEnum| -> Result<NumberEnum, RuntimeError> {
                Ok(left % right)
            },
        };

        Ok(MathOperator {
            data: OperatorData {
                operator,
                left,
                right,
            },
            function,
        })
    }
}

impl Operator for MathOperator {}

impl PartialEq for MathOperator {
    fn eq(&self, other: &Self) -> bool {
        // Compare semantic parts; function is determined by operator
        self.data == other.data
    }
}

impl EvaluatableExpression for MathOperator {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let left_token = &self.data.left.eval(Rc::clone(&context))?;
        let right_token = &self.data.right.eval(context)?;

        match (left_token, right_token) {
            (NumberValue(_left), NumberValue(_right)) => {
                Ok(NumberValue((self.function)(_left.clone(), _right.clone())?))
            }
            // date +/- duration
            (DateValue(ValueOrSv::Value(d)), DurationValue(ValueOrSv::Value(dur))) => {
                match self.data.operator {
                    Addition => match dur.kind {
                        DurationKind::YearsMonths => {
                            let mut year = d.year();
                            let mut month = d.month() as i32; // 1..12
                            let mut add_months = dur.years * 12 + dur.months;
                            if dur.negative { add_months = -add_months; }
                            month += add_months;
                            let mut new_year = year + (month - 1) / 12;
                            let mut new_month = (month - 1) % 12 + 1;
                            if new_month <= 0 {
                                new_year -= 1;
                                new_month += 12;
                            }
                            let new_month_u8 = new_month as u8;
                            let day = d.day();
                            let last = super::super::functions::function_mix::last_day_of_month(new_year, new_month_u8);
                            let new_day = if day > last { last } else { day };
                            let date = time::Date::from_calendar_date(new_year, time::Month::try_from(new_month_u8).unwrap(), new_day).unwrap();
                            Ok(DateValue(ValueOrSv::Value(date)))
                        }
                        DurationKind::DaysTime => {
                            let mut total_days = dur.days;
                            if dur.negative { total_days = -total_days; }
                            let date = *d + TDuration::days(total_days);
                            Ok(DateValue(ValueOrSv::Value(date)))
                        }
                    },
                    Subtraction => match dur.kind {
                        DurationKind::YearsMonths => {
                            // date - (Ym) => date by subtracting months
                            let year = d.year();
                            let mut month = d.month() as i32;
                            let mut sub_months = dur.years * 12 + dur.months;
                            if !dur.negative { sub_months = sub_months; } else { sub_months = -sub_months; }
                            month -= sub_months;
                            let mut new_year = year + (month - 1) / 12;
                            let mut new_month = (month - 1) % 12 + 1;
                            if new_month <= 0 {
                                new_year -= 1;
                                new_month += 12;
                            }
                            let new_month_u8 = new_month as u8;
                            let day = d.day();
                            let last = super::super::functions::function_mix::last_day_of_month(new_year, new_month_u8);
                            let new_day = if day > last { last } else { day };
                            let date = time::Date::from_calendar_date(new_year, time::Month::try_from(new_month_u8).unwrap(), new_day).unwrap();
                            Ok(DateValue(ValueOrSv::Value(date)))
                        }
                        DurationKind::DaysTime => {
                            let mut total_days = dur.days;
                            if dur.negative { total_days = -total_days; }
                            let date = *d - TDuration::days(total_days);
                            Ok(DateValue(ValueOrSv::Value(date)))
                        }
                    },
                    _ => RuntimeError::eval_error("Unsupported operator for date and duration".to_string()).into(),
                }
            }
            // datetime +/- duration
            (DateTimeValue(ValueOrSv::Value(dt)), DurationValue(ValueOrSv::Value(dur))) => {
                match self.data.operator {
                    Addition => match dur.kind {
                        DurationKind::YearsMonths => {
                            let d = dt.date();
                            let t = dt.time();
                            let new_date = {
                                let mut year = d.year();
                                let mut month = d.month() as i32; // 1..12
                                let mut add_months = dur.years * 12 + dur.months;
                                if dur.negative { add_months = -add_months; }
                                month += add_months;
                                let mut new_year = year + (month - 1) / 12;
                                let mut new_month = (month - 1) % 12 + 1;
                                if new_month <= 0 { new_year -= 1; new_month += 12; }
                                let new_month_u8 = new_month as u8;
                                let day = d.day();
                                let last = super::super::functions::function_mix::last_day_of_month(new_year, new_month_u8);
                                let new_day = if day > last { last } else { day };
                                time::Date::from_calendar_date(new_year, time::Month::try_from(new_month_u8).unwrap(), new_day).unwrap()
                            };
                            Ok(DateTimeValue(ValueOrSv::Value(time::PrimitiveDateTime::new(new_date, t))))
                        }
                        DurationKind::DaysTime => {
                            let mut delta = TDuration::days(dur.days.abs());
                            delta = delta + TDuration::hours(dur.hours.abs());
                            delta = delta + TDuration::minutes(dur.minutes.abs());
                            delta = delta + TDuration::seconds(dur.seconds.abs());
                            if dur.negative { delta = -delta; }
                            Ok(DateTimeValue(ValueOrSv::Value(*dt + delta)))
                        }
                    },
                    Subtraction => match dur.kind {
                        DurationKind::YearsMonths => {
                            let d = dt.date();
                            let t = dt.time();
                            let new_date = {
                                let mut year = d.year();
                                let mut month = d.month() as i32;
                                let mut sub_months = dur.years * 12 + dur.months;
                                if !dur.negative { sub_months = sub_months; } else { sub_months = -sub_months; }
                                month -= sub_months;
                                let mut new_year = year + (month - 1) / 12;
                                let mut new_month = (month - 1) % 12 + 1;
                                if new_month <= 0 { new_year -= 1; new_month += 12; }
                                let new_month_u8 = new_month as u8;
                                let day = d.day();
                                let last = super::super::functions::function_mix::last_day_of_month(new_year, new_month_u8);
                                let new_day = if day > last { last } else { day };
                                time::Date::from_calendar_date(new_year, time::Month::try_from(new_month_u8).unwrap(), new_day).unwrap()
                            };
                            Ok(DateTimeValue(ValueOrSv::Value(time::PrimitiveDateTime::new(new_date, t))))
                        }
                        DurationKind::DaysTime => {
                            let mut delta = TDuration::days(dur.days.abs());
                            delta = delta + TDuration::hours(dur.hours.abs());
                            delta = delta + TDuration::minutes(dur.minutes.abs());
                            delta = delta + TDuration::seconds(dur.seconds.abs());
                            if !dur.negative { delta = delta; } else { delta = -delta; }
                            Ok(DateTimeValue(ValueOrSv::Value(*dt - delta)))
                        }
                    },
                    _ => RuntimeError::eval_error("Unsupported operator for datetime and duration".to_string()).into(),
                }
            }
            // time +/- duration (days ignored, only H/M/S applied modulo 24h)
            (TimeValue(ValueOrSv::Value(t)), DurationValue(ValueOrSv::Value(dur))) => {
                match self.data.operator {
                    Addition | Subtraction => {
                        let mut secs: i64 = (t.hour() as i64) * 3600 + (t.minute() as i64) * 60 + (t.second() as i64);
                        let delta: i64 = dur.hours.abs() * 3600 + dur.minutes.abs() * 60 + dur.seconds.abs();
                        if dur.negative ^ matches!(self.data.operator, Subtraction) {
                            // negative addition or positive subtraction => subtract
                            secs -= delta;
                        } else {
                            secs += delta;
                        }
                        let secs_mod = ((secs % 86400) + 86400) % 86400;
                        let h = (secs_mod / 3600) as u8;
                        let m = ((secs_mod % 3600) / 60) as u8;
                        let s = (secs_mod % 60) as u8;
                        let new_time = time::Time::from_hms(h, m, s).unwrap();
                        Ok(TimeValue(ValueOrSv::Value(new_time)))
                    }
                    _ => RuntimeError::eval_error("Unsupported operator for time and duration".to_string()).into(),
                }
            }
            // date - date => duration
            (DateValue(ValueOrSv::Value(a)), DateValue(ValueOrSv::Value(b))) if matches!(self.data.operator, Subtraction) => {
                let diff = (*a - *b).whole_days();
                let neg = diff < 0;
                let days = diff.abs();
                Ok(DurationValue(ValueOrSv::Value(ErDurationValue::dt(days, 0, 0, 0, neg))))
            }
            // time - time => duration (seconds)
            (TimeValue(ValueOrSv::Value(a)), TimeValue(ValueOrSv::Value(b))) if matches!(self.data.operator, Subtraction) => {
                let a_secs = (a.hour() as i64) * 3600 + (a.minute() as i64) * 60 + (a.second() as i64);
                let b_secs = (b.hour() as i64) * 3600 + (b.minute() as i64) * 60 + (b.second() as i64);
                let mut diff = a_secs - b_secs;
                let neg = diff < 0;
                if neg { diff = -diff; }
                let hours = diff / 3600;
                let minutes = (diff % 3600) / 60;
                let seconds = diff % 60;
                Ok(DurationValue(ValueOrSv::Value(ErDurationValue::dt(0, hours, minutes, seconds, neg))))
            }
            // datetime - datetime => duration (days/hours/minutes/seconds)
            (DateTimeValue(ValueOrSv::Value(a)), DateTimeValue(ValueOrSv::Value(b))) if matches!(self.data.operator, Subtraction) => {
                let diff = *a - *b;
                let neg = diff.is_negative();
                let total_secs = diff.whole_seconds().abs();
                let days = total_secs / 86400;
                let rem = total_secs % 86400;
                let hours = rem / 3600;
                let rem2 = rem % 3600;
                let minutes = rem2 / 60;
                let seconds = rem2 % 60;
                Ok(DurationValue(ValueOrSv::Value(ErDurationValue::dt(days, hours, minutes, seconds, neg))))
            }
            _ => RuntimeError::eval_error(format!(
                "Operator '{}' is not implemented for '{} {} {}'",
                self.data.operator, left_token, self.data.operator, right_token
            ))
            .into(),
        }
    }
}

//----------------------------------------------------------------------------------------------
// Display
//----------------------------------------------------------------------------------------------

impl Display for MathOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.data, f)
    }
}

impl Display for MathOperatorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for OperatorData<MathOperatorEnum> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.operator {
            Multiplication | Division | Power => {
                write!(f, "{} {} {}", self.left, self.operator, self.right)
            }
            _ => write!(f, "({} {} {})", self.left, self.operator, self.right),
        }
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub struct NegationOperator {
    pub left: ExpressionEnum,
}

impl NegationOperator {
    pub fn new(left: ExpressionEnum) -> NegationOperator {
        NegationOperator { left }
    }
}

impl Display for NegationOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.left {
            Value(_) | Variable(_) => write!(f, "-{}", self.left),
            _ => write!(f, "-({})", self.left),
        }
    }
}

// impl TypedValue for NegationOperator {
//     fn get_type(&self) -> ValueType {
//         self.left.get_type()
//     }
// }

impl StaticLink for NegationOperator {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        let result = self.left.link(Rc::clone(&ctx))?;
        LinkingError::expect_type(None, result, &[NumberType])
    }
}

impl EvaluatableExpression for NegationOperator {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        match self.left.eval(context)? {
            NumberValue(number) => Ok(NumberValue(number.negate())),
            value => {
                RuntimeError::eval_error(format!("Cannot negate '{}'", value.get_type())).into()
            }
        }
    }
}
