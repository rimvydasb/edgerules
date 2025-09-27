#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::token::ExpressionEnum;
use crate::link::node_data::ContentHolder;
use crate::runtime::edge_rules::EdgeRulesModel;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::RuntimeError;
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::values::{ValueEnum, ValueOrSv};
use js_sys::{Array, Date as JsDate, Object, Reflect};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use time::Month;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Inline JS glue to leverage host RegExp for regexReplace/regexSplit on Web/Node
// without pulling in the Rust regex crate (keeps WASM small).
#[wasm_bindgen(inline_js = r#"
export function __er_regex_replace(s, pattern, flags, repl) {
  try {
    const re = new RegExp(pattern, flags || 'g');
    return String(s).replace(re, repl);
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_regex_split(s, pattern, flags) {
  try {
    const re = new RegExp(pattern, flags || 'g');
    const SEP = "\u001F"; // Unit Separator as rarely-used delimiter
    const parts = String(s).split(re).map(p => p.split(SEP).join(SEP + SEP));
    return parts.join(SEP);
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_to_base64(s) {
  try {
    if (typeof btoa === 'function') {
      return btoa(String(s));
    }
    // Node.js
    return Buffer.from(String(s), 'utf-8').toString('base64');
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_from_base64(s) {
  try {
    if (typeof atob === 'function') {
      return atob(String(s));
    }
    // Node.js
    return Buffer.from(String(s), 'base64').toString('utf-8');
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}
"#)]
extern "C" {
    fn __er_regex_replace(s: &str, pattern: &str, flags: &str, repl: &str) -> String;
    fn __er_regex_split(s: &str, pattern: &str, flags: &str) -> String;
    fn __er_to_base64(s: &str) -> String;
    fn __er_from_base64(s: &str) -> String;
}

// Internal helper used by string functions to call into JS RegExp replace.
// Returns Err with a human-readable message if the pattern or flags are invalid.
pub(crate) fn regex_replace_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
    repl: &str,
) -> Result<String, String> {
    let f = flags.unwrap_or("g");
    let out = __er_regex_replace(s, pattern, f, repl);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

// Calls into JS RegExp split; returns vector of parts.
pub(crate) fn regex_split_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
) -> Result<Vec<String>, String> {
    let f = flags.unwrap_or("g");
    let out = __er_regex_split(s, pattern, f);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        // Split on the Unit Separator and collapse escaped separators
        let sep = '\u{001F}';
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut chars = out.chars().peekable();
        while let Some(c) = chars.next() {
            if c == sep {
                if let Some(next) = chars.peek() {
                    if *next == sep {
                        // Escaped separator -> emit one and consume the duplicate
                        current.push(sep);
                        chars.next();
                        continue;
                    }
                }
                // Segment boundary
                parts.push(current);
                current = String::new();
            } else {
                current.push(c);
            }
        }
        parts.push(current);
        Ok(parts)
    }
}

pub(crate) fn to_base64_js(s: &str) -> Result<String, String> {
    let out = __er_to_base64(s);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

pub(crate) fn from_base64_js(s: &str) -> Result<String, String> {
    let out = __er_from_base64(s);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Provide a stable no-op export when the panic hook feature is disabled,
// so JS/TS that calls `init_panic_hook()` does not break in release builds.
#[cfg(all(not(feature = "console_error_panic_hook")))]
#[wasm_bindgen]
pub fn init_panic_hook() {
    // no-op
}

#[wasm_bindgen]
pub fn evaluate_all(code: &str) -> JsValue {
    match evaluate_all_inner(code) {
        Ok(value) => value,
        Err(err) => throw_js_error(err),
    }
}

#[wasm_bindgen]
pub fn evaluate_expression(code: &str) -> JsValue {
    match evaluate_expression_inner(code) {
        Ok(value) => value,
        Err(err) => throw_js_error(err),
    }
}

#[wasm_bindgen]
pub fn evaluate_field(code: &str, field: &str) -> JsValue {
    match evaluate_field_inner(code, field) {
        Ok(value) => value,
        Err(err) => throw_js_error(err),
    }
}

#[wasm_bindgen]
pub fn evaluate_method(code: &str, method: &str, args: &JsValue) -> JsValue {
    match evaluate_method_inner(code, method, args) {
        Ok(value) => value,
        Err(err) => throw_js_error(err),
    }
}

fn evaluate_all_inner(code: &str) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    service.load_source(code).map_err(|err| err.to_string())?;
    let runtime = service.to_runtime().map_err(|err| err.to_string())?;
    runtime.eval_all().map_err(|err| err.to_string())?;
    let context = Rc::clone(&runtime.context);
    execution_context_to_js(context).map_err(|err| err.to_string())
}

fn evaluate_expression_inner(code: &str) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    let runtime = service
        .to_runtime_snapshot()
        .map_err(|err| err.to_string())?;
    let value = runtime
        .evaluate_expression_str(code)
        .map_err(|err| err.to_string())?;
    value_to_js(&value).map_err(|err| err.to_string())
}

fn evaluate_field_inner(code: &str, field: &str) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    service.load_source(code).map_err(|err| err.to_string())?;
    let runtime = service.to_runtime().map_err(|err| err.to_string())?;
    let value = runtime
        .evaluate_field(field)
        .map_err(|err| err.to_string())?;
    value_to_js(&value).map_err(|err| err.to_string())
}

fn evaluate_method_inner(code: &str, method: &str, args: &JsValue) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    service.load_source(code).map_err(|err| err.to_string())?;
    let runtime = service.to_runtime().map_err(|err| err.to_string())?;
    let expr_args = js_args_to_expressions(args)?;
    let value = runtime
        .call_method(method, expr_args)
        .map_err(|err| err.to_string())?;
    value_to_js(&value).map_err(|err| err.to_string())
}

fn throw_js_error(message: String) -> ! {
    wasm_bindgen::throw_str(&message);
}

fn execution_context_to_js(
    context: Rc<RefCell<ExecutionContext>>,
) -> Result<JsValue, RuntimeError> {
    ExecutionContext::eval_all_fields(Rc::clone(&context))?;

    let js_object = Object::new();
    let field_names: Vec<&'static str> = {
        let ctx_ref = context.borrow();
        let names = {
            let object_ref = ctx_ref.object.borrow();
            object_ref.get_field_names()
        };
        names
    };

    for field_name in field_names {
        let js_value = match {
            let ctx_ref = context.borrow();
            ctx_ref.get(field_name)
        } {
            Ok(EObjectContent::ConstantValue(value)) => value_to_js(&value)?,
            Ok(EObjectContent::ObjectRef(child)) => execution_context_to_js(child)?,
            Ok(EObjectContent::MetaphorRef(_)) => continue,
            Ok(EObjectContent::Definition(_)) => continue,
            Ok(EObjectContent::ExpressionRef(_)) => {
                return Err(RuntimeError::eval_error(format!(
                    "Field '{}' is not evaluated",
                    field_name
                )))
            }
            Err(err) => return Err(RuntimeError::eval_error(err.to_string())),
        };

        Reflect::set(&js_object, &JsValue::from_str(field_name), &js_value).map_err(|_| {
            RuntimeError::eval_error(format!("Failed to set field '{}'", field_name))
        })?;
    }

    Ok(JsValue::from(js_object))
}

fn value_to_js(value: &ValueEnum) -> Result<JsValue, RuntimeError> {
    match value {
        ValueEnum::BooleanValue(flag) => Ok(JsValue::from_bool(*flag)),
        ValueEnum::NumberValue(number) => match number {
            NumberEnum::Real(v) => Ok(JsValue::from_f64(*v)),
            NumberEnum::Int(v) => Ok(JsValue::from_f64(*v as f64)),
            NumberEnum::Fraction(numerator, denominator) => {
                if *denominator == 0 {
                    return Err(RuntimeError::eval_error(
                        "Cannot convert fraction with zero denominator".to_string(),
                    ));
                }
                Ok(JsValue::from_f64(*numerator as f64 / *denominator as f64))
            }
            NumberEnum::SV(sv) => Ok(JsValue::from_str(&sv.to_string())),
        },
        ValueEnum::StringValue(inner) => {
            let text = match inner {
                StringEnum::String(s) => s.clone(),
                StringEnum::Char(c) => c.to_string(),
                StringEnum::SV(sv) => sv.to_string(),
            };
            Ok(JsValue::from_str(&text))
        }
        ValueEnum::Array(items, _) => {
            let js_array = Array::new();
            for item in items {
                let js_item = match item {
                    Ok(inner) => value_to_js(inner)?,
                    Err(err) => return Err(err.clone()),
                };
                js_array.push(&js_item);
            }
            Ok(JsValue::from(js_array))
        }
        ValueEnum::Reference(ctx) => execution_context_to_js(Rc::clone(ctx)),
        ValueEnum::RangeValue(range) => {
            let js_range = Object::new();
            Reflect::set(
                &js_range,
                &JsValue::from_str("start"),
                &JsValue::from_f64(range.start as f64),
            )
            .map_err(|_| RuntimeError::eval_error("Failed to export range.start".to_string()))?;
            Reflect::set(
                &js_range,
                &JsValue::from_str("endExclusive"),
                &JsValue::from_f64(range.end as f64),
            )
            .map_err(|_| RuntimeError::eval_error("Failed to export range.end".to_string()))?;
            Ok(JsValue::from(js_range))
        }
        ValueEnum::DateValue(inner) => match inner {
            ValueOrSv::Value(date) => Ok(JsValue::from_str(&date.to_string())),
            ValueOrSv::Sv(sv) => Ok(JsValue::from_str(&sv.to_string())),
        },
        ValueEnum::TimeValue(inner) => match inner {
            ValueOrSv::Value(time) => Ok(JsValue::from_str(&time.to_string())),
            ValueOrSv::Sv(sv) => Ok(JsValue::from_str(&sv.to_string())),
        },
        ValueEnum::DateTimeValue(inner) => match inner {
            ValueOrSv::Value(dt) => Ok(JsValue::from_str(&dt.to_string())),
            ValueOrSv::Sv(sv) => Ok(JsValue::from_str(&sv.to_string())),
        },
        ValueEnum::DurationValue(inner) => match inner {
            ValueOrSv::Value(duration) => {
                let text = ValueEnum::DurationValue(ValueOrSv::Value(duration.clone())).to_string();
                Ok(JsValue::from_str(&text))
            }
            ValueOrSv::Sv(sv) => Ok(JsValue::from_str(&sv.to_string())),
        },
        ValueEnum::TypeValue(value_type) => Ok(JsValue::from_str(&value_type.to_string())),
    }
}

fn js_args_to_expressions(args: &JsValue) -> Result<Vec<ExpressionEnum>, String> {
    if args.is_undefined() || args.is_null() {
        return Ok(Vec::new());
    }

    if Array::is_array(args) {
        let array = Array::from(args);
        let mut expressions = Vec::with_capacity(array.length() as usize);
        for item in array.iter() {
            let value = js_to_value(&item)?;
            expressions.push(ExpressionEnum::from(value));
        }
        Ok(expressions)
    } else {
        let value = js_to_value(args)?;
        Ok(vec![ExpressionEnum::from(value)])
    }
}

fn js_to_value(js_value: &JsValue) -> Result<ValueEnum, String> {
    if js_value.is_undefined() || js_value.is_null() {
        return Err("null or undefined values are not supported".to_string());
    }

    if let Some(boolean) = js_value.as_bool() {
        return Ok(ValueEnum::BooleanValue(boolean));
    }

    if let Some(number) = js_value.as_f64() {
        if !number.is_finite() {
            return Err("Only finite numbers are supported".to_string());
        }
        return Ok(ValueEnum::from(number));
    }

    if let Some(string) = js_value.as_string() {
        return Ok(ValueEnum::StringValue(StringEnum::from(string)));
    }

    if Array::is_array(js_value) {
        let array = Array::from(js_value);
        let mut elements = Vec::with_capacity(array.length() as usize);
        for item in array.iter() {
            elements.push(js_to_value(&item)?);
        }
        return Ok(ValueEnum::from(elements));
    }

    if js_value.is_instance_of::<JsDate>() {
        let date = JsDate::unchecked_from_js(js_value.clone());
        return js_date_to_value(date);
    }

    Err("Unsupported JS value type for EdgeRules".to_string())
}

fn js_date_to_value(date: JsDate) -> Result<ValueEnum, String> {
    let year = date.get_utc_full_year() as i32;
    let month_index = date.get_utc_month() as u32;
    let day = date.get_utc_date() as u32;

    let month_number = month_index + 1;
    let month = Month::try_from(month_number as u8)
        .map_err(|err| format!("Invalid month value: {}", err))?;
    let day_u8 = u8::try_from(day).map_err(|_| "Invalid day value".to_string())?;

    let feel_date = time::Date::from_calendar_date(year, month, day_u8)
        .map_err(|err| format!("Invalid date: {}", err))?;

    Ok(ValueEnum::DateValue(ValueOrSv::Value(feel_date)))
}
