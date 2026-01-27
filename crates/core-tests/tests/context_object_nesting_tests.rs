use log::info;
use std::rc::Rc;

use edge_rules::ast::context::context_object_builder::ContextObjectBuilder;
use edge_rules::ast::metaphors::functions::FunctionDefinition;
use edge_rules::ast::token::DefinitionEnum;
use edge_rules::ast::token::ExpressionEnum;
use edge_rules::link::linker::{get_till_root, link_parts};
use edge_rules::link::node_data::ContentHolder;
use edge_rules::runtime::edge_rules::EvalError;
use edge_rules::test_support::expr;
use edge_rules::typesystem::types::ToSchema;

mod utilities;
use utilities::init_logger;

type E = ExpressionEnum;

#[test]
fn test_nesting() -> Result<(), EvalError> {
    init_logger();

    info!(">>> test_nesting()");

    let mut builder = ContextObjectBuilder::new();
    builder.add_expression("a", E::from(1.0))?;
    builder.add_expression("b", E::from(2.0))?;

    let child_instance;

    {
        let mut child = ContextObjectBuilder::new();
        child.add_expression("x", E::from("Hello"))?;
        child.add_expression("y", expr("a + b")?)?;
        child.add_definition(DefinitionEnum::UserFunction(FunctionDefinition::build(
            "income".to_string(),
            vec![],
            ContextObjectBuilder::new().build(),
        )?))?;
        let instance = child.build();
        child_instance = Rc::clone(&instance);
        builder.add_expression("c", ExpressionEnum::StaticObject(instance))?;
    }

    let ctx = builder.build();

    link_parts(Rc::clone(&ctx))?;

    assert_eq!(ctx.borrow().to_string(), "{a: 1; b: 2; c: {x: 'Hello'; y: a + b; income() : {}}}");
    assert_eq!(ctx.borrow().to_schema(), "{a: number; b: number; c: {x: string; y: number}}");

    assert_eq!(ctx.borrow().get("a")?.to_string(), "1");
    assert_eq!(ctx.borrow().get("b")?.to_string(), "2");
    assert!(ctx.borrow().get("x").is_err());
    assert_eq!(ctx.borrow().get("c")?.to_string(), "{x: 'Hello'; y: a + b; income() : {}}");

    assert_eq!(get_till_root(Rc::clone(&ctx), "a").unwrap().content.to_string(), "1");
    assert_eq!(get_till_root(Rc::clone(&child_instance), "a").unwrap().content.to_string(), "1");
    assert_eq!(get_till_root(Rc::clone(&child_instance), "x").unwrap().content.to_string(), "'Hello'");

    info!(">>> test_nesting() linking");

    Ok(())
}
