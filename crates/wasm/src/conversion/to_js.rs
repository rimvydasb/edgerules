use crate::conversion::traits::ToJs;
use crate::utils::set_prop;
use edge_rules::ast::context::context_object_type::EObjectContent;
use edge_rules::ast::token::UserTypeBody;
use edge_rules::link::node_data::ContentHolder;
use edge_rules::runtime::execution_context::ExecutionContext;

use edge_rules::typesystem::errors::RuntimeError;
use edge_rules::typesystem::types::number::NumberEnum;
use edge_rules::typesystem::types::string::StringEnum;
use edge_rules::typesystem::types::ValueType;
use edge_rules::typesystem::values::{ArrayValue, ValueEnum, ValueOrSv};
use js_sys::{Array, Object};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsValue;

impl ToJs for ValueType {
    fn to_js(&self) -> Result<JsValue, RuntimeError> {
        match self {
            ValueType::ObjectType(obj) => {
                let js_object = Object::new();
                let borrowed = obj.borrow();
                for name in borrowed.get_field_names() {
                    if let Ok(content) = borrowed.get(name) {
                        match content {
                            EObjectContent::ExpressionRef(entry) => {
                                if let Ok(field_type) = &entry.borrow().field_type {
                                    set_prop(&js_object, name, &field_type.to_js()?)
                                        .map_err(RuntimeError::eval_error)?;
                                }
                            }
                            EObjectContent::UserFunctionRef(entry) => {
                                if let Ok(field_type) = &entry.borrow().field_type {
                                    set_prop(&js_object, name, &field_type.to_js()?)
                                        .map_err(RuntimeError::eval_error)?;
                                }
                            }
                            EObjectContent::ObjectRef(child) => {
                                let child_type = ValueType::ObjectType(child);
                                set_prop(&js_object, name, &child_type.to_js()?)
                                    .map_err(RuntimeError::eval_error)?;
                            }
                            _ => {}
                        }
                    }
                }

                for (name, body) in borrowed.defined_types.iter() {
                    let type_js = match body {
                        UserTypeBody::TypeRef(tref) => {
                            let resolved_type = borrowed
                                .resolve_type_ref(&tref)
                                .map_err(|e| RuntimeError::eval_error(e.to_string()))?;
                            resolved_type.to_js()?
                        }
                        UserTypeBody::TypeObject(ctx) => {
                            let obj_type = ValueType::ObjectType(Rc::clone(ctx));
                            let js_obj = obj_type.to_js()?;
                            js_obj
                        }
                    };
                    set_prop(&js_object, name, &type_js).map_err(RuntimeError::eval_error)?;
                }

                Ok(JsValue::from(js_object))
            }
            ValueType::ListType(Some(inner)) => {
                let js_obj = Object::new();
                set_prop(&js_obj, "type", &JsValue::from_str("list"))
                    .map_err(RuntimeError::eval_error)?;
                set_prop(&js_obj, "itemType", &inner.to_js()?).map_err(RuntimeError::eval_error)?;
                Ok(JsValue::from(js_obj))
            }
            _ => Ok(JsValue::from_str(&self.to_string())),
        }
    }
}

impl ToJs for ValueEnum {
    fn to_js(&self) -> Result<JsValue, RuntimeError> {
        match self {
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
            ValueEnum::Array(array) => array.to_js(),
            ValueEnum::Reference(ctx) => ctx.to_js(),
            ValueEnum::RangeValue(range) => {
                let js_range = Object::new();
                set_prop(&js_range, "start", &JsValue::from_f64(range.start as f64))
                    .map_err(RuntimeError::eval_error)?;
                set_prop(
                    &js_range,
                    "endExclusive",
                    &JsValue::from_f64(range.end as f64),
                )
                .map_err(RuntimeError::eval_error)?;
                Ok(JsValue::from(js_range))
            }
            ValueEnum::DurationValue(inner) => match inner {
                ValueOrSv::Value(duration) => {
                    let text =
                        ValueEnum::DurationValue(ValueOrSv::Value(duration.clone())).to_string();
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
            other => Ok(JsValue::from_str(&other.to_string())),
        }
    }
}

impl ToJs for ArrayValue {
    fn to_js(&self) -> Result<JsValue, RuntimeError> {
        let js_array = Array::new();
        match self {
            ArrayValue::PrimitivesArray { values, .. } => {
                for item in values {
                    js_array.push(&item.to_js()?);
                }
            }
            ArrayValue::ObjectsArray { values, .. } => {
                for item in values {
                    js_array.push(&item.to_js()?);
                }
            }
            ArrayValue::EmptyUntyped => {}
        }
        Ok(JsValue::from(js_array))
    }
}

impl ToJs for Rc<RefCell<ExecutionContext>> {
    fn to_js(&self) -> Result<JsValue, RuntimeError> {
        ExecutionContext::eval_all_fields(self)?;
        let js_object = Object::new();
        // We need to extract field names from the scope to know what to export
        // Scope locking is done inside get_field_names and get
        let field_names = self.borrow().object.borrow().get_field_names();

        for field_name in field_names {
            let field_val_opt = self.borrow().get(field_name);
            match field_val_opt {
                Ok(EObjectContent::ConstantValue(value)) => {
                    set_prop(&js_object, field_name, &value.to_js()?)
                        .map_err(RuntimeError::eval_error)?;
                }
                Ok(EObjectContent::ObjectRef(child)) => {
                    set_prop(&js_object, field_name, &child.to_js()?)
                        .map_err(RuntimeError::eval_error)?;
                }
                Ok(EObjectContent::UserFunctionRef(_)) | Ok(EObjectContent::Definition(_)) => {
                    continue
                }
                Ok(EObjectContent::ExpressionRef(_)) => {
                    return Err(RuntimeError::eval_error(format!(
                        "Field '{}' is not evaluated",
                        field_name
                    )))
                }
                Err(err) => return Err(RuntimeError::eval_error(err.to_string())),
            }
        }
        Ok(JsValue::from(js_object))
    }
}
