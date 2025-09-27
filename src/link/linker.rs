use crate::ast::context::context_object::{ContextObject, ExpressionEntry, MethodEntry};
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::context::context_object_type::EObjectContent::*;
use crate::ast::expression::StaticLink;
use crate::ast::metaphors::metaphor::Metaphor;
use crate::ast::Link;
use crate::link::node_data::{ContentHolder, Node, NodeData};
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::LinkingErrorEnum::*;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::ValueType::ObjectType;
use crate::typesystem::values::ValueEnum;
use crate::utils::intern_field_name;
use log::{error, trace};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

// ---
// **Linker**
// - is responsible for linking all the expressions in the AST.

// @Todo: is it possible or necessary to prevent re-execution of the same code?
pub fn link_parts(context: Rc<RefCell<ContextObject>>) -> Link<()> {
    trace!("link_parts: {}(..)", context.borrow().node().node_type);

    let field_names = context.borrow().get_field_names();
    let mut references = Vec::new();

    for name in field_names {
        match context.borrow().get(name)? {
            EObjectContent::ExpressionRef(expression) => {
                context.borrow().node().lock_field(name)?;

                let linked_type = {
                    match expression.try_borrow_mut() {
                        Ok(mut entry) => {
                            let result = entry.expression.link(Rc::clone(&context));
                            match result {
                                Ok(field_type) => {
                                    entry.field_type = Ok(field_type.clone());
                                    Ok(field_type)
                                }
                                Err(err) => Err(err),
                            }
                        }
                        Err(_) => {
                            let context_name = context.borrow().node().node_type.to_string();
                            Err(LinkingError::new(CyclicReference(
                                context_name,
                                name.to_string(),
                            )))
                        }
                    }
                };

                context.borrow().node().unlock_field(name);

                if let Ok(field_type) = &linked_type {
                    let context_name = context.borrow().node().node_type.to_string();
                    trace!(
                        "expression: {}.{} -> {}",
                        context_name,
                        name,
                        field_type
                    );
                }

                linked_type?;
            }
            EObjectContent::ObjectRef(reference) => {
                references.push((name, reference));
            }
            EObjectContent::Definition(ObjectType(reference)) => {
                references.push((name, reference));
            }
            _ => {
                // Metaphors will be linked when the call will be detected
                // Primitive definitions will not be linked at all
            }
        }
    }

    for (_name, reference) in references {
        link_parts(Rc::clone(&reference))?;
    }

    Ok(())
}

pub fn find_implementation(
    context: Rc<RefCell<ContextObject>>,
    function_name: String,
) -> Link<Rc<RefCell<MethodEntry>>> {
    let mut ctx: Rc<RefCell<ContextObject>> = context;

    loop {
        trace!(
            "find_implementation: searching {} in {} ",
            function_name,
            ctx.borrow().node().node_type
        );

        let implementation = (*ctx).borrow().get_function(function_name.as_str());

        if let Some(definition) = implementation {
            trace!(
                "find_implementation: found {} in {} ",
                function_name,
                ctx.borrow().node().node_type
            );
            return Ok(Rc::clone(&definition));
        } else {
            let maybe_parent = (*ctx).borrow().node().node_type.get_parent();

            if let Some(parent_to_check) = maybe_parent {
                ctx = parent_to_check;
            } else {
                error!(
                    "find_implementation: Cannot find {} in {} ",
                    function_name,
                    ctx.borrow().node().node_type
                );
                return LinkingError::new(FunctionNotFound(format!("{}(...)", function_name)))
                    .into();
            }
        }
    }
}

pub fn get_till_root<T: Node<T>>(
    ctx: Rc<RefCell<T>>,
    name: &str,
) -> Result<BrowseResultFound<T>, LinkingError> {
    let interned = intern_field_name(name);
    ctx.borrow().node().lock_field(interned)?;
    let result;
    match ctx.borrow().get(name) {
        Ok(finding) => {
            result = Ok(BrowseResultFound::new(Rc::clone(&ctx), interned, finding));
        }
        Err(LinkingError {
            error: FieldNotFound(obj_name, field),
            ..
        }) => match ctx.borrow().node().node_type.get_parent() {
            None => {
                error!("get_till_root: Cannot find {} in {} and object upgrade is not possible for `{:?}`", field, obj_name, ctx.borrow().node().node_type);
                result = Err(LinkingError::new(FieldNotFound(obj_name, field)));
            }
            Some(parent) => {
                result = get_till_root(parent, name);
            }
        },
        Err(error) => {
            result = Err(error);
        }
    }
    ctx.borrow().node().unlock_field(interned);
    result
}

#[derive(Debug, Clone)]
pub struct BrowseResultFound<T: Node<T>> {
    pub context: Rc<RefCell<T>>,
    pub field_name: &'static str,
    pub content: EObjectContent<T>,
}

impl<T: Node<T>> BrowseResultFound<T> {
    pub fn new(
        context: Rc<RefCell<T>>,
        field_name: &'static str,
        content: EObjectContent<T>,
    ) -> BrowseResultFound<T> {
        BrowseResultFound {
            context,
            field_name,
            content,
        }
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
                let result = value.borrow_mut().expression.eval(Rc::clone(&self.context));
                // no need to check if in stack, if it was already acquired as expression, it is not in stack
                self.context
                    .borrow()
                    .stack_insert(self.field_name, result.clone());
                result
            }
            MetaphorRef(_value) => {
                todo!("MetaphorRef")
            }
            ObjectRef(value) => {
                NodeData::attach_child(&self.context, value);
                Ok(ValueEnum::Reference(Rc::clone(value)))
            }
            Definition(definition) => Err(RuntimeError::eval_error(format!(
                "Cannot evaluate definition: {}",
                definition
            ))),
        }
    }
}

impl<T: Node<T>> Display for BrowseResultFound<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{} = {}",
            self.context.borrow().node().node_type,
            self.field_name,
            self.content
        )
    }
}

#[derive(Debug, Clone)]
pub enum BrowseResult<'a, T: Node<T>> {
    Found(BrowseResultFound<T>),

    // context, expression, remaining path
    OnExpression(Rc<RefCell<T>>, Rc<RefCell<ExpressionEntry>>, Vec<&'a str>),

    OnObjectType(Rc<RefCell<T>>, Rc<RefCell<ContextObject>>, Vec<&'a str>),
}

impl<'a, T: Node<T>> BrowseResult<'a, T> {
    pub fn found(
        context: Rc<RefCell<T>>,
        field_name: &'a str,
        content: EObjectContent<T>,
    ) -> BrowseResult<'a, T> {
        let interned = intern_field_name(field_name);
        BrowseResult::Found(BrowseResultFound::new(context, interned, content))
    }

    pub fn on_incomplete<FE, FC>(
        self,
        mut on_expression: FE,
        mut on_object_type: FC,
    ) -> Result<BrowseResultFound<T>, LinkingError>
    where
        FE: FnMut(
            Rc<RefCell<T>>,
            Rc<RefCell<ExpressionEntry>>,
            &[&'a str],
        ) -> Result<EObjectContent<T>, LinkingError>,
        FC: FnMut(
            Rc<RefCell<T>>,
            Rc<RefCell<ContextObject>>,
            &[&'a str],
        ) -> Result<EObjectContent<T>, LinkingError>,
    {
        trace!("on_incomplete: {:?}", self);
        match self {
            BrowseResult::Found(result) => Ok(result),
            BrowseResult::OnExpression(ctx, content, path) => {
                let remaining = path.as_slice();
                let result = on_expression(Rc::clone(&ctx), content, remaining)?;
                let continue_result = continue_browse(remaining, 0, (Rc::clone(&ctx), result))?;

                continue_result.on_incomplete(on_expression, on_object_type)
            }
            BrowseResult::OnObjectType(ctx, content, path) => {
                let remaining = path.as_slice();
                let result = on_object_type(Rc::clone(&ctx), content, remaining)?;
                let continue_result = continue_browse(remaining, 0, (Rc::clone(&ctx), result))?;

                continue_result.on_incomplete(on_expression, on_object_type)
            }
        }
    }
}

pub fn browse<'a, T: Node<T>>(
    ctx: Rc<RefCell<T>>,
    path: &[&'a str],
    find_root: bool,
) -> Result<BrowseResult<'a, T>, LinkingError> {
    // Path is empty - this is abnormal and should never happen
    if path.is_empty() {
        return Err(LinkingError::field_not_found(
            ctx.borrow().node().node_type.to_string().as_str(),
            "",
        ));
    }

    // Path is 1
    if path.len() == 1 {
        let field_name = path.first().unwrap();
        return if find_root {
            let result = get_till_root(ctx, field_name)?;
            Ok(BrowseResult::Found(result))
        } else {
            Ok(BrowseResult::found(
                Rc::clone(&ctx),
                field_name,
                ctx.borrow().get(field_name)?,
            ))
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
    path: &[&'a str],
    mut index: usize,
    mut starting: (Rc<RefCell<T>>, EObjectContent<T>),
) -> Result<BrowseResult<'a, T>, LinkingError> {
    trace!("continue_browse(path[{}..], {:?})", index, starting);
    let mut current_search_end: Option<&str> = None;

    #[allow(irrefutable_let_patterns)]
    while let (ref context, ref item) = starting {
        if index >= path.len() {
            return if let Some(current_search) = current_search_end {
                Ok(BrowseResult::found(
                    Rc::clone(context),
                    current_search,
                    item.clone(),
                ))
            } else {
                LinkingError::other_error(format!("Stuck on {}", context.borrow().node().node_type))
                    .into()
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
                        _ => None,
                    },
                    _ => None,
                };

                if let Some(next_value) = maybe_next {
                    // Step into the computed property value and continue browsing
                    starting = (Rc::clone(context), ConstantValue(next_value));
                } else {
                    error!(
                        "Constant value '{:?}' does not have '{}' item",
                        value, current_search
                    );
                    return LinkingError::new(OtherLinkingError(format!(
                        "Value '{}' does not have '{}' item",
                        value, current_search
                    )))
                    .into();
                }
            }
            ExpressionRef(expression) => {
                return Ok(BrowseResult::OnExpression(
                    Rc::clone(context),
                    Rc::clone(expression),
                    path[index - 1..].to_vec(),
                ));
            }
            MetaphorRef(metaphor) => {
                error!(
                    "Metaphor '{:?}' does not have '{}' item",
                    metaphor, current_search
                );
                return LinkingError::new(OtherLinkingError(format!(
                    "Cannot access '{}' from '{}' metaphor",
                    current_search,
                    metaphor.borrow().metaphor.get_name()
                )))
                .into();
            }
            ObjectRef(object) => {
                NodeData::attach_child(context, object);
                let result = object.borrow().get(current_search)?;
                starting = (Rc::clone(object), result)
            }
            Definition(ObjectType(object)) => {
                return Ok(BrowseResult::OnObjectType(
                    Rc::clone(context),
                    Rc::clone(object),
                    path[index - 1..].to_vec(),
                ));
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

                    _ => None,
                };

                if let Some(def) = next_def {
                    starting = (Rc::clone(context), Definition(def));
                } else {
                    error!(
                        "Definition '{:?}' does not have '{}' item",
                        definition, current_search
                    );
                    return LinkingError::new(OtherLinkingError(format!(
                        "Cannot access '{}' from '{}' definition",
                        current_search, definition
                    )))
                    .into();
                }
            }
        }

        current_search_end = Some(current_search);
    }

    Ok(BrowseResult::found(
        starting.0,
        "this should not happen",
        starting.1,
    ))
}
