use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::expression::{missing_for_type, EvaluatableExpression, StaticLink};
use crate::ast::token::{EToken, ExpressionEnum};
use crate::ast::{is_linked, Link};
use crate::link::linker::{browse, build_location_from_context};
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::{
    ErrorStage, LinkingError, LinkingErrorEnum, RuntimeError, RuntimeErrorEnum,
};
use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum;
use crate::utils::intern_field_name;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// *Possible variable usages:*
/// - linking another variable from parameter, for example arg.a
/// - linking variable from a function return value, for example func1().a
/// - linking variable from a field, for example field1.a
/// - linking variable within another expression: x = 1 + b.a
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct VariableLink {
    pub path: Vec<&'static str>,
    pub variable_type: Link<ValueType>,
}

impl VariableLink {
    pub fn new_unlinked(name: String) -> Self {
        let interned = intern_field_name(name.as_str());
        VariableLink {
            path: vec![interned],
            variable_type: LinkingError::not_linked().into(),
        }
    }

    pub fn new_unlinked_path(path: Vec<String>) -> Self {
        let interned_path = path
            .into_iter()
            .map(|segment| intern_field_name(segment.as_str()))
            .collect();
        VariableLink {
            path: interned_path,
            variable_type: LinkingError::not_linked().into(),
        }
    }

    pub fn new_interned_path(path: Vec<&'static str>) -> Self {
        VariableLink {
            path,
            variable_type: LinkingError::not_linked().into(),
        }
    }

    pub fn get_name(&self) -> String {
        if self.path.len() == 1 {
            self.path.first().copied().unwrap().to_string()
        } else {
            self.path.join(".")
        }
    }
}

impl From<VariableLink> for EToken {
    fn from(val: VariableLink) -> Self {
        EToken::Expression(ExpressionEnum::Variable(val))
    }
}

impl Display for VariableLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.path.join("."), f)
    }
}

/// Previously, implemented with: Finder::find_field(context, self.path.get(0).unwrap().as_str())
/// Now only find_path is used, because it solves parent problem
/// - @Todo: static linking must already be done in link part
impl EvaluatableExpression for VariableLink {
    fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        trace!(">>> evaluating variable {:?}", self.path);
        // Alias: `it` inside filter contexts refers to the current context variable
        if self.path.len() == 1 && self.path[0] == "it" {
            return context.borrow().get_context_variable();
        }
        // Support self-qualified references like `calendar.shift` inside the
        // `calendar: { ... }` context by stripping the leading self name and
        // browsing from the current context rather than root.
        let (start_ctx, path_vec, find_root) = {
            if let Some(&first) = self.path.first() {
                let has_parameter = {
                    let exec_borrowed = context.borrow();
                    let ctx_borrowed = exec_borrowed.object.borrow();
                    ctx_borrowed
                        .parameters
                        .iter()
                        .any(|param| intern_field_name(param.name.as_str()) == first)
                };

                if has_parameter {
                    (Rc::clone(&context), &self.path[..], false)
                } else {
                    // climb to nearest ancestor named `first`
                    let mut cursor = Some(Rc::clone(&context));
                    let mut found = None;
                    while let Some(ctx) = cursor {
                        //trace!("variable.eval climb: at {:?}", ctx.borrow().node.node_type);
                        let assigned = ctx.borrow().node.get_assigned_to_field();
                        //trace!("assigned: {:?}", assigned);
                        if assigned == Some(first) {
                            found = Some(ctx);
                            break;
                        }
                        cursor = ctx.borrow().node.node_type.get_parent();
                    }

                    if let Some(ctx) = found {
                        (ctx, &self.path[1..], false)
                    } else {
                        (Rc::clone(&context), &self.path[..], true)
                    }
                }
            } else {
                (Rc::clone(&context), &self.path[..], true)
            }
        };

        let browse_result = match browse(start_ctx, path_vec, find_root) {
            Ok(res) => res,
            Err(link_err) => {
                if let LinkingErrorEnum::FieldNotFound(_, field) = &link_err.error {
                    let expected_type = self
                        .variable_type
                        .clone()
                        .unwrap_or_else(|_| ValueType::UndefinedType);
                    let missing = missing_for_type(&expected_type, Some(field.as_str()), &context)?;
                    return Ok(missing);
                } else {
                    return Err(link_err.into());
                }
            }
        };

        let result = browse_result.on_incomplete(
            |ctx, result, _remaining| match result.borrow().expression.eval(Rc::clone(&ctx)) {
                Ok(intermediate) => Ok(intermediate.into()),
                Err(err) => {
                    if let RuntimeErrorEnum::RuntimeFieldNotFound(_, field) = &err.error {
                        let expected_type = match self.variable_type.clone() {
                            Ok(value_type) => value_type,
                            Err(_) => ValueType::UndefinedType,
                        };
                        let missing = missing_for_type(&expected_type, Some(field.as_str()), &ctx)
                            .map_err(|runtime_err| {
                                LinkingError::other_error(runtime_err.to_string())
                            })?;
                        Ok(EObjectContent::ConstantValue(missing))
                    } else {
                        Err(LinkingError::other_error(err.to_string()))
                    }
                }
            },
            |ctx, _result, _remaining| {
                Err(LinkingError::other_error(format!(
                    "Variable {:?} may be found in context {:?}, but it is not evaluated",
                    self.path,
                    ctx.borrow().node.get_assigned_to_field()
                )))
            },
        )?;

        result.eval()
    }
}

impl StaticLink for VariableLink {
    fn link(&mut self, context: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        trace!(
            "linking variable {:?} in {} with {:?}",
            self.path,
            context.borrow().node.node_type,
            context.borrow().all_field_names.clone()
        );
        if !is_linked(&self.variable_type) {
            // Alias: `it` resolves to the current context variable type if set by the caller
            if self.path.len() == 1 && self.path[0] == "it" {
                if let Some(context_type) = &context.borrow().context_type {
                    return Ok(context_type.clone());
                } else {
                    return LinkingError::not_linked().into();
                }
            }
            // Same self-qualification handling as in eval: treat `contextName.*`
            // inside that context as local browse, not root lookup.
            let (start_ctx, path_vec, find_root) = {
                if let Some(&first) = self.path.first() {
                    let has_parameter = {
                        let borrowed = context.borrow();
                        borrowed
                            .parameters
                            .iter()
                            .any(|param| intern_field_name(param.name.as_str()) == first)
                    };

                    if has_parameter {
                        (Rc::clone(&context), &self.path[..], false)
                    } else {
                        let mut cursor = Some(Rc::clone(&context));
                        let mut found = None;
                        while let Some(ctx) = cursor {
                            let assigned = ctx.borrow().node.get_assigned_to_field();
                            if assigned == Some(first) {
                                found = Some(ctx);
                                break;
                            }
                            cursor = ctx.borrow().node.node_type.get_parent();
                        }

                        if let Some(ctx) = found {
                            (ctx, &self.path[1..], false)
                        } else {
                            (Rc::clone(&context), &self.path[..], true)
                        }
                    }
                } else {
                    (Rc::clone(&context), &self.path[..], true)
                }
            };

        let result = browse(start_ctx, path_vec, find_root).and_then(|r| {
            r.on_incomplete(
                |ctx, result, remaining| {
                    let expression_display = result.borrow().expression.to_string();
                    let field_name = remaining.first().copied();
                    match result.borrow_mut().expression.link(Rc::clone(&ctx)) {
                        Ok(linked_type) => Ok(EObjectContent::Definition(linked_type)),
                        Err(mut err) => {
                            if err.location.is_empty() {
                                if let Some(name) = field_name {
                                    err.location =
                                        build_location_from_context(&ctx, name);
                                }
                            }
                            if err.expression.is_none() {
                                err.expression = Some(expression_display);
                            }
                            if err.stage.is_none() {
                                err.stage = Some(ErrorStage::Linking);
                            }
                            Err(err)
                        }
                    }
                },
                |_ctx, result, _remaining| Ok(EObjectContent::ObjectRef(result)),
            )
        });

        match result {
            Ok(mut value_type) => {
                let linked = value_type.content.link(Rc::clone(&value_type.context));
                match linked {
                    Ok(resolved) => {
                        self.variable_type = Ok(resolved);
                    }
                    Err(mut err) => {
                        if err.location.is_empty() {
                            err.location = build_location_from_context(
                                &value_type.context,
                                value_type.field_name,
                            );
                        }
                        if err.expression.is_none() {
                            err.expression = Some(value_type.content.to_string());
                        }
                        if err.stage.is_none() {
                            err.stage = Some(ErrorStage::Linking);
                        }
                        return Err(err).into();
                    }
                }
            }
            Err(error) => {
                // Defer linking for qualified paths inside unattached inline objects
                let is_unattached_root = matches!(
                    context.borrow().node.node_type,
                    crate::link::node_data::NodeDataEnum::Root()
                        | crate::link::node_data::NodeDataEnum::Isolated()
                );

                // @Todo: must return LinkingErrorEnum::FieldNotFound or the other even better linking error
                if self.path.len() > 1 && is_unattached_root {
                    self.variable_type = LinkingError::not_linked().into();
                    return Ok(ValueType::UndefinedType);
                }
                return error.into();
            }
        }
        }

        self.variable_type.clone()
    }
}
