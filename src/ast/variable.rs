use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use log::{trace};
use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::{is_linked, Link};
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::token::{EToken, ExpressionEnum};
use crate::link::linker::{browse};
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::{ValueType};
use crate::typesystem::values::ValueEnum;

/// *Possible variable usages:*
/// - linking another variable from parameter, for example arg.a
/// - linking variable from a function return value, for example func1().a
/// - linking variable from a field, for example field1.a
/// - linking variable within another expression: x = 1 + b.a
#[derive(Debug, Clone, PartialEq)]
pub struct VariableLink {
    pub path: Vec<String>,
    pub variable_type: Link<ValueType>,
}

impl VariableLink {
    pub fn new_unlinked(path: String) -> Self {
        VariableLink {
            path: vec![path],
            variable_type: LinkingError::not_linked().into(),
        }
    }

    pub fn new_unlinked_path(path: Vec<String>) -> Self {
        VariableLink {
            path,
            variable_type: LinkingError::not_linked().into(),
        }
    }

    pub fn get_name(&self) -> String {
        if self.path.len() == 1 {
            self.path.get(0).unwrap().clone()
        } else {
            self.path.join(".")
        }
    }

    fn path_as_str(&self) -> Vec<&str> {
        self.path.iter().map(|s| s.as_str()).collect()
    }
}

impl Into<EToken> for VariableLink {
    fn into(self) -> EToken {
        EToken::Expression(ExpressionEnum::Variable(self))
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
        // Support self-qualified references like `calendar.shift` inside the
        // `calendar: { ... }` context by stripping the leading self name and
        // browsing from the current context rather than root.
        let (start_ctx, path, find_root) = {
            if let Some(first) = self.path.first() {
                // climb to nearest ancestor named `first`
                let mut cursor = Some(Rc::clone(&context));
                let mut found = None;
                while let Some(ctx) = cursor {
                    trace!("variable.eval climb: at {:?}", ctx.borrow().node.node_type);
                    let assigned = ctx.borrow().node.get_assigned_to_field();
                    trace!("assigned: {:?}", assigned);
                    if assigned.as_deref() == Some(first.as_str()) {
                        found = Some(ctx);
                        break;
                    }
                    cursor = ctx.borrow().node.node_type.get_parent();
                }

                if let Some(ctx) = found {
                    let remaining: Vec<&str> = self.path.iter().skip(1).map(|s| s.as_str()).collect();
                    (ctx, remaining, false)
                } else {
                    (Rc::clone(&context), self.path_as_str(), true)
                }
            } else {
                (Rc::clone(&context), self.path_as_str(), true)
            }
        };

        let result = browse(start_ctx, path, find_root)?
            .on_incomplete(
                |ctx, result, _remaining| {
                    match result.borrow().expression.eval(Rc::clone(&ctx)) {
                        Ok(intermediate) => Ok(intermediate.into()),
                        Err(err) => Err(LinkingError::other_error(err.to_string()))
                    }
                }, |ctx, _result, _remaining| {
                    Err(LinkingError::other_error(format!("Variable {:?} may be found in context {:?}, but it is not evaluated", self.path, ctx.borrow().node.get_assigned_to_field())))
                })?;

        result.eval()
    }
}

impl StaticLink for VariableLink {
    fn link(&mut self, context: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        trace!("linking variable {:?} in {} with {:?}", self.path, context.borrow().node.node_type, context.borrow().all_field_names.clone());
        if !is_linked(&self.variable_type) {
            // Same self-qualification handling as in eval: treat `contextName.*`
            // inside that context as local browse, not root lookup.
            let (start_ctx, path, find_root) = {
                if let Some(first) = self.path.first() {
                    let mut cursor = Some(Rc::clone(&context));
                    let mut found = None;
                    while let Some(ctx) = cursor {
                        let assigned = ctx.borrow().node.get_assigned_to_field();
                        if assigned.as_deref() == Some(first.as_str()) {
                            found = Some(ctx);
                            break;
                        }
                        cursor = ctx.borrow().node.node_type.get_parent();
                    }

                    if let Some(ctx) = found {
                        let remaining: Vec<&str> = self.path.iter().skip(1).map(|s| s.as_str()).collect();
                        (ctx, remaining, false)
                    } else {
                        (Rc::clone(&context), self.path_as_str(), true)
                    }
                } else {
                    (Rc::clone(&context), self.path_as_str(), true)
                }
            };

            let result = browse(start_ctx, path, find_root)
                .and_then(|r| r.on_incomplete(|ctx, result, _remaining| {
                    let linked_type = result.borrow_mut().expression.link(Rc::clone(&ctx))?;
                    Ok(EObjectContent::Definition(linked_type))
                }, |_ctx, result, _remaining| {
                    Ok(EObjectContent::ObjectRef(result))
                }));

            match result {
                Ok(mut value_type) => {
                    self.variable_type = value_type.content.link(value_type.context);
                }
                Err(error) => {
                    // Defer linking for qualified paths inside unattached inline objects
                    let is_unattached_root = matches!(context.borrow().node.node_type,
                        crate::link::node_data::NodeDataEnum::Root() | crate::link::node_data::NodeDataEnum::Isolated());
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
