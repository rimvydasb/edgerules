use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::token::ExpressionEnum;
use crate::ast::token::ExpressionEnum::Variable;
use crate::ast::variable::VariableLink;
use crate::ast::{is_linked, Link};
use crate::runtime::execution_context::*;
use crate::typesystem::errors::ParseErrorEnum::UnknownError;
use crate::typesystem::errors::{ErrorStack, LinkingError, ParseErrorEnum, RuntimeError};
use log::trace;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::typesystem::types::number::NumberEnum::{Int, SV};
use crate::typesystem::types::{SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{Array, BooleanValue, NumberValue, Reference, DateValue, TimeValue, DateTimeValue};
use crate::typesystem::values::ValueOrSv;
use crate::typesystem::types::number::NumberEnum as Num;

//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct ExpressionFilter {
    pub source: ExpressionEnum,
    pub method: ExpressionEnum,
    pub method_type: Link<ValueType>,
    pub return_type: Link<ValueType>,
}

impl Display for ExpressionFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.source, self.method)
    }
}

impl ExpressionFilter {
    /// method could evaluate to:
    /// 1. number: myList[1 + b]
    /// 2. boolean: myList[...> 10]
    ///
    /// source must not be boolean, object
    pub fn build(source: ExpressionEnum, method: ExpressionEnum) -> Result<Self, ParseErrorEnum> {
        Ok(ExpressionFilter {
            source,
            method,
            return_type: LinkingError::not_linked().into(),
            method_type: LinkingError::not_linked().into(),
        })
    }

    fn select_from_list(
        &self,
        values: Vec<Result<ValueEnum, RuntimeError>>,
        list_type: ValueType,
        context: Rc<RefCell<ExecutionContext>>,
    ) -> Result<ValueEnum, RuntimeError> {
        trace!("Selecting from list with method: {}", self.method);
        // @Todo: remove cloning
        if self.method_type.clone()? == ValueType::BooleanType {
            let mut filtered_values = Vec::new();

            for value in values {
                // @Todo: interesting thing is that no sub-context is created, but existing is reused. What if multiple filters are applied?
                context.borrow_mut().context_variable = Some(value?.clone());
                if self.method.eval(Rc::clone(&context))? == BooleanValue(true) {
                    filtered_values.push(Ok(context.borrow_mut().context_variable.take().unwrap()));
                }
            }

            context.borrow_mut().context_variable = None;

            Ok(Array(filtered_values, list_type))
        } else {
            let method = self.method.eval(Rc::clone(&context))?;

            if let NumberValue(Int(number)) = method {
                return if let Some(token) = values.get(number as usize) {
                    token.clone()
                } else {
                    // todo: should determine the type
                    Ok(NumberValue(SV(SpecialValueEnum::Missing)))
                };
            }

            RuntimeError::eval_error(format!("Cannot select a value with '{}'", method)).into()
        }
    }
}

impl StaticLink for ExpressionFilter {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            let source_type = self.source.link(Rc::clone(&ctx))?;

            if let ValueType::ListType(list_type) = source_type {
                let mut builder = ContextObjectBuilder::new_internal(Rc::clone(&ctx));

                // in the iteration the context variable will be of the type of the list
                builder.set_context_type(*list_type.clone());

                self.method_type = self.method.link(Rc::clone(&builder.build()));

                // @Todo: what about List type?
                let static_type = match &self.method_type.clone()? {
                    // expecting multi, value return
                    ValueType::BooleanType | ValueType::RangeType => ValueType::ListType(list_type),

                    // expecting only single value return
                    _ => *list_type,
                };

                self.return_type = Ok(static_type);
            } else {
                self.return_type = LinkingError::expect_array_type(
                    Some(format!("Filter subject `{}`", self.source)),
                    source_type,
                );
            }
        }

        self.return_type.clone()
    }
}

impl EvaluatableExpression for ExpressionFilter {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let source_value = self.source.eval(Rc::clone(&context))?;

        match source_value {
            Array(list, list_item_type) => self.select_from_list(list, list_item_type, context),
            _ => RuntimeError::eval_error(format!(
                "Cannot filter '{}' because data type is {} and not an array",
                self.source,
                source_value.get_type()
            ))
            .into(),
        }
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct FieldSelection {
    pub source: ExpressionEnum,
    pub method: VariableLink,
    pub return_type: Link<ValueType>,
}

impl FieldSelection {
    pub fn build(source: ExpressionEnum, method: ExpressionEnum) -> Result<Self, ParseErrorEnum> {
        match method {
            Variable(variable) => Ok(FieldSelection {
                source,
                method: variable,
                return_type: LinkingError::not_linked().into(),
            }),
            _ => Err(UnknownError(
                "Selection must be variable or variable path".to_string(),
            )),
        }
    }
}

impl StaticLink for FieldSelection {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            trace!("Linking selection '{}'", self);
            match self.source.link(Rc::clone(&ctx)) {
                Ok(ValueType::ObjectType(source_type)) => {
                    self.return_type = self.method.link(source_type);
                }
                Ok(ValueType::DateType) => {
                    // Supported: year, month, day, weekday
                    let name = self.method.get_name();
                    let ret = match name.as_str() {
                        "year" | "month" | "day" | "weekday" => ValueType::NumberType,
                        _ => return LinkingError::other_error(format!("Field '{}' not found in date", name)).into(),
                    };
                    self.return_type = Ok(ret);
                }
                Ok(ValueType::TimeType) => {
                    // Supported: hour, minute, second
                    let name = self.method.get_name();
                    let ret = match name.as_str() {
                        "hour" | "minute" | "second" => ValueType::NumberType,
                        _ => return LinkingError::other_error(format!("Field '{}' not found in time", name)).into(),
                    };
                    self.return_type = Ok(ret);
                }
                Ok(ValueType::DateTimeType) => {
                    // Supported: year, month, day, hour, minute, second, time, weekday
                    let name = self.method.get_name();
                    let ret = match name.as_str() {
                        "year" | "month" | "day" | "hour" | "minute" | "second" | "weekday" => ValueType::NumberType,
                        "time" => ValueType::TimeType,
                        _ => return LinkingError::other_error(format!("Field '{}' not found in date and time", name)).into(),
                    };
                    self.return_type = Ok(ret);
                }
                Err(error) => {
                    return error
                        .with_context(|| format!("While looking at source '{}'", self.source))
                        .into();
                }
                Ok(other) => {
                    return LinkingError::other_error(format!(
                        "Cannot select '{}' because data type is {} and not an object",
                        self.source, other
                    ))
                    .into();
                }
            }
        }

        self.return_type.clone()
    }
}

impl EvaluatableExpression for FieldSelection {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let source_value = self.source.eval(Rc::clone(&context))?;

        trace!("Selecting from {} ({})", self.source, source_value);

        match source_value {
            Reference(reference) => self.method.eval(reference),
            DateValue(ValueOrSv::Value(d)) => {
                let name = self.method.get_name();
                match name.as_str() {
                    "year" => Ok(NumberValue(Num::from(d.year() as i64))),
                    "month" => Ok(NumberValue(Num::from(d.month() as i64))),
                    "day" => Ok(NumberValue(Num::from(d.day() as i64))),
                    "weekday" => Ok(NumberValue(Num::from(d.weekday().number_from_monday() as i64))),
                    _ => RuntimeError::field_not_found(name.as_str(), "date").into(),
                }
            }
            TimeValue(ValueOrSv::Value(t)) => {
                let name = self.method.get_name();
                match name.as_str() {
                    "hour" => Ok(NumberValue(Num::from(t.hour() as i64))),
                    "minute" => Ok(NumberValue(Num::from(t.minute() as i64))),
                    "second" => Ok(NumberValue(Num::from(t.second() as i64))),
                    _ => RuntimeError::field_not_found(name.as_str(), "time").into(),
                }
            }
            DateTimeValue(ValueOrSv::Value(dt)) => {
                let name = self.method.get_name();
                match name.as_str() {
                    "year" => Ok(NumberValue(Num::from(dt.year() as i64))),
                    "month" => Ok(NumberValue(Num::from(dt.month() as i64))),
                    "day" => Ok(NumberValue(Num::from(dt.day() as i64))),
                    "hour" => Ok(NumberValue(Num::from(dt.hour() as i64))),
                    "minute" => Ok(NumberValue(Num::from(dt.minute() as i64))),
                    "second" => Ok(NumberValue(Num::from(dt.second() as i64))),
                    "weekday" => Ok(NumberValue(Num::from(dt.weekday().number_from_monday() as i64))),
                    "time" => Ok(TimeValue(ValueOrSv::Value(dt.time()))),
                    _ => RuntimeError::field_not_found(name.as_str(), "date and time").into(),
                }
            }
            _ => RuntimeError::eval_error(format!(
                "Cannot select '{}' because data type is {} and not an object",
                self.source,
                source_value.get_type()
            ))
            .into(),
        }
    }
}

impl Display for FieldSelection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.source, self.method)
    }
}

//--------------------------------------------------------------------------------------------------
