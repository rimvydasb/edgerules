use edge_rules::runtime::edge_rules::EdgeRulesModel;

#[macro_export]
macro_rules! assert_value {
    ($expr:expr, $expected:expr) => {
        assert_eq!($crate::eval_value(concat!("value : ", $expr)), $expected);
    };
}

pub fn eval_all(code: &str) -> String {
    let mut service = EdgeRulesModel::new();
    match service.load_source(code) {
        Ok(()) => match service.to_runtime() {
            Ok(runtime) => match runtime.eval_all() {
                Ok(()) => runtime.context.borrow().to_code(),
                Err(err) => err.to_string(),
            },
            Err(err) => err.to_string(),
        },
        Err(err) => err.to_string(),
    }
}

pub fn eval_field(code: &str, field: &str) -> String {
    let mut service = EdgeRulesModel::new();
    match service.load_source(code) {
        Ok(()) => match service.to_runtime() {
            Ok(runtime) => match runtime.evaluate_field(field) {
                Ok(value) => value.to_string(),
                Err(err) => err.to_string(),
            },
            Err(err) => err.to_string(),
        },
        Err(err) => err.to_string(),
    }
}

pub fn eval_value(code: &str) -> String {
    eval_field(code, "value")
}

pub fn eval_lines_field(lines: &[&str], field: &str) -> String {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    eval_field(&code, field)
}

pub fn assert_eval_all(lines: &[&str], expected_lines: &[&str]) {
    let model = format!("{{\n{}\n}}", lines.join("\n"));
    let evaluated = eval_all(&model);
    let mut expected = expected_lines.join("\n");
    expected.push('\n');
    assert_eq!(evaluated, expected);
}

/// For tests that must assert link errors (e.g., cyclic/self ref, missing field).
pub fn link_error_contains(code: &str, needles: &[&str]) {
    let mut service = EdgeRulesModel::new();
    let _ = service.load_source(code);
    let err = service.to_runtime().err().map(|e| e.to_string()).unwrap();
    let lower = err.to_lowercase();
    for n in needles {
        assert!(
            lower.contains(&n.to_lowercase()),
            "expected error to contain `{n}`, got: {err}"
        );
    }
}

#[test]
fn test_first() {
    // no-op: ensures test harness initializes cleanly
}
