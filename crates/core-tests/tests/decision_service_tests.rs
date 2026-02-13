use edge_rules::runtime::decision_service::DecisionService;
use edge_rules::runtime::edge_rules::{EdgeRulesModel, EvalError};
use edge_rules::runtime::edge_rules::{ExpressionEnum, InvocationSpec};
use edge_rules::test_support::ValueEnum;
use std::rc::Rc;

mod utilities;
pub use utilities::*;

fn build_request_value(source: &str) -> ValueEnum {
    let mut model = EdgeRulesModel::new();
    let wrapped = format!("{{ requestData: {} }}", source.trim());
    model.append_source(&wrapped).expect("request object should parse");
    let runtime = model.to_runtime().expect("request object should link");
    runtime.evaluate_field("requestData").expect("request field should evaluate")
}

fn value_to_string(value: &ValueEnum) -> String {
    value.to_string().replace(['\n', ' '], "")
}

#[test]
fn set_invocation_invokes_function() {
    let mut model = EdgeRulesModel::new();
    model
        .append_source(
            r#"
        {
            func compute(input): {
                doubled: input * 2
            }
        }
        "#,
        )
        .expect("seed compute function");

    let spec = InvocationSpec {
        method_path: "compute".to_string(),
        arguments: vec![ExpressionEnum::Value(ValueEnum::from(7_i32))],
    };
    model.set_invocation("result", spec).expect("store invocation entry");

    let runtime = model.to_runtime().expect("link invocation model");
    let value = runtime.evaluate_field("result").expect("evaluate invocation field");
    let rendered = value_to_string(&value);
    assert!(rendered.contains("doubled:14"), "expected invocation to call compute(), got {}", rendered);
}

#[test]
fn execute_returns_response_object() {
    let model = r#"
    {
        type RequestType: { amount: <number> }
        func decide(request: RequestType): {
            decision: request.amount * 2
        }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service from source");
    let request = build_request_value("{ amount: 10 }");

    let response = service.execute("decide", Some(vec![request])).expect("decision service should execute");
    let rendered = value_to_string(&response);
    assert!(rendered.contains("decision:20"), "response should include calculated decision, got: {}", rendered);
}

#[test]
fn execute_errors_when_method_is_missing() {
    let mut service = DecisionService::from_source("{ helper: 1 }").expect("service with helper only");
    let request = build_request_value("{ amount: 5 }");

    let err = service.execute("unknownMethod", Some(vec![request])).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("not found"), "expected missing method error, got: {}", err);
}

#[test]
fn execute_errors_when_method_has_wrong_arity() {
    let model = r#"
    {
        func invalid(): { result: true }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service with invalid method");
    let request = build_request_value("{ amount: 5 }");

    let err = service.execute("invalid", Some(vec![request])).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("expected 0 arguments, but got 1"),
        "expected arity error, got: {}",
        err
    );
}

#[test]
fn execute_validation_allows_multiple_arguments() {
    let model = r#"
    {
        func oneArg(a): { res: a }
        func twoArgs(a, b): { res: a + b }
        func threeArgs(a, b, c): { res: a + b + c }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service setup");
    let request = build_request_value("10");

    // 1 arg: Should pass validation and execution
    let res = service.execute("oneArg", Some(vec![request.clone()])).expect("one arg execution");
    assert_string_contains("res:10", value_to_string(&res));

    // 2 args: Should fail with wrong number of arguments
    let err2 = service.execute("twoArgs", Some(vec![request.clone()])).unwrap_err();
    let err2_str = err2.to_string();
    assert!(err2_str.contains("expected 2 arguments, but got 1"), "Should fail due to arg mismatch, got: {}", err2_str);

    // 3 args: Should fail with wrong number of arguments
    let err3 = service.execute("threeArgs", Some(vec![request])).unwrap_err();
    let err3_str = err3.to_string();
    assert!(err3_str.contains("expected 3 arguments, but got 1"), "Should fail due to arg mismatch, got: {}", err3_str);
}

#[test]
fn execute_multi_argument_method() {
    let model = r#"
    {
        func add(a, b): a + b
    }
    "#;
    let mut service = DecisionService::from_source(model).expect("service from source");
    let args = vec![ValueEnum::from(10), ValueEnum::from(20)];
    let res = service.execute("add", Some(args)).expect("execute multi-arg");
    assert_eq!(res, ValueEnum::from(30));
}

#[test]
fn execute_field_evaluation() {
    let model = r#"
    {
        field: 10 + 20
    }
    "#;
    let mut service = DecisionService::from_source(model).expect("service from source");
    let res = service.execute("field", None).expect("execute field evaluation");
    assert_eq!(res, ValueEnum::from(30));
}

#[test]
fn execute_wildcard() {
    let model = r#"
    {
        a: 1
        b: 2
    }
    "#;
    let mut service = DecisionService::from_source(model).expect("service from source");
    let res = service.execute("*", None).expect("execute wildcard");
    let rendered = value_to_string(&res);
    assert!(rendered.contains("a:1"), "should contain a:1");
    assert!(rendered.contains("b:2"), "should contain b:2");
}

#[test]
fn execute_wildcard_with_args_fails() {
    let mut service = DecisionService::from_source("{ a: 1 }").expect("service from source");
    let err = service.execute("*", Some(vec![ValueEnum::from(1)])).unwrap_err();
    assert!(err.to_string().contains("not found"), "expected entry not found error");
}

#[test]
fn invalid_invocation_surfaces_link_error() {
    let model = r#"
    {
        func helper(value): { outcome: value }
        func decide(request): {
            ok: true
        }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service from model");
    {
        let model_ref = service.get_model();
        let mut borrowed = model_ref.borrow_mut();
        let spec = InvocationSpec { method_path: "helper".to_string(), arguments: Vec::new() };
        borrowed.set_invocation("broken", spec).expect("invocation stored");
    }

    let request = build_request_value("{ value: 10 }");
    let err = service.execute("decide", Some(vec![request])).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("expects 1 arguments"),
        "expected link error about argument count, got {}",
        err
    );
}

#[test]
fn execute_relinks_after_model_updates() {
    let model = r#"
    {
        type Request: { amount: <number> }
        func decide(request: Request): {
            value: request.amount + 1
        }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service from model");
    let request = build_request_value("{ amount: 3 }");
    let first = service.execute("decide", Some(vec![request.clone()])).expect("first execution");
    assert_string_contains("value:4", inline_text(first.to_string()));

    let model_ref = service.get_model();
    {
        let mut borrowed = model_ref.borrow_mut();
        borrowed.remove_user_function("decide").expect("remove previous decide function");
        borrowed
            .append_source(
                r#"
            {
                func decide(request: Request): {
                    value: request.amount + 10
                }
            }
            "#,
            )
            .expect("override decide implementation");
    }

    let second = service.execute("decide", Some(vec![request])).expect("execution after edit");
    assert_string_contains("value:13", inline_text(second.to_string()));
}

#[test]
fn from_context_reuses_provided_tree() {
    let model = r#"
    {
        func decide(request): {
            value: request.amount * 3
        }
    }
    "#;

    let mut builder = EdgeRulesModel::new();
    builder.append_source(model).expect("seed model should parse");
    let runtime = builder.to_runtime().expect("seed model should link");
    let context = Rc::clone(&runtime.static_tree);

    let mut service = DecisionService::from_context(context).expect("service from context");
    let request = build_request_value("{ amount: 7 }");
    let response = service.execute("decide", Some(vec![request])).expect("execute from context");
    assert!(value_to_string(&response).contains("value:21"), "expected context-driven result, got {}", response);
}

#[test]
fn test_incompatible_types_in_function_fails_at_runtime() {
    let model = r#"
    {
        func valid(val: number): { result: val > 0 }
        func isEligible(age: number): {
            return: age >= 18 + 'invalid_string'
        }
    }
    "#;

    // The service must not initialize if any root function has linking errors.
    let result = DecisionService::from_source(model);

    match result {
        Err(EvalError::FailedExecution(runtime_err)) => {
            let err_str = runtime_err.to_string();
            assert!(
                err_str.contains("expected 'string'") || err_str.contains("not compatible"),
                "expected linking/runtime error about incompatible types, but got: {}",
                err_str
            );
        }
        Err(EvalError::FailedParsing(parse_errors)) => {
            panic!("Should not have failed parsing, but got: {}", parse_errors);
        }
        Ok(_) => {
            panic!("Service should have failed initialization due to incompatible types in root function");
        }
    }
}

#[test]
fn execute_nested_method_succeeds() {
    let model = r#"
        {
            deeper: {
                func nested(req): { result: req }
            }
        }
    "#;

    let mut service = DecisionService::from_source(model).expect("service with nested function");
    let request = ValueEnum::from(1);

    let res = service.execute("deeper.nested", Some(vec![request])).expect("nested execution");
    assert!(value_to_string(&res).contains("result:1"), "expected successful nested execution, got: {}", res);
}

#[test]
fn execute_nested_paths() {
    let model = r#"
    {
        nested: {
            field: 42
            func double(x): x * 2
        }
    }
    "#;
    let mut service = DecisionService::from_source(model).expect("service from source");

    // Nested field
    let res1 = service.execute("nested.field", None).expect("execute nested field");
    assert_eq!(res1, ValueEnum::from(42));

    // Nested function
    let res2 = service.execute("nested.double", Some(vec![ValueEnum::from(21)])).expect("execute nested function");
    assert_eq!(res2, ValueEnum::from(42));
}

#[test]
fn execute_zero_arg_method_as_field_returns_context() {
    let model = r#"
    {
        func getTen(): 10
    }
    "#;
    let mut service = DecisionService::from_source(model).expect("service from source");

    // Calling execute as a field evaluation
    let res = service.execute("getTen", None).expect("execute as field");

    // It should return the context object (Reference), not the result 10
    match res {
        ValueEnum::Reference(_) => {}
        _ => panic!("Expected function context Reference, got: {}", res),
    }

    // Now try executing it as a method with 0 arguments
    let res2 = service.execute("getTen", Some(vec![])).expect("execute as method with 0 args");
    assert_eq!(res2, ValueEnum::from(10));
}
