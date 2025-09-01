use core::fmt;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use crate::ast::context::context_object::{ContextObject, ExpressionEntry, MethodEntry};
use crate::ast::context::context_object_type::EObjectContent::{ConstantValue, Definition, ExpressionRef, MetaphorRef, ObjectRef};
use crate::ast::expression::StaticLink;
use crate::ast::Link;
use crate::link::linker::link_parts;
use crate::link::node_data::Node;
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;

#[derive(Debug, Clone, PartialEq)]
pub struct FormalParameter {
    pub name: String,
    pub value_type: ValueType,
}

impl FormalParameter {
    pub(crate) fn new(name: String, value_type: ValueType) -> FormalParameter {
        FormalParameter { name, value_type }
    }
}

impl Display for FormalParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.value_type == ValueType::UndefinedType {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}: {}", self.name, self.value_type)
        }
    }
}

/// ---
#[derive(Debug, Clone)]
pub enum EObjectContent<T: Node<T>> {
    ConstantValue(ValueEnum),
    ExpressionRef(Rc<RefCell<ExpressionEntry>>),
    MetaphorRef(Rc<RefCell<MethodEntry>>),
    ObjectRef(Rc<RefCell<T>>),
    Definition(ValueType),
}

impl StaticLink for EObjectContent<ContextObject> {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        match self {
            ConstantValue(value) => Ok(value.get_type()),
            ExpressionRef(value) => {
                let field_type = value.borrow_mut().expression.link(ctx);
                value.borrow_mut().field_type = field_type.clone();
                field_type
            }
            MetaphorRef(_metaphor) => {
                todo!("MetaphorRef")
            }
            ObjectRef(value) => match link_parts(Rc::clone(value)) {
                Ok(_) => Ok(ValueType::ObjectType(Rc::clone(value))),
                Err(err) => Err(err)
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
            MetaphorRef(value) => write!(f, "{}", value.borrow().metaphor),
            ObjectRef(obj) => write!(f, "{}", obj.borrow()),
            Definition(definition) => write!(f, "{}", definition),
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::rc::Rc;
    use log::info;
    use crate::ast::context::context_object_builder::*;
    use crate::ast::metaphors::functions::FunctionDefinition;
    use crate::ast::token::ExpressionEnum;
    use crate::ast::token::DefinitionEnum::MetaphorDefinition;
    use crate::link::linker::link_parts;
    use crate::runtime::edge_rules::{EvalError, expr};

    use crate::utils::test::init_logger;

    type E = ExpressionEnum;

    #[test]
    fn test_builder() -> Result<(), EvalError> {
        init_logger();

        info!(">>> test_builder()");

        let mut builder = ContextObjectBuilder::new();
        builder.add_expression("a", E::from(1.0));
        builder.add_expression("b", E::from(2.0));

        let obj = builder.build();

        link_parts(Rc::clone(&obj))?;

        assert_eq!(obj.borrow().expressions.len(), 2);
        assert_eq!(obj.borrow().metaphors.len(), 0);
        assert_eq!(obj.borrow().all_field_names.len(), 2);
        assert_eq!(obj.borrow().to_string(), "{a : 1; b : 2}");
        assert_eq!(obj.borrow().to_type_string(), "Type<a: number, b: number>");

        let mut builder2 = ContextObjectBuilder::new();
        builder2.add_expression("x", E::from("Hello"));
        builder2.add_expression("y", expr("1 + 2")?);

        let obj2 = builder2.build();

        link_parts(Rc::clone(&obj2))?;

        assert_eq!(obj2.borrow().to_type_string(), "Type<x: string, y: number>");

        let mut builder3 = ContextObjectBuilder::new();
        builder3.add_expression("x", E::from("Hello"));
        builder3.add_expression("y", expr("a + b")?);
        builder3.append(Rc::clone(&obj));

        let obj3 = builder3.build();

        link_parts(Rc::clone(&obj3))?;

        assert_eq!(obj3.borrow().to_type_string(), "Type<a: number, b: number, x: string, y: number>");

        Ok(())
    }

    #[test]
    fn test_nesting() -> Result<(), EvalError> {
        init_logger();

        info!(">>> test_nesting()");

        let mut builder = ContextObjectBuilder::new();
        builder.add_expression("a", E::from(1.0));
        builder.add_expression("b", E::from(2.0));

        {
            let mut child = ContextObjectBuilder::new();
            child.add_expression("x", E::from("Hello"));
            child.add_expression("y", expr("a + b")?);
            child.add_definition(MetaphorDefinition(FunctionDefinition::build(
                vec![],
                "income".to_string(),
                vec![],
                ContextObjectBuilder::new().build()).into()));
            builder.add_expression("c", ExpressionEnum::StaticObject(child.build()));
        }

        let obj = builder.build();

        link_parts(Rc::clone(&obj))?;

        assert_eq!(obj.borrow().to_string(), "{a : 1; b : 2; c : {x : 'Hello'; y : a + b; income() : {}}}");
        assert_eq!(obj.borrow().to_type_string(), "Type<a: number, b: number, c: Type<x: string, y: number>>");

        Ok(())
    }
}
