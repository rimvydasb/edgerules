use edge_rules::runtime::decision_service::DecisionService;
use edge_rules::runtime::edge_rules::{EdgeRulesModel, EvalError};
use edge_rules::test_support::ValueEnum;
use std::rc::Rc;

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

#[cfg(feature = "mutable_decision_service")]
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

    let response = service.execute("decide", request).expect("decision service should execute");
    let rendered = value_to_string(&response);
    assert!(rendered.contains("decision:20"), "response should include calculated decision, got: {}", rendered);
}

#[test]
fn execute_errors_when_method_is_missing() {
    let mut service = DecisionService::from_source("{ helper: 1 }").expect("service with helper only");
    let request = build_request_value("{ amount: 5 }");

    let err = service.execute("unknownMethod", request).unwrap_err();
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

    let err = service.execute("invalid", request).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("exactly one argument"), "expected arity error, got: {}", err);
}

#[cfg(feature = "mutable_decision_service")]
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
    let err = service.execute("decide", request).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("expects 1 arguments"),
        "expected link error about argument count, got {}",
        err
    );
}

#[cfg(feature = "mutable_decision_service")]
#[test]
fn execute_relinks_after_model_updates() {
    let model = r#"
    {
        func decide(request): {
            value: request.amount + 1
        }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service from model");
    let request = build_request_value("{ amount: 3 }");
    let first = service.execute("decide", request.clone()).expect("first execution");
    assert!(value_to_string(&first).contains("value:4"), "expected request.amount+1 result, got {}", first);

    let model_ref = service.get_model();
    {
        let mut borrowed = model_ref.borrow_mut();
        borrowed.remove_user_function("decide").expect("remove previous decide function");
        borrowed
            .append_source(
                r#"
            {
                func decide(request): {
                    value: request.amount + 10
                }
            }
            "#,
            )
            .expect("override decide implementation");
    }

    let second = service.execute("decide", request).expect("execution after edit");
    assert!(value_to_string(&second).contains("value:13"), "expected updated decide result, got {}", second);
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
    let response = service.execute("decide", request).expect("execute from context");
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
fn execute_nested_method_fails() {
    // Nested method execution is not supported by design.
    // The execute() method expects the function to be in the root context.
    let model = r#"
        {
            deeper: {
                func nested(req): { result: true }
            }
        }
    "#;

    let mut service = DecisionService::from_source(model).expect("service with nested function");
    let request = ValueEnum::from(1);

    // This should fail because "nested" is not in the root context,
    // and "deeper.nested" path resolution is not supported for execution entry points.
    let err = service.execute("deeper.nested", request).unwrap_err();
    let err_str = err.to_string().to_lowercase();
    
    assert!(
        err_str.contains("not found") || err_str.contains("entry 'nested' not found"),
        "expected error about missing function, got: {}",
        err_str
    );
}
