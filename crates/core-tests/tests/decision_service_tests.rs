use edge_rules::runtime::decision_service::DecisionService;
use edge_rules::runtime::edge_rules::{EdgeRulesModel, EvalError};
use edge_rules::test_support::ValueEnum;
use std::rc::Rc;

/// Builds a request value from a source string for use in decision service tests.
///
/// # Example
/// ```ignore
/// let request_value = build_request_value("{ amount: 10 }");
/// ```
fn build_request_value(source: &str) -> ValueEnum {
    let mut model = EdgeRulesModel::new();
    let wrapped = format!("{{ requestData: {} }}", source.trim());
    model.append_source(&wrapped).expect("request object should parse");
    let runtime = model.to_runtime().expect("request object should link");
    runtime.evaluate_field("requestData").expect("request field should evaluate")
}

/// Normalizes a ValueEnum to a string without whitespace for assertion comparisons.
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

/// Tests that DecisionService.execute() returns a properly calculated response object.
#[test]
fn execute_returns_response_object() {
    // Arrange: Define a decision function that doubles the input amount
    let decision_model_source = r#"
    {
        type RequestType: { amount: <number> }
        func decide(request: RequestType): {
            decision: request.amount * 2
        }
    }
    "#;

    let mut decision_service = DecisionService::from_source(decision_model_source).expect("service from source");
    let request_value = build_request_value("{ amount: 10 }");

    // Act: Execute the decision function with the request
    let response = decision_service.execute("decide", request_value).expect("decision service should execute");

    // Assert: Response contains the calculated decision (10 * 2 = 20)
    let rendered_response = value_to_string(&response);
    assert!(
        rendered_response.contains("decision:20"),
        "response should include calculated decision, got: {}",
        rendered_response
    );
}

/// Tests that DecisionService.execute() returns an error when the method is not defined.
#[test]
fn execute_errors_when_method_is_missing() {
    // Arrange: Create a service without the requested method
    let mut decision_service = DecisionService::from_source("{ helper: 1 }").expect("service with helper only");
    let request_value = build_request_value("{ amount: 5 }");

    // Act: Attempt to execute a non-existent method
    let execution_error = decision_service.execute("unknownMethod", request_value).unwrap_err();

    // Assert: Error message indicates missing method
    assert!(
        execution_error.to_string().to_lowercase().contains("not found"),
        "expected missing method error, got: {}",
        execution_error
    );
}

/// Tests that DecisionService.execute() returns an error when the method has wrong arity.
#[test]
fn execute_errors_when_method_has_wrong_arity() {
    // Arrange: Define a function with no arguments (invalid for decision service)
    let zero_arity_model = r#"
    {
        func invalid(): { result: true }
    }
    "#;

    let mut decision_service = DecisionService::from_source(zero_arity_model).expect("service with invalid method");
    let request_value = build_request_value("{ amount: 5 }");

    // Act: Attempt to execute a function that takes no arguments
    let arity_error = decision_service.execute("invalid", request_value).unwrap_err();

    // Assert: Error message indicates arity mismatch
    assert!(
        arity_error.to_string().to_lowercase().contains("exactly one argument"),
        "expected arity error, got: {}",
        arity_error
    );
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

/// Tests that DecisionService can be initialized from an existing context tree.
#[test]
fn from_context_reuses_provided_tree() {
    // Arrange: Build a model and extract its static context tree
    let decision_model_source = r#"
    {
        func decide(request): {
            value: request.amount * 3
        }
    }
    "#;

    let mut model_builder = EdgeRulesModel::new();
    model_builder.append_source(decision_model_source).expect("seed model should parse");
    let initial_runtime = model_builder.to_runtime().expect("seed model should link");
    let shared_context_tree = Rc::clone(&initial_runtime.static_tree);

    // Act: Create decision service from the extracted context
    let mut decision_service = DecisionService::from_context(shared_context_tree).expect("service from context");
    let request_value = build_request_value("{ amount: 7 }");
    let response = decision_service.execute("decide", request_value).expect("execute from context");

    // Assert: Calculation uses the shared context (7 * 3 = 21)
    assert!(value_to_string(&response).contains("value:21"), "expected context-driven result, got {}", response);
}

/// Tests that incompatible types in function definitions cause initialization failure.
#[test]
fn test_incompatible_types_in_function_fails_at_runtime() {
    // Arrange: Model with type-incompatible expression in isEligible function
    let model_with_type_error = r#"
    {
        func valid(val: number): { result: val > 0 }
        func isEligible(age: number): {
            return: age >= 18 + 'invalid_string'
        }
    }
    "#;

    // Act: Attempt to create service from model with type error
    let initialization_result = DecisionService::from_source(model_with_type_error);

    // Assert: Service must not initialize if root function has linking errors
    match initialization_result {
        Err(EvalError::FailedExecution(runtime_error)) => {
            let error_message = runtime_error.to_string();
            assert!(
                error_message.contains("expected 'string'") || error_message.contains("not compatible"),
                "expected linking/runtime error about incompatible types, but got: {}",
                error_message
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

/// Tests that nested method execution is not supported by design.
///
/// The execute() method expects the function to be in the root context.
/// Paths like "deeper.nested" are not supported for execution entry points.
#[test]
fn execute_nested_method_fails() {
    // Arrange: Model with function defined in nested context
    let model_with_nested_function = r#"
        {
            deeper: {
                func nested(req): { result: true }
            }
        }
    "#;

    let mut decision_service =
        DecisionService::from_source(model_with_nested_function).expect("service with nested function");
    let request_value = ValueEnum::from(1);

    // Act: Attempt to execute nested function path
    let path_error = decision_service.execute("deeper.nested", request_value).unwrap_err();

    // Assert: Error indicates function not found at root level
    let error_message = path_error.to_string().to_lowercase();
    assert!(
        error_message.contains("not found") || error_message.contains("entry 'nested' not found"),
        "expected error about missing function, got: {}",
        error_message
    );
}
