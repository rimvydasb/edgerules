use rstest::rstest;

mod utilities;
pub use utilities::*;

// ============================================================================
// Parameterized Logic Tests - Comparisons
// ============================================================================

#[rstest]
#[case("1 = 2", "false")]
#[case("1 < 2", "true")]
#[case("1 <= 2", "true")]
#[case("2 > 1", "true")]
#[case("2 >= 1", "true")]
#[case("1 = 1", "true")]
#[case("1 = 1 + 1", "false")]
fn test_numeric_comparisons(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - Boolean ops with numbers in conditionals
// ============================================================================

#[rstest]
#[case("1 = 2 and 5 = 5", "false")]
#[case("1 + 1 = 2 and 5 = 5", "true")]
#[case("1 = 2 or 5 = 5", "true")]
#[case("1 = 2 or 5 = 5 + 1", "false")]
#[case("1 = 2 xor 5 = 5 + 1", "false")]
#[case("1 = 2 xor 5 = 4 + 1", "true")]
#[case("1 = 2 - 1 xor 5 = 5 + 1", "true")]
#[case("1 = 2 or 5 = 5 and 1 = 1", "true")]
#[case("1 = 2 or 5 = 5 and 1 = 1 + 1", "false")]
fn test_boolean_ops_with_comparisons(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - If-Then-Else
// ============================================================================

#[rstest]
#[case("if 1 > 2 then 3 else 4", "4")]
#[case("if 1 < 2 then 3 else 4", "3")]
#[case("if 1 < 2 then 3 + 1 else 5", "4")]
#[case("if 1 > 2 then 3 + 1 else 5 * 10", "50")]
#[case("if 1 > 2 then 3 + 1 else (if 1 < 2 then 5 * 10 else 0)", "50")]
#[case("if 1 > 2 then 3 + 1 else (if 1 > 2 then 5 * 10 else 0)", "0")]
#[case("if 1 < 2 then (if 5 > 2 then 5 * 10 else 0) else 1", "50")]
#[case("(if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1", "51")]
#[case("1 + (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1", "52")]
#[case("2 * (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1", "101")]
fn test_if_then_else(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - Boolean Literals Comparators
// ============================================================================

#[rstest]
#[case("true = true", "true")]
#[case("true = false", "false")]
#[case("true <> true", "false")]
#[case("true <> false", "true")]
fn test_boolean_literals_comparators(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

#[rstest]
#[case("true = (1 = 1)", "true")]
#[case("false = (1 = 2)", "true")]
fn test_boolean_expression_comparisons(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

#[rstest]
#[case("value: true < false", "operation '<' not supported for types 'boolean' and 'boolean'")]
#[case("value: true <= false", "operation '<=' not supported for types 'boolean' and 'boolean'")]
#[case("value: true > false", "operation '>' not supported for types 'boolean' and 'boolean'")]
#[case("value: true >= false", "operation '>=' not supported for types 'boolean' and 'boolean'")]
fn test_boolean_comparators_unsupported(#[case] code: &str, #[case] expected_error: &str) {
    init_logger();
    link_error_contains(code, &[expected_error]);
}

// ============================================================================
// Parameterized Logic Tests - Boolean OR Truth Table
// ============================================================================

#[rstest]
#[case("true  or true", "true")]
#[case("true  or false", "true")]
#[case("false or true", "true")]
#[case("false or false", "false")]
fn test_boolean_or(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - Boolean AND Truth Table
// ============================================================================

#[rstest]
#[case("true  and true", "true")]
#[case("true  and false", "false")]
#[case("false and true", "false")]
#[case("false and false", "false")]
fn test_boolean_and(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - Boolean XOR Truth Table
// ============================================================================

#[rstest]
#[case("true  xor true", "false")]
#[case("true  xor false", "true")]
#[case("false xor true", "true")]
#[case("false xor false", "false")]
fn test_boolean_xor(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - Boolean NOT
// ============================================================================

#[rstest]
#[case("not true", "false")]
#[case("not false", "true")]
#[case("not (1 = 1)", "false")]
#[case("not (1 = 2)", "true")]
fn test_boolean_not(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - Mixed Boolean Expressions
// ============================================================================

#[rstest]
#[case("true and (1 < 2)", "true")]
#[case("(1 = 1) and false", "false")]
#[case("(1 = 1) or false", "true")]
#[case("true and not false", "true")]
#[case("(1 < 2) and not (2 < 1)", "true")]
fn test_boolean_mixed(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Logic Tests - Complex Boolean Expressions
// ============================================================================

#[rstest]
#[case("(true and (1 < 2)) or (false and (3 = 4))", "true")]
#[case("(true xor (1 = 1 and false)) or (2 < 1)", "true")]
#[case("(true and true) xor (false or (1 < 1))", "true")]
#[case("(true and (2 > 1 and (3 > 2))) and (false or (5 = 5))", "true")]
fn test_boolean_complex(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}
