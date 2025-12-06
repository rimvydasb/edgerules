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
    assert!(err.location.is_empty(), "location should be empty for now");
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
    assert!(err.location.is_empty(), "location should be empty for now");
    assert!(
        err.to_string().to_lowercase().contains("invalid date"),
        "got: {err}"
    );
}

mod utilities;
pub use utilities::*;
