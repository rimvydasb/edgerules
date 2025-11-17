use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::expression::StaticLink;
use crate::ast::sequence::CollectionExpression;
use crate::ast::token::ExpressionEnum;
use crate::link::linker;
use crate::link::node_data::ContentHolder;
use crate::runtime::edge_rules::EdgeRulesModel;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::RuntimeError;
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::ValueType;
use crate::typesystem::values::{ArrayValue, ValueEnum, ValueOrSv};
use js_sys::{Array, Date as JsDate, Object, Reflect};
use std::cell::RefCell;
use std::rc::Rc;
use time::{Month, PrimitiveDateTime, Time};
use wasm_bindgen::{JsCast, JsValue};

pub fn evaluate_all_inner(code: &str) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(|err| err.to_string())?;
    let runtime = service.to_runtime().map_err(|err| err.to_string())?;
    runtime.eval_all().map_err(|err| err.to_string())?;
    let context = Rc::clone(&runtime.context);
    execution_context_to_js(context).map_err(|err| err.to_string())
}

pub fn evaluate_expression_inner(code: &str) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    let runtime = service
        .to_runtime_snapshot()
        .map_err(|err| err.to_string())?;
    let value = runtime
        .evaluate_expression_str(code)
        .map_err(|err| err.to_string())?;
    value_to_js(&value).map_err(|err| err.to_string())
}

pub fn evaluate_field_inner(code: &str, field: &str) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(|err| err.to_string())?;
    let runtime = service.to_runtime().map_err(|err| err.to_string())?;
    let value = runtime
        .evaluate_field(field)
        .map_err(|err| err.to_string())?;
    value_to_js(&value).map_err(|err| err.to_string())
}

pub fn evaluate_method_inner(code: &str, method: &str, args: &JsValue) -> Result<JsValue, String> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(|err| err.to_string())?;
    let runtime = service.to_runtime().map_err(|err| err.to_string())?;
    let expr_args = js_args_to_expressions(args)?;
    let value = runtime
        .call_method(method, expr_args)
        .map_err(|err| err.to_string())?;
    value_to_js(&value).map_err(|err| err.to_string())
}

pub fn throw_js_error(message: String) -> ! {
    wasm_bindgen::throw_str(&message);
}

fn execution_context_to_js(
    context: Rc<RefCell<ExecutionContext>>,
) -> Result<JsValue, RuntimeError> {
    ExecutionContext::eval_all_fields(&context)?;

    let js_object = Object::new();
    let field_names: Vec<&'static str> = {
        let ctx_ref = context.borrow();
        let object = ctx_ref.object.borrow();
        object.get_field_names()
    };

    for field_name in field_names {
        let js_value = match {
            let ctx_ref = context.borrow();
            ctx_ref.get(field_name)
        } {
            Ok(EObjectContent::ConstantValue(value)) => value_to_js(&value)?,
            Ok(EObjectContent::ObjectRef(child)) => execution_context_to_js(child)?,
            Ok(EObjectContent::UserFunctionRef(_)) => continue,
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

pub(crate) fn value_to_js(value: &ValueEnum) -> Result<JsValue, RuntimeError> {
    match value {
        ValueEnum::BooleanValue(flag) => Ok(JsValue::from_bool(*flag)),
        ValueEnum::NumberValue(number) => match number {
            NumberEnum::Real(v) => Ok(JsValue::from_f64(*v)),
            NumberEnum::Int(v) => Ok(JsValue::from_f64(*v as f64)),
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
        ValueEnum::Array(array) => {
            let js_array = Array::new();
            match array {
                ArrayValue::PrimitivesArray { values, .. } => {
                    for item in values {
                        let js_item = value_to_js(item)?;
                        js_array.push(&js_item);
                    }
                }
                ArrayValue::ObjectsArray { values, .. } => {
                    for item in values {
                        let js_item = execution_context_to_js(Rc::clone(item))?;
                        js_array.push(&js_item);
                    }
                }
                ArrayValue::EmptyUntyped => {}
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
        ValueEnum::PeriodValue(inner) => match inner {
            ValueOrSv::Value(period) => {
                let text = ValueEnum::PeriodValue(ValueOrSv::Value(period.clone())).to_string();
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

pub(crate) fn js_to_value(js_value: &JsValue) -> Result<ValueEnum, String> {
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
        if elements.is_empty() {
            return Ok(ValueEnum::Array(ArrayValue::EmptyUntyped));
        }

        let list_type = infer_js_array_list_type(&elements).unwrap_or(ValueType::ListType(None));

        if let ValueType::ListType(Some(inner)) = &list_type {
            if let ValueType::ObjectType(object_type) = inner.as_ref() {
                if elements
                    .iter()
                    .all(|value| matches!(value, ValueEnum::Reference(_)))
                {
                    let contexts = elements
                        .into_iter()
                        .map(|value| match value {
                            ValueEnum::Reference(ctx) => ctx,
                            _ => unreachable!("expected object reference in array"),
                        })
                        .collect();
                    return Ok(ValueEnum::Array(ArrayValue::ObjectsArray {
                        values: contexts,
                        object_type: Rc::clone(object_type),
                    }));
                }
            }
        }

        let item_type = match list_type {
            ValueType::ListType(Some(inner)) => *inner,
            ValueType::ListType(None) => ValueType::ListType(None),
            other => other,
        };

        return Ok(ValueEnum::Array(ArrayValue::PrimitivesArray {
            values: elements,
            item_type,
        }));
    }

    if js_value.is_instance_of::<JsDate>() {
        let date = JsDate::unchecked_from_js(js_value.clone());
        return js_date_to_value(date);
    }

    if js_value.is_object() {
        if js_value.is_function() {
            return Err("Functions are not supported as EdgeRules values".to_string());
        }
        let obj: Object = js_value
            .clone()
            .dyn_into()
            .map_err(|_| "Unsupported JS object value for EdgeRules".to_string())?;
        return js_object_to_value(obj);
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

    let hour = u8::try_from(date.get_utc_hours() as u32)
        .map_err(|_| "Invalid hour value for Date".to_string())?;
    let minute = u8::try_from(date.get_utc_minutes() as u32)
        .map_err(|_| "Invalid minute value for Date".to_string())?;
    let second = u8::try_from(date.get_utc_seconds() as u32)
        .map_err(|_| "Invalid second value for Date".to_string())?;
    let millisecond = u16::try_from(date.get_utc_milliseconds() as u32)
        .map_err(|_| "Invalid millisecond value for Date".to_string())?;

    if hour == 0 && minute == 0 && second == 0 && millisecond == 0 {
        return Ok(ValueEnum::DateValue(ValueOrSv::Value(feel_date)));
    }

    let feel_time = Time::from_hms_milli(hour, minute, second, millisecond)
        .map_err(|err| format!("Invalid time: {}", err))?;
    let datetime = PrimitiveDateTime::new(feel_date, feel_time);
    Ok(ValueEnum::DateTimeValue(ValueOrSv::Value(datetime)))
}

fn js_object_to_value(object: Object) -> Result<ValueEnum, String> {
    let entries = Object::entries(&object);
    let mut builder = ContextObjectBuilder::new();

    for entry in entries.iter() {
        let pair = Array::from(&entry);
        let key = pair
            .get(0)
            .as_string()
            .ok_or_else(|| "Object keys must be strings".to_string())?;
        let value_js = pair.get(1);
        let value_enum = js_to_value(&value_js)?;
        builder
            .add_expression(key.as_str(), ExpressionEnum::from(value_enum.clone()))
            .map_err(|err| err.to_string())?;
    }

    let static_context = builder.build();
    linker::link_parts(Rc::clone(&static_context)).map_err(|err| err.to_string())?;
    let exec_ctx = ExecutionContext::create_isolated_context(Rc::clone(&static_context));
    ExecutionContext::eval_all_fields(&exec_ctx).map_err(|err| err.to_string())?;
    Ok(ValueEnum::Reference(exec_ctx))
}

fn infer_js_array_list_type(elements: &[ValueEnum]) -> Option<ValueType> {
    if elements.is_empty() {
        return Some(ValueType::ListType(None));
    }

    let expressions: Vec<ExpressionEnum> =
        elements.iter().cloned().map(ExpressionEnum::from).collect();

    let mut collection = CollectionExpression::build(expressions);
    let ctx = ContextObjectBuilder::new().build();
    collection.link(ctx).ok()
}
