use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::number::NumberEnum::{Int, Real, SV};
use crate::typesystem::types::ValueType::{
    DateTimeType, DateType, DurationType, ListType, NumberType, RangeType, TimeType, UndefinedType,
};
use crate::typesystem::types::{Integer, SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{
    Array, DateTimeValue, DateValue, DurationValue as DurationVariant, NumberValue, RangeValue,
    TimeValue,
};
use crate::typesystem::values::{
    ArrayValue, DurationValue as DurationStruct, ValueEnum, ValueOrSv,
};
use std::cmp::Ordering;
use std::f64::consts::PI;

// Helper to extract NumberEnum
fn get_number(v: &ValueEnum) -> Option<NumberEnum> {
    match v {
        NumberValue(n) => Some(n.clone()),
        _ => None,
    }
}

pub fn eval_ln(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => {
            if n <= 0.0 {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("ln of non-positive number"))))
            } else {
                Ok(NumberValue(Real(n.ln())))
            }
        },
        NumberValue(Int(n)) => {
            if n <= 0 {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("ln of non-positive number"))))
            } else {
                Ok(NumberValue(Real((n as f64).ln())))
            }
        },
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_log10(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => {
            if n <= 0.0 {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("log10 of non-positive number"))))
            } else {
                Ok(NumberValue(Real(n.log10())))
            }
        },
        NumberValue(Int(n)) => {
            if n <= 0 {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("log10 of non-positive number"))))
            } else {
                Ok(NumberValue(Real((n as f64).log10())))
            }
        },
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_exp(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.exp()))),
        NumberValue(Int(n)) => Ok(NumberValue(Real((n as f64).exp()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_pi(_args: Vec<Result<ValueEnum, RuntimeError>>, _ret: ValueType) -> Result<ValueEnum, RuntimeError> {
    Ok(NumberValue(Real(PI)))
}

pub fn eval_degrees(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.to_degrees()))),
        NumberValue(Int(n)) => Ok(NumberValue(Real((n as f64).to_degrees()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_radians(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.to_radians()))),
        NumberValue(Int(n)) => Ok(NumberValue(Real((n as f64).to_radians()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_sin(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.sin()))),
        NumberValue(Int(n)) => Ok(NumberValue(Real((n as f64).sin()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_cos(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.cos()))),
        NumberValue(Int(n)) => Ok(NumberValue(Real((n as f64).cos()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_tan(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.tan()))),
        NumberValue(Int(n)) => Ok(NumberValue(Real((n as f64).tan()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_asin(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => {
            if !(-1.0..=1.0).contains(&n) {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("asin input out of range [-1, 1]"))))
            } else {
                Ok(NumberValue(Real(n.asin())))
            }
        },
        NumberValue(Int(n)) => {
            let val = n as f64;
             if !(-1.0..=1.0).contains(&val) {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("asin input out of range [-1, 1]"))))
            } else {
                Ok(NumberValue(Real(val.asin())))
            }
        },
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_acos(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => {
            if !(-1.0..=1.0).contains(&n) {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("acos input out of range [-1, 1]"))))
            } else {
                Ok(NumberValue(Real(n.acos())))
            }
        },
        NumberValue(Int(n)) => {
            let val = n as f64;
             if !(-1.0..=1.0).contains(&val) {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable("acos input out of range [-1, 1]"))))
            } else {
                Ok(NumberValue(Real(val.acos())))
            }
        },
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_atan(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.atan()))),
        NumberValue(Int(n)) => Ok(NumberValue(Real((n as f64).atan()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_atan2(y_val: ValueEnum, x_val: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(y), Some(x)) = (get_number(&y_val), get_number(&x_val)) {
        match (y, x) {
            (SV(sv), _) | (_, SV(sv)) => Ok(NumberValue(SV(sv))),
            (Real(y_f), Real(x_f)) => Ok(NumberValue(Real(y_f.atan2(x_f)))),
            (Int(y_i), Int(x_i)) => Ok(NumberValue(Real((y_i as f64).atan2(x_i as f64)))),
            (Real(y_f), Int(x_i)) => Ok(NumberValue(Real(y_f.atan2(x_i as f64)))),
            (Int(y_i), Real(x_f)) => Ok(NumberValue(Real((y_i as f64).atan2(x_f)))),
        }
    } else {
        RuntimeError::type_not_supported(y_val.get_type()).into()
    }
}

pub fn eval_abs(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.abs()))),
        NumberValue(Int(n)) => Ok(NumberValue(Int(n.abs()))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_round(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    // validation ensures 1 or 2 arguments
    let number = get_number(&vals[0])
        .ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let digits = if vals.len() == 2 {
        match get_number(&vals[1]) {
            Some(Int(d)) => d,
            Some(Real(d)) => d as i64,
            Some(SV(sv)) => return Ok(NumberValue(SV(sv))),
            _ => return RuntimeError::type_not_supported(vals[1].get_type()).into(),
        }
    } else {
        0
    };

    match number {
        Real(n) => {
            let multiplier = 10f64.powi(digits as i32);
            let val = n * multiplier;
            let rounded = val.round_ties_even();
            Ok(NumberValue(Real(rounded / multiplier)))
        }
        Int(n) => {
            if digits >= 0 {
                Ok(NumberValue(Int(n)))
            } else {
                let multiplier = 10f64.powi(digits as i32);
                let val = (n as f64) * multiplier;
                let rounded = val.round_ties_even();
                Ok(NumberValue(Real(rounded / multiplier)))
            }
        }
        SV(sv) => Ok(NumberValue(SV(sv))),
    }
}

pub fn eval_round_up(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    // validation ensures 1 or 2 arguments
    let number = get_number(&vals[0])
        .ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let digits = if vals.len() == 2 {
        match get_number(&vals[1]) {
            Some(Int(d)) => d,
            Some(Real(d)) => d as i64,
            Some(SV(sv)) => return Ok(NumberValue(SV(sv))),
            _ => return RuntimeError::type_not_supported(vals[1].get_type()).into(),
        }
    } else {
        0
    };

    match number {
        Real(n) => {
            let multiplier = 10f64.powi(digits as i32);
            let val = n * multiplier;
            // Round away from zero: sign * ceil(abs)
            let rounded = val.signum() * val.abs().ceil();
            Ok(NumberValue(Real(rounded / multiplier)))
        }
        Int(n) => {
            if digits >= 0 {
                Ok(NumberValue(Int(n)))
            } else {
                let multiplier = 10f64.powi(digits as i32);
                let val = (n as f64) * multiplier;
                let rounded = val.signum() * val.abs().ceil();
                Ok(NumberValue(Real(rounded / multiplier)))
            }
        }
        SV(sv) => Ok(NumberValue(SV(sv))),
    }
}

pub fn eval_round_down(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    // validation ensures 1 or 2 arguments
    let number = get_number(&vals[0])
        .ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let digits = if vals.len() == 2 {
        match get_number(&vals[1]) {
            Some(Int(d)) => d,
            Some(Real(d)) => d as i64,
            Some(SV(sv)) => return Ok(NumberValue(SV(sv))),
            _ => return RuntimeError::type_not_supported(vals[1].get_type()).into(),
        }
    } else {
        0
    };

    match number {
        Real(n) => {
            let multiplier = 10f64.powi(digits as i32);
            let val = n * multiplier;
            // Round toward zero: trunc
            let rounded = val.trunc();
            Ok(NumberValue(Real(rounded / multiplier)))
        }
        Int(n) => {
            if digits >= 0 {
                Ok(NumberValue(Int(n)))
            } else {
                let multiplier = 10f64.powi(digits as i32);
                let val = (n as f64) * multiplier;
                let rounded = val.trunc();
                Ok(NumberValue(Real(rounded / multiplier)))
            }
        }
        SV(sv) => Ok(NumberValue(SV(sv))),
    }
}

pub fn eval_floor(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.floor()))),
        NumberValue(Int(n)) => Ok(NumberValue(Int(n))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_ceiling(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.ceil()))),
        NumberValue(Int(n)) => Ok(NumberValue(Int(n))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_trunc(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => Ok(NumberValue(Real(n.trunc()))),
        NumberValue(Int(n)) => Ok(NumberValue(Int(n))),
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_modulo(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(a), Some(b)) = (get_number(&left), get_number(&right)) {
        match (a, b) {
            (SV(sv), _) | (_, SV(sv)) => Ok(NumberValue(SV(sv))),
            (Real(r1), Real(r2)) => {
                if r2 == 0.0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                let rem = r1 % r2;
                let res = if rem != 0.0 && rem.signum() != r2.signum() {
                    rem + r2
                } else {
                    rem
                };
                Ok(NumberValue(Real(res)))
            },
            (Int(i1), Int(i2)) => {
                if i2 == 0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                let rem = i1 % i2;
                let res = if rem != 0 && rem.signum() != i2.signum() {
                    rem + i2
                } else {
                    rem
                };
                Ok(NumberValue(Int(res)))
            },
            (Real(r1), Int(i2)) => {
                if i2 == 0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                let r2 = i2 as f64;
                let rem = r1 % r2;
                let res = if rem != 0.0 && rem.signum() != r2.signum() {
                    rem + r2
                } else {
                    rem
                };
                Ok(NumberValue(Real(res)))
            },
            (Int(i1), Real(r2)) => {
                if r2 == 0.0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                let r1 = i1 as f64;
                let rem = r1 % r2;
                let res = if rem != 0.0 && rem.signum() != r2.signum() {
                    rem + r2
                } else {
                    rem
                };
                Ok(NumberValue(Real(res)))
            },
        }
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}

pub fn eval_idiv(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(a), Some(b)) = (get_number(&left), get_number(&right)) {
        match (a, b) {
            (SV(sv), _) | (_, SV(sv)) => Ok(NumberValue(SV(sv))),
            (Real(r1), Real(r2)) => {
                if r2 == 0.0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                Ok(NumberValue(Real((r1 / r2).floor())))
            },
            (Int(i1), Int(i2)) => {
                if i2 == 0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                let d = i1 / i2;
                let r = i1 % i2;
                let res = if (r > 0 && i2 < 0) || (r < 0 && i2 > 0) {
                    d - 1
                } else {
                    d
                };
                Ok(NumberValue(Int(res)))
            },
            (Real(r1), Int(i2)) => {
                 if i2 == 0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                 Ok(NumberValue(Real((r1 / (i2 as f64)).floor())))
            },
            (Int(i1), Real(r2)) => {
                 if r2 == 0.0 { return RuntimeError::eval_error("Division by zero".to_string()).into(); }
                 Ok(NumberValue(Real(((i1 as f64) / r2).floor())))
            },
        }
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}

pub fn eval_sqrt(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(Real(n)) => {
            if n < 0.0 {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable(
                    "sqrt of negative number",
                ))))
            } else {
                Ok(NumberValue(Real(n.sqrt())))
            }
        }
        NumberValue(Int(n)) => {
            if n < 0 {
                Ok(NumberValue(SV(SpecialValueEnum::not_applicable(
                    "sqrt of negative number",
                ))))
            } else {
                Ok(NumberValue(Real((n as f64).sqrt())))
            }
        }
        NumberValue(SV(sv)) => Ok(NumberValue(SV(sv))),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_clamp(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    // validation ensures 3 arguments
    let n = get_number(&vals[0])
        .ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let min = get_number(&vals[1])
        .ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let max = get_number(&vals[2])
        .ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?;

    // If any is SV, return SV
    if let SV(sv) = n { return Ok(NumberValue(SV(sv))); }
    if let SV(sv) = min { return Ok(NumberValue(SV(sv))); }
    if let SV(sv) = max { return Ok(NumberValue(SV(sv))); }

    // min(max(n, min), max) logic
    let lower = if n < min { min } else { n };
    let result = if lower > max { max } else { lower };
    Ok(NumberValue(result))
}

// Validators

pub fn validate_unary_number(arg: ValueType) -> Link<()> {
    LinkingError::expect_type(None, arg, &[NumberType]).map(|_| ())
}

pub fn validate_binary_number_number(left: ValueType, right: ValueType) -> Link<()> {
    LinkingError::expect_type(None, left, &[NumberType])?;
    LinkingError::expect_type(None, right, &[NumberType])?;
    Ok(())
}

pub fn validate_round_args(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() || args.len() > 2 {
        return LinkingError::other_error("round functions expect 1 or 2 arguments".to_string()).into();
    }
    LinkingError::expect_type(None, args[0].clone(), &[NumberType])?;
    if args.len() == 2 {
        LinkingError::expect_type(None, args[1].clone(), &[NumberType])?;
    }
    Ok(())
}

pub fn validate_clamp_args(args: Vec<ValueType>) -> Link<()> {
    if args.len() != 3 {
        return LinkingError::other_error("clamp expects 3 arguments".to_string()).into();
    }
    for arg in args {
        LinkingError::expect_type(None, arg, &[NumberType])?;
    }
    Ok(())
}

pub fn validate_zero_args(args: Vec<ValueType>) -> Link<()> {
    if !args.is_empty() {
        return LinkingError::other_error("Expects 0 arguments".to_string()).into();
    }
    Ok(())
}


#[derive(Clone, Copy)]
enum ExtremaOrder {
    Min,
    Max,
}

impl ExtremaOrder {
    fn should_replace(self, ordering: Ordering) -> bool {
        match self {
            ExtremaOrder::Max => ordering == Ordering::Less,
            ExtremaOrder::Min => ordering == Ordering::Greater,
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExtremaKind {
    Number,
    Date,
    Time,
    DateTime,
    Duration,
}

fn kind_from_value_type(value_type: &ValueType) -> Option<ExtremaKind> {
    match value_type {
        NumberType | RangeType => Some(ExtremaKind::Number),
        DateType => Some(ExtremaKind::Date),
        TimeType => Some(ExtremaKind::Time),
        DateTimeType => Some(ExtremaKind::DateTime),
        DurationType => Some(ExtremaKind::Duration),
        ListType(Some(inner)) => kind_from_value_type(inner),
        _ => None,
    }
}

fn kind_from_value(value: &ValueEnum) -> Option<ExtremaKind> {
    match value {
        NumberValue(_) => Some(ExtremaKind::Number),
        DateValue(_) => Some(ExtremaKind::Date),
        TimeValue(_) => Some(ExtremaKind::Time),
        DateTimeValue(_) => Some(ExtremaKind::DateTime),
        DurationVariant(_) => Some(ExtremaKind::Duration),
        _ => None,
    }
}

fn detect_extrema_kind(
    list_type: &ValueType,
    values: &[ValueEnum],
) -> Result<ExtremaKind, RuntimeError> {
    if let Some(kind) = kind_from_value_type(list_type) {
        return Ok(kind);
    }

    if let Some(value) = values.first() {
        if let Some(kind) = kind_from_value(value) {
            return Ok(kind);
        } else {
            return RuntimeError::type_not_supported(value.get_type()).into();
        }
    }

    Ok(ExtremaKind::Number)
}

fn missing_extrema_value(kind: ExtremaKind) -> ValueEnum {
    match kind {
        ExtremaKind::Number => NumberValue(SV(SpecialValueEnum::missing_for(None))),
        ExtremaKind::Date => DateValue(ValueOrSv::Sv(SpecialValueEnum::missing_for(None))),
        ExtremaKind::Time => TimeValue(ValueOrSv::Sv(SpecialValueEnum::missing_for(None))),
        ExtremaKind::DateTime => DateTimeValue(ValueOrSv::Sv(SpecialValueEnum::missing_for(None))),
        ExtremaKind::Duration => {
            DurationVariant(ValueOrSv::Sv(SpecialValueEnum::missing_for(None)))
        }
    }
}

fn should_replace_value_or_sv_ord<T: Ord>(
    order: ExtremaOrder,
    current: &ValueOrSv<T, SpecialValueEnum>,
    candidate: &ValueOrSv<T, SpecialValueEnum>,
) -> bool {
    match (current, candidate) {
        (ValueOrSv::Value(a), ValueOrSv::Value(b)) => order.should_replace(a.cmp(b)),
        (ValueOrSv::Sv(_), ValueOrSv::Value(_)) => matches!(order, ExtremaOrder::Max),
        (ValueOrSv::Value(_), ValueOrSv::Sv(_)) => matches!(order, ExtremaOrder::Min),
        (ValueOrSv::Sv(_), ValueOrSv::Sv(_)) => false,
    }
}

fn should_replace_duration(
    order: ExtremaOrder,
    current: &ValueOrSv<DurationStruct, SpecialValueEnum>,
    candidate: &ValueOrSv<DurationStruct, SpecialValueEnum>,
) -> Result<bool, RuntimeError> {
    match (current, candidate) {
        (ValueOrSv::Value(a), ValueOrSv::Value(b)) => match a.partial_cmp(b) {
            Some(ordering) => Ok(order.should_replace(ordering)),
            None => {
                RuntimeError::eval_error("Cannot compare durations of different kinds".to_string())
                    .into()
            }
        },
        (ValueOrSv::Sv(_), ValueOrSv::Value(_)) => Ok(matches!(order, ExtremaOrder::Max)),
        (ValueOrSv::Value(_), ValueOrSv::Sv(_)) => Ok(matches!(order, ExtremaOrder::Min)),
        (ValueOrSv::Sv(_), ValueOrSv::Sv(_)) => Ok(false),
    }
}

fn eval_extrema_all(
    values: Vec<ValueEnum>,
    list_type: ValueType,
    order: ExtremaOrder,
) -> Result<ValueEnum, RuntimeError> {
    let kind = detect_extrema_kind(&list_type, &values)?;

    if values.is_empty() {
        return Ok(missing_extrema_value(kind));
    }

    match kind {
        ExtremaKind::Number => {
            let mut best: Option<NumberEnum> = None;
            for value in values {
                match value {
                    NumberValue(number) => match &mut best {
                        Some(current) => {
                            let should_replace = match order {
                                ExtremaOrder::Max => *current < number,
                                ExtremaOrder::Min => *current > number,
                            };
                            if should_replace {
                                *current = number;
                            }
                        }
                        None => best = Some(number),
                    },
                    other => return RuntimeError::type_not_supported(other.get_type()).into(),
                }
            }

            Ok(best
                .map(NumberValue)
                .unwrap_or_else(|| missing_extrema_value(ExtremaKind::Number)))
        }
        ExtremaKind::Date => {
            let mut best: Option<ValueOrSv<time::Date, SpecialValueEnum>> = None;
            for value in values {
                match value {
                    DateValue(candidate) => match &mut best {
                        Some(current) => {
                            if should_replace_value_or_sv_ord(order, &*current, &candidate) {
                                *current = candidate;
                            }
                        }
                        None => best = Some(candidate),
                    },
                    other => return RuntimeError::type_not_supported(other.get_type()).into(),
                }
            }

            Ok(best
                .map(DateValue)
                .unwrap_or_else(|| missing_extrema_value(ExtremaKind::Date)))
        }
        ExtremaKind::Time => {
            let mut best: Option<ValueOrSv<time::Time, SpecialValueEnum>> = None;
            for value in values {
                match value {
                    TimeValue(candidate) => match &mut best {
                        Some(current) => {
                            if should_replace_value_or_sv_ord(order, &*current, &candidate) {
                                *current = candidate;
                            }
                        }
                        None => best = Some(candidate),
                    },
                    other => return RuntimeError::type_not_supported(other.get_type()).into(),
                }
            }

            Ok(best
                .map(TimeValue)
                .unwrap_or_else(|| missing_extrema_value(ExtremaKind::Time)))
        }
        ExtremaKind::DateTime => {
            let mut best: Option<ValueOrSv<time::PrimitiveDateTime, SpecialValueEnum>> = None;
            for value in values {
                match value {
                    DateTimeValue(candidate) => match &mut best {
                        Some(current) => {
                            if should_replace_value_or_sv_ord(order, &*current, &candidate) {
                                *current = candidate;
                            }
                        }
                        None => best = Some(candidate),
                    },
                    other => return RuntimeError::type_not_supported(other.get_type()).into(),
                }
            }

            Ok(best
                .map(DateTimeValue)
                .unwrap_or_else(|| missing_extrema_value(ExtremaKind::DateTime)))
        }
        ExtremaKind::Duration => {
            let mut best: Option<ValueOrSv<DurationStruct, SpecialValueEnum>> = None;
            for value in values {
                match value {
                    DurationVariant(candidate) => match &mut best {
                        Some(current) => {
                            if should_replace_duration(order, &*current, &candidate)? {
                                *current = candidate;
                            }
                        }
                        None => best = Some(candidate),
                    },
                    other => return RuntimeError::type_not_supported(other.get_type()).into(),
                }
            }

            Ok(best
                .map(DurationVariant)
                .unwrap_or_else(|| missing_extrema_value(ExtremaKind::Duration)))
        }
    }
}

pub fn eval_max_all(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    eval_extrema_all(values, list_type, ExtremaOrder::Max)
}

pub fn eval_max_multi(
    values: Vec<Result<ValueEnum, RuntimeError>>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let resolved = into_valid(values)?;
    eval_max_all(resolved, list_type)
}

pub fn eval_max(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(_) | DateValue(_) | TimeValue(_) | DateTimeValue(_) | DurationVariant(_) => {
            Ok(value)
        }
        Array(ArrayValue::ObjectsArray {
            values: _,
            object_type,
        }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        Array(ArrayValue::EmptyUntyped) => Ok(NumberValue(SV(SpecialValueEnum::missing_for(None)))),
        Array(ArrayValue::PrimitivesArray { values, item_type }) => eval_max_all(values, item_type),
        RangeValue(range) => match range.max() {
            None => RuntimeError::eval_error(
                "Max is not implemented for this particular range".to_string(),
            )
            .into(),
            Some(max) => Ok(NumberValue(NumberEnum::from(max))),
        },
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_min_all(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    eval_extrema_all(values, list_type, ExtremaOrder::Min)
}

pub fn eval_min_multi(
    values: Vec<Result<ValueEnum, RuntimeError>>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let resolved = into_valid(values)?;
    eval_min_all(resolved, list_type)
}

pub fn eval_min(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(_) | DateValue(_) | TimeValue(_) | DateTimeValue(_) | DurationVariant(_) => {
            Ok(value)
        }
        Array(ArrayValue::ObjectsArray {
            values: _,
            object_type,
        }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        Array(ArrayValue::EmptyUntyped) => Ok(NumberValue(SV(SpecialValueEnum::missing_for(None)))),
        Array(ArrayValue::PrimitivesArray { values, item_type }) => eval_min_all(values, item_type),
        RangeValue(range) => match range.min() {
            None => RuntimeError::eval_error(
                "Min is not implemented for this particular range".to_string(),
            )
            .into(),
            Some(min) => Ok(NumberValue(NumberEnum::from(min))),
        },
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_sum_all(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    if matches!(list_type, ValueType::DurationType) {
        return sum_duration_values(values);
    }

    if values.is_empty() {
        return Ok(NumberValue(NumberEnum::from(0_i64)));
    }

    let mut acc: Option<NumberEnum> = None;

    for token in values {
        if let NumberValue(number) = token {
            acc = Some(match acc {
                Some(existing) => existing + number,
                None => number,
            });
        } else {
            return RuntimeError::type_not_supported(list_type.clone()).into();
        }
    }

    match acc {
        Some(total) => Ok(NumberValue(total)),
        None => Ok(NumberValue(SV(SpecialValueEnum::missing_for(None)))),
    }
}

fn sum_duration_values(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut seconds_total: i128 = 0;
    let mut has_value = false;
    let mut special: Option<SpecialValueEnum> = None;

    for value in values {
        match value {
            DurationVariant(ValueOrSv::Value(duration)) => {
                seconds_total += duration.signed_seconds();
                has_value = true;
            }
            DurationVariant(ValueOrSv::Sv(sv)) => {
                special = Some(sv);
                break;
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    if let Some(sv) = special {
        Ok(DurationVariant(ValueOrSv::Sv(sv)))
    } else if has_value {
        let result = DurationStruct::from_signed_seconds(seconds_total)?;
        Ok(DurationVariant(ValueOrSv::Value(result)))
    } else {
        Ok(DurationVariant(ValueOrSv::Sv(
            SpecialValueEnum::missing_for(None),
        )))
    }
}

pub fn eval_sum_multi(
    values: Vec<Result<ValueEnum, RuntimeError>>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let resolved = into_valid(values)?;
    eval_sum_all(resolved, list_type)
}

pub fn eval_sum(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(number) => Ok(NumberValue(number)),
        DurationVariant(_) => Ok(value),
        Array(array) => match array {
            ArrayValue::EmptyUntyped => Ok(NumberValue(0.into())),
            ArrayValue::ObjectsArray { object_type, .. } => RuntimeError::type_not_supported(
                ValueType::list_of(ValueType::ObjectType(object_type)),
            )
            .into(),
            ArrayValue::PrimitivesArray { values, item_type } => eval_sum_all(values, item_type),
        },
        RangeValue(range) => Ok(ValueEnum::from(range.sum::<Integer>())),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_count(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(_) => Ok(NumberValue(NumberEnum::Int(1))),
        ValueEnum::Array(array) => {
            let count = array.len();
            Ok(NumberValue(NumberEnum::Int(count as Integer)))
        }
        RangeValue(range) => Ok(ValueEnum::from(range.count() as Integer)),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_find(maybe_array: ValueEnum, search: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let ValueEnum::Array(array) = maybe_array {
        match array {
            ArrayValue::EmptyUntyped => Ok(NumberValue(SV(SpecialValueEnum::missing_for(None)))),
            ArrayValue::ObjectsArray { object_type, .. } => RuntimeError::type_not_supported(
                ValueType::list_of(ValueType::ObjectType(object_type)),
            )
            .into(),
            ArrayValue::PrimitivesArray { values, .. } => {
                let maybe_index = values.iter().position(|value| value.eq(&search));
                match maybe_index {
                    Some(index) => Ok(ValueEnum::from(index as Integer)),
                    None => Ok(NumberValue(SV(SpecialValueEnum::missing_for(None)))),
                }
            }
        }
    } else {
        RuntimeError::type_not_supported(maybe_array.get_type()).into()
    }
}

pub fn list_item_as_second_arg(left: ValueType, right: ValueType) -> Link<()> {
    let item_type = LinkingError::expect_array_type(Some("function arguments".to_string()), left)?;
    if !matches!(item_type, ValueType::UndefinedType) {
        LinkingError::expect_same_types("function arguments", item_type, right)?;
    }
    Ok(())
}

pub fn number_range_or_any_list(value_type: ValueType) -> Link<()> {
    match &value_type {
        NumberType | RangeType | ListType(_) => Ok(()),
        _ => LinkingError::types_not_compatible(
            None,
            value_type,
            Some(vec![
                NumberType,
                RangeType,
                ListType(Some(Box::new(NumberType))),
            ]),
        )
        .into(),
    }
}

fn is_extrema_scalar_type(value_type: &ValueType) -> bool {
    matches!(
        value_type,
        NumberType | DateType | TimeType | DateTimeType | DurationType
    )
}

pub fn validate_extrema_input(value_type: ValueType) -> Link<()> {
    if matches!(
        value_type,
        NumberType | RangeType | DateType | TimeType | DateTimeType | DurationType
    ) {
        return Ok(());
    }

    if let ListType(Some(inner)) = &value_type {
        if is_extrema_scalar_type(inner) {
            return Ok(());
        }
    } else if matches!(value_type, ListType(None)) {
        return Ok(());
    }

    LinkingError::types_not_compatible(
        None,
        value_type,
        Some(vec![
            NumberType,
            RangeType,
            DateType,
            TimeType,
            DateTimeType,
            DurationType,
            ListType(Some(Box::new(NumberType))),
            ListType(Some(Box::new(DateType))),
            ListType(Some(Box::new(TimeType))),
            ListType(Some(Box::new(DateTimeType))),
            ListType(Some(Box::new(DurationType))),
            ListType(None),
        ]),
    )
    .into()
}

pub fn validate_multi_extrema_args(args: Vec<ValueType>) -> Link<()> {
    let mut expected: Option<ValueType> = None;

    for arg in args {
        if matches!(arg, UndefinedType) {
            continue;
        }

        if is_extrema_scalar_type(&arg) {
            if let Some(existing) = expected.clone() {
                LinkingError::expect_same_types("function arguments", existing, arg.clone())?;
            } else {
                expected = Some(arg.clone());
            }
        } else {
            return LinkingError::types_not_compatible(
                None,
                arg,
                Some(vec![
                    NumberType,
                    DateType,
                    TimeType,
                    DateTimeType,
                    DurationType,
                ]),
            )
            .into();
        }
    }

    Ok(())
}

fn is_sum_scalar_type(value_type: &ValueType) -> bool {
    matches!(value_type, NumberType | DurationType)
}

pub fn validate_sum_input(value_type: ValueType) -> Link<()> {
    if matches!(value_type, NumberType | RangeType | DurationType) {
        return Ok(());
    }

    if let ListType(Some(inner)) = &value_type {
        if is_sum_scalar_type(inner) {
            return Ok(());
        }
    } else if matches!(value_type, ListType(None)) {
        return Ok(());
    }

    LinkingError::types_not_compatible(
        None,
        value_type,
        Some(vec![
            NumberType,
            RangeType,
            DurationType,
            ListType(Some(Box::new(NumberType))),
            ListType(Some(Box::new(DurationType))),
            ListType(None),
        ]),
    )
    .into()
}

pub fn validate_multi_sum_args(args: Vec<ValueType>) -> Link<()> {
    let mut expected: Option<ValueType> = None;

    for arg in args {
        if matches!(arg, UndefinedType) {
            continue;
        }

        if is_sum_scalar_type(&arg) {
            if let Some(existing) = expected.clone() {
                LinkingError::expect_same_types("function arguments", existing, arg.clone())?;
            } else {
                expected = Some(arg.clone());
            }
        } else {
            return LinkingError::types_not_compatible(
                None,
                arg,
                Some(vec![NumberType, DurationType]),
            )
            .into();
        }
    }

    Ok(())
}

pub fn return_binary_same_as_right_arg(_left: ValueType, right: ValueType) -> ValueType {
    right
}

pub fn return_binary_same_as_left_arg(left: ValueType, _right: ValueType) -> ValueType {
    left
}

pub fn return_uni_number(_arg: ValueType) -> ValueType {
    NumberType
}

pub fn return_uni_extrema(arg: ValueType) -> ValueType {
    match arg {
        RangeType => NumberType,
        ListType(Some(inner)) => return_uni_extrema(*inner),
        ListType(None) => UndefinedType,
        other => other,
    }
}

pub fn return_multi_extrema(args: &[ValueType]) -> ValueType {
    args.first()
        .cloned()
        .map(return_uni_extrema)
        .unwrap_or(UndefinedType)
}
