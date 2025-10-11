use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::number::NumberEnum::SV;
use crate::typesystem::types::ValueType::{
    DateTimeType, DateType, DurationType, ListType, NumberType, RangeType, TimeType, UndefinedType,
};
use crate::typesystem::types::{Integer, SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{
    Array, DateTimeValue, DateValue, DurationValue as DurationVariant, NumberValue, RangeValue,
    TimeValue,
};
use crate::typesystem::values::{
    ArrayValue, DurationKind, DurationValue as DurationStruct, ValueEnum, ValueOrSv,
};
use std::cmp::Ordering;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    for value in values {
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
        (ValueOrSv::Value(a), ValueOrSv::Value(b)) => {
            if a.kind != b.kind {
                return RuntimeError::eval_error(
                    "Cannot compare durations of different kinds".to_string(),
                )
                .into();
            }
            match a.partial_cmp(b) {
                Some(ordering) => Ok(order.should_replace(ordering)),
                None => RuntimeError::eval_error(
                    "Cannot compare durations of different kinds".to_string(),
                )
                .into(),
            }
        }
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
    let mut accumulator: Option<(DurationKind, i128)> = None;
    let mut special: Option<SpecialValueEnum> = None;

    for value in values {
        match value {
            DurationVariant(ValueOrSv::Value(duration)) => {
                let addition = match duration.kind {
                    DurationKind::YearsMonths => duration.signed_months(),
                    DurationKind::DaysTime => duration.signed_seconds(),
                };
                match &mut accumulator {
                    Some((kind, total)) => {
                        if *kind != duration.kind {
                            return RuntimeError::eval_error(
                                "Cannot sum durations of different kinds".to_string(),
                            )
                            .into();
                        }
                        *total += addition;
                    }
                    None => accumulator = Some((duration.kind.clone(), addition)),
                }
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
    } else if let Some((kind, total)) = accumulator {
        let result = match kind {
            DurationKind::YearsMonths => DurationStruct::from_total_months(total),
            DurationKind::DaysTime => DurationStruct::from_total_seconds(total),
        };
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
