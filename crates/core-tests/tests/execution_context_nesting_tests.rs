use edge_rules::ast::context::context_object_builder::ContextObjectBuilder;
use edge_rules::ast::metaphors::functions::FunctionDefinition;
use edge_rules::ast::token::DefinitionEnum::UserFunction as UserFunctionDef;
use edge_rules::ast::token::ExpressionEnum;
use log::info;
use std::rc::Rc;

use edge_rules::link::linker::link_parts;
use edge_rules::link::node_data::ContentHolder;
use edge_rules::runtime::edge_rules::EvalError;
use edge_rules::runtime::execution_context::ExecutionContext;
use edge_rules::test_support::expr;
use edge_rules::typesystem::types::TypedValue;
use edge_rules::typesystem::values::ValueEnum;
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

    {
        let mut child = ContextObjectBuilder::new();
        child.add_expression("x", E::from("Hello"))?;
        child.add_expression("y", expr("a + b")?)?;
        child.add_definition(UserFunctionDef(FunctionDefinition::build(
            "income".to_string(),
            vec![],
            ContextObjectBuilder::new().build(),
        )?))?;
        builder.add_expression("c", ExpressionEnum::StaticObject(child.build()))?;
    }

    let ctx = builder.build();

    link_parts(Rc::clone(&ctx))?;

    let ex = ExecutionContext::create_root_context(ctx);

    ex.borrow().stack_insert("a", Ok(ValueEnum::from(88.0)));
    ex.borrow().stack_insert("b", Ok(ValueEnum::from(99.0)));

    assert_eq!(ex.borrow().get("a")?.to_string(), "88");
    assert_eq!(ex.borrow().get("b")?.to_string(), "99");
    assert!(ex.borrow().get("x").is_err());
    assert_eq!(
        ex.borrow().to_string(),
        "{a: 88; b: 99; c: {x: 'Hello'; y: a + b; income() : {}}}"
    );
    assert_eq!(
        ex.borrow().get_type().to_string(),
        "{a: number; b: number; c: {x: string; y: number}}"
    );
    assert_eq!(
        ex.borrow().get("c")?.to_string(),
        "{x: 'Hello'; y: a + b; income() : {}}"
    );

    // @Todo: update tests
    // {
    //     let result = linker::find_variable(Rc::clone(&ex), "a")?;
    //     assert_eq!(result.to_string(), "88");
    //     assert_eq!(result.get_type().to_string(), "number");
    // }
    //
    // {
    //     let result = linker::find_path(Rc::clone(&ex), vec!["c","x"])?;
    //     assert_eq!(result.to_string(), "'Hello'");
    //     assert_eq!(result.get_type().to_string(), "string");
    // }
    //
    // {
    //     let result = linker::find_path(Rc::clone(&ex), vec!["c","y"])?;
    //     assert_eq!(result.to_string(), "(a + b)");
    //     assert_eq!(result.get_type().to_string(), "number");
    // }

    Ok(())
}
