use crate::ast::functions::function_numeric::list_item_as_second_arg;
use crate::ast::functions::function_string as strf;
use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum::{Char as SChar, String as SString};
use crate::typesystem::types::ValueType::{BooleanType, ListType, NumberType, StringType};
use crate::typesystem::types::{Integer, SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{Array, BooleanValue, NumberValue, StringValue};
use crate::typesystem::values::{ArrayValue, ValueEnum};
use log::trace;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

fn as_int(v: &ValueEnum) -> Option<i64> {
    match v {
        NumberValue(NumberEnum::Int(i)) => Some(*i),
        NumberValue(NumberEnum::Real(r)) => Some(*r as i64),
        _ => None,
    }
}

fn clone_array_parts(value: &ValueEnum) -> Result<(Vec<ValueEnum>, ValueType), RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, item_type }) => {
            Ok((values.clone(), item_type.clone()))
        }
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok((Vec::new(), ValueType::UndefinedType)),
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(Rc::clone(
                object_type,
            ))))
            .into()
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

fn build_array_from_parts(
    values: Vec<ValueEnum>,
    mut item_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    if matches!(item_type, ValueType::UndefinedType) {
        if let Some(first) = values.first() {
            item_type = first.get_type();
        }
    }

    Ok(ValueEnum::Array(ArrayValue::PrimitivesArray {
        values,
        item_type,
    }))
}

fn merge_item_type(target: &mut ValueType, candidate: ValueType) -> Result<(), RuntimeError> {
    if matches!(candidate, ValueType::UndefinedType) {
        return Ok(());
    }

    if matches!(target, ValueType::UndefinedType) {
        *target = candidate;
        return Ok(());
    }

    if *target == candidate {
        Ok(())
    } else {
        Err(RuntimeError::type_not_supported(candidate))
    }
}

fn flatten_list_value_type_owned(value_type: ValueType) -> ValueType {
    match value_type {
        ValueType::ListType(Some(inner)) => flatten_list_value_type_owned(*inner),
        other => other,
    }
}

fn merge_item_type_from_value(
    target: &mut ValueType,
    value: &ValueEnum,
) -> Result<(), RuntimeError> {
    merge_item_type(target, value.get_type())
}

// ---------------- Validators and return type helpers ----------------

pub fn validate_unary_list(arg: ValueType) -> Link<()> {
    if let ListType(_) = arg {
        Ok(())
    } else {
        LinkingError::expect_array_type(None, arg).map(|_| ())
    }
}

pub fn validate_unary_list_numbers(arg: ValueType) -> Link<()> {
    match arg {
        ListType(Some(inner)) => {
            let flattened = flatten_list_value_type_owned(*inner);
            LinkingError::expect_type(None, flattened, &[NumberType]).map(|_| ())
        }
        ListType(None) => LinkingError::types_not_compatible(
            None,
            ValueType::ListType(None),
            Some(vec![ListType(Some(Box::new(NumberType)))]),
        )
        .into(),
        other => LinkingError::expect_type(None, other, &[ListType(Some(Box::new(NumberType)))])
            .map(|_| ()),
    }
}

pub fn validate_unary_boolean_list(arg: ValueType) -> Link<()> {
    match arg {
        ListType(Some(inner)) => {
            LinkingError::expect_type(None, *inner, &[BooleanType]).map(|_| ())
        }
        ListType(None) => LinkingError::types_not_compatible(
            None,
            ValueType::ListType(None),
            Some(vec![ListType(Some(Box::new(BooleanType)))]),
        )
        .into(),
        other => LinkingError::expect_type(None, other, &[ListType(Some(Box::new(BooleanType)))])
            .map(|_| ()),
    }
}

pub fn return_same_list_type(arg: ValueType) -> ValueType {
    arg
}
pub fn return_list_undefined(_args: &[ValueType]) -> ValueType {
    ListType(None)
}

pub fn return_flatten_type(arg: ValueType) -> ValueType {
    match arg {
        ListType(Some(inner)) => {
            let mut t = *inner;
            while let ListType(Some(next)) = t.clone() {
                t = *next;
            }
            ListType(Some(Box::new(t)))
        }
        other => other,
    }
}

pub fn validate_binary_list_number(left: ValueType, right: ValueType) -> Link<()> {
    if let ListType(_) = left {
        LinkingError::expect_type(None, right, &[NumberType]).map(|_| ())
    } else {
        LinkingError::expect_array_type(None, left).map(|_| ())
    }
}

pub fn validate_binary_contains_mixed(left: ValueType, right: ValueType) -> Link<()> {
    // Allow string,string OR list+item
    if matches!(left, StringType) {
        strf::validate_binary_string_string(left, right)
    } else {
        list_item_as_second_arg(left, right)
    }
}

pub fn validate_binary_index_of_mixed(left: ValueType, right: ValueType) -> Link<()> {
    // Allow string,string OR list+item
    if matches!(left, StringType) {
        strf::validate_binary_string_string(left, right)
    } else {
        list_item_as_second_arg(left, right)
    }
}

pub fn return_index_of_type(left: ValueType, _right: ValueType) -> ValueType {
    if matches!(left, StringType) {
        NumberType
    } else {
        ListType(Some(Box::new(NumberType)))
    }
}

pub fn validate_unary_reverse_mixed(arg: ValueType) -> Link<()> {
    if matches!(arg, StringType) {
        strf::validate_unary_string(arg)
    } else {
        validate_unary_list(arg)
    }
}

//

// ---------------- Implementations ----------------

pub fn eval_contains_mixed(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match left {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok(BooleanValue(false)),
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, .. }) => {
            Ok(BooleanValue(values.iter().any(|v| v == &right)))
        }
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        other => strf::eval_contains(other, right),
    }
}

pub fn eval_product(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok(NumberValue(NumberEnum::SV(
            SpecialValueEnum::missing_for(None),
        ))),
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, .. }) => {
            let mut acc: Option<NumberEnum> = None;
            for v in values {
                if let NumberValue(n) = v {
                    acc = Some(match acc {
                        Some(a) => a * n,
                        None => n,
                    });
                } else {
                    return RuntimeError::type_not_supported(v.get_type()).into();
                }
            }
            match acc {
                Some(total) => Ok(NumberValue(total)),
                None => Ok(NumberValue(NumberEnum::SV(SpecialValueEnum::missing_for(
                    None,
                )))),
            }
        }
        NumberValue(n) => Ok(NumberValue(n)),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_mean(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok(NumberValue(NumberEnum::SV(
            SpecialValueEnum::missing_for(None),
        ))),
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, .. }) => {
            let mut sum = 0.0f64;
            let mut count = 0.0f64;
            for v in values {
                match v {
                    NumberValue(NumberEnum::Int(i)) => {
                        sum += i as f64;
                        count += 1.0;
                    }
                    NumberValue(NumberEnum::Real(r)) => {
                        sum += r;
                        count += 1.0;
                    }
                    _ => return RuntimeError::type_not_supported(v.get_type()).into(),
                }
            }
            if count == 0.0 {
                Ok(NumberValue(NumberEnum::SV(SpecialValueEnum::missing_for(
                    None,
                ))))
            } else {
                Ok(NumberValue(NumberEnum::from(sum / count)))
            }
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_median(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok(NumberValue(NumberEnum::SV(
            SpecialValueEnum::missing_for(None),
        ))),
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        ValueEnum::Array(ArrayValue::PrimitivesArray { mut values, .. }) => {
            if values.is_empty() {
                return Ok(NumberValue(NumberEnum::SV(SpecialValueEnum::missing_for(
                    None,
                ))));
            }
            let mut nums: Vec<f64> = Vec::with_capacity(values.len());
            for v in values.drain(..) {
                match v {
                    NumberValue(NumberEnum::Int(i)) => nums.push(i as f64),
                    NumberValue(NumberEnum::Real(r)) => nums.push(r),
                    _ => return RuntimeError::type_not_supported(v.get_type()).into(),
                }
            }
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            let n = nums.len();
            let med = if n % 2 == 1 {
                nums[n / 2]
            } else {
                (nums[n / 2 - 1] + nums[n / 2]) / 2.0
            };
            Ok(NumberValue(NumberEnum::from(med)))
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_stddev(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok(NumberValue(NumberEnum::SV(
            SpecialValueEnum::missing_for(None),
        ))),
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, .. }) => {
            if values.is_empty() {
                return Ok(NumberValue(NumberEnum::SV(SpecialValueEnum::missing_for(
                    None,
                ))));
            }
            let mut nums: Vec<f64> = Vec::with_capacity(values.len());
            for v in values {
                match v {
                    NumberValue(NumberEnum::Int(i)) => nums.push(i as f64),
                    NumberValue(NumberEnum::Real(r)) => nums.push(r),
                    _ => return RuntimeError::type_not_supported(v.get_type()).into(),
                }
            }
            let mean = nums.iter().copied().sum::<f64>() / (nums.len() as f64);
            let var =
                nums.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (nums.len() as f64);
            Ok(NumberValue(NumberEnum::from(var.sqrt())))
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_mode(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => {
            Ok(ValueEnum::Array(ArrayValue::EmptyUntyped))
        }
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, item_type }) => {
            let mut uniques: Vec<ValueEnum> = Vec::new();
            let mut counts: Vec<i64> = Vec::new();
            for v in values {
                if let Some(pos) = uniques.iter().position(|u| u == &v) {
                    counts[pos] += 1;
                } else {
                    uniques.push(v);
                    counts.push(1);
                }
            }
            let maxc = counts.iter().copied().max().unwrap_or(0);
            let out: Vec<ValueEnum> = uniques
                .into_iter()
                .zip(counts)
                .filter_map(|(v, c)| if c == maxc && maxc > 0 { Some(v) } else { None })
                .collect();
            build_array_from_parts(out, item_type)
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_all(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok(BooleanValue(true)),
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, .. }) => {
            for v in values {
                match v {
                    BooleanValue(true) => {}
                    BooleanValue(false) => return Ok(BooleanValue(false)),
                    _ => return RuntimeError::type_not_supported(v.get_type()).into(),
                }
            }
            Ok(BooleanValue(true))
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_any(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        ValueEnum::Array(ArrayValue::EmptyUntyped) => Ok(BooleanValue(false)),
        ValueEnum::Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        ValueEnum::Array(ArrayValue::PrimitivesArray { values, .. }) => {
            for v in values {
                match v {
                    BooleanValue(true) => return Ok(BooleanValue(true)),
                    BooleanValue(false) => {}
                    _ => return RuntimeError::type_not_supported(v.get_type()).into(),
                }
            }
            Ok(BooleanValue(false))
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_sublist(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if !(vals.len() == 2 || vals.len() == 3) {
        return RuntimeError::eval_error("sublist expects 2 or 3 args".to_string()).into();
    }
    let (items, source_item_type) = match &vals[0] {
        ValueEnum::Array(array) => match array {
            ArrayValue::ObjectsArray { object_type, .. } => {
                return RuntimeError::type_not_supported(ValueType::list_of(
                    ValueType::ObjectType(Rc::clone(object_type)),
                ))
                .into();
            }
            ArrayValue::PrimitivesArray { .. } | ArrayValue::EmptyUntyped => {
                let item_type = array.item_type().unwrap_or(ValueType::UndefinedType);
                (
                    array.clone_primitive_values().unwrap_or_default(),
                    item_type,
                )
            }
        },
        other => return RuntimeError::type_not_supported(other.get_type()).into(),
    };
    let start =
        as_int(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?; // 1-based
    let len_opt = if vals.len() == 3 {
        Some(as_int(&vals[2]).ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?)
    } else {
        None
    };
    let n = items.len() as i64;
    let i = (start - 1).max(0).min(n);
    let j = match len_opt {
        Some(l) => (i + l).min(n),
        None => n,
    };
    let (ii, jj) = (i as usize, j as usize);
    let out: Vec<ValueEnum> = items.iter().take(jj).skip(ii).cloned().collect();
    let result_item_type = match ret {
        ValueType::ListType(Some(inner)) => *inner,
        ValueType::ListType(None) => source_item_type,
        other => other,
    };
    build_array_from_parts(out, result_item_type)
}

pub fn validate_multi_sublist(args: Vec<ValueType>) -> Link<()> {
    if !(args.len() == 2 || args.len() == 3) {
        return LinkingError::other_error("sublist expects 2 or 3 arguments".to_string()).into();
    }
    LinkingError::expect_array_type(None, args[0].clone())?;
    LinkingError::expect_type(None, args[1].clone(), &[NumberType])?;
    if args.len() == 3 {
        LinkingError::expect_type(None, args[2].clone(), &[NumberType])?;
    }
    Ok(())
}

pub fn eval_append(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    if args.is_empty() {
        return RuntimeError::eval_error("append expects at least 1 argument".to_string()).into();
    }
    let vals = into_valid(args)?;
    let (mut items, mut item_type) = match &vals[0] {
        ValueEnum::Array(array) => match array {
            ArrayValue::ObjectsArray { object_type, .. } => {
                return RuntimeError::type_not_supported(ValueType::list_of(
                    ValueType::ObjectType(Rc::clone(object_type)),
                ))
                .into();
            }
            ArrayValue::PrimitivesArray { .. } | ArrayValue::EmptyUntyped => {
                let item_type = array.item_type().unwrap_or(ValueType::UndefinedType);
                (
                    array.clone_primitive_values().unwrap_or_default(),
                    item_type,
                )
            }
        },
        _ => return RuntimeError::type_not_supported(vals[0].get_type()).into(),
    };
    for v in vals.into_iter().skip(1) {
        if matches!(item_type, ValueType::UndefinedType) {
            item_type = v.get_type();
        }
        items.push(v);
    }
    let out: Vec<ValueEnum> = items;
    build_array_from_parts(out, item_type)
}

pub fn validate_multi_append(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() {
        return LinkingError::other_error("append expects at least 1 argument".to_string()).into();
    }
    let list_t = LinkingError::expect_array_type(None, args[0].clone())?;
    for t in args.into_iter().skip(1) {
        LinkingError::expect_same_types("append", list_t.clone(), t)?;
    }
    Ok(())
}

pub fn eval_concatenate(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    let mut out_items: Vec<ValueEnum> = Vec::new();
    let mut item_type = ValueType::UndefinedType;

    for value in vals {
        match value {
            Array(ArrayValue::EmptyUntyped) => {}
            Array(ArrayValue::PrimitivesArray {
                values,
                item_type: array_item_type,
            }) => {
                merge_item_type(&mut item_type, array_item_type.clone())?;
                for v in values {
                    merge_item_type_from_value(&mut item_type, &v)?;
                    out_items.push(v);
                }
            }
            Array(ArrayValue::ObjectsArray { object_type, .. }) => {
                return RuntimeError::type_not_supported(ValueType::list_of(
                    ValueType::ObjectType(object_type),
                ))
                .into();
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    build_array_from_parts(out_items, item_type)
}

pub fn validate_multi_concatenate(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() {
        return LinkingError::other_error("concatenate expects at least 1 argument".to_string())
            .into();
    }
    let mut base_item: Option<ValueType> = None;
    for t in args {
        let inner = LinkingError::expect_array_type(None, t)?;
        if let Some(b) = &base_item {
            LinkingError::expect_same_types("concatenate", b.clone(), inner.clone())?;
        } else {
            base_item = Some(inner);
        }
    }
    Ok(())
}

pub fn eval_insert_before(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if vals.len() != 3 {
        return RuntimeError::eval_error("insertBefore expects 3 arguments".to_string()).into();
    }

    let (mut items, mut item_type) = clone_array_parts(&vals[0])?;
    let pos =
        as_int(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let idx = ((pos - 1).max(0) as usize).min(items.len());

    let value = vals[2].clone();
    merge_item_type_from_value(&mut item_type, &value)?;
    items.insert(idx, value);

    build_array_from_parts(items, item_type)
}

pub fn validate_multi_insert_before(args: Vec<ValueType>) -> Link<()> {
    if args.len() != 3 {
        return LinkingError::other_error("insertBefore expects 3 arguments".to_string()).into();
    }
    let inner = LinkingError::expect_array_type(None, args[0].clone())?;
    LinkingError::expect_type(None, args[1].clone(), &[NumberType])?;
    LinkingError::expect_same_types("insertBefore", inner, args[2].clone()).map(|_| ())
}

pub fn eval_remove(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let pos = as_int(&right).ok_or_else(|| RuntimeError::type_not_supported(right.get_type()))?;

    match left {
        Array(ArrayValue::EmptyUntyped) => Ok(Array(ArrayValue::EmptyUntyped)),
        Array(ArrayValue::PrimitivesArray { values, item_type }) => {
            let mut res: Vec<ValueEnum> = Vec::with_capacity(values.len());
            for (i, v) in values.into_iter().enumerate() {
                if (i as i64) != (pos - 1) {
                    res.push(v);
                }
            }
            build_array_from_parts(res, item_type)
        }
        Array(ArrayValue::ObjectsArray {
            values,
            object_type,
        }) => {
            let mut res: Vec<Rc<RefCell<ExecutionContext>>> = Vec::with_capacity(values.len());
            for (i, v) in values.into_iter().enumerate() {
                if (i as i64) != (pos - 1) {
                    res.push(v);
                }
            }
            Ok(Array(ArrayValue::ObjectsArray {
                values: res,
                object_type,
            }))
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_reverse_mixed(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        Array(ArrayValue::EmptyUntyped) => Ok(Array(ArrayValue::EmptyUntyped)),
        Array(ArrayValue::PrimitivesArray {
            mut values,
            item_type,
        }) => {
            values.reverse();
            Ok(Array(ArrayValue::PrimitivesArray { values, item_type }))
        }
        Array(ArrayValue::ObjectsArray {
            mut values,
            object_type,
        }) => {
            values.reverse();
            Ok(Array(ArrayValue::ObjectsArray {
                values,
                object_type,
            }))
        }
        other => strf::eval_reverse(other),
    }
}

pub fn eval_index_of_mixed(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match left {
        Array(ArrayValue::EmptyUntyped) => {
            build_array_from_parts(Vec::new(), ValueType::NumberType)
        }
        Array(ArrayValue::PrimitivesArray { values, .. }) => {
            let mut pos: Vec<ValueEnum> = Vec::new();
            for (i, v) in values.into_iter().enumerate() {
                if v == right {
                    pos.push(ValueEnum::from((i as Integer) + 1));
                }
            }
            build_array_from_parts(pos, ValueType::NumberType)
        }
        Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        other => strf::eval_index_of(other, right),
    }
}

pub fn eval_union(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    let mut out: Vec<ValueEnum> = Vec::new();
    let mut item_type = ValueType::UndefinedType;

    for value in vals {
        match value {
            Array(ArrayValue::EmptyUntyped) => {}
            Array(ArrayValue::PrimitivesArray {
                values,
                item_type: array_item_type,
            }) => {
                merge_item_type(&mut item_type, array_item_type.clone())?;
                for v in values {
                    merge_item_type_from_value(&mut item_type, &v)?;
                    if !out.iter().any(|existing| existing == &v) {
                        out.push(v);
                    }
                }
            }
            Array(ArrayValue::ObjectsArray { object_type, .. }) => {
                return RuntimeError::type_not_supported(ValueType::list_of(
                    ValueType::ObjectType(object_type),
                ))
                .into();
            }
            other => return RuntimeError::type_not_supported(other.get_type()).into(),
        }
    }

    build_array_from_parts(out, item_type)
}

pub fn validate_multi_union(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() {
        return LinkingError::other_error("union expects at least 1 argument".to_string()).into();
    }
    let mut base: Option<ValueType> = None;
    for t in args {
        let inner = LinkingError::expect_array_type(None, t)?;
        if let Some(b) = &base {
            LinkingError::expect_same_types("union", b.clone(), inner.clone())?;
        } else {
            base = Some(inner);
        }
    }
    Ok(())
}

pub fn eval_distinct(values: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match values {
        Array(ArrayValue::EmptyUntyped) => Ok(Array(ArrayValue::EmptyUntyped)),
        Array(ArrayValue::PrimitivesArray { values, item_type }) => {
            let mut out: Vec<ValueEnum> = Vec::new();
            for v in values {
                if !out.iter().any(|x| x == &v) {
                    out.push(v);
                }
            }
            build_array_from_parts(out, item_type)
        }
        Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_duplicates(values: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match values {
        Array(ArrayValue::EmptyUntyped) => Ok(Array(ArrayValue::EmptyUntyped)),
        Array(ArrayValue::PrimitivesArray { values, item_type }) => {
            let mut uniq: Vec<ValueEnum> = Vec::new();
            let mut dups: Vec<ValueEnum> = Vec::new();
            for v in values {
                if uniq.iter().any(|x| x == &v) {
                    if !dups.iter().any(|x| x == &v) {
                        dups.push(v);
                    }
                } else {
                    uniq.push(v);
                }
            }
            build_array_from_parts(dups, item_type)
        }
        Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_flatten(values: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    fn collect(value: ValueEnum, acc: &mut Vec<ValueEnum>) -> Result<ValueType, RuntimeError> {
        match value {
            Array(ArrayValue::PrimitivesArray { values, item_type }) => {
                let mut last_type = item_type;
                for v in values {
                    last_type = collect(v, acc)?;
                }
                Ok(last_type)
            }
            Array(ArrayValue::EmptyUntyped) => Ok(ValueType::UndefinedType),
            Array(ArrayValue::ObjectsArray { object_type, .. }) => {
                RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(
                    object_type,
                )))
                .into()
            }
            other => {
                let value_type = other.get_type();
                acc.push(other);
                Ok(value_type)
            }
        }
    }

    match values {
        Array(ArrayValue::EmptyUntyped) => Ok(Array(ArrayValue::EmptyUntyped)),
        Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        Array(ArrayValue::PrimitivesArray { values, .. }) => {
            let mut acc: Vec<ValueEnum> = Vec::new();
            let mut flattened_type = ValueType::UndefinedType;

            for v in values {
                let t = collect(v, &mut acc)?;
                merge_item_type(&mut flattened_type, t)?;
            }

            if matches!(flattened_type, ValueType::UndefinedType) && !acc.is_empty() {
                flattened_type = acc[0].get_type();
            }

            build_array_from_parts(acc, flattened_type)
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

fn value_ordering(left: &ValueEnum, right: &ValueEnum) -> Ordering {
    match (left, right) {
        (NumberValue(NumberEnum::Int(x)), NumberValue(NumberEnum::Int(y))) => x.cmp(y),
        (NumberValue(NumberEnum::Real(x)), NumberValue(NumberEnum::Real(y))) => {
            x.partial_cmp(y).unwrap_or(Ordering::Equal)
        }
        (StringValue(SString(a)), StringValue(SString(b))) => a.cmp(b),
        (StringValue(SString(a)), StringValue(SChar(b))) => a.cmp(&b.to_string()),
        (StringValue(SChar(a)), StringValue(SString(b))) => a.to_string().cmp(b),
        (StringValue(SChar(a)), StringValue(SChar(b))) => a.cmp(b),
        _ => left.to_string().cmp(&right.to_string()),
    }
}

pub fn eval_sort(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        Array(ArrayValue::EmptyUntyped) => Ok(Array(ArrayValue::EmptyUntyped)),
        Array(ArrayValue::PrimitivesArray {
            mut values,
            item_type,
        }) => {
            trace!(
                "eval_sort: sorting {} elements ascending (element type: {:?})",
                values.len(),
                item_type
            );
            values.sort_by(value_ordering);
            Ok(Array(ArrayValue::PrimitivesArray { values, item_type }))
        }
        Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_sort_desc(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        Array(ArrayValue::EmptyUntyped) => Ok(Array(ArrayValue::EmptyUntyped)),
        Array(ArrayValue::PrimitivesArray {
            mut values,
            item_type,
        }) => {
            trace!(
                "eval_sort_desc: sorting {} elements descending (element type: {:?})",
                values.len(),
                item_type
            );
            values.sort_by(|a, b| value_ordering(b, a));
            Ok(Array(ArrayValue::PrimitivesArray { values, item_type }))
        }
        Array(ArrayValue::ObjectsArray { object_type, .. }) => {
            RuntimeError::type_not_supported(ValueType::list_of(ValueType::ObjectType(object_type)))
                .into()
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn validate_binary_partition(left: ValueType, right: ValueType) -> Link<()> {
    LinkingError::expect_array_type(None, left)?;
    LinkingError::expect_type(None, right, &[NumberType]).map(|_| ())
}

pub fn return_partition_type(left: ValueType, _right: ValueType) -> ValueType {
    match left {
        ListType(inner) => ValueType::list_of(ValueType::ListType(inner)),
        other => other,
    }
}

pub fn eval_join(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if vals.is_empty() {
        return RuntimeError::eval_error("join expects at least 1 argument".to_string()).into();
    }

    let (items, _) = clone_array_parts(&vals[0])?;

    let delim = if vals.len() >= 2 {
        match &vals[1] {
            StringValue(SString(s)) => s.clone(),
            StringValue(SChar(c)) => c.to_string(),
            _ => return RuntimeError::type_not_supported(vals[1].get_type()).into(),
        }
    } else {
        String::new()
    };

    let (prefix, suffix) = if vals.len() >= 4 {
        let p = match &vals[2] {
            StringValue(SString(s)) => s.clone(),
            StringValue(SChar(c)) => c.to_string(),
            _ => return RuntimeError::type_not_supported(vals[2].get_type()).into(),
        };
        let s = match &vals[3] {
            StringValue(SString(s)) => s.clone(),
            StringValue(SChar(c)) => c.to_string(),
            _ => return RuntimeError::type_not_supported(vals[3].get_type()).into(),
        };
        (p, s)
    } else {
        (String::new(), String::new())
    };

    let mut parts: Vec<String> = Vec::new();
    for v in items {
        if let StringValue(SString(s)) = v {
            parts.push(s);
        } else if let StringValue(SChar(c)) = v {
            parts.push(c.to_string());
        }
    }
    let joined = format!("{}{}{}", prefix, parts.join(&delim), suffix);
    Ok(StringValue(SString(joined)))
}

pub fn validate_multi_join(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() {
        return LinkingError::other_error("join expects at least 1 argument".to_string()).into();
    }
    let inner = LinkingError::expect_array_type(None, args[0].clone())?;
    LinkingError::expect_type(None, inner, &[StringType])?;
    if args.len() >= 2 {
        LinkingError::expect_type(None, args[1].clone(), &[StringType])?;
    }
    if args.len() == 4 {
        LinkingError::expect_type(None, args[2].clone(), &[StringType])?;
        LinkingError::expect_type(None, args[3].clone(), &[StringType])?;
    }
    Ok(())
}

pub fn eval_is_empty(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(array) = value {
        Ok(BooleanValue(array.is_empty()))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

pub fn eval_partition(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let size = as_int(&right).ok_or_else(|| RuntimeError::type_not_supported(right.get_type()))?;

    match left {
        Array(ArrayValue::EmptyUntyped) => {
            if size <= 0 {
                let chunk = Array(ArrayValue::PrimitivesArray {
                    values: Vec::new(),
                    item_type: ValueType::UndefinedType,
                });
                build_array_from_parts(vec![chunk], ValueType::list_of(ValueType::UndefinedType))
            } else {
                build_array_from_parts(Vec::new(), ValueType::list_of(ValueType::UndefinedType))
            }
        }
        Array(ArrayValue::PrimitivesArray {
            values,
            mut item_type,
        }) => {
            if matches!(item_type, ValueType::UndefinedType) && !values.is_empty() {
                item_type = values[0].get_type();
            }

            let chunk_item_type = ValueType::list_of(item_type.clone());
            let mut chunks: Vec<ValueEnum> = Vec::new();

            if size <= 0 {
                chunks.push(build_array_from_parts(Vec::new(), item_type.clone())?);
            } else {
                let mut idx = 0usize;
                while idx < values.len() {
                    let end = (((idx as i64) + size).min(values.len() as i64)) as usize;
                    let chunk_values: Vec<ValueEnum> = values[idx..end].to_vec();
                    chunks.push(build_array_from_parts(chunk_values, item_type.clone())?);
                    idx = end;
                }
            }

            build_array_from_parts(chunks, chunk_item_type)
        }
        Array(ArrayValue::ObjectsArray {
            values,
            object_type,
        }) => {
            let chunk_item_type =
                ValueType::list_of(ValueType::ObjectType(Rc::clone(&object_type)));
            let mut chunks: Vec<ValueEnum> = Vec::new();

            if size <= 0 {
                chunks.push(Array(ArrayValue::ObjectsArray {
                    values: Vec::new(),
                    object_type: Rc::clone(&object_type),
                }));
            } else {
                let mut idx = 0usize;
                while idx < values.len() {
                    let end = (((idx as i64) + size).min(values.len() as i64)) as usize;
                    let chunk_values: Vec<Rc<RefCell<ExecutionContext>>> =
                        values[idx..end].to_vec();
                    chunks.push(Array(ArrayValue::ObjectsArray {
                        values: chunk_values,
                        object_type: Rc::clone(&object_type),
                    }));
                    idx = end;
                }
            }

            build_array_from_parts(chunks, chunk_item_type)
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}
