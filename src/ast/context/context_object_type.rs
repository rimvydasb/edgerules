use crate::ast::context::context_object::{ContextObject, ExpressionEntry, MethodEntry};
use crate::ast::context::context_object_type::EObjectContent::{
    ConstantValue, Definition, ExpressionRef, ObjectRef, UserFunctionRef,
};
use crate::ast::expression::StaticLink;
use crate::ast::token::ComplexTypeRef;
use crate::ast::Link;
use crate::link::linker::link_parts;
use crate::link::node_data::Node;
use crate::typesystem::errors::LinkingError;
use crate::typesystem::errors::LinkingErrorEnum::CyclicReference;
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use core::fmt;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct FormalParameter {
    pub name: String,
    pub parameter_type: ComplexTypeRef,
}

impl FormalParameter {
    pub(crate) fn with_type_ref(name: String, parameter_type: ComplexTypeRef) -> FormalParameter {
        FormalParameter {
            name,
            parameter_type,
        }
    }

    pub fn declared_type(&self) -> Option<&ComplexTypeRef> {
        if self.parameter_type.is_undefined() {
            None
        } else {
            Some(&self.parameter_type)
        }
    }

    pub fn with_runtime_type(&self, value_type: ValueType) -> FormalParameter {
        FormalParameter {
            name: self.name.clone(),
            parameter_type: ComplexTypeRef::from_value_type(value_type),
        }
    }

    pub fn runtime_value_type(&self) -> Option<ValueType> {
        match &self.parameter_type {
            ComplexTypeRef::BuiltinType(value_type) => Some(value_type.clone()),
            _ => None,
        }
    }
}

impl Display for FormalParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.parameter_type.is_undefined() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.parameter_type)
        }
    }
}

/// ---
#[derive(Debug, Clone)]
pub enum EObjectContent<T: Node<T>> {
    ConstantValue(ValueEnum),
    ExpressionRef(Rc<RefCell<ExpressionEntry>>),
    UserFunctionRef(Rc<RefCell<MethodEntry>>),
    ObjectRef(Rc<RefCell<T>>),
    Definition(ValueType),
}

impl StaticLink for EObjectContent<ContextObject> {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        match self {
            ConstantValue(value) => Ok(value.get_type()),
            ExpressionRef(value) => match value.try_borrow_mut() {
                Ok(mut entry) => {
                    let field_type = entry.expression.link(Rc::clone(&ctx));
                    if let Ok(field_type_value) = &field_type {
                        entry.field_type = Ok(field_type_value.clone());
                    }
                    field_type
                }
                Err(_) => {
                    let ctx_ref = ctx.borrow();
                    let context_name = ctx_ref.node().node_type.to_string();
                    let field_name = ctx_ref.node().node_type.to_code();
                    let field_label = if field_name.is_empty() {
                        "<self>".to_string()
                    } else {
                        field_name
                    };
                    Err(LinkingError::new(CyclicReference(
                        context_name,
                        field_label,
                    )))
                }
            },
            UserFunctionRef(_metaphor) => {
                todo!("UserFunctionRef")
            }
            ObjectRef(value) => match link_parts(Rc::clone(value)) {
                Ok(_) => Ok(ValueType::ObjectType(Rc::clone(value))),
                Err(err) => Err(err),
            },
            Definition(definition) => Ok(definition.clone()),
        }
    }
}

impl<T: Node<T>> Display for EObjectContent<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConstantValue(value) => write!(f, "{}", value),
            ExpressionRef(value) => write!(f, "{}", value.borrow().expression),
            UserFunctionRef(value) => write!(f, "{}", value.borrow().function_definition),
            ObjectRef(obj) => write!(f, "{}", obj.borrow()),
            Definition(definition) => write!(f, "{}", definition),
        }
    }
}
