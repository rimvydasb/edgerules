use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::expression::StaticLink;
use crate::ast::token::ExpressionEnum;
use crate::ast::utils::array_to_code_sep;
use crate::ast::{is_linked, Link};
use crate::link::linker;
use crate::link::node_data::NodeDataEnum;
use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum::Array;
use crate::typesystem::values::{ArrayValue, ValueEnum};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct CollectionExpression {
    pub elements: Vec<ExpressionEnum>,

    /// @Todo: do not allow non-homogeneous collections: throw linking error if any following element has different type
    /// @Todo: collection still can contain multiple objects with very different structure - that is fine, however
    /// to support multi-typed objects, need to aggregate all elements to one super-type of ObjectType(Rc<RefCell<ContextObject>>)
    pub list_type: Link<ValueType>,
}

impl StaticLink for CollectionExpression {
    /// Returns ListType or LinkingErrorEnum
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.list_type) {
            if self.elements.is_empty() {
                self.list_type = Ok(ValueType::ListType(None));
            } else {
                let mut aggregated_type: Option<ValueType> = None;
                for arg in self.elements.iter_mut() {
                    if let ExpressionEnum::StaticObject(obj) = arg {
                        {
                            let mut object = obj.borrow_mut();
                            object.node.node_type = NodeDataEnum::Internal(Rc::downgrade(&ctx), None);
                        }
                        linker::link_parts(Rc::clone(obj))?;
                    }
                    let element_type = arg.link(Rc::clone(&ctx))?;
                    aggregated_type = Some(match aggregated_type {
                        None => element_type,
                        Some(existing) => merge_collection_types(existing, element_type)?,
                    });
                }

                let inner = aggregated_type.map(Box::new);
                self.list_type = Ok(ValueType::ListType(inner));
            }
        }

        self.list_type.clone()
    }
}

fn merge_collection_types(existing: ValueType, new_type: ValueType) -> Link<ValueType> {
    use ValueType::*;

    if existing == new_type {
        return Ok(existing);
    }

    match (existing, new_type) {
        (ObjectType(base), ObjectType(extra)) => {
            let mut builder = ContextObjectBuilder::new();
            builder.append(Rc::clone(&base)).map_err(|err| LinkingError::other_error(err.to_string()))?;
            builder
                .append_if_missing(Rc::clone(&extra))
                .map_err(|err| LinkingError::other_error(err.to_string()))?;
            Ok(ObjectType(builder.build()))
        }
        (ListType(Some(base)), ListType(Some(extra))) => {
            let merged_inner = merge_collection_types(*base, *extra)?;
            Ok(ListType(Some(Box::new(merged_inner))))
        }
        (ListType(None), ListType(Some(inner))) | (ListType(Some(inner)), ListType(None)) => Ok(ListType(Some(inner))),
        (ListType(None), ListType(None)) => Ok(ListType(None)),
        (left, right) => {
            LinkingError::other_error(format!("Only homogeneous arrays are supported. Found`{}` and `{}`", left, right))
                .into()
        }
    }
}

impl CollectionExpression {
    pub fn build(elements: Vec<ExpressionEnum>) -> Self {
        CollectionExpression { elements, list_type: LinkingError::not_linked().into() }
    }

    pub fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let list_type = self.list_type.clone()?;

        // We cannot lose the type information even if the array is empty
        if self.elements.is_empty() {
            return Ok(Array(ArrayValue::PrimitivesArray { values: vec![], item_type: list_type }));
        }

        match list_type {
            ValueType::ListType(Some(inner_type)) => match *inner_type {
                ValueType::ObjectType(object_type) => {
                    let mut results: Vec<Rc<RefCell<ExecutionContext>>> = Vec::with_capacity(self.elements.len());

                    for expr in self.elements.iter() {
                        match expr.eval(Rc::clone(&context))? {
                            ValueEnum::Reference(eval_context) => {
                                // To simplify the overall execution, all fields of objects inside array are evaluated immediately
                                // @Todo: consider lazy evaluation if performance becomes an issue (sometimes not all objects are needed from the collection if filter or selection is applied later)
                                ExecutionContext::eval_all_fields(&eval_context)?;
                                results.push(eval_context);
                            }
                            other => {
                                return RuntimeError::type_not_supported(other.get_type()).into();
                            }
                        }
                    }

                    Ok(Array(ArrayValue::ObjectsArray { values: results, object_type }))
                }
                other_type => {
                    let mut results: Vec<ValueEnum> = Vec::with_capacity(self.elements.len());

                    for expr in self.elements.iter() {
                        results.push(expr.eval(Rc::clone(&context))?);
                    }

                    Ok(Array(ArrayValue::PrimitivesArray { values: results, item_type: other_type }))
                }
            },
            ValueType::ListType(None) => {
                let mut results: Vec<ValueEnum> = Vec::with_capacity(self.elements.len());

                for expr in self.elements.iter() {
                    results.push(expr.eval(Rc::clone(&context))?);
                }

                Ok(Array(ArrayValue::PrimitivesArray { values: results, item_type: ValueType::UndefinedType }))
            }

            // This should never happen, but just in case
            other => RuntimeError::type_not_supported(other).into(),
        }
    }
}

impl Display for CollectionExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", array_to_code_sep(self.elements.iter(), ", "))
    }
}
