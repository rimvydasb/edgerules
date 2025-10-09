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
use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::Array;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug)]
pub struct CollectionExpression {
    pub elements: Vec<ExpressionEnum>,

    /// @Todo: do not allow non-homogeneous collections: throw linking error if any following element has different type
    /// @Todo: collection still can contain multiple objects with very different structure - that is fine, however
    /// to support multi-typed objects, need to aggregate all elements to one super-type of ObjectType(Rc<RefCell<ContextObject>>)
    pub collection_item_type: Link<ValueType>,
}

impl StaticLink for CollectionExpression {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.collection_item_type) {
            let mut aggregated_type: Option<ValueType> = None;
            for arg in self.elements.iter_mut() {
                if let ExpressionEnum::StaticObject(obj) = arg {
                    {
                        let mut object = obj.borrow_mut();
                        object.node.node_type = NodeDataEnum::Internal(Rc::downgrade(&ctx));
                    }
                    linker::link_parts(Rc::clone(obj))?;
                }
                let element_type = arg.link(Rc::clone(&ctx))?;
                aggregated_type = Some(match aggregated_type {
                    None => element_type,
                    Some(existing) => merge_collection_types(existing, element_type)?,
                });
            }
            self.collection_item_type = Ok(aggregated_type.unwrap_or(ValueType::UndefinedType));
        }

        // @Todo: different type must be assigned if collection is multityped
        //args.iter().any(|arg| arg.get_type() != type_of_sequence)

        self.collection_item_type.clone()
    }
}

fn merge_collection_types(existing: ValueType, new_type: ValueType) -> Link<ValueType> {
    use ValueType::*;

    match (&existing, &new_type) {
        (UndefinedType, _) => return Ok(new_type),
        (_, UndefinedType) => return Ok(existing),
        _ => {}
    }

    if existing == new_type {
        return Ok(existing);
    }

    match (existing, new_type) {
        (ObjectType(base), ObjectType(extra)) => {
            let mut builder = ContextObjectBuilder::new();
            builder
                .append(Rc::clone(&base))
                .map_err(|err| LinkingError::other_error(err.to_string()))?;
            builder
                .append_if_missing(Rc::clone(&extra))
                .map_err(|err| LinkingError::other_error(err.to_string()))?;
            Ok(ObjectType(builder.build()))
        }
        (ListType(base), ListType(extra)) => {
            let merged_inner = merge_collection_types(*base, *extra)?;
            Ok(ListType(Box::new(merged_inner)))
        }
        (left, right) => LinkingError::other_error(format!(
            "Only homogeneous arrays are supported. Found`{}` and `{}`",
            left, right
        ))
        .into(),
    }
}

impl CollectionExpression {
    pub fn build(elements: Vec<ExpressionEnum>) -> Self {
        CollectionExpression {
            elements,
            collection_item_type: LinkingError::not_linked().into(),
        }
    }

    pub fn eval(&self, context: Rc<RefCell<ExecutionContext>>) -> Result<ValueEnum, RuntimeError> {
        let results = self
            .elements
            .iter()
            .map(|expr| expr.eval(Rc::clone(&context)))
            .collect();
        Ok(Array(results, self.collection_item_type.clone()?))
    }
}

impl Display for CollectionExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", array_to_code_sep(self.elements.iter(), ", "))
    }
}
