use edge_rules::runtime::edge_rules::EdgeRulesModel;

#[macro_export]
macro_rules! assert_value {
    // &["a","b","c"] form
    (&[ $($line:expr),* $(,)? ], $expected:expr) => {{
        let lines: &[&str] = &[$($line),*];
        assert_eq!($crate::eval_lines_field(lines, "value"), $expected, "for lines: {:?}", lines)
    }};
    // Raw string / string literal block form (e.g., r#"..."#)
    ($src:literal, $expected:expr) => {{
        let body = $src.trim_matches(|c| c == '\n' || c == '\r').trim();
        if (body.starts_with('{') && body.ends_with('}')) {
            assert_eq!($crate::eval_field(body, "value"), $expected, "for body: {:?}", body);
        } else if body.contains('\n') {
            let code = {
                let mut s = ::std::string::String::new();
                s.push_str("{\n");
                s.push_str(body);
                s.push_str("\n}");
                s
            };
            assert_eq!($crate::eval_field(&code, "value"), $expected, "for body: {:?}", $src);
        } else {
            if body.starts_with("value:") || body.starts_with("value :") || body.starts_with("value\t:") {
                let code = {
                    let mut s = ::std::string::String::new();
                    s.push_str("{\n");
                    s.push_str(body);
                    s.push_str("\n}");
                    s
                };
                assert_eq!($crate::eval_field(&code, "value"), $expected, "for body: {:?}", body);
            } else {
                assert_eq!($crate::eval_value(&format!("value : {}", body)), $expected, "for body: {:?}", body);
            }
        }
    }};
    // Expression string form (fallback)
    ($expr:expr, $expected:expr) => {
        assert_eq!($crate::eval_value(&format!("value : {}", $expr)), $expected);
    };
}

#[macro_export]
macro_rules! assert_string_contains {
    ($needle:expr, $haystack:expr) => {
        assert!(
            $haystack.contains($needle),
            "expected `{}` to contain `{}`",
            $haystack,
            $needle
        );
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

fn wrap_in_object(lines: &str) -> String {
    if lines.trim().starts_with('{') && lines.trim().ends_with('}') {
        return lines.trim().to_string();
    }

    format!("{{{}}}", lines.trim().to_string())
}

pub fn assert_eval_all(lines: &str, expected_lines: &[&str]) {
    let model = wrap_in_object(lines);
    let evaluated = eval_all(&*model);
    assert_eq!(
        evaluated.lines().map(|l| l.trim()).collect::<Vec<_>>(),
        expected_lines.iter().map(|l| l.trim()).collect::<Vec<_>>()
    );
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

/// For tests that must assert parse errors (e.g., invalid syntax, duplicate fields).
pub fn parse_error_contains(code: &str, needles: &[&str]) {
    let mut service = EdgeRulesModel::new();
    let err = service.load_source(code);

    match (err.err().map(|e| e.to_string())) {
        None => {
            panic!("expected parse error, got none\ncode:\n{code}");
        }
        Some(err) => {
            let lower = err.to_lowercase();
            for n in needles {
                assert!(
                    lower.contains(&n.to_lowercase()),
                    "expected error to contain `{n}`, got: {err}\ncode:\n{code}"
                );
            }
        }
    }
}

pub fn to_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn test_first() {
    // no-op: ensures test harness initializes cleanly
}
