use crate::ast::context::context_object::{ContextObject, ExpressionEntry, MethodEntry};
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::context::context_object_type::EObjectContent::*;
use crate::ast::expression::StaticLink;
use crate::ast::metaphors::metaphor::UserFunction;
use crate::ast::Link;
use crate::link::node_data::{ContentHolder, Node, NodeData};
use crate::runtime::execution_context::{build_location_from_execution_context, ExecutionContext};
use crate::typesystem::errors::LinkingErrorEnum::*;
use crate::typesystem::errors::{ErrorStage, LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::ValueType::ObjectType;
use crate::typesystem::values::{number_value_from_i128, ValueEnum};
use crate::utils::intern_field_name;
use log::error;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;

// ---
// **Linker**
// - is responsible for linking all the expressions in the AST.

// @Todo: is it possible or necessary to prevent re-execution of the same code?
pub fn link_parts(context: Rc<RefCell<ContextObject>>) -> Link<Rc<RefCell<ContextObject>>> {
    //trace!("link_parts: {}(..)", context.borrow().node().node_type);

    let field_names = context.borrow().get_field_names();
    let mut references = Vec::new();

    for name in field_names {
        match context.borrow().get(name)? {
            ExpressionRef(expression) => {
                if let Ok(entry) = expression.try_borrow() {
                    if entry.field_type.is_ok() {
                        continue;
                    }
                }
                context.borrow().node().lock_field(name)?;

                let linked_type = {
                    match expression.try_borrow_mut() {
                        Ok(mut entry) => {
                            let expression_display = entry.expression.to_string();
                            let result = entry.expression.link(Rc::clone(&context));
                            match result {
                                Ok(field_type) => {
                                    entry.field_type = Ok(field_type.clone());
                                    Ok(field_type)
                                }
                                Err(mut err) => {
                                    if err.location().is_empty() {
                                        *err.location_mut() = build_location_from_context(&context, name);
                                    }
                                    if !err.has_expression() {
                                        err.set_expression(expression_display);
                                    }
                                    if !err.has_stage() {
                                        err.set_stage(ErrorStage::Linking);
                                    }
                                    Err(err)
                                }
                            }
                        }
                        Err(_) => {
                            let context_name = context.borrow().node().node_type.to_string();
                            Err(LinkingError::cyclic_reference(&context_name, name))
                        }
                    }
                };

                context.borrow().node().unlock_field(name);

                if let Ok(_field_type) = &linked_type {
                    let _context_name = context.borrow().node().node_type.to_string();
                    //trace!("expression: {}.{} -> {}", context_name, name, field_type);
                }

                linked_type?;
            }
            val @ (ObjectRef(_) | Definition(ObjectType(_))) => {
                let (reference, default_name) = match val {
                    ObjectRef(r) => (r, "<child>"),
                    Definition(ObjectType(r)) => (r, "<definition>"),
                    _ => unreachable!(),
                };

                let assigned_name = reference.borrow().node().get_assigned_to_field().unwrap_or(default_name);
                if reference.try_borrow_mut().is_ok() {
                    references.push((name, reference.clone()));
                } else {
                    let context_name = context.borrow().node.node_type.to_string();
                    return Err(LinkingError::cyclic_reference(&context_name, assigned_name));
                }
            }
            _ => {
                // User functions will be linked when the call will be detected
                // Primitive definitions will not be linked at all
            }
        }
    }

    for (_name, reference) in references {
        link_parts(Rc::clone(&reference))?;
    }

    Ok(context)
}

pub fn find_implementation(
    context: Rc<RefCell<ContextObject>>,
    function_name: String,
) -> Link<Rc<RefCell<MethodEntry>>> {
    if function_name.contains('.') {
        let segments: Vec<&str> = function_name.split('.').collect();
        let browse_result = browse(Rc::clone(&context), &segments, true)?;
        return match browse_result {
            BrowseResult::Found(found) => match found.content {
                UserFunctionRef(method) => Ok(method),
                _ => {
                    let known_metaphors = collect_known_implementations(Rc::clone(&context));
                    LinkingError::new(FunctionNotFound { name: function_name, known_metaphors }).into()
                }
            },
            _ => {
                let known_metaphors = collect_known_implementations(Rc::clone(&context));
                LinkingError::new(FunctionNotFound { name: function_name, known_metaphors }).into()
            }
        };
    }

    let mut ctx: Rc<RefCell<ContextObject>> = context;

    loop {
        trace!("find_implementation: searching {} in {} ", function_name, ctx.borrow().node().node_type);

        let implementation = (*ctx).borrow().get_function(function_name.as_str());

        if let Some(definition) = implementation {
            // trace!(
            //     "find_implementation: found {} in {} ",
            //     function_name,
            //     ctx.borrow().node().node_type
            // );
            return Ok(Rc::clone(&definition));
        } else {
            let maybe_parent = (*ctx).borrow().node().node_type.get_parent();

            if let Some(parent_to_check) = maybe_parent {
                ctx = parent_to_check;
            } else {
                error!("find_implementation: Cannot find {} in {} ", function_name, ctx.borrow().node().node_type);

                let known_metaphors = collect_known_implementations(Rc::clone(&ctx));

                return LinkingError::new(FunctionNotFound { name: function_name, known_metaphors }).into();
            }
        }
    }
}

/// This method will be used for debugging and error reporting purposes inside find_implementation
/// when user needs to be informed about all known implementations of a given function
pub fn collect_known_implementations(context: Rc<RefCell<ContextObject>>) -> Vec<String> {
    let mut ctx: Rc<RefCell<ContextObject>> = context;
    let mut implementations = Vec::new();
    let mut visited = HashSet::new();

    loop {
        let maybe_parent = {
            let borrowed = ctx.borrow();
            for name in borrowed.metaphors.keys() {
                if visited.insert(*name) {
                    implementations.push((*name).to_string());
                }
            }
            borrowed.node().node_type.get_parent()
        };

        match maybe_parent {
            Some(parent_to_check) => {
                ctx = parent_to_check;
            }
            None => break,
        }
    }

    implementations.sort();

    implementations
}

pub(crate) fn build_location_from_context(context: &Rc<RefCell<ContextObject>>, field_name: &str) -> Vec<String> {
    let mut location = vec![field_name.to_string()];
    let mut current = Some(Rc::clone(context));

    while let Some(ctx) = current {
        let (parent, assigned) = {
            let borrowed = ctx.borrow();
            (borrowed.node().node_type.get_parent(), borrowed.node().node_type.get_assigned_name())
        };

        if let Some(name) = assigned {
            location.insert(0, name.to_string());
        }

        current = parent;
    }

    location
}

pub fn get_till_root<T: Node<T>>(ctx: Rc<RefCell<T>>, name: &str) -> Result<BrowseResultFound<T>, LinkingError> {
    let interned = intern_field_name(name);
    ctx.borrow().node().lock_field(interned)?;
    let result;
    match ctx.borrow().get(name) {
        Ok(finding) => {
            result = Ok(BrowseResultFound::new(Rc::clone(&ctx), interned, finding));
        }
        Err(err) if matches!(err.kind(), FieldNotFound(_, _)) => {
            if let FieldNotFound(obj_name, field) = err.kind() {
                match ctx.borrow().node().node_type.get_parent() {
                    None => {
                        result = Err(LinkingError::new(FieldNotFound(obj_name.clone(), field.clone())));
                    }
                    Some(parent) => {
                        result = get_till_root(parent, name);
                    }
                }
            } else {
                result = Err(err);
            }
        }
        Err(error) => {
            result = Err(error);
        }
    }
    ctx.borrow().node().unlock_field(interned);
    result
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone)]
pub struct BrowseResultFound<T: Node<T>> {
    pub context: Rc<RefCell<T>>,
    pub field_name: &'static str,
    pub content: EObjectContent<T>,
}

impl<T: Node<T>> BrowseResultFound<T> {
    pub fn new(context: Rc<RefCell<T>>, field_name: &'static str, content: EObjectContent<T>) -> BrowseResultFound<T> {
        BrowseResultFound { context, field_name, content }
    }
}

impl BrowseResultFound<ExecutionContext> {
    /// ObjectContent was retrieved, but to evaluate it it is necessary to provide context:
    /// - context is the execution context where this object is being acquired
    /// - content_name is the name that was used to acquire this content
    pub fn eval(&self) -> Result<ValueEnum, RuntimeError> {
        match &self.content {
            ConstantValue(value) => Ok(value.clone()),
            ExpressionRef(value) => {
                // since linking did it's work, no need to lock again
                let result = match value.borrow().expression.eval(Rc::clone(&self.context)) {
                    Ok(v) => Ok(v),
                    Err(mut err) => {
                        if err.location().is_empty() {
                            *err.location_mut() = build_location_from_execution_context(&self.context, self.field_name);
                        }
                        if !err.has_expression() {
                            err.set_expression(value.borrow().expression.to_string());
                        }
                        Err(err)
                    }
                };
                let final_result = result?;

                // no need to check if in stack, if it was already acquired as expression, it is not in stack
                self.context.borrow().stack_insert(self.field_name, Ok(final_result.clone()));
                Ok(final_result)
            }
            UserFunctionRef(value) => {
                let definition = value
                    .borrow()
                    .function_definition
                    .create_context(vec![], Some(Rc::clone(&self.context.borrow().object)))?;
                let eval_context = definition.create_eval_context(vec![], Rc::clone(&self.context))?;
                ExecutionContext::eval_all_fields(&eval_context)?;
                Ok(ValueEnum::Reference(eval_context))
            }
            ObjectRef(value) => {
                NodeData::attach_child(&self.context, value);
                Ok(ValueEnum::Reference(Rc::clone(value)))
            }
            Definition(definition) => {
                Err(RuntimeError::eval_error(format!("Cannot evaluate definition: {}", definition)))
            }
        }
    }
}

impl<T: Node<T>> Display for BrowseResultFound<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{} = {}", self.context.borrow().node().node_type, self.field_name, self.content)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone)]
pub enum BrowseResult<'a, T: Node<T>> {
    Found(BrowseResultFound<T>),

    // context, expression, remaining path
    OnExpression(Rc<RefCell<T>>, Rc<RefCell<ExpressionEntry>>, &'a [&'a str]),

    OnObjectType(Rc<RefCell<T>>, Rc<RefCell<ContextObject>>, &'a [&'a str]),
}

impl<'a, T: Node<T>> BrowseResult<'a, T> {
    pub fn found(context: Rc<RefCell<T>>, field_name: &'a str, content: EObjectContent<T>) -> BrowseResult<'a, T> {
        let interned = intern_field_name(field_name);
        BrowseResult::Found(BrowseResultFound::new(context, interned, content))
    }

    pub fn on_incomplete<FE, FC>(
        self,
        mut on_expression: FE,
        mut on_object_type: FC,
    ) -> Result<BrowseResultFound<T>, LinkingError>
    where
        FE: FnMut(Rc<RefCell<T>>, Rc<RefCell<ExpressionEntry>>, &[&'a str]) -> Result<EObjectContent<T>, LinkingError>,
        FC: FnMut(Rc<RefCell<T>>, Rc<RefCell<ContextObject>>, &[&'a str]) -> Result<EObjectContent<T>, LinkingError>,
    {
        //trace!("on_incomplete: {:?}", self);
        match self {
            BrowseResult::Found(result) => Ok(result),
            BrowseResult::OnExpression(ctx, content, path) => {
                let result = on_expression(Rc::clone(&ctx), content, path)?;
                let continue_result = continue_browse(path, 0, (Rc::clone(&ctx), result))?;

                continue_result.on_incomplete(on_expression, on_object_type)
            }
            BrowseResult::OnObjectType(ctx, content, path) => {
                let result = on_object_type(Rc::clone(&ctx), content, path)?;
                let continue_result = continue_browse(path, 0, (Rc::clone(&ctx), result))?;

                continue_result.on_incomplete(on_expression, on_object_type)
            }
        }
    }
}

pub fn browse<'a, T: Node<T>>(
    ctx: Rc<RefCell<T>>,
    path: &'a [&'a str],
    find_root: bool,
) -> Result<BrowseResult<'a, T>, LinkingError> {
    // Path is empty - this is abnormal and should never happen
    if path.is_empty() {
        return Err(LinkingError::field_not_found(ctx.borrow().node().node_type.to_string().as_str(), ""));
    }

    // Path is 1
    if path.len() == 1 {
        let field_name = path.first().unwrap();
        return if find_root {
            let result = get_till_root(ctx, field_name)?;
            Ok(BrowseResult::Found(result))
        } else {
            Ok(BrowseResult::found(Rc::clone(&ctx), field_name, ctx.borrow().get(field_name)?))
        };
    }

    // Path > 1
    let mut index = 0usize;
    let starting = if find_root {
        let root = get_till_root(ctx, path[index])?;
        index += 1;
        (Rc::clone(&root.context), root.content)
    } else {
        let first = path[index];
        index += 1;
        (Rc::clone(&ctx), ctx.borrow().get(first)?)
    };

    continue_browse(path, index, starting)
}

// (Intentionally no browse_ids helper to avoid dead code until an interner lands.)

fn constant_value_label(value: &ValueEnum) -> &'static str {
    match value {
        ValueEnum::DateValue(_) => "date",
        ValueEnum::TimeValue(_) => "time",
        ValueEnum::DateTimeValue(_) => "datetime",
        ValueEnum::DurationValue(_) => "duration",
        ValueEnum::PeriodValue(_) => "period",
        _ => "value",
    }
}

/**
 * Continue browsing through the object structure
 *
 * - `browse` is a queue of remaining path elements to browse
 * - `starting` is the current context and the current item to browse into
 *
 * Returns:
 * - `BrowseResult::Found` if the entire path was successfully browsed
 * - `BrowseResult::OnExpression` if an expression was encountered before the end of the path
 * - `BrowseResult::OnObjectType` if an object type was encountered before the end of the path
 */
fn continue_browse<'a, T: Node<T>>(
    path: &'a [&'a str],
    mut index: usize,
    mut starting: (Rc<RefCell<T>>, EObjectContent<T>),
) -> Result<BrowseResult<'a, T>, LinkingError> {
    //trace!("continue_browse(path[{}..], {:?})", index, starting);
    let mut current_search_end: Option<&str> = path.first().copied();

    #[allow(irrefutable_let_patterns)]
    while let (ref context, ref item) = starting {
        if index >= path.len() {
            return if let Some(current_search) = current_search_end {
                Ok(BrowseResult::found(Rc::clone(context), current_search, item.clone()))
            } else {
                // Path is empty so there is nothing to resolve for this context.
                return LinkingError::field_not_found(&context.borrow().node().node_type.to_string(), "<empty>").into();
            };
        }

        let current_search = path[index];
        index += 1;

        match item {
            ConstantValue(value) => {
                // Allow field-like access on runtime constant Date/Time/DateTime values
                use crate::typesystem::values::ValueEnum as VE;
                use crate::typesystem::values::ValueOrSv;
                let maybe_next: Option<VE> = match value {
                    VE::DateValue(ValueOrSv::Value(d)) => match current_search {
                        "year" => Some(VE::from(d.year())),
                        "month" => Some(VE::from(d.month())),
                        "day" => Some(VE::from(d.day())),
                        "weekday" => Some(VE::from(d.weekday().number_from_monday())),
                        _ => None,
                    },
                    VE::TimeValue(ValueOrSv::Value(t)) => match current_search {
                        "hour" => Some(VE::from(t.hour())),
                        "minute" => Some(VE::from(t.minute())),
                        "second" => Some(VE::from(t.second())),
                        _ => None,
                    },
                    VE::DateTimeValue(ValueOrSv::Value(dt)) => match current_search {
                        "year" => Some(VE::from(dt.year())),
                        "month" => Some(VE::from(dt.month())),
                        "day" => Some(VE::from(dt.day())),
                        "hour" => Some(VE::from(dt.hour())),
                        "minute" => Some(VE::from(dt.minute())),
                        "second" => Some(VE::from(dt.second())),
                        "weekday" => Some(VE::from(dt.weekday().number_from_monday())),
                        "time" => Some(VE::TimeValue(ValueOrSv::Value(dt.time()))),
                        "date" => Some(VE::DateValue(ValueOrSv::Value(dt.date()))),
                        _ => None,
                    },
                    VE::DurationValue(ValueOrSv::Value(dur)) => {
                        let (days, hours, minutes, seconds) = dur.normalized_components();
                        match current_search {
                            "days" => Some(number_value_from_i128(days)),
                            "hours" => Some(number_value_from_i128(hours)),
                            "minutes" => Some(number_value_from_i128(minutes)),
                            "seconds" => Some(number_value_from_i128(seconds)),
                            "totalSeconds" => Some(number_value_from_i128(dur.total_seconds_signed())),
                            "totalMinutes" => Some(VE::NumberValue(NumberEnum::from(dur.total_minutes()))),
                            "totalHours" => Some(VE::NumberValue(NumberEnum::from(dur.total_hours()))),
                            _ => None,
                        }
                    }
                    VE::PeriodValue(ValueOrSv::Value(period)) => {
                        let (years, months) = period.normalized_years_months();
                        match current_search {
                            "years" => Some(number_value_from_i128(years)),
                            "months" => Some(number_value_from_i128(months)),
                            "days" => Some(number_value_from_i128(period.total_days_signed())),
                            "totalMonths" => Some(number_value_from_i128(period.total_months_signed())),
                            "totalDays" => Some(number_value_from_i128(period.total_days_signed())),
                            _ => None,
                        }
                    }
                    _ => None,
                };

                if let Some(next_value) = maybe_next {
                    // Step into the computed property value and continue browsing
                    starting = (Rc::clone(context), ConstantValue(next_value));
                } else {
                    let label = constant_value_label(value);
                    return LinkingError::field_not_found(label, current_search).into();
                }
            }
            ExpressionRef(expression) => {
                return Ok(BrowseResult::OnExpression(Rc::clone(context), Rc::clone(expression), &path[index - 1..]));
            }
            UserFunctionRef(metaphor) => {
                let metaphor_name = metaphor.borrow().function_definition.get_name();
                let object_name = format!("function {}", metaphor_name);
                return LinkingError::field_not_found(&object_name, current_search).into();
            }
            ObjectRef(object) => {
                NodeData::attach_child(context, object);
                let result = object.borrow().get(current_search)?;
                starting = (Rc::clone(object), result)
            }
            Definition(ObjectType(object)) => {
                return Ok(BrowseResult::OnObjectType(Rc::clone(context), Rc::clone(object), &path[index - 1..]));
            }
            Definition(definition) => {
                use crate::typesystem::types::ValueType;
                // Provide synthetic member types for primitive definitions (static typing)
                let next_def = match (&definition, current_search) {
                    // Date -> numeric components
                    (ValueType::DateType, "year")
                    | (ValueType::DateType, "month")
                    | (ValueType::DateType, "day")
                    | (ValueType::DateType, "weekday") => Some(ValueType::NumberType),

                    // Time -> numeric components
                    (ValueType::TimeType, "hour")
                    | (ValueType::TimeType, "minute")
                    | (ValueType::TimeType, "second") => Some(ValueType::NumberType),

                    // DateTime -> numeric components and time
                    (ValueType::DateTimeType, "year")
                    | (ValueType::DateTimeType, "month")
                    | (ValueType::DateTimeType, "day")
                    | (ValueType::DateTimeType, "hour")
                    | (ValueType::DateTimeType, "minute")
                    | (ValueType::DateTimeType, "second")
                    | (ValueType::DateTimeType, "weekday") => Some(ValueType::NumberType),
                    (ValueType::DateTimeType, "time") => Some(ValueType::TimeType),
                    (ValueType::DateTimeType, "date") => Some(ValueType::DateType),

                    // Duration -> numeric components
                    (ValueType::DurationType, "days")
                    | (ValueType::DurationType, "hours")
                    | (ValueType::DurationType, "minutes")
                    | (ValueType::DurationType, "seconds")
                    | (ValueType::DurationType, "totalSeconds")
                    | (ValueType::DurationType, "totalMinutes")
                    | (ValueType::DurationType, "totalHours") => Some(ValueType::NumberType),

                    // Period -> numeric components
                    (ValueType::PeriodType, "years")
                    | (ValueType::PeriodType, "months")
                    | (ValueType::PeriodType, "days")
                    | (ValueType::PeriodType, "totalMonths")
                    | (ValueType::PeriodType, "totalDays") => Some(ValueType::NumberType),

                    _ => None,
                };

                if let Some(def) = next_def {
                    starting = (Rc::clone(context), Definition(def));
                } else {
                    let object_name =
                        current_search_end.map(|name| name.to_string()).unwrap_or_else(|| definition.to_string());
                    return LinkingError::field_not_found(object_name.as_str(), current_search).into();
                }
            }
        }

        current_search_end = Some(current_search);
    }

    // Path iteration is exhausted; this branch should be unreachable because we return earlier
    // once `index >= path.len()`. Keep a defensive fallback to avoid panics.
    Ok(BrowseResult::found(starting.0, "this should not happen", starting.1))
}
