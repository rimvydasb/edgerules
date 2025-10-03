use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::StaticLink;
use crate::ast::token::ExpressionEnum;
use crate::ast::utils::array_to_code_sep;
use crate::ast::{is_linked, Link};
use crate::link::linker;
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
    pub collection_item_type: Link<ValueType>,
}

impl StaticLink for CollectionExpression {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        if !is_linked(&self.collection_item_type) {
            let mut first_type = ValueType::UndefinedType;
            for arg in self.elements.iter_mut() {
                if let ExpressionEnum::StaticObject(obj) = arg {
                    linker::link_parts(Rc::clone(obj))?;
                }
                first_type = arg.link(Rc::clone(&ctx))?;
            }
            self.collection_item_type = Ok(first_type);
        }

        // @Todo: different type must be assigned if collection is multityped
        //args.iter().any(|arg| arg.get_type() != type_of_sequence)

        self.collection_item_type.clone()
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
