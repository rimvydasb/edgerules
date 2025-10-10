use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::number::NumberEnum::SV;
use crate::typesystem::types::ValueType::{ListType, NumberType, RangeType};
use crate::typesystem::types::{Integer, SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{Array, NumberValue, RangeValue};
use crate::typesystem::values::{ArrayValue, ValueEnum};

pub fn eval_max_all(
    values: Vec<ValueEnum>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let mut maximum: Option<NumberEnum> = None;

    for value in values {
        match value {
            NumberValue(ref number) => {
                if let Some(ref check) = maximum {
                    if check < number {
                        maximum = Some(number.clone());
                    }
                } else {
                    maximum = Some(number.clone());
                }
            }
            _ => return RuntimeError::type_not_supported(list_type).into(),
        }
    }

    if let Some(max) = maximum {
        Ok(NumberValue(max))
    } else {
        Ok(NumberValue(SV(SpecialValueEnum::missing_for(None))))
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
    let mut minimum: Option<NumberEnum> = None;

    for value in values {
        match value {
            NumberValue(ref number) => {
                if let Some(ref current) = minimum {
                    if number < current {
                        minimum = Some(number.clone());
                    }
                } else {
                    minimum = Some(number.clone());
                }
            }
            _ => return RuntimeError::type_not_supported(list_type.clone()).into(),
        }
    }

    if let Some(min) = minimum {
        Ok(NumberValue(min))
    } else {
        Ok(NumberValue(SV(SpecialValueEnum::missing_for(None))))
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
        ListType(Some(list_type)) => matches!(**list_type, NumberType),
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
