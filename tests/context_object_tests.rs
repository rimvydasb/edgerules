mod utilities;

use std::rc::Rc;

use edge_rules::runtime::edge_rules::DefinitionEnum::Metaphor as MetaphorDef;
use edge_rules::runtime::{
    edge_rules::{
        expr, link_parts, ContextObjectBuilder, EvalError, ExpressionEnum, FunctionDefinition,
    },
    ToSchema,
};
use log::info;
pub use utilities::*;

type E = ExpressionEnum;

#[test]
fn test_builder() -> Result<(), EvalError> {
    init_logger();

    info!(">>> test_builder()");

    let mut builder = ContextObjectBuilder::new();
    builder.add_expression("a", E::from(1.0))?;
    builder.add_expression("b", E::from(2.0))?;

    let obj = builder.build();

    link_parts(Rc::clone(&obj))?;

    assert_eq!(obj.borrow().expressions.len(), 2);
    assert_eq!(obj.borrow().metaphors.len(), 0);
    assert_eq!(obj.borrow().all_field_names.len(), 2);
    assert_eq!(obj.borrow().to_string(), "{a: 1; b: 2}");
    assert_eq!(obj.borrow().to_schema(), "{a: number; b: number}");

    let mut builder2 = ContextObjectBuilder::new();
    builder2.add_expression("x", E::from("Hello"))?;
    builder2.add_expression("y", expr("1 + 2")?)?;

    let obj2 = builder2.build();

    link_parts(Rc::clone(&obj2))?;

    assert_eq!(obj2.borrow().to_schema(), "{x: string; y: number}");

    let mut builder3 = ContextObjectBuilder::new();
    builder3.add_expression("x", E::from("Hello"))?;
    builder3.add_expression("y", expr("a + b")?)?;
    builder3.append(Rc::clone(&obj))?;

    let obj3 = builder3.build();

    link_parts(Rc::clone(&obj3))?;

    assert_eq!(
        obj3.borrow().to_schema(),
        "{x: string; y: number; a: number; b: number}"
    );

    Ok(())
}

#[test]
fn test_nesting() -> Result<(), EvalError> {
    init_logger();

    info!(">>> test_nesting()");

    let mut builder = ContextObjectBuilder::new();
    builder.add_expression("a", E::from(1.0))?;
    builder.add_expression("b", E::from(2.0))?;

    {
        let mut child = ContextObjectBuilder::new();
        child.add_expression("x", E::from("Hello"))?;
        child.add_expression("y", expr("a + b")?)?;
        child.add_definition(MetaphorDef(
            FunctionDefinition::build(
                vec![],
                "income".to_string(),
                vec![],
                ContextObjectBuilder::new().build(),
            )?
            .into(),
        ))?;
        builder.add_expression("c", ExpressionEnum::StaticObject(child.build()))?;
    }

    let obj = builder.build();

    link_parts(Rc::clone(&obj))?;

    assert_eq!(
        obj.borrow().to_string(),
        "{a: 1; b: 2; c: {x: 'Hello'; y: a + b; income() : {}}}"
    );
    assert_eq!(
        obj.borrow().to_schema(),
        "{a: number; b: number; c: {x: string; y: number}}"
    );

    Ok(())
}
