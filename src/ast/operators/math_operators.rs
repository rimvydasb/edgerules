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
use crate::typesystem::types::string::StringEnum as TStringEnum;
use crate::typesystem::types::ValueType::StringType;
use crate::typesystem::types::ValueType::{
    DateTimeType, DateType, DurationType, NumberType, TimeType,
};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{
    DateTimeValue, DateValue, DurationValue, NumberValue, StringValue, TimeValue,
};
use crate::typesystem::values::{DurationValue as ErDurationValue, ValueOrSv};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use time::Duration as TDuration;

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
        if matches!(left_type, ValueType::UndefinedType)
            || matches!(right_type, ValueType::UndefinedType)
        {
            return Ok(NumberType);
        }

        // Numeric operations default
        if matches!(left_type, NumberType) && matches!(right_type, NumberType) {
            return Ok(NumberType);
        }

        match self.data.operator {
            Addition => {
                // First handle string combinations explicitly for readability
                match (left_type.clone(), right_type.clone()) {
                    (StringType, StringType) => Ok(StringType),
                    (lt, StringType) => LinkingError::expect_single_type(
                        "Left side of operator '+'",
                        lt,
                        &StringType,
                    ),
                    (StringType, rt) => LinkingError::expect_single_type(
                        "Right side of operator '+'",
                        rt,
                        &StringType,
                    ),
                    // durations
                    (DurationType, DurationType) => Ok(DurationType),
                    // date +/- duration (commutative)
                    (DateType, DurationType) | (DurationType, DateType) => Ok(DateType),
                    // datetime +/- duration (commutative)
                    (DateTimeType, DurationType) | (DurationType, DateTimeType) => Ok(DateTimeType),
                    // time +/- duration (commutative)
                    (TimeType, DurationType) | (DurationType, TimeType) => Ok(TimeType),
                    // date mixed with datetime (date treated as midnight)
                    (DateType, DateTimeType) | (DateTimeType, DateType) => Ok(DateTimeType),
                    // Fallback: not a supported '+' combo
                    _ => LinkingError::types_not_compatible(
                        Some("Operator '+'".to_string()),
                        left_type,
                        Some(vec![
                            NumberType,
                            DateType,
                            DateTimeType,
                            TimeType,
                            DurationType,
                        ]),
                    )
                    .into(),
                }
            }
            Subtraction => {
                // date - date => duration; time - time => duration; datetime - datetime => duration
                // date - duration => date; datetime - duration => datetime; time - duration => time
                match (left_type.clone(), right_type.clone()) {
                    (DateType, DateType) | (TimeType, TimeType) | (DateTimeType, DateTimeType) => {
                        Ok(DurationType)
                    }
                    (DateType, DurationType) => Ok(DateType),
                    (DateTimeType, DurationType) => Ok(DateTimeType),
                    (TimeType, DurationType) => Ok(TimeType),
                    (DurationType, DurationType) => Ok(DurationType),
                    (DateType, DateTimeType) | (DateTimeType, DateType) => Ok(DurationType),
                    _ => LinkingError::types_not_compatible(
                        Some("Operator '-'".to_string()),
                        left_type,
                        Some(vec![
                            NumberType,
                            DateType,
                            DateTimeType,
                            TimeType,
                            DurationType,
                        ]),
                    )
                    .into(),
                }
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

fn operate_duration_values(
    operator: &MathOperatorEnum,
    left: &ErDurationValue,
    right: &ErDurationValue,
) -> Result<ErDurationValue, RuntimeError> {
    match operator {
        MathOperatorEnum::Addition | MathOperatorEnum::Subtraction => {}
        other => {
            return Err(RuntimeError::eval_error(format!(
                "Unsupported operator '{}' for duration values",
                other.as_str()
            )))
        }
    }

    let (left_months, left_seconds) = left.components();
    let (right_months, right_seconds) = right.components();

    let (months_total, seconds_total) = if matches!(operator, MathOperatorEnum::Addition) {
        (left_months + right_months, left_seconds + right_seconds)
    } else {
        (left_months - right_months, left_seconds - right_seconds)
    };

    ErDurationValue::from_components(months_total, seconds_total)
}

fn shift_date_by_months(date: time::Date, months_delta: i128) -> Result<time::Date, RuntimeError> {
    if months_delta == 0 {
        return Ok(date);
    }

    let delta_i32 = i32::try_from(months_delta).map_err(|_| {
        RuntimeError::eval_error("Month offset is out of range for date adjustment".to_string())
    })?;

    let year = date.year();
    let mut month = date.month() as i32;
    month += delta_i32;
    let mut new_year = year + (month - 1) / 12;
    let mut new_month = (month - 1) % 12 + 1;
    if new_month <= 0 {
        new_year -= 1;
        new_month += 12;
    }
    let new_month_u8 = new_month as u8;
    let day = date.day();
    let last = super::super::functions::function_date::last_day_of_month(new_year, new_month_u8);
    let new_day = if day > last { last } else { day };
    time::Date::from_calendar_date(
        new_year,
        time::Month::try_from(new_month_u8).unwrap(),
        new_day,
    )
    .map_err(|_| {
        RuntimeError::eval_error("Invalid date produced by duration adjustment".to_string())
    })
}

fn apply_seconds_to_date(
    date: time::Date,
    seconds_delta: i128,
) -> Result<time::Date, RuntimeError> {
    if seconds_delta == 0 {
        return Ok(date);
    }

    let seconds_i64 = i64::try_from(seconds_delta).map_err(|_| {
        RuntimeError::eval_error("Second offset is out of range for date adjustment".to_string())
    })?;
    let duration = TDuration::seconds(seconds_i64);
    let adjusted = datetime_at_midnight(date) + duration;
    Ok(adjusted.date())
}

fn apply_duration_to_date(
    date: time::Date,
    duration: &ErDurationValue,
    operator: &MathOperatorEnum,
) -> Result<time::Date, RuntimeError> {
    let (mut months_delta, mut seconds_delta) = duration.components();
    if matches!(operator, MathOperatorEnum::Subtraction) {
        months_delta = -months_delta;
        seconds_delta = -seconds_delta;
    }

    let mut current = shift_date_by_months(date, months_delta)?;
    current = apply_seconds_to_date(current, seconds_delta)?;
    Ok(current)
}

fn apply_duration_to_datetime(
    datetime: time::PrimitiveDateTime,
    duration: &ErDurationValue,
    operator: &MathOperatorEnum,
) -> Result<time::PrimitiveDateTime, RuntimeError> {
    let (mut months_delta, mut seconds_delta) = duration.components();
    if matches!(operator, MathOperatorEnum::Subtraction) {
        months_delta = -months_delta;
        seconds_delta = -seconds_delta;
    }

    let mut current = datetime;
    if months_delta != 0 {
        let adjusted_date = shift_date_by_months(current.date(), months_delta)?;
        current = time::PrimitiveDateTime::new(adjusted_date, current.time());
    }
    if seconds_delta != 0 {
        let seconds_i64 = i64::try_from(seconds_delta).map_err(|_| {
            RuntimeError::eval_error(
                "Second offset is out of range for datetime adjustment".to_string(),
            )
        })?;
        let delta = TDuration::seconds(seconds_i64);
        current = current + delta;
    }
    Ok(current)
}

fn apply_duration_to_time(
    time: time::Time,
    duration: &ErDurationValue,
    operator: &MathOperatorEnum,
) -> Result<time::Time, RuntimeError> {
    let (months_delta, mut seconds_delta) = duration.components();
    if months_delta != 0 {
        return RuntimeError::eval_error(
            "Cannot apply month-based duration to time values".to_string(),
        )
        .into();
    }

    if matches!(operator, MathOperatorEnum::Subtraction) {
        seconds_delta = -seconds_delta;
    }

    let mut total_secs: i128 =
        (time.hour() as i128) * 3_600 + (time.minute() as i128) * 60 + time.second() as i128;
    total_secs += seconds_delta;

    let secs_mod = total_secs.rem_euclid(86_400);
    let hours = (secs_mod / 3_600) as u8;
    let minutes = ((secs_mod % 3_600) / 60) as u8;
    let seconds = (secs_mod % 60) as u8;
    time::Time::from_hms(hours, minutes, seconds).map_err(|_| {
        RuntimeError::eval_error("Invalid time produced by duration adjustment".to_string())
    })
}

fn datetime_at_midnight(date: time::Date) -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT)
}

fn merge_date_with_datetime(
    date: time::Date,
    datetime: &time::PrimitiveDateTime,
) -> time::PrimitiveDateTime {
    let base = datetime_at_midnight(date);
    let start_of_day = datetime_at_midnight(datetime.date());
    base + (*datetime - start_of_day)
}

fn duration_from_time_diff(diff: time::Duration) -> ErDurationValue {
    let neg = diff.is_negative();
    let total_secs = diff.whole_seconds().abs() as i128;
    ErDurationValue::dt_from_total_seconds(total_secs, neg)
}

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
            // string + string -> concatenate raw contents (no quotes added)
            (StringValue(ls), StringValue(rs)) if matches!(self.data.operator, Addition) => {
                let mut out = String::new();
                match ls {
                    TStringEnum::String(s) => out.push_str(s),
                    TStringEnum::Char(c) => out.push(*c),
                    TStringEnum::SV(_) => {}
                }
                match rs {
                    TStringEnum::String(s) => out.push_str(s),
                    TStringEnum::Char(c) => out.push(*c),
                    TStringEnum::SV(_) => {}
                }
                Ok(ValueEnum::StringValue(TStringEnum::String(out)))
            }
            (
                DurationValue(ValueOrSv::Value(left_dur)),
                DurationValue(ValueOrSv::Value(right_dur)),
            ) => {
                let combined = operate_duration_values(&self.data.operator, left_dur, right_dur)?;
                Ok(DurationValue(ValueOrSv::Value(combined)))
            }
            (DateValue(ValueOrSv::Value(date)), DurationValue(ValueOrSv::Value(duration))) => {
                let result = apply_duration_to_date(*date, duration, &self.data.operator)?;
                Ok(DateValue(ValueOrSv::Value(result)))
            }
            (DurationValue(ValueOrSv::Value(duration)), DateValue(ValueOrSv::Value(date)))
                if matches!(self.data.operator, Addition) =>
            {
                let result = apply_duration_to_date(*date, duration, &MathOperatorEnum::Addition)?;
                Ok(DateValue(ValueOrSv::Value(result)))
            }
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                DurationValue(ValueOrSv::Value(duration)),
            ) => {
                let result = apply_duration_to_datetime(*datetime, duration, &self.data.operator)?;
                Ok(DateTimeValue(ValueOrSv::Value(result)))
            }
            (
                DurationValue(ValueOrSv::Value(duration)),
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) if matches!(self.data.operator, Addition) => {
                let result =
                    apply_duration_to_datetime(*datetime, duration, &MathOperatorEnum::Addition)?;
                Ok(DateTimeValue(ValueOrSv::Value(result)))
            }
            (TimeValue(ValueOrSv::Value(time)), DurationValue(ValueOrSv::Value(duration))) => {
                let result = apply_duration_to_time(*time, duration, &self.data.operator)?;
                Ok(TimeValue(ValueOrSv::Value(result)))
            }
            (DurationValue(ValueOrSv::Value(duration)), TimeValue(ValueOrSv::Value(time)))
                if matches!(self.data.operator, Addition) =>
            {
                let result = apply_duration_to_time(*time, duration, &MathOperatorEnum::Addition)?;
                Ok(TimeValue(ValueOrSv::Value(result)))
            }
            (DateValue(ValueOrSv::Value(date)), DateTimeValue(ValueOrSv::Value(dt))) => {
                match self.data.operator {
                    Addition => {
                        let combined = merge_date_with_datetime(*date, dt);
                        Ok(DateTimeValue(ValueOrSv::Value(combined)))
                    }
                    Subtraction => {
                        let left_dt = datetime_at_midnight(*date);
                        let diff = left_dt - *dt;
                        Ok(DurationValue(ValueOrSv::Value(duration_from_time_diff(
                            diff,
                        ))))
                    }
                    _ => RuntimeError::eval_error(
                        "Unsupported operator for date and datetime".to_string(),
                    )
                    .into(),
                }
            }
            (DateTimeValue(ValueOrSv::Value(dt)), DateValue(ValueOrSv::Value(date))) => {
                match self.data.operator {
                    Addition => {
                        let combined = merge_date_with_datetime(*date, dt);
                        Ok(DateTimeValue(ValueOrSv::Value(combined)))
                    }
                    Subtraction => {
                        let right_dt = datetime_at_midnight(*date);
                        let diff = *dt - right_dt;
                        Ok(DurationValue(ValueOrSv::Value(duration_from_time_diff(
                            diff,
                        ))))
                    }
                    _ => RuntimeError::eval_error(
                        "Unsupported operator for datetime and date".to_string(),
                    )
                    .into(),
                }
            }
            // date - date => duration
            (DateValue(ValueOrSv::Value(a)), DateValue(ValueOrSv::Value(b)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let diff = (*a - *b).whole_days();
                let neg = diff < 0;
                let days = diff.abs();
                Ok(DurationValue(ValueOrSv::Value(ErDurationValue::dt(
                    days, 0, 0, 0, neg,
                ))))
            }
            // time - time => duration (seconds)
            (TimeValue(ValueOrSv::Value(a)), TimeValue(ValueOrSv::Value(b)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let a_secs =
                    (a.hour() as i64) * 3600 + (a.minute() as i64) * 60 + (a.second() as i64);
                let b_secs =
                    (b.hour() as i64) * 3600 + (b.minute() as i64) * 60 + (b.second() as i64);
                let mut diff = a_secs - b_secs;
                let neg = diff < 0;
                if neg {
                    diff = -diff;
                }
                let hours = diff / 3600;
                let minutes = (diff % 3600) / 60;
                let seconds = diff % 60;
                Ok(DurationValue(ValueOrSv::Value(ErDurationValue::dt(
                    0, hours, minutes, seconds, neg,
                ))))
            }
            // datetime - datetime => duration (days/hours/minutes/seconds)
            (DateTimeValue(ValueOrSv::Value(a)), DateTimeValue(ValueOrSv::Value(b)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let diff = *a - *b;
                Ok(DurationValue(ValueOrSv::Value(duration_from_time_diff(
                    diff,
                ))))
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
