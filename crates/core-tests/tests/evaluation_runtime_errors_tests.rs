use edge_rules::runtime::edge_rules::EdgeRulesModel;

#[test]
fn runtime_error_exposes_stage_at_root() {
    let code = r#"
{
    value: date("not-a-date")
}
"#;

    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    let err = runtime
        .evaluate_field("value")
        .err()
        .expect("expected runtime error");

    assert!(err.stage.is_some());
    assert_eq!(err.location, vec!["value"]);
    assert!(
        err.to_string().to_lowercase().contains("invalid date"),
        "got: {err}"
    );
}

#[test]
fn runtime_error_in_nested_context_has_stage() {
    let code = r#"
{
    nested: { bad: date("not-a-date") }
    value: nested.bad
}
"#;

    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    let err = runtime
        .evaluate_field("value")
        .err()
        .expect("expected runtime error");

    assert!(err.stage.is_some());
    assert_eq!(err.location, vec!["nested", "bad"]);
    assert!(
        err.to_string().to_lowercase().contains("invalid date"),
        "got: {err}"
    );
}

mod utilities;
pub use utilities::*;

#[test]
fn runtime_error_deep_dependency_chain() {
    let code = r#"
{
    source: {
        value: date('invalid')
    }
    intermediate: {
        calc: source.value
    }
    result: intermediate.calc
}
"#;

    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // Evaluate 'result', which depends on 'intermediate.calc', which depends on 'source.value'
    // The error originates in 'source.value'
    let err = runtime
        .evaluate_field("result")
        .err()
        .expect("expected runtime error");

    assert!(err.stage.is_some());
    // The location should point to the source of the error, not the top-level field
    assert_eq!(err.location, vec!["source", "value"]);
    assert_eq!(err.expression, Some("date('invalid')".to_string()));
}
