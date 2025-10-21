use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::expression::{missing_for_type, EvaluatableExpression, StaticLink};
use crate::ast::token::ExpressionEnum;
use crate::ast::token::ExpressionEnum::Variable;
use crate::ast::variable::VariableLink;
use crate::ast::{is_linked, Link};
use crate::runtime::execution_context::*;
use crate::typesystem::errors::ParseErrorEnum::UnknownError;
use crate::typesystem::errors::{
    ErrorStack, LinkingError, ParseErrorEnum, RuntimeError, RuntimeErrorEnum,
};
use log::trace;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::typesystem::types::number::NumberEnum as Num;
use crate::typesystem::types::number::NumberEnum::Int;
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::{
    BooleanValue, DateTimeValue, DateValue, DurationValue as DurationVariant, NumberValue,
    PeriodValue as PeriodVariant, Reference, TimeValue,
};
use crate::typesystem::values::{number_value_from_i128, ArrayValue, ValueEnum, ValueOrSv};

fn flatten_list_type(value_type: ValueType) -> ValueType {
    match value_type {
        ValueType::ListType(Some(inner)) => flatten_list_type(*inner),
        other => other,
    }
}

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
        array: ArrayValue,
        list_type: ValueType,
        context: Rc<RefCell<ExecutionContext>>,
    ) -> Result<ValueEnum, RuntimeError> {
        trace!("Selecting from list with method: {}", self.method);

        if self.method_type.clone()? == ValueType::BooleanType {
            match array {
                ArrayValue::EmptyUntyped => Ok(ValueEnum::Array(ArrayValue::EmptyUntyped)),
                ArrayValue::PrimitivesArray { values, item_type } => {
                    let mut filtered = Vec::new();
                    for candidate in values {
                        if self.evaluate_predicate(candidate.clone(), Rc::clone(&context))? {
                            filtered.push(candidate);
                        }
                    }
                    Ok(ValueEnum::Array(ArrayValue::PrimitivesArray {
                        values: filtered,
                        item_type,
                    }))
                }
                ArrayValue::ObjectsArray {
                    values,
                    object_type,
                } => {
                    let mut filtered = Vec::new();
                    for reference in values {
                        if self.evaluate_predicate(
                            ValueEnum::Reference(Rc::clone(&reference)),
                            Rc::clone(&context),
                        )? {
                            filtered.push(reference);
                        }
                    }
                    Ok(ValueEnum::Array(ArrayValue::ObjectsArray {
                        values: filtered,
                        object_type,
                    }))
                }
            }
        } else {
            let method = self.method.eval(Rc::clone(&context))?;

            if let NumberValue(Int(number)) = method {
                if number < 0 {
                    let element_type = list_type
                        .get_list_type()
                        .unwrap_or(ValueType::UndefinedType);
                    return missing_for_type(&element_type, None, &context);
                }
                let idx = number as usize;
                match array {
                    ArrayValue::EmptyUntyped => {
                        let element_type = list_type
                            .get_list_type()
                            .unwrap_or(ValueType::UndefinedType);
                        missing_for_type(&element_type, None, &context)
                    }
                    ArrayValue::PrimitivesArray { values, item_type } => {
                        if let Some(value) = values.into_iter().nth(idx) {
                            Ok(value)
                        } else {
                            missing_for_type(&item_type, None, &context)
                        }
                    }
                    ArrayValue::ObjectsArray {
                        values,
                        object_type,
                    } => {
                        if let Some(reference) = values.into_iter().nth(idx) {
                            Ok(ValueEnum::Reference(reference))
                        } else {
                            missing_for_type(
                                &ValueType::ObjectType(Rc::clone(&object_type)),
                                None,
                                &context,
                            )
                        }
                    }
                }
            } else {
                RuntimeError::eval_error(format!("Cannot select a value with '{}'", method)).into()
            }
        }
    }

    fn evaluate_predicate(
        &self,
        candidate: ValueEnum,
        context: Rc<RefCell<ExecutionContext>>,
    ) -> Result<bool, RuntimeError> {
        let interpret_result = |result: Result<ValueEnum, RuntimeError>| match result {
            Ok(BooleanValue(true)) => Ok(true),
            Ok(BooleanValue(false)) => Ok(false),
            Ok(_) => Ok(false),
            Err(err) => match err.error {
                RuntimeErrorEnum::RuntimeFieldNotFound(_, _) => Ok(false),
                _ => Err(err),
            },
        };

        match &candidate {
            Reference(reference_ctx) => {
                let element_ctx = Rc::clone(reference_ctx);
                let previous_context_variable = element_ctx.borrow().context_variable.clone();

                element_ctx.borrow_mut().context_variable = Some(candidate.clone());
                let evaluation = self.method.eval(Rc::clone(&element_ctx));
                element_ctx.borrow_mut().context_variable = previous_context_variable;

                interpret_result(evaluation)
            }
            _ => {
                let tmp_ctx = ExecutionContext::create_temp_child_context(
                    Rc::clone(&context),
                    ContextObjectBuilder::new().build(),
                );
                {
                    tmp_ctx.borrow_mut().context_variable = Some(candidate.clone());
                }
                interpret_result(self.method.eval(Rc::clone(&tmp_ctx)))
            }
        }
    }
}

impl StaticLink for ExpressionFilter {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.return_type) {
            let source_type = self.source.link(Rc::clone(&ctx))?;

            if let ValueType::ListType(inner) = source_type {
                let mut builder = ContextObjectBuilder::new_internal(Rc::clone(&ctx));

                if let Some(inner_type) = inner.as_ref() {
                    if let ValueType::ObjectType(object_type) = inner_type.as_ref() {
                        if let Err(err) = builder.append_if_missing(Rc::clone(object_type)) {
                            let linking_error = LinkingError::other_error(err.to_string());
                            self.method_type = Err(linking_error.clone());
                            self.return_type = Err(linking_error.clone());
                            return Err(linking_error);
                        }

                        if let ExpressionEnum::Collection(collection) = &self.source {
                            for element in &collection.elements {
                                if let ExpressionEnum::StaticObject(obj) = element {
                                    if let Err(err) = builder.append_if_missing(Rc::clone(obj)) {
                                        let linking_error =
                                            LinkingError::other_error(err.to_string());
                                        self.method_type = Err(linking_error.clone());
                                        self.return_type = Err(linking_error.clone());
                                        return Err(linking_error);
                                    }
                                }
                            }
                        }
                    }
                }

                let element_type = inner
                    .as_ref()
                    .map(|boxed| (**boxed).clone())
                    .unwrap_or(ValueType::UndefinedType);
                let element_type = flatten_list_type(element_type);

                builder.set_context_type(element_type.clone());

                let method_context = builder.build();

                self.method_type = self.method.link(Rc::clone(&method_context));
                let static_type = match &self.method_type.clone()? {
                    ValueType::BooleanType | ValueType::RangeType => {
                        ValueType::ListType(inner.clone())
                    }
                    _ => element_type,
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
            ValueEnum::Array(array) => {
                let list_type = array.list_type();
                self.select_from_list(array, list_type, context)
            }
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
                        _ => {
                            return LinkingError::other_error(format!(
                                "date does not have '{}' item",
                                name
                            ))
                            .into()
                        }
                    };
                    self.return_type = Ok(ret);
                }
                Ok(ValueType::TimeType) => {
                    // Supported: hour, minute, second
                    let name = self.method.get_name();
                    let ret = match name.as_str() {
                        "hour" | "minute" | "second" => ValueType::NumberType,
                        _ => {
                            return LinkingError::other_error(format!(
                                "time does not have '{}' item",
                                name
                            ))
                            .into()
                        }
                    };
                    self.return_type = Ok(ret);
                }
                Ok(ValueType::DateTimeType) => {
                    // Supported: year, month, day, hour, minute, second, time, weekday
                    let name = self.method.get_name();
                    let ret = match name.as_str() {
                        "year" | "month" | "day" | "hour" | "minute" | "second" | "weekday" => {
                            ValueType::NumberType
                        }
                        "time" => ValueType::TimeType,
                        _ => {
                            return LinkingError::other_error(format!(
                                "datetime does not have '{}' item",
                                name
                            ))
                            .into()
                        }
                    };
                    self.return_type = Ok(ret);
                }
                Ok(ValueType::DurationType) => {
                    // Supported: days, hours, minutes, seconds, totals
                    let name = self.method.get_name();
                    let ret = match name.as_str() {
                        "days" | "hours" | "minutes" | "seconds" | "totalSeconds"
                        | "totalMinutes" | "totalHours" => ValueType::NumberType,
                        _ => {
                            return LinkingError::other_error(format!(
                                "duration does not have '{}' item",
                                name
                            ))
                            .into()
                        }
                    };
                    self.return_type = Ok(ret);
                }
                Ok(ValueType::PeriodType) => {
                    // Supported: years, months, days, totals
                    let name = self.method.get_name();
                    let ret = match name.as_str() {
                        "years" | "months" | "days" | "totalMonths" | "totalDays" => {
                            ValueType::NumberType
                        }
                        _ => {
                            return LinkingError::other_error(format!(
                                "period does not have '{}' item",
                                name
                            ))
                            .into()
                        }
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
            Reference(reference) => {
                let value = self.method.eval(Rc::clone(&reference))?;
                if let Reference(child_ctx) = &value {
                    ExecutionContext::eval_all_fields(child_ctx)?;
                }
                Ok(value)
            }
            DateValue(ValueOrSv::Value(d)) => {
                let name = self.method.get_name();
                match name.as_str() {
                    "year" => Ok(NumberValue(Num::from(d.year() as i64))),
                    "month" => Ok(NumberValue(Num::from(d.month() as i64))),
                    "day" => Ok(NumberValue(Num::from(d.day() as i64))),
                    "weekday" => Ok(NumberValue(Num::from(
                        d.weekday().number_from_monday() as i64
                    ))),
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
                    "weekday" => Ok(NumberValue(Num::from(
                        dt.weekday().number_from_monday() as i64
                    ))),
                    "time" => Ok(TimeValue(ValueOrSv::Value(dt.time()))),
                    _ => RuntimeError::field_not_found(name.as_str(), "date and time").into(),
                }
            }
            DurationVariant(ValueOrSv::Value(dur)) => {
                let name = self.method.get_name();
                match name.as_str() {
                    "days" => {
                        let (days, _, _, _) = dur.normalized_components();
                        Ok(number_value_from_i128(days))
                    }
                    "hours" => {
                        let (_, hours, _, _) = dur.normalized_components();
                        Ok(number_value_from_i128(hours))
                    }
                    "minutes" => {
                        let (_, _, minutes, _) = dur.normalized_components();
                        Ok(number_value_from_i128(minutes))
                    }
                    "seconds" => {
                        let (_, _, _, seconds) = dur.normalized_components();
                        Ok(number_value_from_i128(seconds))
                    }
                    "totalSeconds" => Ok(number_value_from_i128(dur.total_seconds_signed())),
                    "totalMinutes" => Ok(NumberValue(Num::from(dur.total_minutes()))),
                    "totalHours" => Ok(NumberValue(Num::from(dur.total_hours()))),
                    _ => RuntimeError::field_not_found(name.as_str(), "duration").into(),
                }
            }
            PeriodVariant(ValueOrSv::Value(period)) => {
                let name = self.method.get_name();
                match name.as_str() {
                    "years" => {
                        let (years, _) = period.normalized_years_months();
                        Ok(number_value_from_i128(years))
                    }
                    "months" => {
                        let (_, months) = period.normalized_years_months();
                        Ok(number_value_from_i128(months))
                    }
                    "days" => Ok(number_value_from_i128(period.total_days_signed())),
                    "totalMonths" => Ok(number_value_from_i128(period.total_months_signed())),
                    "totalDays" => Ok(number_value_from_i128(period.total_days_signed())),
                    _ => RuntimeError::field_not_found(name.as_str(), "period").into(),
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
