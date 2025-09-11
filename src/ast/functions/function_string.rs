use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::link::node_data::ContentHolder;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum::{Char as SChar, String as SString};
use crate::typesystem::types::ValueType::{
    BooleanType, ListType as VTList, NumberType, StringType,
};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{Array, BooleanValue, NumberValue, StringValue};
#[cfg(feature = "base64_functions")]
use base64::{engine::general_purpose, Engine as _};
#[cfg(feature = "regex_functions")]
use regex::RegexBuilder;
use std::rc::Rc;

fn as_string(v: &ValueEnum) -> Option<String> {
    match v {
        StringValue(SString(s)) => Some(s.clone()),
        StringValue(SChar(c)) => Some(c.to_string()),
        _ => None,
    }
}

fn as_int(v: &ValueEnum) -> Option<i64> {
    match v {
        NumberValue(NumberEnum::Int(i)) => Some(*i),
        NumberValue(NumberEnum::Real(r)) => Some(*r as i64),
        NumberValue(NumberEnum::SV(_)) => Some(0),
        _ => None,
    }
}

// Validators and return type helpers
pub fn validate_unary_string(arg: ValueType) -> Link<()> {
    LinkingError::expect_type(None, arg, &[StringType]).map(|_| ())
}
pub fn validate_binary_string_string(left: ValueType, right: ValueType) -> Link<()> {
    LinkingError::expect_type(None, left, &[StringType])?;
    LinkingError::expect_type(None, right, &[StringType])?;
    Ok(())
}
pub fn validate_binary_string_number(left: ValueType, right: ValueType) -> Link<()> {
    LinkingError::expect_type(None, left, &[StringType])?;
    LinkingError::expect_type(None, right, &[NumberType])?;
    Ok(())
}
pub fn validate_binary_string_any(left: ValueType, _right: ValueType) -> Link<()> {
    LinkingError::expect_type(None, left, &[StringType]).map(|_| ())
}
pub fn validate_multi_substring(args: Vec<ValueType>) -> Link<()> {
    if !(args.len() == 2 || args.len() == 3) {
        return LinkingError::other_error("substring expects 2 or 3 arguments".to_string()).into();
    }
    LinkingError::expect_type(None, args[0].clone(), &[StringType])?;
    for (i, t) in args.iter().enumerate().skip(1) {
        LinkingError::expect_type(Some(format!("arg{}", i + 1)), t.clone(), &[NumberType])?;
    }
    Ok(())
}
pub fn validate_multi_replace(args: Vec<ValueType>) -> Link<()> {
    if !(args.len() == 3 || args.len() == 4) {
        return LinkingError::other_error("replace expects 3 or 4 arguments".to_string()).into();
    }
    for t in args.iter().take(3) {
        LinkingError::expect_type(None, t.clone(), &[StringType])?;
    }
    if args.len() == 4 {
        LinkingError::expect_type(None, args[3].clone(), &[StringType])?;
    }
    Ok(())
}
pub fn validate_multi_from_char_code(args: Vec<ValueType>) -> Link<()> {
    for t in args {
        LinkingError::expect_type(None, t, &[NumberType])?;
    }
    Ok(())
}
pub fn validate_multi_pad(args: Vec<ValueType>) -> Link<()> {
    if args.len() != 3 {
        return LinkingError::other_error("padStart/padEnd expects 3 arguments".to_string()).into();
    }
    LinkingError::expect_type(None, args[0].clone(), &[StringType])?;
    LinkingError::expect_type(None, args[1].clone(), &[NumberType])?;
    LinkingError::expect_type(None, args[2].clone(), &[StringType])?;
    Ok(())
}
pub fn return_string_type_unary(_: ValueType) -> ValueType {
    StringType
}
pub fn return_boolean_type_binary(_: ValueType, _: ValueType) -> ValueType {
    BooleanType
}
pub fn return_string_type_binary(_: ValueType, _: ValueType) -> ValueType {
    StringType
}
pub fn return_string_type_multi() -> ValueType {
    StringType
}
pub fn return_string_list_type_binary(_: ValueType, _: ValueType) -> ValueType {
    VTList(Box::new(StringType))
}
pub fn return_number_type_binary(_: ValueType, _: ValueType) -> ValueType {
    NumberType
}

// Implementations
pub fn eval_length(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        Ok(NumberValue(NumberEnum::from(s.chars().count() as i64)))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
pub fn eval_to_upper(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        Ok(StringValue(SString(s.to_uppercase())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
pub fn eval_to_lower(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        Ok(StringValue(SString(s.to_lowercase())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
pub fn eval_trim(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        Ok(StringValue(SString(s.trim().to_string())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
pub fn eval_contains(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(n)) = (as_string(&left), as_string(&right)) {
        Ok(BooleanValue(h.contains(&n)))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_starts_with(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(p)) = (as_string(&left), as_string(&right)) {
        Ok(BooleanValue(h.starts_with(&p)))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_ends_with(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(s)) = (as_string(&left), as_string(&right)) {
        Ok(BooleanValue(h.ends_with(&s)))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_substring(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if !(vals.len() == 2 || vals.len() == 3) {
        return RuntimeError::eval_error("substring expects 2 or 3 args".to_string()).into();
    }
    let s =
        as_string(&vals[0]).ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let start =
        as_int(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let len_opt = if vals.len() == 3 {
        Some(as_int(&vals[2]).ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?)
    } else {
        None
    };
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len() as i64;
    let mut idx = if start > 0 {
        start - 1
    } else if start < 0 {
        n + start
    } else {
        0
    };
    if idx < 0 {
        idx = 0;
    }
    if idx > n {
        idx = n;
    }
    let end = match len_opt {
        Some(l) if l >= 0 => (idx + l).min(n),
        Some(l) if l < 0 => (idx + l).max(0),
        _ => n,
    };
    let (i, j) = (idx as usize, end as usize);
    let out: String = if j >= i {
        chars[i..j].iter().collect()
    } else {
        chars[j..i].iter().collect()
    };
    Ok(StringValue(SString(out)))
}
pub fn eval_substring_before(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(p)) = (as_string(&left), as_string(&right)) {
        if let Some(pos) = h.find(&p) {
            Ok(StringValue(SString(h[..pos].to_string())))
        } else {
            Ok(StringValue(SString(String::new())))
        }
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_substring_after(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(p)) = (as_string(&left), as_string(&right)) {
        if let Some(pos) = h.find(&p) {
            Ok(StringValue(SString(h[(pos + p.len())..].to_string())))
        } else {
            Ok(StringValue(SString(String::new())))
        }
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}

// -----------------------------------------
// Basic, non-regex split/replace operations
// -----------------------------------------

pub fn eval_split(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(pat)) = (as_string(&left), as_string(&right)) {
        let parts: Vec<Result<ValueEnum, RuntimeError>> = h
            .split(&pat)
            .map(|s| Ok(StringValue(SString(s.to_string()))))
            .collect();
        Ok(Array(parts, VTList(Box::new(StringType))))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}

pub fn eval_replace(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if !(vals.len() == 3 || vals.len() == 4) {
        return RuntimeError::eval_error("replace expects 3 or 4 arguments".to_string()).into();
    }
    let s =
        as_string(&vals[0]).ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let pattern =
        as_string(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let repl =
        as_string(&vals[2]).ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?;

    // Fast path: standard replace or empty pattern
    if vals.len() == 3 || pattern.is_empty() {
        return Ok(StringValue(SString(s.replace(&pattern, &repl))));
    }

    // Optional flags (currently supports: 'i' for case-insensitive)
    let flags = as_string(&vals[3]).unwrap_or_default();
    if flags.contains('i') {
        // Prefer regex-based, escaping the literal pattern
        #[cfg(feature = "regex_functions")]
        {
            let mut builder = RegexBuilder::new(&regex::escape(&pattern));
            builder.case_insensitive(true);
            let re = builder
                .build()
                .map_err(|e| RuntimeError::eval_error(e.to_string()))?;
            return Ok(StringValue(SString(
                re.replace_all(&s, repl.as_str()).into_owned(),
            )));
        }

        // Fallback (no-regex build): ASCII case-insensitive replace of all occurrences
        #[cfg(not(feature = "regex_functions"))]
        {
            let s_lower = s.to_ascii_lowercase();
            let pat_lower = pattern.to_ascii_lowercase();
            if pat_lower.is_empty() {
                return Ok(StringValue(SString(s.replace(&pattern, &repl))));
            }
            let mut out = String::with_capacity(s.len());
            let mut i: usize = 0;
            let pat_len = pattern.len();
            while i <= s_lower.len() {
                if let Some(pos) = s_lower[i..].find(&pat_lower) {
                    let real = i + pos;
                    out.push_str(&s[i..real]);
                    out.push_str(&repl);
                    i = real + pat_len;
                } else {
                    out.push_str(&s[i..]);
                    break;
                }
            }
            return Ok(StringValue(SString(out)));
        }
    }

    // No flags handled -> default replace
    Ok(StringValue(SString(s.replace(&pattern, &repl))))
}

pub fn eval_replace_first(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if vals.len() != 3 {
        return RuntimeError::eval_error("replaceFirst expects 3 arguments".to_string()).into();
    }
    let s =
        as_string(&vals[0]).ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let pattern =
        as_string(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let repl =
        as_string(&vals[2]).ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?;

    if pattern.is_empty() {
        return Ok(StringValue(SString(format!("{}{}", repl, s))));
    }

    if let Some(pos) = s.find(&pattern) {
        let mut out = String::with_capacity(s.len() + repl.len());
        out.push_str(&s[..pos]);
        out.push_str(&repl);
        out.push_str(&s[pos + pattern.len()..]);
        Ok(StringValue(SString(out)))
    } else {
        Ok(StringValue(SString(s)))
    }
}

pub fn eval_replace_last(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if vals.len() != 3 {
        return RuntimeError::eval_error("replaceLast expects 3 arguments".to_string()).into();
    }
    let s =
        as_string(&vals[0]).ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let pattern =
        as_string(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let repl =
        as_string(&vals[2]).ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?;

    if pattern.is_empty() {
        return Ok(StringValue(SString(format!("{}{}", s, repl))));
    }

    if let Some(pos) = s.rfind(&pattern) {
        let mut out = String::with_capacity(s.len() + repl.len());
        out.push_str(&s[..pos]);
        out.push_str(&repl);
        out.push_str(&s[pos + pattern.len()..]);
        Ok(StringValue(SString(out)))
    } else {
        Ok(StringValue(SString(s)))
    }
}

#[cfg(feature = "regex_functions")]
pub fn eval_regex_split(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(pat)) = (as_string(&left), as_string(&right)) {
        let re = RegexBuilder::new(&pat)
            .build()
            .map_err(|e| RuntimeError::eval_error(e.to_string()))?;
        let parts: Vec<Result<ValueEnum, RuntimeError>> = re
            .split(&h)
            .map(|s| Ok(StringValue(SString(s.to_string()))))
            .collect();
        Ok(Array(parts, VTList(Box::new(StringType))))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}

#[cfg(not(feature = "regex_functions"))]
pub fn eval_regex_split(_left: ValueEnum, _right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    RuntimeError::eval_error("regex_functions feature is disabled".to_string()).into()
}

#[cfg(feature = "base64_functions")]
pub fn eval_to_base64(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        Ok(StringValue(SString(general_purpose::STANDARD.encode(s))))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
#[cfg(not(feature = "base64_functions"))]
pub fn eval_to_base64(_value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    RuntimeError::eval_error("base64_functions feature is disabled".to_string()).into()
}
#[cfg(feature = "base64_functions")]
pub fn eval_from_base64(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        match general_purpose::STANDARD.decode(s) {
            Ok(bytes) => Ok(StringValue(SString(
                String::from_utf8_lossy(&bytes).to_string(),
            ))),
            Err(e) => RuntimeError::eval_error(e.to_string()).into(),
        }
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
#[cfg(not(feature = "base64_functions"))]
pub fn eval_from_base64(_value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    RuntimeError::eval_error("base64_functions feature is disabled".to_string()).into()
}
#[cfg(feature = "regex_functions")]
pub fn eval_regex_replace(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    if !(vals.len() == 3 || vals.len() == 4) {
        return RuntimeError::eval_error("replace expects 3 or 4 args".to_string()).into();
    }
    let s =
        as_string(&vals[0]).ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let pattern =
        as_string(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let repl =
        as_string(&vals[2]).ok_or_else(|| RuntimeError::type_not_supported(vals[2].get_type()))?;
    let mut builder = RegexBuilder::new(&pattern);
    if vals.len() == 4 {
        if let Some(flags) = as_string(&vals[3]) {
            if flags.contains('i') {
                builder.case_insensitive(true);
            }
        }
    }
    let re = builder
        .build()
        .map_err(|e| RuntimeError::eval_error(e.to_string()))?;
    Ok(StringValue(SString(
        re.replace_all(&s, repl.as_str()).into_owned(),
    )))
}

#[cfg(not(feature = "regex_functions"))]
pub fn eval_regex_replace(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    RuntimeError::eval_error("regex_functions feature is disabled".to_string()).into()
}

pub fn eval_char_at(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(s), Some(i)) = (as_string(&left), as_int(&right)) {
        let mut iter = s.chars();
        let ch = iter.nth(i as usize).unwrap_or('\0');
        Ok(StringValue(SString(if ch == '\0' {
            String::new()
        } else {
            ch.to_string()
        })))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_char_code_at(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(s), Some(i)) = (as_string(&left), as_int(&right)) {
        let code = s.chars().nth(i as usize).map(|c| c as u32).unwrap_or(0);
        Ok(NumberValue(NumberEnum::from(code as i64)))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_index_of(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(n)) = (as_string(&left), as_string(&right)) {
        if let Some(pos) = h.find(&n) {
            Ok(NumberValue(NumberEnum::from(pos as i64)))
        } else {
            Ok(NumberValue(NumberEnum::from(-1)))
        }
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_last_index_of(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(h), Some(n)) = (as_string(&left), as_string(&right)) {
        if let Some(pos) = h.rfind(&n) {
            Ok(NumberValue(NumberEnum::from(pos as i64)))
        } else {
            Ok(NumberValue(NumberEnum::from(-1)))
        }
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_from_char_code(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    let mut s = String::new();
    for v in vals {
        if let Some(i) = as_int(&v) {
            if let Some(ch) = char::from_u32(i as u32) {
                s.push(ch);
            }
        }
    }
    Ok(StringValue(SString(s)))
}
pub fn eval_pad_start(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    let s =
        as_string(&vals[0]).ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let target =
        as_int(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let pad = as_string(&vals[2]).unwrap_or(" ".to_string());
    let mut out = s.clone();
    if target as usize > s.chars().count() {
        let pad_ch = pad.chars().next().unwrap_or(' ');
        let need = target as usize - s.chars().count();
        let prefix: String = std::iter::repeat_n(pad_ch, need).collect();
        out = format!("{}{}", prefix, s);
    }
    Ok(StringValue(SString(out)))
}
pub fn eval_pad_end(
    args: Vec<Result<ValueEnum, RuntimeError>>,
    _ret: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let vals = into_valid(args)?;
    let s =
        as_string(&vals[0]).ok_or_else(|| RuntimeError::type_not_supported(vals[0].get_type()))?;
    let target =
        as_int(&vals[1]).ok_or_else(|| RuntimeError::type_not_supported(vals[1].get_type()))?;
    let pad = as_string(&vals[2]).unwrap_or(" ".to_string());
    let mut out = s.clone();
    if target as usize > s.chars().count() {
        let pad_ch = pad.chars().next().unwrap_or(' ');
        let need = target as usize - s.chars().count();
        let suffix: String = std::iter::repeat_n(pad_ch, need).collect();
        out = format!("{}{}", s, suffix);
    }
    Ok(StringValue(SString(out)))
}
pub fn eval_repeat(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let (Some(s), Some(n)) = (as_string(&left), as_int(&right)) {
        let times = if n < 0 { 0 } else { n as usize };
        Ok(StringValue(SString(s.repeat(times))))
    } else {
        RuntimeError::type_not_supported(left.get_type()).into()
    }
}
pub fn eval_reverse(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        Ok(StringValue(SString(s.chars().rev().collect())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
pub fn eval_sanitize_filename(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Some(s) = as_string(&value) {
        let filtered: String = s
            .chars()
            .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
            .collect();
        Ok(StringValue(SString(filtered)))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}
pub fn eval_interpolate(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    let template =
        as_string(&left).ok_or_else(|| RuntimeError::type_not_supported(left.get_type()))?;
    if let ValueEnum::Reference(ctx) = right {
        let mut out = String::new();
        let mut i = 0;
        let bytes = template.as_bytes();
        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'$' && bytes[i + 1] == b'{' {
                // parse until '}'
                let mut j = i + 2;
                while j < bytes.len() && bytes[j] != b'}' {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'}' {
                    let key = &template[(i + 2)..j];
                    // Look up in context and append unquoted value for strings
                    match ctx.borrow().get(key) {
                        Ok(EObjectContent::ExpressionRef(expr)) => {
                            match expr.borrow().expression.eval(Rc::clone(&ctx)) {
                                Ok(ValueEnum::StringValue(SString(s))) => out.push_str(&s),
                                Ok(ValueEnum::StringValue(SChar(c))) => out.push(c),
                                Ok(val) => out.push_str(&val.to_string()),
                                Err(_) => {}
                            }
                        }
                        Ok(EObjectContent::ConstantValue(ValueEnum::StringValue(SString(s)))) => {
                            out.push_str(&s)
                        }
                        Ok(EObjectContent::ConstantValue(ValueEnum::StringValue(SChar(c)))) => {
                            out.push(c)
                        }
                        Ok(EObjectContent::ConstantValue(v)) => out.push_str(&v.to_string()),
                        _ => {}
                    }
                    i = j + 1;
                    continue;
                }
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        Ok(StringValue(SString(out)))
    } else {
        Ok(StringValue(SString(template)))
    }
}
