use edge_rules::runtime::edge_rules::EdgeRules;

#[macro_export]
macro_rules! assert_value {
    ($expr:expr, $expected:expr) => {
        assert_eq!(crate::eval_value(concat!("value : ", $expr)), $expected);
    };
}

pub fn eval_all(code: &str) -> String {
    let service = EdgeRules::new();
    service.evaluate_all(code)
}

pub fn eval_field(code: &str, field: &str) -> String {
    let mut service = EdgeRules::new();
    let _ = service.load_source(code);
    service.evaluate_field(field)
}

pub fn eval_value(code: &str) -> String {
    eval_field(code, "value")
}

pub fn eval_lines_field(lines: &[&str], field: &str) -> String {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    eval_field(&code, field)
}

/// For tests that must assert link errors (e.g., cyclic/self ref, missing field).
pub fn link_error_contains(code: &str, needles: &[&str]) {
    let mut service = EdgeRules::new();
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
