#![allow(dead_code)]

use edge_rules::runtime::edge_rules::{EdgeRulesModel, EdgeRulesRuntime, ExpressionEnum, ParseErrors};
use edge_rules::runtime::ToSchema;
use edge_rules::test_support::ParseErrorEnum::Stacked;
use edge_rules::test_support::{LinkingError, LinkingErrorEnum, NumberEnum, ParseErrorEnum, StaticLink, ValueEnum};
use edge_rules::utils::to_display;
use env_logger::Builder;
use log::info;
use std::fmt::Display;
use std::mem::discriminant;
use std::sync::Once;

pub fn inline_text<S: AsRef<str>>(code: S) -> String {
    code.as_ref().replace('\n', " ").replace(" ", "").replace(",", "")
}

#[track_caller]
pub fn assert_string_contains<S1: AsRef<str>, S2: AsRef<str>>(needle: S1, haystack: S2) {
    let n = needle.as_ref();
    let h = haystack.as_ref();
    assert!(h.contains(n), "expected `{}` to contain `{}`", h, n);
}

#[track_caller]
pub fn assert_eval_field<R: Into<EdgeRulesRuntime>>(runtime: R, field: &str, expected: &str) {
    let runtime = runtime.into();
    match runtime.evaluate_field(field) {
        Ok(value) => {
            let mut actual = value.to_string();
            // Objects evaluated via field often have "fieldName: { ... }" prefix
            let prefix = format!("{}:", field);
            if actual.starts_with(&prefix) {
                actual = actual.strip_prefix(&prefix).unwrap().trim().to_string();
            }
            assert_eq!(inline_text(actual), inline_text(expected));
        }
        Err(err) => panic!("evaluation failed: {}\nfield: {}", err, field),
    }
}

#[track_caller]
pub fn assert_eval_value<R: Into<EdgeRulesRuntime>>(runtime: R, expected: &str) {
    assert_eval_field(runtime, "value", expected);
}

#[track_caller]
pub fn assert_expression_value(expression: &str, expected: &str) {
    let code = format!("value: {}", expression);
    assert_eval_value(code.as_str(), expected);
}

#[track_caller]
pub fn expression_value_contains(expression: &str, needles: &[&str]) {
    let code = format!("value: {}", expression);
    let runtime = EdgeRulesRuntime::from(code.as_str());
    match runtime.evaluate_field("value") {
        Ok(value) => {
            let rendered = inline_text(value.to_string());
            for n in needles {
                let needle = inline_text(n);
                assert!(rendered.contains(&needle), "expected `{}` to contain `{}`", rendered, needle);
            }
        }
        Err(err) => panic!("evaluation failed: {}", err),
    }
}

pub fn get_runtime(code: &str) -> EdgeRulesRuntime {
    init_logger();
    EdgeRulesRuntime::from(code)
}

pub fn eval_all(code: &str) -> String {
    let mut service = EdgeRulesModel::new();
    match service.append_source(code) {
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
    init_logger();
    let runtime = EdgeRulesRuntime::from(code);
    match runtime.evaluate_field(field) {
        Ok(value) => value.to_string(),
        Err(err) => err.to_string(),
    }
}

pub fn eval_value(code: &str) -> String {
    eval_field(code, "value")
}

pub fn init_logger() {
    static LOGGER: Once = Once::new();

    LOGGER.call_once(|| {
        let _ = Builder::from_default_env().is_test(true).try_init();
    });
}

pub fn init_test(name: &str) {
    init_logger();
    info!(">>> starting test {}", name);
}

pub fn wrap_in_object(lines: &str) -> String {
    if lines.trim().starts_with('{') && lines.trim().ends_with('}') {
        return lines.trim().to_string();
    }

    format!("{{{}}}", lines.trim())
}

#[track_caller]
pub fn assert_eval_all<R: Into<EdgeRulesRuntime>>(runtime: R, expected_lines: &[&str]) {
    let runtime = runtime.into();
    if let Err(err) = runtime.eval_all() {
        panic!("eval_all failed: {}", err);
    }
    let evaluated = runtime.context.borrow().to_code();
    assert_eq!(
        evaluated.lines().map(|l| inline_text(l.trim())).collect::<Vec<_>>(),
        expected_lines.iter().map(|l| inline_text(l.trim())).collect::<Vec<_>>()
    );
}

/// For tests that must assert link errors (e.g., cyclic/self ref, missing field).
#[track_caller]
pub fn link_error_contains(code: &str, needles: &[&str]) {
    init_logger();
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(code);

    match service.to_runtime() {
        Ok(ok_but_unexpected) => {
            let static_tree = ok_but_unexpected.static_tree.borrow().to_string();
            println!("static_tree:\n{}\n", static_tree);
            panic!("expected link error, got none\ncode:{}", code);
        }
        Err(err) => {
            let lower = err.to_string().to_lowercase();
            for n in needles {
                assert!(lower.contains(&n.to_lowercase()), "expected error to contain `{n}`, got: {err}\ncode:{code}");
            }
        }
    }
}

#[track_caller]
pub fn link_error_location(
    code: &str,
    expected_location: &[&str],
    expected_expression: &str,
    error: LinkingErrorEnum,
) -> Vec<String> {
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(code);

    match service.to_runtime() {
        Ok(_) => panic!("expected link error, got none\ncode:{}", code),
        Err(err) => {
            let expected = expected_location.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            assert_eq!(err.location(), expected, "location mismatch for code:\n{code}");
            assert_eq!(
                err.expression().map(|s| s.as_str()),
                Some(expected_expression),
                "expression mismatch for code:\n{code}"
            );
            assert!(err.stage().is_some());
            assert_eq!(err.kind(), &error);
            err.location().to_vec()
        }
    }
}

/// For tests that must assert parse errors (e.g., invalid syntax, duplicate fields).
#[track_caller]
pub fn parse_error_contains(code: &str, needles: &[&str]) {
    let mut service = EdgeRulesModel::new();
    let err = service.append_source(code);

    match err.err().map(|e| e.to_string()) {
        None => {
            panic!("expected parse error, got none\ncode:{}", code);
        }
        Some(err) => {
            let lower = err.to_lowercase();
            for n in needles {
                assert!(lower.contains(&n.to_lowercase()), "expected error to contain `{n}`, got: {err}\ncode:{code}");
            }
        }
    }
}

#[track_caller]
pub fn runtime_error_contains(code: &str, needles: &[&str]) {
    init_logger();
    let runtime = EdgeRulesRuntime::from(code);
    let output = match runtime.evaluate_field("value") {
        Ok(v) => v.to_string(),
        Err(err) => err.to_string(),
    }
    .to_lowercase();

    for n in needles {
        let needle = n.to_lowercase();
        assert!(output.contains(&needle), "expected error to contain `{n}`, got: {output}\ncode:{code}");
    }
}

pub fn to_lines(text: &str) -> Vec<&str> {
    text.lines().map(|line| line.trim()).filter(|line| !line.is_empty()).collect()
}

pub fn test_code(code: &str) -> TestServiceBuilder {
    TestServiceBuilder::build(code)
}

pub fn test_code_lines<T: Display>(code: &[T]) -> TestServiceBuilder {
    TestServiceBuilder::build(format!("{{{}}}", to_display(code, "\n")).as_str())
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
                panic!("Expected runtime, but got nothing: `{}`", self.original_code);
            }
            Some(runtime) => {
                assert_eq!(runtime.static_tree.borrow().to_schema(), expected_type);
            }
        }

        self
    }

    pub fn expect_num(&self, variable: &str, expected: NumberEnum) {
        self.expect(&mut ExpressionEnum::variable(variable), ValueEnum::NumberValue(expected))
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

            if errors.errors().iter().any(|err| matches_error(err, &expected)) {
                return self;
            }

            panic!("Expected parse error `{}`, but got: `{:?}`", expected, errors);
        } else {
            panic!("Expected parse error, but got no errors: `{}`", self.original_code);
        }
    }

    pub fn expect_no_errors(&self) -> &Self {
        if let Some(errors) = &self.parse_errors {
            panic!("Expected no errors, but got parse errors: `{}`\nFailed to parse:\n{}", errors, self.original_code);
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
            assert_eq!(&expected, errors.kind(), "Testing:\n{}", self.original_code);
        } else {
            panic!("Expected linking error, but got no errors: `{}`", self.original_code);
        }

        self
    }

    pub fn expect(&self, _expr: &mut ExpressionEnum, _expected: ValueEnum) {
        self.expect_no_errors();

        if let Err(error) = _expr.link(self.runtime.as_ref().unwrap().static_tree.clone()) {
            panic!("Expected value, but got linking errors: `{:?}`\nFailed to parse:\n{}", error, _expr);
        }

        match _expr.eval(self.runtime.as_ref().unwrap().context.clone()) {
            Ok(value) => {
                assert_eq!(value, _expected, "Context:\n{}", self.runtime.as_ref().unwrap().context.borrow());
            }
            Err(error) => {
                panic!("Evaluation failed: {}\nExpression: {:?}", error, _expr);
            }
        }
    }
}

#[test]
fn test_first() {
    // no-op: ensures test harness initializes cleanly
}
