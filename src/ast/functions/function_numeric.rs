use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::number::NumberEnum::SV;
use crate::typesystem::types::ValueType::{ListType, NumberType, RangeType};
use crate::typesystem::types::{Integer, SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{Array, NumberValue, RangeValue};

pub fn eval_max_all(
    values: Vec<Result<ValueEnum, RuntimeError>>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let mut maximum: Option<NumberEnum> = None;

    for value in values {
        match value? {
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
        Ok(NumberValue(SV(SpecialValueEnum::Missing)))
    }
}

pub fn eval_max(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(_) => Ok(value),
        Array(values, list_type) => eval_max_all(values, list_type),
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

pub fn eval_sum_all(
    values: Vec<Result<ValueEnum, RuntimeError>>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    if values.is_empty() {
        return Ok(ValueEnum::from(0));
    }

    let mut acc: NumberEnum = match values.first().unwrap() {
        Ok(NumberValue(NumberEnum::Real(_))) => NumberEnum::Real(0.0),
        Ok(NumberValue(NumberEnum::Int(_))) => NumberEnum::Int(0),
        _ => return RuntimeError::type_not_supported(list_type).into(),
    };

    for token in values {
        if let NumberValue(number) = token? {
            acc = acc + number;
        } else {
            return RuntimeError::type_not_supported(list_type).into();
        }
    }

    Ok(NumberValue(acc))
}

pub fn eval_sum(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(number) => Ok(NumberValue(number)),
        Array(items, list_type) => eval_sum_all(items, list_type),
        RangeValue(range) => Ok(ValueEnum::from(range.sum::<Integer>())),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_count(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(_) => Ok(NumberValue(NumberEnum::Int(1))),
        Array(items, _) => {
            let count = items.len();
            Ok(NumberValue(NumberEnum::Int(count as Integer)))
        }
        RangeValue(range) => Ok(ValueEnum::from(range.count() as Integer)),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_find(maybe_array: ValueEnum, search: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _) = maybe_array {
        let valid = into_valid(values)?;

        let maybe_index = valid.iter().position(|value| value.eq(&search));

        match maybe_index {
            Some(index) => Ok(ValueEnum::from(index as Integer)),

            // todo: should determine the type
            None => Ok(NumberValue(SV(SpecialValueEnum::Missing))),
        }
    } else {
        RuntimeError::type_not_supported(maybe_array.get_type()).into()
    }
}

pub fn list_item_as_second_arg(left: ValueType, right: ValueType) -> Link<()> {
    let list_type = LinkingError::expect_array_type(Some("function arguments".to_string()), left)?;
    LinkingError::expect_same_types("function arguments", list_type, right)?;
    Ok(())
}

pub fn number_range_or_any_list(value_type: ValueType) -> Link<()> {
    match &value_type {
        NumberType | RangeType | ListType(_) => Ok(()),
        _ => LinkingError::types_not_compatible(
            None,
            value_type,
            Some(vec![NumberType, RangeType, ListType(Box::new(NumberType))]),
        )
        .into(),
    }
}

pub fn number_range_or_number_list(value_type: ValueType) -> Link<()> {
    if match &value_type {
        NumberType | RangeType => true,
        ListType(list_type) => matches!(*list_type.clone(), NumberType),
        _ => false,
    } {
        Ok(())
    } else {
        LinkingError::types_not_compatible(
            None,
            value_type,
            Some(vec![NumberType, RangeType, ListType(Box::new(NumberType))]),
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
