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
    DateTimeType, DateType, DurationType, NumberType, PeriodType, TimeType,
};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{
    DateTimeValue, DateValue, DurationValue as DurationVariant, NumberValue,
    PeriodValue as PeriodVariant, StringValue, TimeValue,
};
use crate::typesystem::values::{
    DurationValue as ErDurationValue, PeriodValue as ErPeriodValue, ValueOrSv,
};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::fmt::Debug;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use time::Duration as TDuration;

//----------------------------------------------------------------------------------------------
// Operator

pub type BinaryNumberFunction =
    fn(a: NumberEnum, b: NumberEnum) -> Result<NumberEnum, RuntimeError>;

#[cfg(not(target_arch = "wasm32"))]
pub trait Operator: Display + Debug + EvaluatableExpression {}

#[cfg(target_arch = "wasm32")]
pub trait Operator: Display + EvaluatableExpression {}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq)]
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

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
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

    pub fn build_from_char(operator: char) -> EToken {
        match MathOperatorEnum::try_from(operator) {
            Ok(operator) => operator.into(),
            Err(error) => EToken::ParseError(error),
        }
    }
}

impl TryFrom<&str> for MathOperatorEnum {
    type Error = ParseErrorEnum;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut chars = value.chars();
        if let Some(ch) = chars.next() {
            if chars.next().is_none() {
                return MathOperatorEnum::try_from(ch);
            }
        }

        Err(ParseErrorEnum::WrongFormat(format!(
            "Unknown operator: {}",
            value
        )))
    }
}

impl TryFrom<char> for MathOperatorEnum {
    type Error = ParseErrorEnum;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Addition),
            '-' => Ok(Subtraction),
            '*' | '×' => Ok(Multiplication),
            '/' | '÷' => Ok(Division),
            '^' => Ok(Power),
            '%' => Ok(Modulus),
            _ => Err(ParseErrorEnum::WrongFormat(format!(
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
            Err(ParseErrorEnum::WrongFormat(format!(
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

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
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
                    (DurationType, DurationType) => Ok(DurationType),
                    (PeriodType, PeriodType) => Ok(PeriodType),
                    (DateType, DurationType) | (DurationType, DateType) => Ok(DateTimeType),
                    (DateTimeType, DurationType) | (DurationType, DateTimeType) => Ok(DateTimeType),
                    (TimeType, DurationType) | (DurationType, TimeType) => Ok(TimeType),
                    (DateType, PeriodType) => Ok(DateType),
                    (PeriodType, DateType) => Ok(DateType),
                    (DateTimeType, PeriodType) => Ok(DateTimeType),
                    (PeriodType, DateTimeType) => Ok(DateTimeType),
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
                            PeriodType,
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
                    (DateType, DurationType) => Ok(DateTimeType),
                    (DateTimeType, DurationType) => Ok(DateTimeType),
                    (TimeType, DurationType) => Ok(TimeType),
                    (DurationType, DurationType) => Ok(DurationType),
                    (PeriodType, PeriodType) => Ok(PeriodType),
                    (DateType, PeriodType) => Ok(DateType),
                    (DateTimeType, PeriodType) => Ok(DateTimeType),
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
                match right {
                    NumberEnum::Int(0) => return Err(RuntimeError::division_by_zero()),
                    NumberEnum::Real(r) if r == 0.0 => return Err(RuntimeError::division_by_zero()),
                    _ => {}
                }
                Ok(left / right)
            },
            Power => |left: NumberEnum, right: NumberEnum| -> Result<NumberEnum, RuntimeError> {
                match (left, right) {
                    (Int(left), Int(right)) => {
                        if right < 0 {
                            Ok(NumberEnum::from((left as f64).powf(right as f64)))
                        } else {
                            Ok(NumberEnum::from(left.pow(right as u32)))
                        }
                    }
                    (Real(left), Int(right)) => Ok(NumberEnum::from(left.powi(right as i32))),
                    (Int(left), Real(right)) => Ok(NumberEnum::from((left as f64).powf(right))),
                    (Real(left), Real(right)) => Ok(NumberEnum::from(left.powf(right))),
                    _ => RuntimeError::internal_integrity_error(100).into(),
                }
            },
            Modulus => |left: NumberEnum, right: NumberEnum| -> Result<NumberEnum, RuntimeError> {
                match right {
                    NumberEnum::Int(0) => return Err(RuntimeError::division_by_zero()),
                    NumberEnum::Real(r) if r == 0.0 => return Err(RuntimeError::division_by_zero()),
                    _ => {}
                }
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
        _ => return Err(RuntimeError::internal_integrity_error(101).into()),
    }

    let left_seconds = left.signed_seconds();
    let right_seconds = right.signed_seconds();

    let total = if matches!(operator, MathOperatorEnum::Addition) {
        left_seconds + right_seconds
    } else {
        left_seconds - right_seconds
    };

    ErDurationValue::from_signed_seconds(total)
}

fn operate_period_values(
    operator: &MathOperatorEnum,
    left: &ErPeriodValue,
    right: &ErPeriodValue,
) -> Result<ErPeriodValue, RuntimeError> {
    match operator {
        MathOperatorEnum::Addition | MathOperatorEnum::Subtraction => {}
        _ => return Err(RuntimeError::internal_integrity_error(102).into()),
    }

    let (left_months, left_days) = left.signed_components();
    let (right_months, right_days) = right.signed_components();

    let (months_total, days_total) = if matches!(operator, MathOperatorEnum::Addition) {
        (left_months + right_months, left_days + right_days)
    } else {
        (left_months - right_months, left_days - right_days)
    };

    ErPeriodValue::from_signed_parts(months_total, days_total)
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

fn apply_duration_to_date(
    date: time::Date,
    duration: &ErDurationValue,
    operator: &MathOperatorEnum,
) -> Result<time::PrimitiveDateTime, RuntimeError> {
    let mut seconds_delta = duration.signed_seconds();
    if matches!(operator, MathOperatorEnum::Subtraction) {
        seconds_delta = -seconds_delta;
    }

    let seconds_i64 = i64::try_from(seconds_delta).map_err(|_| {
        RuntimeError::eval_error("Second offset is out of range for date adjustment".to_string())
    })?;
    let delta = TDuration::seconds(seconds_i64);
    datetime_at_midnight(date)
        .checked_add(delta)
        .ok_or_else(|| {
            RuntimeError::eval_error("Date adjustment with duration overflowed".to_string())
        })
}

fn apply_duration_to_datetime(
    datetime: time::PrimitiveDateTime,
    duration: &ErDurationValue,
    operator: &MathOperatorEnum,
) -> Result<time::PrimitiveDateTime, RuntimeError> {
    let mut seconds_delta = duration.signed_seconds();
    if matches!(operator, MathOperatorEnum::Subtraction) {
        seconds_delta = -seconds_delta;
    }

    let seconds_i64 = i64::try_from(seconds_delta).map_err(|_| {
        RuntimeError::eval_error(
            "Second offset is out of range for datetime adjustment".to_string(),
        )
    })?;
    let delta = TDuration::seconds(seconds_i64);
    datetime.checked_add(delta).ok_or_else(|| {
        RuntimeError::eval_error("Datetime adjustment with duration overflowed".to_string())
    })
}

fn apply_duration_to_time(
    time: time::Time,
    duration: &ErDurationValue,
    operator: &MathOperatorEnum,
) -> Result<time::Time, RuntimeError> {
    let mut seconds_delta = duration.signed_seconds();
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

fn apply_days_to_date(date: time::Date, days_delta: i128) -> Result<time::Date, RuntimeError> {
    if days_delta == 0 {
        return Ok(date);
    }

    let days_i64 = i64::try_from(days_delta).map_err(|_| {
        RuntimeError::eval_error("Day offset is out of range for date adjustment".to_string())
    })?;
    let delta = TDuration::days(days_i64);
    date.checked_add(delta).ok_or_else(|| {
        RuntimeError::eval_error("Date adjustment with period overflowed".to_string())
    })
}

fn apply_period_to_date(
    date: time::Date,
    period: &ErPeriodValue,
    operator: &MathOperatorEnum,
) -> Result<time::Date, RuntimeError> {
    let (mut months_delta, mut days_delta) = period.signed_components();
    if matches!(operator, MathOperatorEnum::Subtraction) {
        months_delta = -months_delta;
        days_delta = -days_delta;
    }

    let current = shift_date_by_months(date, months_delta)?;
    apply_days_to_date(current, days_delta)
}

fn apply_period_to_datetime(
    datetime: time::PrimitiveDateTime,
    period: &ErPeriodValue,
    operator: &MathOperatorEnum,
) -> Result<time::PrimitiveDateTime, RuntimeError> {
    let (mut months_delta, mut days_delta) = period.signed_components();
    if matches!(operator, MathOperatorEnum::Subtraction) {
        months_delta = -months_delta;
        days_delta = -days_delta;
    }

    let mut date = shift_date_by_months(datetime.date(), months_delta)?;
    date = apply_days_to_date(date, days_delta)?;
    Ok(time::PrimitiveDateTime::new(date, datetime.time()))
}

fn datetime_at_midnight(date: time::Date) -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT)
}

fn duration_from_time_diff(diff: time::Duration) -> Result<ErDurationValue, RuntimeError> {
    let total_secs = i128::from(diff.whole_seconds());
    ErDurationValue::from_signed_seconds(total_secs)
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
                DurationVariant(ValueOrSv::Value(left_dur)),
                DurationVariant(ValueOrSv::Value(right_dur)),
            ) => {
                let combined = operate_duration_values(&self.data.operator, left_dur, right_dur)?;
                Ok(DurationVariant(ValueOrSv::Value(combined)))
            }
            (
                PeriodVariant(ValueOrSv::Value(left_period)),
                PeriodVariant(ValueOrSv::Value(right_period)),
            ) => {
                let combined =
                    operate_period_values(&self.data.operator, left_period, right_period)?;
                Ok(PeriodVariant(ValueOrSv::Value(combined)))
            }
            (DateValue(ValueOrSv::Value(date)), DurationVariant(ValueOrSv::Value(duration))) => {
                if matches!(self.data.operator, Addition | Subtraction) {
                    let result = apply_duration_to_date(*date, duration, &self.data.operator)?;
                    Ok(DateTimeValue(ValueOrSv::Value(result)))
                } else {
                    RuntimeError::internal_integrity_error(103).into()
                }
            }
            (DurationVariant(ValueOrSv::Value(duration)), DateValue(ValueOrSv::Value(date)))
                if matches!(self.data.operator, Addition) =>
            {
                let result = apply_duration_to_date(*date, duration, &MathOperatorEnum::Addition)?;
                Ok(DateTimeValue(ValueOrSv::Value(result)))
            }
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                DurationVariant(ValueOrSv::Value(duration)),
            ) => {
                if matches!(self.data.operator, Addition | Subtraction) {
                    let result =
                        apply_duration_to_datetime(*datetime, duration, &self.data.operator)?;
                    Ok(DateTimeValue(ValueOrSv::Value(result)))
                } else {
                    RuntimeError::internal_integrity_error(104).into()
                }
            }
            (
                DurationVariant(ValueOrSv::Value(duration)),
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) if matches!(self.data.operator, Addition) => {
                let result =
                    apply_duration_to_datetime(*datetime, duration, &MathOperatorEnum::Addition)?;
                Ok(DateTimeValue(ValueOrSv::Value(result)))
            }
            (TimeValue(ValueOrSv::Value(time)), DurationVariant(ValueOrSv::Value(duration))) => {
                if matches!(self.data.operator, Addition | Subtraction) {
                    let result = apply_duration_to_time(*time, duration, &self.data.operator)?;
                    Ok(TimeValue(ValueOrSv::Value(result)))
                } else {
                    RuntimeError::internal_integrity_error(105).into()
                }
            }
            (DurationVariant(ValueOrSv::Value(duration)), TimeValue(ValueOrSv::Value(time)))
                if matches!(self.data.operator, Addition) =>
            {
                let result = apply_duration_to_time(*time, duration, &MathOperatorEnum::Addition)?;
                Ok(TimeValue(ValueOrSv::Value(result)))
            }
            (DateValue(ValueOrSv::Value(left)), DateValue(ValueOrSv::Value(right)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let diff_days = (*left - *right).whole_days();
                let seconds = i128::from(diff_days) * 86_400;
                let duration = ErDurationValue::from_signed_seconds(seconds)?;
                Ok(DurationVariant(ValueOrSv::Value(duration)))
            }
            (TimeValue(ValueOrSv::Value(left)), TimeValue(ValueOrSv::Value(right)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let left_secs = i128::from(left.hour()) * 3_600
                    + i128::from(left.minute()) * 60
                    + i128::from(left.second());
                let right_secs = i128::from(right.hour()) * 3_600
                    + i128::from(right.minute()) * 60
                    + i128::from(right.second());
                let diff = left_secs - right_secs;
                let duration = ErDurationValue::from_signed_seconds(diff)?;
                Ok(DurationVariant(ValueOrSv::Value(duration)))
            }
            (DateTimeValue(ValueOrSv::Value(left)), DateTimeValue(ValueOrSv::Value(right)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let diff = *left - *right;
                let duration = duration_from_time_diff(diff)?;
                Ok(DurationVariant(ValueOrSv::Value(duration)))
            }
            (DateValue(ValueOrSv::Value(date)), DateTimeValue(ValueOrSv::Value(datetime)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let left_dt = datetime_at_midnight(*date);
                let diff = left_dt - *datetime;
                let duration = duration_from_time_diff(diff)?;
                Ok(DurationVariant(ValueOrSv::Value(duration)))
            }
            (DateTimeValue(ValueOrSv::Value(datetime)), DateValue(ValueOrSv::Value(date)))
                if matches!(self.data.operator, Subtraction) =>
            {
                let right_dt = datetime_at_midnight(*date);
                let diff = *datetime - right_dt;
                let duration = duration_from_time_diff(diff)?;
                Ok(DurationVariant(ValueOrSv::Value(duration)))
            }
            (DateValue(ValueOrSv::Value(date)), PeriodVariant(ValueOrSv::Value(period))) => {
                if matches!(self.data.operator, Addition | Subtraction) {
                    let result = apply_period_to_date(*date, period, &self.data.operator)?;
                    Ok(DateValue(ValueOrSv::Value(result)))
                } else {
                    RuntimeError::internal_integrity_error(106).into()
                }
            }
            (PeriodVariant(ValueOrSv::Value(period)), DateValue(ValueOrSv::Value(date)))
                if matches!(self.data.operator, Addition) =>
            {
                let result = apply_period_to_date(*date, period, &MathOperatorEnum::Addition)?;
                Ok(DateValue(ValueOrSv::Value(result)))
            }
            (
                DateTimeValue(ValueOrSv::Value(datetime)),
                PeriodVariant(ValueOrSv::Value(period)),
            ) => {
                if matches!(self.data.operator, Addition | Subtraction) {
                    let result = apply_period_to_datetime(*datetime, period, &self.data.operator)?;
                    Ok(DateTimeValue(ValueOrSv::Value(result)))
                } else {
                    RuntimeError::internal_integrity_error(107).into()
                }
            }
            (
                PeriodVariant(ValueOrSv::Value(period)),
                DateTimeValue(ValueOrSv::Value(datetime)),
            ) if matches!(self.data.operator, Addition) => {
                let result =
                    apply_period_to_datetime(*datetime, period, &MathOperatorEnum::Addition)?;
                Ok(DateTimeValue(ValueOrSv::Value(result)))
            }
            (DurationVariant(ValueOrSv::Value(_)), PeriodVariant(ValueOrSv::Value(_)))
            | (PeriodVariant(ValueOrSv::Value(_)), DurationVariant(ValueOrSv::Value(_))) => {
                RuntimeError::internal_integrity_error(108).into()
            }
            _ => RuntimeError::internal_integrity_error(109).into(),
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

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq)]
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
            _ => RuntimeError::internal_integrity_error(110).into(),
        }
    }
}
