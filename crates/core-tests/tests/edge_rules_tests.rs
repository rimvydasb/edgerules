use edge_rules::runtime::edge_rules::{
    ContextObjectBuilder, ContextQueryErrorEnum, EdgeRulesModel, EdgeRulesRuntime, EvalError,
    ExpressionEnum, ParseErrors,
};
use edge_rules::runtime::ToSchema;
use edge_rules::test_support::NumberEnum::Int;
use edge_rules::test_support::ParseErrorEnum::{Stacked, UnexpectedToken, WrongFormat};
use edge_rules::test_support::SpecialValueEnum::Missing;
use edge_rules::test_support::{
    expr, ComplexTypeRef, EToken, EUnparsedToken, FunctionDefinition, LinkingError,
    LinkingErrorEnum, NumberEnum, ParseErrorEnum, StaticLink, UserTypeBody, ValueEnum, ValueType,
};
use edge_rules::utils::to_display;
use log::error;
use std::fmt::Display;
use std::mem::discriminant;
use std::rc::Rc;

mod test_utils;
use test_utils::test::init_logger;

pub fn test_code(code: &str) -> TestServiceBuilder {
    TestServiceBuilder::build(code)
}

pub fn test_code_lines<T: Display>(code: &[T]) -> TestServiceBuilder {
    TestServiceBuilder::build(format!("{{{}}}", to_display(code, "\n")).as_str())
}

pub fn inline<S: AsRef<str>>(code: S) -> String {
    code.as_ref().replace('\n', " ").replace(" ", "")
}

pub struct TestServiceBuilder {
    original_code: String,
    runtime: Option<EdgeRulesRuntime>,
    parse_errors: Option<ParseErrors>,
    linking_errors: Option<LinkingError>,
}

impl TestServiceBuilder {
    pub fn build(code: &str) -> Self {
        let mut service = EdgeRulesModel::new();

        match service.append_source(code) {
            Ok(_model) => match service.to_runtime() {
                Ok(runtime) => TestServiceBuilder {
                    original_code: code.to_string(),
                    runtime: Some(runtime),
                    parse_errors: None,
                    linking_errors: None,
                },
                Err(linking_errors) => TestServiceBuilder {
                    original_code: code.to_string(),
                    runtime: None,
                    parse_errors: None,
                    linking_errors: Some(linking_errors),
                },
            },
            Err(errors) => TestServiceBuilder {
                original_code: code.to_string(),
                runtime: None,
                parse_errors: Some(errors),
                linking_errors: None,
            },
        }
    }

    pub fn expect_type(&self, expected_type: &str) -> &Self {
        self.expect_no_errors();

        match &self.runtime {
            None => {
                panic!(
                    "Expected runtime, but got nothing: `{}`",
                    self.original_code
                );
            }
            Some(runtime) => {
                assert_eq!(runtime.static_tree.borrow().to_schema(), expected_type);
            }
        }

        self
    }

    pub fn expect_num(&self, variable: &str, expected: NumberEnum) {
        self.expect(
            &mut ExpressionEnum::variable(variable),
            ValueEnum::NumberValue(expected),
        )
    }

    pub fn expect_parse_error(&self, expected: ParseErrorEnum) -> &Self {
        if let Some(errors) = &self.parse_errors {
            fn matches_error(found: &ParseErrorEnum, expected: &ParseErrorEnum) -> bool {
                if discriminant(found) == discriminant(expected) {
                    return true;
                }

                if let Stacked(inner) = found {
                    return inner.iter().any(|err| matches_error(err, expected));
                }

                false
            }

            if errors
                .errors()
                .iter()
                .any(|err| matches_error(err, &expected))
            {
                return self;
            }

            panic!(
                "Expected parse error `{}`, but got: `{:?}`",
                expected, errors
            );
        } else {
            panic!(
                "Expected parse error, but got no errors: `{}`",
                self.original_code
            );
        }
    }

    pub fn expect_no_errors(&self) -> &Self {
        if let Some(errors) = &self.parse_errors {
            panic!(
                "Expected no errors, but got parse errors: `{}`\nFailed to parse:\n{}",
                errors, self.original_code
            );
        }

        if let Some(errors) = &self.linking_errors {
            panic!(
                "Expected no errors, but got linking errors: `{}`\nFailed to parse:\n{}",
                errors, self.original_code
            );
        }

        self
    }

    pub fn expect_link_error(&self, expected: LinkingErrorEnum) -> &Self {
        if let Some(errors) = &self.parse_errors {
            panic!(
                "Expected linking error, but got parse errors: `{:?}`\nFailed to parse:\n{}",
                errors, self.original_code
            );
        }

        if let Some(errors) = &self.linking_errors {
            assert_eq!(expected, errors.error, "Testing:\n{}", self.original_code);
        } else {
            panic!(
                "Expected linking error, but got no errors: `{}`",
                self.original_code
            );
        }

        self
    }

    pub fn expect(&self, _expr: &mut ExpressionEnum, _expected: ValueEnum) {
        self.expect_no_errors();

        if let Err(error) = _expr.link(self.runtime.as_ref().unwrap().static_tree.clone()) {
            panic!(
                "Expected value, but got linking errors: `{:?}`\nFailed to parse:\n{}",
                error, _expr
            );
        }

        match _expr.eval(self.runtime.as_ref().unwrap().context.clone()) {
            Ok(value) => {
                assert_eq!(
                    value,
                    _expected,
                    "Context:\n{}",
                    self.runtime.as_ref().unwrap().context.borrow()
                );
            }
            Err(error) => {
                error!("{}", error);
                panic!("Failed to parse: `{:?}`", _expr);
            }
        }
    }
}

#[test]
fn test_service() -> Result<(), EvalError> {
    init_logger();

    {
        let mut service = EdgeRulesModel::new();
        service.append_source("value: 2 + 2")?;
        service
            .append_source("value: 2 + 3")
            .expect("append second expression");

        let runtime = service.to_runtime()?;
        let result = runtime.evaluate_field("value")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(5)));
    }

    test_code("value").expect_parse_error(UnexpectedToken(
        Box::new(EToken::Unparsed(EUnparsedToken::Comma)),
        None,
    ));
    test_code("value: 2 + 2").expect_num("value", Int(4));
    test_code("value: 2 + ").expect_parse_error(WrongFormat("any".to_string()));
    test_code("{ value: 2 + 2 }").expect_num("value", Int(4));
    test_code("{ v1: 100; value: v1 + v1 }").expect_num("value", Int(200));

    Ok(())
}

#[test]
fn test_service_evaluate_field_with_existing_state() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source("{ value: 3 }")?;
    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_field("value")?;
    assert_eq!(result.to_string(), "3");

    service
        .append_source("value: 2 + 2")
        .expect("append new expression");

    let runtime = service.to_runtime_snapshot()?;
    let updated = runtime.evaluate_field("value")?;
    assert_eq!(updated.to_string(), "4");

    service.append_source("extra: value + 2")?;
    let runtime = service.to_runtime_snapshot()?;
    let extra = runtime.evaluate_field("extra")?;
    assert_eq!(extra.to_string(), "6");

    Ok(())
}

#[test]
fn test_service_evaluate_field_with_path_depth() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service
        .append_source("{ calendar: { config: { start: 7 }; sub: { inner: { value: 42 } } } }")?;

    let runtime = service.to_runtime_snapshot()?;
    let out1 = runtime.evaluate_field("calendar.config.start")?;
    assert_eq!(out1.to_string(), "7");

    let out2 = runtime.evaluate_field("calendar.sub.inner.value")?;
    assert_eq!(out2.to_string(), "42");

    let duplicate = service.append_source("{ calendar: { config: { start: 7; end: start + 5 } } }");
    let errors = duplicate.expect_err("duplicate calendar should fail");
    let first_error = errors
        .errors()
        .first()
        .expect("expected duplicate calendar error");
    assert!(first_error.to_string().contains("calendar"));

    let runtime = service.to_runtime_snapshot()?;
    let start = runtime.evaluate_field("calendar.config.start")?;
    assert_eq!(start.to_string(), "7");

    // @Todo: this test is incorrect, `calendar.config.end` cannot be linked and link error should occur
    // @Todo: find and fix if self.path.len() > 1 && is_unattached_root {...
    let end = runtime.evaluate_field("calendar.config.end")?;
    assert_eq!(
        end.to_string(),
        ValueEnum::NumberValue(NumberEnum::SV(Missing("end".to_string()))).to_string()
    );

    Ok(())
}

#[test]
fn test_evaluate_expression_with_loaded_context() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source("{ value: 3 }")?;

    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_expression_str("2 + value")?;
    assert_eq!(result, ValueEnum::NumberValue(Int(5)));

    Ok(())
}

#[test]
fn test_evaluate_pure_expression_without_context() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_expression_str("2 + 3")?;
    assert_eq!(result, ValueEnum::NumberValue(Int(5)));

    let result = runtime.evaluate_expression_str("sum(1,2,3)")?;
    assert_eq!(result, ValueEnum::NumberValue(Int(6)));

    Ok(())
}

#[test]
fn test_evaluate_expression_unknown_variable_fails() {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let runtime = service
        .to_runtime_snapshot()
        .expect("Failed to build runtime snapshot");
    let err = runtime.evaluate_expression_str("x + 1").unwrap_err();
    match err {
        EvalError::FailedExecution(_e) => {}
        other => panic!("Expected runtime error, got: {:?}", other),
    }
}

#[test]
fn append_source_accepts_user_function_definition() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source("func inc(value): { result: value + 1 }")?;
    service.append_source("{ value: inc(2).result }")?;

    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_expression_str("value")?;
    assert_eq!(result, ValueEnum::NumberValue(Int(3)));

    Ok(())
}

#[test]
fn merge_context_object_appends_new_fields() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let mut builder = ContextObjectBuilder::new();
    builder
        .set_expression("value", expr("1 + 1")?)
        .expect("set expression");
    let context = builder.build();

    service
        .merge_context_object(context)
        .expect("merge context object");
    let runtime = service.to_runtime()?;
    let result = runtime.evaluate_field("value")?;
    assert_eq!(result.to_string(), "2");

    Ok(())
}

#[test]
fn merge_context_object_rejects_duplicate_fields() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service
        .set_expression("value", expr("1")?)
        .expect("set expression");

    let mut builder = ContextObjectBuilder::new();
    builder
        .set_expression("value", expr("2")?)
        .expect("set expression");
    let context = builder.build();

    match service.merge_context_object(context) {
        Err(err) => {
            assert_eq!(err.name, "value");
        }
        other => panic!("expected duplicate error, got {:?}", other),
    }

    Ok(())
}

#[test]
fn set_expression_adds_root_field() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service
        .set_expression("enabled", ExpressionEnum::from(true))
        .expect("set root expression");

    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_field("enabled")?;
    assert_eq!(result.to_string(), "true");

    Ok(())
}

#[test]
fn set_expression_overrides_existing_field() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service
        .set_expression("enabled", ExpressionEnum::from(true))
        .expect("set root expression");
    service
        .set_expression("enabled", ExpressionEnum::from(false))
        .expect("override expression");

    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_field("enabled")?;
    assert_eq!(result.to_string(), "false");

    Ok(())
}

#[test]
fn set_expression_errors_when_context_missing() {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let error = service
        .set_expression("other.enabled", ExpressionEnum::from(true))
        .expect_err("missing context should fail");

    match error {
        ContextQueryErrorEnum::ContextNotFoundError(path) => {
            assert_eq!(path, "other");
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[test]
fn set_expression_updates_nested_context_when_present() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let nested = ContextObjectBuilder::new().build();
    service
        .set_expression("other", ExpressionEnum::StaticObject(nested))
        .expect("insert nested context");
    service
        .set_expression("other.enabled", ExpressionEnum::from(true))
        .expect("set nested expression");

    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_field("other.enabled")?;
    assert_eq!(result.to_string(), "true");

    Ok(())
}

#[test]
fn set_expression_root_context_accepts_complex_expression() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let value_expression = expr("2 + 3")?;
    service
        .set_expression("sum", value_expression)
        .expect("set sum expression");

    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_field("sum")?;
    assert_eq!(result.to_string(), "5");

    Ok(())
}

#[test]
fn set_expression_supports_multi_segment_paths() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service
        .set_expression(
            "settings",
            ExpressionEnum::StaticObject(ContextObjectBuilder::new().build()),
        )
        .expect("create settings context");
    service
        .set_expression(
            "settings.network",
            ExpressionEnum::StaticObject(ContextObjectBuilder::new().build()),
        )
        .expect("create settings.network context");
    service
        .set_expression("settings.network.enabled", ExpressionEnum::from(true))
        .expect("set nested flag");

    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_field("settings.network.enabled")?;
    assert_eq!(result.to_string(), "true");

    Ok(())
}

#[test]
fn expressions_api_gets_and_removes_fields() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service
        .set_expression("enabled", ExpressionEnum::from(true))
        .expect("set root expression");
    assert!(service.get_expression("enabled").is_ok());

    service
        .remove_expression("enabled")
        .expect("remove root expression");
    match service.get_expression("enabled") {
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        _ => panic!("expected EntryNotFoundError"),
    }

    let mut nested_builder = ContextObjectBuilder::new();
    nested_builder
        .add_expression("enabled", ExpressionEnum::from(false))
        .expect("add nested field");
    let nested_context = nested_builder.build();

    service
        .set_expression(
            "settings",
            ExpressionEnum::StaticObject(Rc::clone(&nested_context)),
        )
        .expect("attach nested context");
    service
        .set_expression("settings.mode", ExpressionEnum::from("auto"))
        .expect("set nested field");

    assert!(service.get_expression("settings.enabled").is_ok());
    assert!(service.get_expression("settings.mode").is_ok());

    service
        .remove_expression("settings.enabled")
        .expect("remove nested expression");
    match service.get_expression("settings.enabled") {
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        _ => panic!("expected EntryNotFoundError"),
    }
    assert!(service.get_expression("settings.mode").is_ok());

    Ok(())
}

#[test]
fn user_type_api_supports_root_and_nested_contexts() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let base_type = UserTypeBody::TypeRef(ComplexTypeRef::BuiltinType(ValueType::BooleanType));

    service
        .set_user_type("IsEnabled", base_type.clone())
        .expect("set root user type");
    assert!(service.get_user_type("IsEnabled").is_ok());

    service
        .remove_user_type("IsEnabled")
        .expect("remove root user type");
    match service.get_user_type("IsEnabled") {
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        _ => panic!("expected EntryNotFoundError"),
    }

    let nested_builder = ContextObjectBuilder::new();
    let nested = nested_builder.build();
    service
        .set_expression("settings", ExpressionEnum::StaticObject(Rc::clone(&nested)))
        .expect("attach nested context");

    service
        .set_user_type("settings.Point", base_type.clone())
        .expect("set nested user type");
    assert!(service.get_user_type("settings.Point").is_ok());

    service
        .remove_user_type("settings.Point")
        .expect("remove nested user type");
    match service.get_user_type("settings.Point") {
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        _ => panic!("expected EntryNotFoundError"),
    }

    let error = service
        .set_user_type("unknown.Point", base_type)
        .expect_err("missing context should fail");
    match error {
        ContextQueryErrorEnum::ContextNotFoundError(path) => assert_eq!(path, "unknown"),
        other => panic!("unexpected error: {:?}", other),
    }

    Ok(())
}

#[test]
fn user_function_api_supports_root_and_nested_contexts() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    let root_fn = FunctionDefinition::build(
        "inc".to_string(),
        vec![],
        ContextObjectBuilder::new().build(),
    )
    .expect("build root function");
    service
        .set_user_function(root_fn, None)
        .expect("set root user function");
    assert!(service.get_user_function("inc").is_ok());

    service
        .remove_user_function("inc")
        .expect("remove root user function");
    match service.get_user_function("inc") {
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        _ => panic!("expected EntryNotFoundError"),
    }

    let nested_builder = ContextObjectBuilder::new();
    let nested = nested_builder.build();
    service
        .set_expression("other", ExpressionEnum::StaticObject(Rc::clone(&nested)))
        .expect("attach nested context");

    let nested_fn = FunctionDefinition::build(
        "compute".to_string(),
        vec![],
        ContextObjectBuilder::new().build(),
    )
    .expect("build nested function");
    service
        .set_user_function(nested_fn, Some(vec!["other"]))
        .expect("set nested user function");
    assert!(service.get_user_function("other.compute").is_ok());

    service
        .remove_user_function("other.compute")
        .expect("remove nested user function");
    match service.get_user_function("other.compute") {
        Err(ContextQueryErrorEnum::EntryNotFoundError(_)) => {}
        _ => panic!("expected EntryNotFoundError"),
    }

    let err = service
        .set_user_function(
            FunctionDefinition::build(
                "missing".to_string(),
                vec![],
                ContextObjectBuilder::new().build(),
            )
            .expect("build missing function"),
            Some(vec!["missing"]),
        )
        .expect_err("missing context should fail");
    match err {
        ContextQueryErrorEnum::ContextNotFoundError(path) => assert_eq!(path, "missing"),
        other => panic!("unexpected error: {:?}", other),
    }

    Ok(())
}

#[test]
fn test_evaluate_expression_with_function_indirect() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source("{ func f(a): { result: a + 1 }; tmp: f(2).result }")?;
    let runtime = service.to_runtime_snapshot()?;
    let result = runtime.evaluate_expression_str("tmp")?;
    assert_eq!(result, ValueEnum::NumberValue(Int(3)));

    Ok(())
}

#[test]
fn call_method_errors_when_function_missing() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source("{ value: 1 }")?;
    let runtime = service.to_runtime_snapshot()?;

    let err = runtime
        .call_method("missing", vec![])
        .expect_err("expected missing function error");

    let message = err.to_string();
    assert!(
        message.contains("Function 'missing(...)"),
        "unexpected error: {message}"
    );

    Ok(())
}

#[test]
fn call_method_errors_when_argument_count_mismatches() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source("{ func greet(name, age): { result: name } }")?;
    let runtime = service.to_runtime_snapshot()?;

    let err = runtime
        .call_method("greet", vec![expr("'tom'")?])
        .expect_err("expected argument mismatch error");

    let message = err.to_string();
    assert!(
        message.contains("Function greet expects 2 arguments, but 1 were provided"),
        "unexpected error: {message}"
    );

    Ok(())
}

#[test]
fn call_method_happy_path_with_single_and_multiple_arguments() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source(
        "{ func inc(x): { result: x + 1 }; func add(left, right): { result: left + right } }",
    )?;
    let runtime = service.to_runtime_snapshot()?;

    let single = runtime.call_method("inc", vec![expr("41")?])?;
    assert_eq!(inline(single.to_string()), inline("{result: 42}"));

    let multiple = runtime.call_method("add", vec![expr("1")?, expr("2")?])?;
    assert_eq!(inline(multiple.to_string()), inline("{result: 3}"));

    Ok(())
}

#[test]
fn call_method_type_mismatch_does_not_poison_context() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source(
            "{ type LoanOffer: { amount: <number> }; func inc(offer: LoanOffer): { result: offer.amount + 1 } }",
        )?;
    let runtime = service.to_runtime_snapshot()?;

    let err = runtime
        .call_method("inc", vec![expr("1")?])
        .expect_err("expected type mismatch error");
    let message = err.to_string();
    assert!(
        message.contains("Argument `offer` of function `inc`"),
        "unexpected error: {message}"
    );

    let first = runtime.call_method("inc", vec![expr("{amount: 10}")?])?;
    assert_eq!(inline(first.to_string()), inline("{result: 11}"));

    let second = runtime.call_method("inc", vec![expr("{amount: 20}")?])?;
    assert_eq!(inline(second.to_string()), inline("{result: 21}"));

    Ok(())
}

#[test]
fn call_method_list_iteration() -> Result<(), EvalError> {
    init_logger();

    let mut service = EdgeRulesModel::new();
    service.append_source(
        r#"
        {
            func interpolate(baseline: number[]) : {
               resultset : for x in baseline return x * 2
            }
        }
        "#,
    )?;

    let runtime = service.to_runtime_snapshot()?;

    let first = runtime.call_method("interpolate", vec![expr("[1,2,3,4,5]")?])?;
    assert_eq!(
        inline(first.to_string()),
        inline("{resultset: [2, 4, 6, 8, 10]}")
    );

    Ok(())
}

#[test]
fn test_linking() -> Result<(), EvalError> {
    init_logger();

    test_code("{ a: 1; b: a  }")
        .expect_type("{a: number; b: number}")
        .expect_num("a", Int(1));

    test_code("{ a: z; b: a; z: 8 * 2  }")
        .expect_type("{a: number; b: number; z: number}")
        .expect_num("a", Int(16));

    test_code("{ a: {x: 1}; b: a.x }")
        .expect_type("{a: {x: number}; b: number}")
        .expect_num("b", Int(1));

    test_code("{ c: b; a: {x: 1}; b: a.x }")
        .expect_type("{c: number; a: {x: number}; b: number}")
        .expect_num("c", Int(1));

    // roundtrip test
    test_code("{ c: b; a: {x: 1; aa: b}; b: a.x }")
        .expect_type("{c: number; a: {x: number; aa: number}; b: number}")
        .expect_num("c", Int(1));

    // messy handover test
    test_code("{ c: b; a: {x: {y: 1}}; b: a.x; d: c.y }")
        .expect_type("{c: {y: number}; a: {x: {y: number}}; b: {y: number}; d: number}")
        .expect_num("d", Int(1));

    // deep roundtrip test
    test_code("{ c: b; a: {x: {x: 1; aa: b}}; b: a.x.x }")
        .expect_type("{c: number; a: {x: {x: number; aa: number}}; b: number}")
        .expect_num("c", Int(1));

    test_code("{ func f(arg1):  { a: arg1 } }").expect_type("{}");

    test_code("{ func f(arg1):  { a: arg1 }; b: 1 }")
        .expect_type("{b: number}")
        .expect_num("b", Int(1));

    test_code("{ func f(arg1):  { a: arg1 }; b: f(1) }").expect_type("{b: {a: number}}");

    test_code("{ func f(arg1):  { a: arg1 }; b: f(1).a }")
        .expect_type("{b: number}")
        .expect_num("b", Int(1));

    // possibility to call a function from a sub-context
    test_code_lines(&[
        "func func1(a): { result: a }",
        "subContext: {",
        "subResult: func1(35).result",
        "}",
        "value: subContext.subResult",
    ])
    .expect_num("value", Int(35));

    // argument as a parameter works well
    test_code_lines(&[
        "myInput: 35",
        "func func1(a): { result: a }",
        "subContext: {",
        "subResult: func1(myInput).result",
        "}",
        "value: subContext.subResult",
    ])
    .expect_num("value", Int(35));

    Ok(())
}

#[test]
fn calendar_self_reference_in_array_elements() -> Result<(), EvalError> {
    init_logger();

    let tb = test_code_lines(&[
        "calendar: {",
        "    shift: 2",
        "    days: [",
        "        { start: calendar.shift + 1 },",
        "        { start: calendar.shift + 31 }",
        "    ]",
        "    firstDay: days[0].start",
        "    secondDay: days[1].start",
        "}",
    ]);
    tb.expect_num("calendar.firstDay", Int(3));
    tb.expect_num("calendar.secondDay", Int(33));

    Ok(())
}

#[test]
fn pass_self_context_to_function_should_fail() {
    init_logger();

    // Users cannot pass the context object itself into a function defined in that same context.
    test_code_lines(&[
            "calendar: {",
            "    shift: 2",
            "    func start1(calendar): { result: calendar.shift + 1 }",
            "    firstDay: start1(calendar).result",
            "}",
        ])
            .expect_link_error(LinkingErrorEnum::OtherLinkingError(
                "Cannot pass context `calendar` as argument to function `start1` defined in the same context".to_string(),
            ));
}

#[test]
fn pass_context_to_function_should_not_fail() {
    init_logger();

    // Users can pass the context object into a function defined in upper or another context.
    test_code_lines(&[
            "func start1(calendar): { result: calendar.shift + 1 }",
            "calendar: {",
            "    shift: 2",
            "    firstDay: start1(calendar).result",
            "}",
        ])
            .expect_link_error(LinkingErrorEnum::OtherLinkingError(
                "Cannot pass context `calendar` as argument to function `start1` defined in the same context".to_string(),
            ));
}

#[test]
fn pass_self_context_as_second_argument_should_fail() {
    init_logger();

    // The guard applies to any argument position, not just the first one.
    test_code_lines(&[
            "calendar: {",
            "    shift: 2",
            "    func start2(x, cal): { result: cal.shift + x }",
            "    firstDay: start2(1, calendar).result",
            "}",
        ])
            .expect_link_error(LinkingErrorEnum::OtherLinkingError(
                "Cannot pass context `calendar` as argument to function `start2` defined in the same context".to_string(),
            ));
}

#[test]
fn pass_sub_context_to_function() -> Result<(), EvalError> {
    init_logger();

    let tb = test_code_lines(&[
        "calendar: {",
        "    config: { shift: 2 }",
        "    func start1(calendar): { result: calendar.shift + 1 }",
        "    firstDay: start1(config).result",
        "}",
    ]);
    tb.expect_num("calendar.firstDay", Int(3));

    Ok(())
}
