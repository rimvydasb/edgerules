use crate::ast::functions::function_numeric::list_item_as_second_arg;
use crate::link::node_data::ContentHolder;
use crate::ast::functions::function_string as strf;
use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::ValueType::{ListType, NumberType, StringType};
use crate::typesystem::types::{Integer, ValueType, TypedValue};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::types::string::StringEnum::{String as SString, Char as SChar};
use crate::typesystem::values::ValueEnum::{Array, BooleanValue, NumberValue, StringValue, Reference};
use std::cmp::Ordering;
use std::rc::Rc;

fn as_int(v: &ValueEnum) -> Option<i64> {
    match v {
        NumberValue(NumberEnum::Int(i)) => Some(*i),
        NumberValue(NumberEnum::Real(r)) => Some(*r as i64),
        _ => None,
    }
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
    if let ListType(inner) = arg {
        if matches!(*inner, NumberType) {
            Ok(())
        } else {
            LinkingError::expect_type(None, *inner, &[NumberType]).map(|_| ())
        }
    } else {
        LinkingError::expect_type(None, arg, &[ListType(Box::new(NumberType))]).map(|_| ())
    }
}

pub fn return_same_list_type(arg: ValueType) -> ValueType {
    arg
}
pub fn return_list_undefined() -> ValueType {
    ListType(Box::new(ValueType::UndefinedType))
}

pub fn return_flatten_type(arg: ValueType) -> ValueType {
    match arg {
        ListType(inner) => {
            let mut t = *inner;
            while let ListType(next) = t.clone() {
                t = *next;
            }
            ListType(Box::new(t))
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
    if matches!(left, StringType) { NumberType } else { ListType(Box::new(NumberType)) }
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
        Array(values, _t) => {
            let vals = into_valid(values)?;
            Ok(BooleanValue(vals.iter().any(|v| v == &right)))
        }
        other => strf::eval_contains(other, right),
    }
}

pub fn eval_min(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        Array(values, _t) => {
            let vals = into_valid(values)?;
            let mut best: Option<NumberEnum> = None;
            for v in vals {
                if let NumberValue(n) = v {
                    best = Some(match best { Some(b) if n < b => n, Some(b) => b, None => n });
                } else {
                    return RuntimeError::type_not_supported(v.get_type()).into();
                }
            }
            Ok(best.map(NumberValue).unwrap_or_else(|| NumberValue(NumberEnum::from(0))))
        }
        NumberValue(_) => Ok(value),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_product(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        Array(values, _t) => {
            let vals = into_valid(values)?;
            let mut acc: Option<NumberEnum> = None;
            for v in vals {
                if let NumberValue(n) = v {
                    acc = Some(match acc { Some(a) => a * n, None => n });
                } else {
                    return RuntimeError::type_not_supported(v.get_type()).into();
                }
            }
            Ok(NumberValue(acc.unwrap_or(NumberEnum::from(1))))
        }
        NumberValue(n) => Ok(NumberValue(n)),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_mean(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _t) = value {
        let vals = into_valid(values)?;
        let mut sum = 0.0f64;
        let mut count = 0.0f64;
        for v in vals {
            match v {
                NumberValue(NumberEnum::Int(i)) => { sum += i as f64; count += 1.0; }
                NumberValue(NumberEnum::Real(r)) => { sum += r; count += 1.0; }
                _ => return RuntimeError::type_not_supported(v.get_type()).into(),
            }
        }
        let avg = if count == 0.0 { 0.0 } else { sum / count };
        Ok(NumberValue(NumberEnum::from(avg)))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

pub fn eval_median(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _t) = value {
        let vals = into_valid(values)?;
        let mut nums: Vec<f64> = Vec::new();
        for v in vals { match v { NumberValue(NumberEnum::Int(i)) => nums.push(i as f64), NumberValue(NumberEnum::Real(r)) => nums.push(r), _ => return RuntimeError::type_not_supported(v.get_type()).into(), } }
        if nums.is_empty() { return Ok(NumberValue(NumberEnum::from(0))); }
        nums.sort_by(|a,b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        let n = nums.len();
        let med = if n % 2 == 1 { nums[n/2] } else { (nums[n/2 - 1] + nums[n/2]) / 2.0 };
        Ok(NumberValue(NumberEnum::from(med)))
    } else { RuntimeError::type_not_supported(value.get_type()).into() }
}

pub fn eval_stddev(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _t) = value {
        let vals = into_valid(values)?;
        let mut nums: Vec<f64> = Vec::new();
        for v in vals { match v { NumberValue(NumberEnum::Int(i)) => nums.push(i as f64), NumberValue(NumberEnum::Real(r)) => nums.push(r), _ => return RuntimeError::type_not_supported(v.get_type()).into(), } }
        if nums.is_empty() { return Ok(NumberValue(NumberEnum::from(0))); }
        let mean = nums.iter().copied().sum::<f64>() / (nums.len() as f64);
        let var = nums.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (nums.len() as f64);
        Ok(NumberValue(NumberEnum::from(var.sqrt())))
    } else { RuntimeError::type_not_supported(value.get_type()).into() }
}

pub fn eval_mode(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, list_t) = value {
        let vals = into_valid(values)?;
        let mut uniques: Vec<ValueEnum> = Vec::new();
        let mut counts: Vec<i64> = Vec::new();
        for v in vals {
            if let Some(pos) = uniques.iter().position(|u| u == &v) {
                counts[pos] += 1;
            } else {
                uniques.push(v);
                counts.push(1);
            }
        }
        let maxc = counts.iter().copied().max().unwrap_or(0);
        let out: Vec<Result<ValueEnum, RuntimeError>> = uniques
            .into_iter()
            .zip(counts.into_iter())
            .filter_map(|(v,c)| if c == maxc && maxc > 0 { Some(Ok(v)) } else { None })
            .collect();
        Ok(Array(out, list_t))
    } else { RuntimeError::type_not_supported(value.get_type()).into() }
}

pub fn eval_all(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _t) = value {
        let vals = into_valid(values)?;
        for v in vals { match v { BooleanValue(true) => {}, BooleanValue(false) => return Ok(BooleanValue(false)), _ => return RuntimeError::type_not_supported(v.get_type()).into(), } }
        Ok(BooleanValue(true))
    } else { RuntimeError::type_not_supported(value.get_type()).into() }
}

pub fn eval_any(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _t) = value {
        let vals = into_valid(values)?;
        for v in vals { match v { BooleanValue(true) => return Ok(BooleanValue(true)), BooleanValue(false) => {}, _ => return RuntimeError::type_not_supported(v.get_type()).into(), } }
        Ok(BooleanValue(false))
    } else { RuntimeError::type_not_supported(value.get_type()).into() }
}

pub fn eval_sublist(args: Vec<Result<ValueEnum, RuntimeError>>, ret: ValueType) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if !(vals.len() == 2 || vals.len() == 3) { return RuntimeError::eval_error("sublist expects 2 or 3 args".to_string()).into(); }
    let list = match &vals[0] { Array(v, t) => (into_valid(v.clone())?, t.clone()), other => return RuntimeError::type_not_supported(other.get_type()).into() };
    let start = as_int(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?; // 1-based
    let len_opt = if vals.len() == 3 { Some(as_int(&vals[2]).ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?) } else { None };
    let items = list.0;
    let n = items.len() as i64;
    let i = (start - 1).max(0).min(n);
    let j = match len_opt { Some(l) => (i + l).min(n), None => n };
    let mut out: Vec<Result<ValueEnum, RuntimeError>> = Vec::new();
    let (ii, jj) = (i as usize, j as usize);
    for k in ii..jj { out.push(Ok(items[k].clone())); }
    Ok(Array(out, ret))
}

pub fn validate_multi_sublist(args: Vec<ValueType>) -> Link<()> {
    if !(args.len() == 2 || args.len() == 3) { return LinkingError::other_error("sublist expects 2 or 3 arguments".to_string()).into(); }
    LinkingError::expect_array_type(None, args[0].clone())?;
    LinkingError::expect_type(None, args[1].clone(), &[NumberType])?;
    if args.len() == 3 { LinkingError::expect_type(None, args[2].clone(), &[NumberType])?; }
    Ok(())
}


pub fn eval_append(args: Vec<Result<ValueEnum, RuntimeError>>, _ret: ValueType) -> Result<ValueEnum, RuntimeError> {
    if args.is_empty() { return RuntimeError::eval_error("append expects at least 1 argument".to_string()).into(); }
    let vals = into_valid(args)?;
    let (mut items, item_type) = match &vals[0] { Array(v, t) => (into_valid(v.clone())?, t.get_list_type().unwrap_or(ValueType::UndefinedType)), _ => return RuntimeError::type_not_supported(vals[0].get_type()).into() };
    for v in vals.into_iter().skip(1) { items.push(v); }
    let out: Vec<Result<ValueEnum, RuntimeError>> = items.into_iter().map(Ok).collect();
    Ok(Array(out, ListType(Box::new(item_type))))
}

pub fn validate_multi_append(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() { return LinkingError::other_error("append expects at least 1 argument".to_string()).into(); }
    let list_t = LinkingError::expect_array_type(None, args[0].clone())?;
    for t in args.into_iter().skip(1) { LinkingError::expect_same_types("append", list_t.clone(), t)?; }
    Ok(())
}

pub fn eval_concatenate(args: Vec<Result<ValueEnum, RuntimeError>>, _ret: ValueType) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    let mut out_items: Vec<ValueEnum> = Vec::new();
    let mut item_type = ValueType::UndefinedType;
    for v in vals {
        if let Array(arr, t) = v { item_type = t.get_list_type().unwrap_or(item_type); out_items.extend(into_valid(arr)?); } else { return RuntimeError::type_not_supported(v.get_type()).into(); }
    }
    let out: Vec<Result<ValueEnum, RuntimeError>> = out_items.into_iter().map(Ok).collect();
    Ok(Array(out, ListType(Box::new(item_type))))
}

pub fn validate_multi_concatenate(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() { return LinkingError::other_error("concatenate expects at least 1 argument".to_string()).into(); }
    let mut base_item: Option<ValueType> = None;
    for t in args {
        let inner = LinkingError::expect_array_type(None, t)?;
        if let Some(b) = &base_item { LinkingError::expect_same_types("concatenate", b.clone(), inner.clone())?; } else { base_item = Some(inner); }
    }
    Ok(())
}

pub fn eval_insert_before(args: Vec<Result<ValueEnum, RuntimeError>>, ret: ValueType) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if vals.len() != 3 { return RuntimeError::eval_error("insertBefore expects 3 arguments".to_string()).into(); }
    let (mut items, _t) = match &vals[0] { Array(v, t) => (into_valid(v.clone())?, t.clone()), _ => return RuntimeError::type_not_supported(vals[0].get_type()).into() };
    let pos = as_int(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?; // 1-based
    let idx = ((pos - 1).max(0) as usize).min(items.len());
    items.insert(idx, vals[2].clone());
    let out: Vec<Result<ValueEnum, RuntimeError>> = items.into_iter().map(Ok).collect();
    Ok(Array(out, ret))
}

pub fn validate_multi_insert_before(args: Vec<ValueType>) -> Link<()> {
    if args.len() != 3 { return LinkingError::other_error("insertBefore expects 3 arguments".to_string()).into(); }
    let inner = LinkingError::expect_array_type(None, args[0].clone())?;
    LinkingError::expect_type(None, args[1].clone(), &[NumberType])?;
    LinkingError::expect_same_types("insertBefore", inner, args[2].clone()).map(|_| ())
}

pub fn eval_remove(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let (items, t) = match left { Array(v, t) => (into_valid(v)?, t), _ => return RuntimeError::type_not_supported(left.get_type()).into() };
    let pos = as_int(&right).ok_or_else(|| RuntimeError::type_not_supported(right.get_type()))?; // 1-based
    let mut res: Vec<ValueEnum> = Vec::new();
    for (i, v) in items.into_iter().enumerate() { if (i as i64) != (pos - 1) { res.push(v); } }
    Ok(Array(res.into_iter().map(Ok).collect(), t))
}

pub fn eval_reverse_mixed(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        Array(values, t) => {
            let mut vals = into_valid(values)?;
            vals.reverse();
            Ok(Array(vals.into_iter().map(Ok).collect(), t))
        }
        other => strf::eval_reverse(other),
    }
}

pub fn eval_index_of_mixed(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match left {
        Array(values, _t) => {
            let vals = into_valid(values)?;
            let mut pos: Vec<Result<ValueEnum, RuntimeError>> = Vec::new();
            for (i, v) in vals.into_iter().enumerate() { if v == right { pos.push(Ok(ValueEnum::from((i as Integer) + 1))); } }
            Ok(Array(pos, ListType(Box::new(NumberType))))
        }
        other => strf::eval_index_of(other, right),
    }
}

pub fn eval_union(args: Vec<Result<ValueEnum, RuntimeError>>, _ret: ValueType) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    let mut out: Vec<ValueEnum> = Vec::new();
    let mut item_type = ValueType::UndefinedType;
    for v in vals { if let Array(arr, t) = v { item_type = t.get_list_type().unwrap_or(item_type); for x in into_valid(arr)? { if !out.iter().any(|y| y == &x) { out.push(x); } } } else { return RuntimeError::type_not_supported(v.get_type()).into(); } }
    Ok(Array(out.into_iter().map(Ok).collect(), ListType(Box::new(item_type))))
}

pub fn validate_multi_union(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() { return LinkingError::other_error("union expects at least 1 argument".to_string()).into(); }
    let mut base: Option<ValueType> = None;
    for t in args { let inner = LinkingError::expect_array_type(None, t)?; if let Some(b) = &base { LinkingError::expect_same_types("union", b.clone(), inner.clone())?; } else { base = Some(inner); } }
    Ok(())
}

pub fn eval_distinct(values: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let (items, t) = match values { Array(v, t) => (into_valid(v)?, t), _ => return RuntimeError::type_not_supported(values.get_type()).into() };
    let mut out: Vec<ValueEnum> = Vec::new();
    for v in items { if !out.iter().any(|x| x == &v) { out.push(v); } }
    Ok(Array(out.into_iter().map(Ok).collect(), t))
}

pub fn eval_duplicates(values: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let (items, t) = match values { Array(v, t) => (into_valid(v)?, t), _ => return RuntimeError::type_not_supported(values.get_type()).into() };
    let mut uniq: Vec<ValueEnum> = Vec::new();
    let mut dups: Vec<ValueEnum> = Vec::new();
    for v in items { if uniq.iter().any(|x| x == &v) { if !dups.iter().any(|x| x == &v) { dups.push(v); } } else { uniq.push(v); } }
    Ok(Array(dups.into_iter().map(Ok).collect(), t))
}

pub fn eval_flatten(values: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    fn collect(v: ValueEnum, acc: &mut Vec<ValueEnum>) -> Result<ValueType, RuntimeError> {
        match v {
            Array(items, inner_t) => {
                let vals = into_valid(items)?;
                let mut last_t = inner_t;
                for x in vals { last_t = collect(x, acc)?; }
                Ok(last_t)
            }
            other => { acc.push(other.clone()); Ok(other.get_type()) }
        }
    }
    match values {
        Array(items, _t) => {
            let vals = into_valid(items)?;
            let mut acc: Vec<ValueEnum> = Vec::new();
            let mut base: Option<ValueType> = None;
            for v in vals { let t = collect(v, &mut acc)?; base = Some(base.unwrap_or(t)); }
            Ok(Array(acc.into_iter().map(Ok).collect(), ListType(Box::new(base.unwrap_or(ValueType::UndefinedType)))))
        }
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_sort(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let (items, t) = match left { Array(v, t) => (into_valid(v)?, t), _ => return RuntimeError::type_not_supported(left.get_type()).into() };
    let mut arr = items;

    // If right is a string: treat as field name for object sorting
    if let StringValue(SString(field)) = right {
        fn key_for(v: &ValueEnum, field: &str) -> ValueEnum {
            match v {
                Reference(ctx) => match ctx.borrow().get(field) {
                    Ok(crate::ast::context::context_object_type::EObjectContent::ConstantValue(val)) => val.clone(),
                    Ok(crate::ast::context::context_object_type::EObjectContent::ExpressionRef(expr)) => expr.borrow().expression.eval(Rc::clone(ctx)).unwrap_or_else(|_| v.clone()),
                    Ok(crate::ast::context::context_object_type::EObjectContent::ObjectRef(obj)) => Reference(Rc::clone(&obj)),
                    _ => v.clone(),
                },
                _ => v.clone(),
            }
        }
        arr.sort_by(|a, b| {
            let ka = key_for(a, &field);
            let kb = key_for(b, &field);
            match (&ka, &kb) {
                (NumberValue(NumberEnum::Int(x)), NumberValue(NumberEnum::Int(y))) => x.cmp(y),
                (NumberValue(NumberEnum::Real(x)), NumberValue(NumberEnum::Real(y))) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
                (StringValue(sa), StringValue(sb)) => sa.to_string().cmp(&sb.to_string()),
                _ => ka.to_string().cmp(&kb.to_string()),
            }
        });
        return Ok(Array(arr.into_iter().map(Ok).collect(), t));
    }

    // default: numbers ascending, strings lexicographic, else by Display
    arr.sort_by(|a, b| match (a, b) {
        (NumberValue(NumberEnum::Int(x)), NumberValue(NumberEnum::Int(y))) => x.cmp(y),
        (NumberValue(NumberEnum::Real(x)), NumberValue(NumberEnum::Real(y))) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
        (StringValue(sa), StringValue(sb)) => sa.to_string().cmp(&sb.to_string()),
        _ => a.to_string().cmp(&b.to_string()),
    });
    Ok(Array(arr.into_iter().map(Ok).collect(), t))
}

pub fn validate_binary_sort(left: ValueType, right: ValueType) -> Link<()> {
    LinkingError::expect_array_type(None, left)?;
    // accept any comparator placeholder for now; if string, treat as field name
    if matches!(right, StringType) {
        Ok(())
    } else {
        Ok(())
    }
}

pub fn validate_binary_partition(left: ValueType, right: ValueType) -> Link<()> {
    LinkingError::expect_array_type(None, left)?;
    LinkingError::expect_type(None, right, &[NumberType]).map(|_| ())
}

pub fn return_partition_type(left: ValueType, _right: ValueType) -> ValueType {
    match left {
        ListType(inner) => ListType(Box::new(ListType(inner))),
        other => other,
    }
}

pub fn eval_join(args: Vec<Result<ValueEnum, RuntimeError>>, _ret: ValueType) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if vals.is_empty() { return RuntimeError::eval_error("join expects at least 1 argument".to_string()).into(); }
    let (items, _t) = match &vals[0] { Array(v, _t) => (into_valid(v.clone())?, _t.clone()), _ => return RuntimeError::type_not_supported(vals[0].get_type()).into() };
    let delim = if vals.len() >= 2 {
        match &vals[1] { StringValue(SString(s)) => s.clone(), StringValue(SChar(c)) => c.to_string(), _ => return RuntimeError::type_not_supported(vals[1].get_type()).into() }
    } else { String::new() };
    let (prefix, suffix) = if vals.len() >= 4 {
        let p = match &vals[2] { StringValue(SString(s)) => s.clone(), StringValue(SChar(c)) => c.to_string(), _ => return RuntimeError::type_not_supported(vals[2].get_type()).into() };
        let s = match &vals[3] { StringValue(SString(s)) => s.clone(), StringValue(SChar(c)) => c.to_string(), _ => return RuntimeError::type_not_supported(vals[3].get_type()).into() };
        (p, s)
    } else { (String::new(), String::new()) };
    let mut parts: Vec<String> = Vec::new();
    for v in items {
        if let StringValue(SString(s)) = v { parts.push(s); }
        else if let StringValue(SChar(c)) = v { parts.push(c.to_string()); }
        else { /* ignore non-strings (like nulls if added later) */ }
    }
    let joined = format!("{}{}{}", prefix, parts.join(&delim), suffix);
    Ok(StringValue(joined.into()))
}

pub fn validate_multi_join(args: Vec<ValueType>) -> Link<()> {
    if args.is_empty() { return LinkingError::other_error("join expects at least 1 argument".to_string()).into(); }
    let inner = LinkingError::expect_array_type(None, args[0].clone())?;
    LinkingError::expect_type(None, inner, &[StringType])?;
    if args.len() >= 2 { LinkingError::expect_type(None, args[1].clone(), &[StringType])?; }
    if args.len() == 4 { LinkingError::expect_type(None, args[2].clone(), &[StringType])?; LinkingError::expect_type(None, args[3].clone(), &[StringType])?; }
    Ok(())
}

pub fn eval_is_empty(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _t) = value { Ok(BooleanValue(values.len() == 0)) } else { RuntimeError::type_not_supported(value.get_type()).into() }
}

pub fn eval_partition(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let (items, inner_t) = match left { Array(v, t) => (into_valid(v)?, t.get_list_type().unwrap_or(ValueType::UndefinedType)), _ => return RuntimeError::type_not_supported(left.get_type()).into() };
    let size = as_int(&right).ok_or_else(|| RuntimeError::type_not_supported(right.get_type()))?;
    if size <= 0 { return Ok(Array(vec![Ok(Array(Vec::new(), ListType(Box::new(inner_t.clone()))))], ListType(Box::new(ListType(Box::new(inner_t)))))); }
    let mut chunks: Vec<Result<ValueEnum, RuntimeError>> = Vec::new();
    let mut idx = 0usize;
    while idx < items.len() {
        let end = (idx as i64 + size).min(items.len() as i64) as usize;
        let chunk: Vec<Result<ValueEnum, RuntimeError>> = items[idx..end].iter().cloned().map(Ok).collect();
        chunks.push(Ok(Array(chunk, ListType(Box::new(inner_t.clone())))));
        idx = end;
    }
    Ok(Array(chunks, ListType(Box::new(ListType(Box::new(inner_t))))))
}
