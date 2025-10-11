use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::number::NumberEnum::SV;
use crate::typesystem::types::ValueType::{
    DateTimeType, DateType, DurationType, ListType, NumberType, RangeType, TimeType,
};
use crate::typesystem::types::{Integer, SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{
    Array, DateTimeValue, DateValue, DurationValue, NumberValue, RangeValue, TimeValue,
};
use crate::typesystem::values::{
    ArrayValue, DurationKind, DurationValue as RuntimeDurationValue, ValueEnum, ValueOrSv,
};
use std::cmp::Ordering;
use time::{Date, PrimitiveDateTime, Time};

#[derive(Clone, Copy, Debug, PartialEq)]
enum DurationTotal {
    YearsMonths(i128),
    DaysTime(i128),
}

impl DurationTotal {
    fn from_value(value: &RuntimeDurationValue) -> Self {
        match value.kind {
            DurationKind::YearsMonths => {
                let total_months = i128::from(value.years) * 12 + i128::from(value.months);
                let total = if value.negative {
                    -total_months
                } else {
                    total_months
                };
                DurationTotal::YearsMonths(total)
            }
            DurationKind::DaysTime => {
                let total_seconds = i128::from(value.days) * 86_400
                    + i128::from(value.hours) * 3_600
                    + i128::from(value.minutes) * 60
                    + i128::from(value.seconds);
                let total = if value.negative {
                    -total_seconds
                } else {
                    total_seconds
                };
                DurationTotal::DaysTime(total)
            }
        }
    }

    fn compare(&self, other: &Self) -> Result<Ordering, RuntimeError> {
        match (self, other) {
            (DurationTotal::YearsMonths(left), DurationTotal::YearsMonths(right)) => {
                Ok(left.cmp(right))
            }
            (DurationTotal::DaysTime(left), DurationTotal::DaysTime(right)) => Ok(left.cmp(right)),
            _ => Err(RuntimeError::eval_error(
                "Duration comparison requires values of the same kind".to_string(),
            )),
        }
    }

    fn add_assign(&mut self, other: &Self) -> Result<(), RuntimeError> {
        match (self, other) {
            (DurationTotal::YearsMonths(total), DurationTotal::YearsMonths(extra)) => {
                *total = (*total).checked_add(*extra).ok_or_else(|| {
                    RuntimeError::eval_error(
                        "Duration addition overflowed total months".to_string(),
                    )
                })?;
                Ok(())
            }
            (DurationTotal::DaysTime(total), DurationTotal::DaysTime(extra)) => {
                *total = (*total).checked_add(*extra).ok_or_else(|| {
                    RuntimeError::eval_error(
                        "Duration addition overflowed total seconds".to_string(),
                    )
                })?;
                Ok(())
            }
            _ => Err(RuntimeError::eval_error(
                "Duration addition requires values of the same kind".to_string(),
            )),
        }
    }

    fn to_duration_value(self) -> Result<RuntimeDurationValue, RuntimeError> {
        match self {
            DurationTotal::YearsMonths(total_months) => {
                if total_months == 0 {
                    return Ok(RuntimeDurationValue::ym(0, 0, false));
                }
                let negative = total_months < 0;
                let absolute = if negative {
                    total_months.checked_neg().ok_or_else(|| {
                        RuntimeError::eval_error(
                            "Duration conversion overflowed total months".to_string(),
                        )
                    })?
                } else {
                    total_months
                };
                let years = i32::try_from(absolute / 12).map_err(|_| {
                    RuntimeError::eval_error("Duration years exceed supported range".to_string())
                })?;
                let months = i32::try_from(absolute % 12).map_err(|_| {
                    RuntimeError::eval_error("Duration months exceed supported range".to_string())
                })?;
                Ok(RuntimeDurationValue::ym(years, months, negative))
            }
            DurationTotal::DaysTime(total_seconds) => {
                if total_seconds == 0 {
                    return Ok(RuntimeDurationValue::dt(0, 0, 0, 0, false));
                }
                let negative = total_seconds < 0;
                let absolute = if negative {
                    total_seconds.checked_neg().ok_or_else(|| {
                        RuntimeError::eval_error(
                            "Duration conversion overflowed total seconds".to_string(),
                        )
                    })?
                } else {
                    total_seconds
                };
                let days = i64::try_from(absolute / 86_400).map_err(|_| {
                    RuntimeError::eval_error("Duration days exceed supported range".to_string())
                })?;
                let rem = absolute % 86_400;
                let hours = i64::try_from(rem / 3_600).map_err(|_| {
                    RuntimeError::eval_error("Duration hours exceed supported range".to_string())
                })?;
                let rem = rem % 3_600;
                let minutes = i64::try_from(rem / 60).map_err(|_| {
                    RuntimeError::eval_error("Duration minutes exceed supported range".to_string())
                })?;
                let seconds = i64::try_from(rem % 60).map_err(|_| {
                    RuntimeError::eval_error("Duration seconds exceed supported range".to_string())
                })?;
                Ok(RuntimeDurationValue::dt(
                    days, hours, minutes, seconds, negative,
                ))
            }
        }
    }
}

fn missing_value(value_type: ValueType) -> ValueEnum {
    match value_type {
        NumberType | RangeType => NumberValue(SV(SpecialValueEnum::missing_for(None))),
        DateType => DateValue(ValueOrSv::Sv(SpecialValueEnum::missing_for(None))),
        TimeType => TimeValue(ValueOrSv::Sv(SpecialValueEnum::missing_for(None))),
        DateTimeType => DateTimeValue(ValueOrSv::Sv(SpecialValueEnum::missing_for(None))),
        DurationType => DurationValue(ValueOrSv::Sv(SpecialValueEnum::missing_for(None))),
        _ => NumberValue(SV(SpecialValueEnum::missing_for(None))),
    }
}

fn eval_max_numbers(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let mut maximum: Option<NumberEnum> = None;

    for value in values {
        match value {
            NumberValue(number) => {
                if let Some(ref check) = maximum {
                    if check < &number {
                        maximum = Some(number);
                    }
                } else {
                    maximum = Some(number);
                }
            }
            _ => return RuntimeError::type_not_supported(list_type).into(),
        }
    }

    Ok(match maximum {
        Some(max) => NumberValue(max),
        None => missing_value(list_type),
    })
}

fn eval_min_numbers(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let mut minimum: Option<NumberEnum> = None;

    for value in values {
        match value {
            NumberValue(number) => {
                if let Some(ref current) = minimum {
                    if &number < current {
                        minimum = Some(number);
                    }
                } else {
                    minimum = Some(number);
                }
            }
            _ => return RuntimeError::type_not_supported(list_type).into(),
        }
    }

    Ok(match minimum {
        Some(min) => NumberValue(min),
        None => missing_value(list_type),
    })
}

fn eval_max_dates(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut maximum: Option<Date> = None;

    for value in values {
        match value {
            DateValue(ValueOrSv::Value(date)) => {
                if let Some(existing) = maximum {
                    if date > existing {
                        maximum = Some(date);
                    }
                } else {
                    maximum = Some(date);
                }
            }
            DateValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::DateType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match maximum {
        Some(date) => DateValue(ValueOrSv::Value(date)),
        None => missing_value(ValueType::DateType),
    })
}

fn eval_min_dates(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut minimum: Option<Date> = None;

    for value in values {
        match value {
            DateValue(ValueOrSv::Value(date)) => {
                if let Some(existing) = minimum {
                    if date < existing {
                        minimum = Some(date);
                    }
                } else {
                    minimum = Some(date);
                }
            }
            DateValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::DateType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match minimum {
        Some(date) => DateValue(ValueOrSv::Value(date)),
        None => missing_value(ValueType::DateType),
    })
}

fn eval_max_times(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut maximum: Option<Time> = None;

    for value in values {
        match value {
            TimeValue(ValueOrSv::Value(time)) => {
                if let Some(existing) = maximum {
                    if time > existing {
                        maximum = Some(time);
                    }
                } else {
                    maximum = Some(time);
                }
            }
            TimeValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::TimeType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match maximum {
        Some(time) => TimeValue(ValueOrSv::Value(time)),
        None => missing_value(ValueType::TimeType),
    })
}

fn eval_min_times(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut minimum: Option<Time> = None;

    for value in values {
        match value {
            TimeValue(ValueOrSv::Value(time)) => {
                if let Some(existing) = minimum {
                    if time < existing {
                        minimum = Some(time);
                    }
                } else {
                    minimum = Some(time);
                }
            }
            TimeValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::TimeType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match minimum {
        Some(time) => TimeValue(ValueOrSv::Value(time)),
        None => missing_value(ValueType::TimeType),
    })
}

fn eval_max_datetimes(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut maximum: Option<PrimitiveDateTime> = None;

    for value in values {
        match value {
            DateTimeValue(ValueOrSv::Value(dt)) => {
                if let Some(existing) = maximum {
                    if dt > existing {
                        maximum = Some(dt);
                    }
                } else {
                    maximum = Some(dt);
                }
            }
            DateTimeValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::DateTimeType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match maximum {
        Some(dt) => DateTimeValue(ValueOrSv::Value(dt)),
        None => missing_value(ValueType::DateTimeType),
    })
}

fn eval_min_datetimes(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut minimum: Option<PrimitiveDateTime> = None;

    for value in values {
        match value {
            DateTimeValue(ValueOrSv::Value(dt)) => {
                if let Some(existing) = minimum {
                    if dt < existing {
                        minimum = Some(dt);
                    }
                } else {
                    minimum = Some(dt);
                }
            }
            DateTimeValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::DateTimeType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match minimum {
        Some(dt) => DateTimeValue(ValueOrSv::Value(dt)),
        None => missing_value(ValueType::DateTimeType),
    })
}

fn eval_max_durations(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut maximum_total: Option<DurationTotal> = None;
    let mut maximum_value: Option<RuntimeDurationValue> = None;

    for value in values {
        match value {
            DurationValue(ValueOrSv::Value(duration)) => {
                let total = DurationTotal::from_value(&duration);
                if let Some(existing_total) = maximum_total.as_mut() {
                    let ordering = existing_total.compare(&total)?;
                    if ordering == Ordering::Less {
                        *existing_total = total;
                        maximum_value = Some(duration);
                    }
                } else {
                    maximum_total = Some(total);
                    maximum_value = Some(duration);
                }
            }
            DurationValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::DurationType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match maximum_value {
        Some(duration) => DurationValue(ValueOrSv::Value(duration)),
        None => missing_value(ValueType::DurationType),
    })
}

fn eval_min_durations(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut minimum_total: Option<DurationTotal> = None;
    let mut minimum_value: Option<RuntimeDurationValue> = None;

    for value in values {
        match value {
            DurationValue(ValueOrSv::Value(duration)) => {
                let total = DurationTotal::from_value(&duration);
                if let Some(existing_total) = minimum_total.as_mut() {
                    let ordering = existing_total.compare(&total)?;
                    if ordering == Ordering::Greater {
                        *existing_total = total;
                        minimum_value = Some(duration);
                    }
                } else {
                    minimum_total = Some(total);
                    minimum_value = Some(duration);
                }
            }
            DurationValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::DurationType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match minimum_value {
        Some(duration) => DurationValue(ValueOrSv::Value(duration)),
        None => missing_value(ValueType::DurationType),
    })
}

fn eval_sum_numbers(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let mut acc: Option<NumberEnum> = None;

    for value in values {
        match value {
            NumberValue(number) => {
                acc = Some(match acc {
                    Some(existing) => existing + number,
                    None => number,
                });
            }
            _ => return RuntimeError::type_not_supported(list_type.clone()).into(),
        }
    }

    Ok(match acc {
        Some(total) => NumberValue(total),
        None => missing_value(list_type),
    })
}

fn eval_sum_durations(values: Vec<ValueEnum>) -> Result<ValueEnum, RuntimeError> {
    let mut acc: Option<DurationTotal> = None;

    for value in values {
        match value {
            DurationValue(ValueOrSv::Value(duration)) => {
                let part = DurationTotal::from_value(&duration);
                if let Some(total) = acc.as_mut() {
                    total.add_assign(&part)?;
                } else {
                    acc = Some(part);
                }
            }
            DurationValue(ValueOrSv::Sv(_)) => {
                return RuntimeError::type_not_supported(ValueType::DurationType).into()
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    Ok(match acc {
        Some(total) => DurationValue(ValueOrSv::Value(total.to_duration_value()?)),
        None => missing_value(ValueType::DurationType),
    })
}

pub fn eval_max_all(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    match list_type {
        NumberType | RangeType => eval_max_numbers(values, list_type),
        DateType => eval_max_dates(values),
        TimeType => eval_max_times(values),
        DateTimeType => eval_max_datetimes(values),
        DurationType => eval_max_durations(values),
        ValueType::UndefinedType => eval_max_numbers(values, NumberType),
        other => RuntimeError::type_not_supported(other).into(),
    }
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
        NumberValue(_) => Ok(value),
        DateValue(_) => Ok(value),
        TimeValue(_) => Ok(value),
        DateTimeValue(_) => Ok(value),
        DurationValue(_) => Ok(value),
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
    match list_type {
        NumberType | RangeType => eval_min_numbers(values, list_type),
        DateType => eval_min_dates(values),
        TimeType => eval_min_times(values),
        DateTimeType => eval_min_datetimes(values),
        DurationType => eval_min_durations(values),
        ValueType::UndefinedType => eval_min_numbers(values, NumberType),
        other => RuntimeError::type_not_supported(other).into(),
    }
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
        NumberValue(_) => Ok(value),
        DateValue(_) => Ok(value),
        TimeValue(_) => Ok(value),
        DateTimeValue(_) => Ok(value),
        DurationValue(_) => Ok(value),
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
    match list_type {
        NumberType | RangeType => eval_sum_numbers(values, list_type),
        DurationType => eval_sum_durations(values),
        ValueType::UndefinedType => eval_sum_numbers(values, NumberType),
        other => RuntimeError::type_not_supported(other).into(),
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
        DurationValue(_) => Ok(value),
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

pub fn number_range_or_number_list(value_type: ValueType) -> Link<()> {
    if match &value_type {
        NumberType | RangeType => true,
        ListType(Some(list_type)) => matches!(
            **list_type,
            NumberType
                | ValueType::DateType
                | ValueType::TimeType
                | ValueType::DateTimeType
                | ValueType::DurationType
        ),
        ListType(None) => true,
        _ => false,
    } {
        Ok(())
    } else {
        //println!("Type not compatible: {:?}", value_type);
        LinkingError::types_not_compatible(
            None,
            value_type,
            Some(vec![
                NumberType,
                RangeType,
                ListType(Some(Box::new(NumberType))),
            ]),
        )
        .into()
    }
}

pub fn validate_multi_all_args_numbers(args: Vec<ValueType>) -> Link<()> {
    for arg in args {
        if !matches!(arg, NumberType) {
            return LinkingError::types_not_compatible(None, arg, Some(vec![NumberType])).into();
        }
    }

    Ok(())
}

pub fn return_binary_same_as_right_arg(_left: ValueType, right: ValueType) -> ValueType {
    right
}

pub fn return_uni_number(_arg: ValueType) -> ValueType {
    NumberType
}

pub fn return_multi_number() -> ValueType {
    NumberType
}
