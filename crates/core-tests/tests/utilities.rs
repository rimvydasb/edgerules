#![allow(dead_code)]

use std::sync::Once;

use edge_rules::runtime::edge_rules::EdgeRulesModel;
use edge_rules::runtime::edge_rules::EdgeRulesRuntime;
use edge_rules::test_support::LinkingErrorEnum;
use edge_rules::test_support::ValueType;
use env_logger::Builder;

pub fn inline<S: AsRef<str>>(code: S) -> String {
    code.as_ref().replace('\n', " ").replace(" ", "")
}

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
                assert_eq!(inline($crate::eval_value(&format!("value : {}", body))), inline($expected), "for body: {:?}", body);
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
        assert!($haystack.contains($needle), "expected `{}` to contain `{}`", $haystack, $needle);
    };
}

pub fn exe_field(runtime: &EdgeRulesRuntime, path: &str) -> String {
    inline(runtime.evaluate_field(path).unwrap().to_string())
}

pub fn get_runtime(code: &str) -> EdgeRulesRuntime {
    init_logger();
    let mut service = EdgeRulesModel::new();
    match service.append_source(&wrap_in_object(code)) {
        Ok(()) => match service.to_runtime() {
            Ok(runtime) => runtime,
            Err(err) => panic!("link error: {err}\ncode:\n{code}"),
        },
        Err(err) => panic!("parse error: {err}\ncode:\n{code}"),
    }
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
    let mut service = EdgeRulesModel::new();
    match service.append_source(&wrap_in_object(code)) {
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

pub fn init_logger() {
    static LOGGER: Once = Once::new();

    LOGGER.call_once(|| {
        let _ = Builder::from_default_env().is_test(true).try_init();
    });
}

pub fn wrap_in_object(lines: &str) -> String {
    if lines.trim().starts_with('{') && lines.trim().ends_with('}') {
        return lines.trim().to_string();
    }

    format!("{{{}}}", lines.trim())
}

pub fn assert_eval_all(lines: &str, expected_lines: &[&str]) {
    let model = wrap_in_object(lines);
    let evaluated = eval_all(&model);
    assert_eq!(
        evaluated.lines().map(|l| inline(l.trim())).collect::<Vec<_>>(),
        expected_lines.iter().map(|l| inline(l.trim())).collect::<Vec<_>>()
    );
}

/// For tests that must assert link errors (e.g., cyclic/self ref, missing field).
pub fn link_error_contains(code: &str, needles: &[&str]) {
    init_logger();
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(code);

    match service.to_runtime() {
        Ok(ok_but_unexpected) => {
            let static_tree = ok_but_unexpected.static_tree.borrow().to_string();
            println!("static_tree:\n{}\n", static_tree);
            panic!("expected link error, got none\ncode:\n{code}");
        }
        Err(err) => {
            let lower = err.to_string().to_lowercase();
            for n in needles {
                assert!(
                    lower.contains(&n.to_lowercase()),
                    "expected error to contain `{n}`, got: {err}\ncode:\n{code}"
                );
            }
        }
    }
}

pub fn link_error_location(
    code: &str,
    expected_location: &[&str],
    expected_expression: &str,
    error: LinkingErrorEnum,
) -> Vec<String> {
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(code);

    match service.to_runtime() {
        Ok(_) => panic!("expected link error, got none\ncode:\n{code}"),
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
pub fn parse_error_contains(code: &str, needles: &[&str]) {
    let mut service = EdgeRulesModel::new();
    let err = service.append_source(code);

    match err.err().map(|e| e.to_string()) {
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
    text.lines().map(|line| line.trim()).filter(|line| !line.is_empty()).collect()
}

// ============================================================================
// Test Builders - ExpressionTest
// ============================================================================

/// A fluent builder for testing multiple expression cases with the same function.
///
/// # Example
///
/// ```ignore
/// ExpressionTest::new("abs")
///     .case("abs(10)", "10")
///     .case("abs(-10)", "10")
///     .case("abs(0)", "0")
///     .run_all();
/// ```
pub struct ExpressionTest {
    function_name: String,
    cases: Vec<(String, String)>,
}

impl ExpressionTest {
    /// Creates a new ExpressionTest for the given function name.
    pub fn new(function_name: &str) -> Self {
        init_logger();
        Self { function_name: function_name.to_string(), cases: Vec::new() }
    }

    /// Adds a test case with the given expression and expected result.
    pub fn case(mut self, expression: &str, expected: &str) -> Self {
        self.cases.push((expression.to_string(), expected.to_string()));
        self
    }

    /// Runs all test cases and panics on the first failure.
    pub fn run_all(&self) {
        for (expression, expected) in &self.cases {
            let actual = eval_value(&format!("value: {}", expression));
            assert_eq!(
                actual,
                expected.as_str(),
                "Function '{}' failed for expression '{}': got '{}', expected '{}'",
                self.function_name,
                expression,
                actual,
                expected
            );
        }
    }

    /// Returns the number of test cases.
    pub fn case_count(&self) -> usize {
        self.cases.len()
    }
}

// ============================================================================
// Test Builders - UnaryFunctionValidator
// ============================================================================

/// A fluent builder for validating unary function argument requirements.
///
/// # Example
///
/// ```ignore
/// UnaryFunctionValidator::for_number_functions(&["abs", "floor", "ceiling"])
///     .expect_parse_error_when_no_args()
///     .expect_link_error_when_wrong_type("'abc'", ValueType::StringType)
///     .validate();
/// ```
pub struct UnaryFunctionValidator {
    function_names: Vec<String>,
    expected_arg_type: Option<ValueType>,
    check_no_args: bool,
    wrong_type_input: Option<(String, ValueType)>,
}

impl UnaryFunctionValidator {
    /// Creates a validator for functions that expect number arguments.
    pub fn for_number_functions(functions: &[&str]) -> Self {
        init_logger();
        Self {
            function_names: functions.iter().map(|s| s.to_string()).collect(),
            expected_arg_type: Some(ValueType::NumberType),
            check_no_args: false,
            wrong_type_input: None,
        }
    }

    /// Creates a validator for functions that expect string arguments.
    pub fn for_string_functions(functions: &[&str]) -> Self {
        init_logger();
        Self {
            function_names: functions.iter().map(|s| s.to_string()).collect(),
            expected_arg_type: Some(ValueType::StringType),
            check_no_args: false,
            wrong_type_input: None,
        }
    }

    /// Creates a validator for functions that expect list arguments.
    pub fn for_list_functions(functions: &[&str]) -> Self {
        init_logger();
        Self {
            function_names: functions.iter().map(|s| s.to_string()).collect(),
            expected_arg_type: Some(ValueType::ListType(None)),
            check_no_args: false,
            wrong_type_input: None,
        }
    }

    /// Configures the validator to check for parse errors when no arguments are provided.
    pub fn expect_parse_error_when_no_args(mut self) -> Self {
        self.check_no_args = true;
        self
    }

    /// Configures the validator to check for link errors when wrong type is provided.
    pub fn expect_link_error_when_wrong_type(mut self, input: &str, wrong_type: ValueType) -> Self {
        self.wrong_type_input = Some((input.to_string(), wrong_type));
        self
    }

    /// Validates all configured checks for all functions.
    pub fn validate(&self) {
        for func in &self.function_names {
            if self.check_no_args {
                let code = format!("{{ value: {}() }}", func);
                parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
            }

            if let Some((input, wrong_type)) = &self.wrong_type_input {
                let code = format!("{{ value: {}({}) }}", func, input);
                if let Some(expected_type) = &self.expected_arg_type {
                    link_error_location(
                        &code,
                        &["value"],
                        &format!("{}({})", func, input),
                        LinkingErrorEnum::TypesNotCompatible(
                            None,
                            wrong_type.clone(),
                            Some(vec![expected_type.clone()]),
                        ),
                    );
                }
            }
        }
    }
}

// ============================================================================
// Test Builders - ErrorTestBuilder
// ============================================================================

/// A fluent builder for testing error conditions.
///
/// # Example
///
/// ```ignore
/// ErrorTestBuilder::new()
///     .with_code("value: value + 1")
///     .expect_link_error_containing("cyclic")
///     .run();
/// ```
pub struct ErrorTestBuilder {
    code: Option<String>,
    expected_link_errors: Vec<String>,
    expected_parse_errors: Vec<String>,
}

impl ErrorTestBuilder {
    pub fn new() -> Self {
        init_logger();
        Self { code: None, expected_link_errors: Vec::new(), expected_parse_errors: Vec::new() }
    }

    pub fn with_code(mut self, code: &str) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn expect_link_error_containing(mut self, needle: &str) -> Self {
        self.expected_link_errors.push(needle.to_string());
        self
    }

    pub fn expect_parse_error_containing(mut self, needle: &str) -> Self {
        self.expected_parse_errors.push(needle.to_string());
        self
    }

    pub fn run(&self) {
        let code = self.code.as_ref().expect("Code must be set before running");

        if !self.expected_link_errors.is_empty() {
            let needles: Vec<&str> = self.expected_link_errors.iter().map(|s| s.as_str()).collect();
            link_error_contains(code, &needles);
        }

        if !self.expected_parse_errors.is_empty() {
            let needles: Vec<&str> = self.expected_parse_errors.iter().map(|s| s.as_str()).collect();
            parse_error_contains(code, &needles);
        }
    }
}

impl Default for ErrorTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_first() {
    // no-op: ensures test harness initializes cleanly
}

#[test]
fn test_expression_test_builder() {
    ExpressionTest::new("abs")
        .case("abs(10)", "10")
        .case("abs(-10)", "10")
        .case("abs(0)", "0")
        .run_all();
}

#[test]
fn test_error_test_builder() {
    ErrorTestBuilder::new().with_code("value: value + 1").expect_link_error_containing("cyclic").run();
}

#[test]
fn test_unary_function_validator_number_functions() {
    // Test that abs() with no arguments produces parse error
    // and abs('abc') produces link error for wrong type
    UnaryFunctionValidator::for_number_functions(&["abs"])
        .expect_parse_error_when_no_args()
        .expect_link_error_when_wrong_type("'abc'", ValueType::StringType)
        .validate();
}

#[test]
fn test_unary_function_validator_string_functions() {
    // Test that length() with no arguments produces parse error
    // and length(123) produces link error for wrong type
    UnaryFunctionValidator::for_string_functions(&["length"])
        .expect_parse_error_when_no_args()
        .expect_link_error_when_wrong_type("123", ValueType::NumberType)
        .validate();
}
