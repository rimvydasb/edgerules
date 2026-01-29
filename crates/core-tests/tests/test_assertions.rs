//! Domain-specific test assertion macros for EdgeRules.
//!
//! This module provides custom assertion macros that improve test readability
//! and reduce boilerplate when testing EdgeRules expressions, errors, and outputs.
//!
//! # Usage
//!
//! Import macros in your test file:
//! ```ignore
//! mod test_assertions;
//! pub use test_assertions::*;
//! ```

mod utilities;
pub use utilities::*;

/// Asserts that evaluating the given expression results in a link error
/// containing the expected message fragments.
///
/// # Examples
///
/// ```ignore
/// assert_link_error!("value: value + 1", "cyclic");
/// assert_link_error!(
///     "{ record1: 15 + record2; record2: 7 + record3; record3: record1 * 10 }",
///     "cyclic", "record1"
/// );
/// ```
#[macro_export]
macro_rules! assert_link_error {
    ($code:expr, $( $needle:expr ),+ $(,)?) => {{
        let needles: &[&str] = &[$($needle),+];
        link_error_contains($code, needles)
    }};
}

/// Asserts that evaluating the given expression results in a parse error
/// containing the expected message fragments.
///
/// # Examples
///
/// ```ignore
/// assert_parse_error!("func badFunc(())", "badFunc");
/// assert_parse_error!("{ type BadType: }", "assignment side is not complete");
/// ```
#[macro_export]
macro_rules! assert_parse_error {
    ($code:expr, $( $needle:expr ),+ $(,)?) => {{
        let needles: &[&str] = &[$($needle),+];
        parse_error_contains($code, needles)
    }};
}

/// Asserts that a runtime error occurs at the specified location
/// with the expected expression.
///
/// # Examples
///
/// ```ignore
/// let runtime = get_runtime("value: date('invalid')");
/// // Note: For runtime errors, the test setup must allow successful linking
/// // but fail at evaluation time.
/// ```
#[macro_export]
macro_rules! assert_runtime_error {
    ($runtime:expr, $field:expr, $expected_location:expr, $expected_expr:expr) => {{
        let err = $runtime.evaluate_field($field).expect_err("expected runtime error");
        assert_eq!(err.location(), $expected_location, "Location mismatch");
        assert_eq!(err.expression().map(|s| s.as_str()), Some($expected_expr), "Expression mismatch");
    }};
}

/// Asserts that two code outputs are equivalent after whitespace normalization.
///
/// # Examples
///
/// ```ignore
/// assert_code_eq!("{ a: 1 }", "{a:1}");
/// ```
#[macro_export]
macro_rules! assert_code_eq {
    ($actual:expr, $expected:expr) => {{
        let actual_normalized = inline($actual);
        let expected_normalized = inline($expected);
        assert_eq!(
            actual_normalized, expected_normalized,
            "Code mismatch:\nActual: {}\nExpected: {}",
            $actual, $expected
        );
    }};
}

/// Asserts that evaluating the expression produces the expected string result.
///
/// This is a simplified version of `assert_value!` for single expression tests.
///
/// # Examples
///
/// ```ignore
/// assert_expr_value!("1 + 2", "3");
/// assert_expr_value!("abs(-5)", "5");
/// ```
#[macro_export]
macro_rules! assert_expr_value {
    ($expr:expr, $expected:expr) => {{
        let actual = eval_value(&format!("value: {}", $expr));
        assert_eq!(actual, $expected, "Expression '{}' evaluated to '{}', expected '{}'", $expr, actual, $expected);
    }};
}

/// Asserts that evaluating the expression produces an error containing the given fragments.
///
/// # Examples
///
/// ```ignore
/// assert_eval_error!("10 / 0", "Division by zero");
/// ```
#[macro_export]
macro_rules! assert_eval_error {
    ($expr:expr, $( $needle:expr ),+ $(,)?) => {{
        let result = eval_value(&format!("value: {}", $expr));
        $(
            assert!(
                result.to_lowercase().contains(&$needle.to_lowercase()),
                "Expected error containing '{}', got: {}",
                $needle, result
            );
        )+
    }};
}

#[test]
fn test_assert_link_error_macro() {
    assert_link_error!("value: value + 1", "cyclic");
}

#[test]
fn test_assert_parse_error_macro() {
    assert_parse_error!("func badFunc(())", "badFunc");
}

#[test]
fn test_assert_code_eq_macro() {
    assert_code_eq!("{ a: 1; b: 2 }", "{a:1;b:2}");
}

#[test]
fn test_assert_expr_value_macro() {
    assert_expr_value!("1 + 2", "3");
    assert_expr_value!("abs(-5)", "5");
}

#[test]
fn test_assert_eval_error_macro() {
    assert_eval_error!("10 / 0", "Division by zero");
}

#[test]
fn test_assert_runtime_error_macro() {
    // Create a runtime with an invalid date that will fail at evaluation time
    let runtime = get_runtime("value: date('invalid-date')");
    let err = runtime.evaluate_field("value").expect_err("expected runtime error");

    // Verify the error has the expected properties
    assert_eq!(err.location(), vec!["value"]);
    assert!(err.expression().is_some());
}
