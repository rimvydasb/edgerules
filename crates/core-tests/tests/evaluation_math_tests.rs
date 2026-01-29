use rstest::rstest;

mod utilities;
pub use utilities::*;

// ============================================================================
// Parameterized Math Tests - Absolute Value
// ============================================================================

#[rstest]
#[case("abs(10)", "10")]
#[case("abs(-10)", "10")]
#[case("abs(0)", "0")]
#[case("abs(-1.5)", "1.5")]
fn test_math_abs(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Rounding Functions (floor, ceiling, trunc)
// ============================================================================

#[rstest]
// floor: rounds toward negative infinity
#[case("floor(1.1)", "1")]
#[case("floor(1.9)", "1")]
#[case("floor(-1.1)", "-2")]
#[case("floor(-1.9)", "-2")]
// ceiling: rounds toward positive infinity
#[case("ceiling(1.1)", "2")]
#[case("ceiling(1.9)", "2")]
#[case("ceiling(-1.1)", "-1")]
#[case("ceiling(-1.9)", "-1")]
// trunc: rounds toward zero
#[case("trunc(1.1)", "1")]
#[case("trunc(1.9)", "1")]
#[case("trunc(-1.1)", "-1")]
#[case("trunc(-1.9)", "-1")]
fn test_math_rounding_basic(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Round (banker's rounding)
// ============================================================================

#[rstest]
// Banker's rounding (round half to even)
#[case("round(2.5)", "2")]
#[case("round(3.5)", "4")]
#[case("round(1.2)", "1")]
#[case("round(1.8)", "2")]
// With digits
#[case("round(1.2345, 2)", "1.23")]
#[case("round(1.235, 2)", "1.24")]
// Negative digits (rounding to tens, etc)
#[case("round(123, -1)", "120")]
#[case("round(125, -1)", "120")]
#[case("round(135, -1)", "140")]
fn test_math_round(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - RoundUp (away from zero)
// ============================================================================

#[rstest]
// Away from zero
#[case("roundUp(1.1)", "2")]
#[case("roundUp(1.9)", "2")]
#[case("roundUp(-1.1)", "-2")]
#[case("roundUp(-1.9)", "-2")]
// With digits
#[case("roundUp(1.11, 1)", "1.2")]
#[case("roundUp(1.19, 1)", "1.2")]
#[case("roundUp(-1.11, 1)", "-1.2")]
fn test_math_round_up(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - RoundDown (toward zero)
// ============================================================================

#[rstest]
// Toward zero (same as trunc)
#[case("roundDown(1.1)", "1")]
#[case("roundDown(1.9)", "1")]
#[case("roundDown(-1.1)", "-1")]
#[case("roundDown(-1.9)", "-1")]
// With digits
#[case("roundDown(1.19, 1)", "1.1")]
#[case("roundDown(-1.19, 1)", "-1.1")]
fn test_math_round_down(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Division (modulo, idiv)
// ============================================================================

#[rstest]
// modulo: sign matches divisor
#[case("modulo(10, 3)", "1")]
#[case("modulo(10, -3)", "-2")]
#[case("modulo(-10, 3)", "2")]
#[case("modulo(-10, -3)", "-1")]
// idiv: floor division
#[case("idiv(10, 3)", "3")]
#[case("idiv(10, -3)", "-4")]
#[case("idiv(-10, 3)", "-4")]
#[case("idiv(-10, -3)", "3")]
fn test_math_division(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Division by Zero
// ============================================================================

#[rstest]
#[case("modulo(10, 0)", "[runtime] Division by zero")]
#[case("idiv(10, 0)", "[runtime] Division by zero")]
#[case("10 / 0", "[runtime] Division by zero")]
#[case("10 % 0", "[runtime] Division by zero")]
fn test_math_division_by_zero(#[case] expression: &str, #[case] expected_error: &str) {
    init_logger();
    let result = eval_value(&format!("value : {}", expression));
    assert_string_contains!(expected_error, result);
}

// ============================================================================
// Parameterized Math Tests - Square Root
// ============================================================================

#[rstest]
#[case("sqrt(4)", "2")]
#[case("sqrt(2.25)", "1.5")]
#[case("sqrt(0)", "0")]
#[case("sqrt(-1)", "NotApplicable('sqrt of negative number')")]
fn test_math_sqrt(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Clamp
// ============================================================================

#[rstest]
#[case("clamp(5, 0, 10)", "5")]
#[case("clamp(-5, 0, 10)", "0")]
#[case("clamp(15, 0, 10)", "10")]
#[case("clamp(5, 10, 0)", "0")]
fn test_math_clamp(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Basic Arithmetic
// ============================================================================

#[rstest]
#[case("1 + 2", "3")]
#[case("10 - 4", "6")]
#[case("2 * 3", "6")]
#[case("10 / 2", "5")]
#[case("10.5 + 0.5", "11")]
#[case("1 + 2 * 3", "7")]
#[case("(1 + 2) * 3", "9")]
fn test_math_arithmetic(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Power
// ============================================================================

#[rstest]
#[case("2 ^ 3", "8")]
#[case("4 ^ 0.5", "2")]
#[case("2 ^ -1", "0.5")]
#[case("(-2) ^ 2", "4")]
#[case("(-2) ^ 3", "-8")]
#[case("10 ^ 0", "1")]
fn test_math_power(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Unary Minus
// ============================================================================

#[rstest]
#[case("-5", "-5")]
#[case("- -5", "5")]
#[case("5 + -3", "2")]
#[case("- (2 + 3)", "-5")]
fn test_math_unary_minus(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Precedence
// ============================================================================

#[rstest]
// Parentheses
#[case("(1 + 2) * 3", "9")]
// Function call precedence
#[case("abs(-5) + 1", "6")]
#[case("2 * abs(-5)", "10")]
// Unary minus (DMN FEEL: higher precedence than exponentiation)
#[case("-2 ^ 2", "4")]
// Power
#[case("2 * 3 ^ 2", "18")]
// Multiply/Divide
#[case("1 + 2 * 3", "7")]
// Add/Subtract (left associative)
#[case("10 - 5 + 2", "7")]
// Comparators
#[case("1 + 2 = 3", "true")]
#[case("2 ^ 3 > 5", "true")]
// Logical not
#[case("not 1 = 2", "true")]
#[case("not (1 = 2)", "true")]
// Logical operators
#[case("1 = 1 or 2 = 2 and 3 = 4", "true")]
fn test_math_precedence(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Rounding Edge Cases
// ============================================================================

#[rstest]
// Rounding to negative digits (tens, hundreds)
#[case("round(12345, -2)", "12300")]
#[case("round(12350, -2)", "12400")]
// RoundUp negative numbers
#[case("roundUp(-123.45, 1)", "-123.5")]
// RoundDown negative numbers (toward zero)
#[case("roundDown(-123.45, 1)", "-123.4")]
fn test_math_rounding_edge_cases(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Advanced Functions (constants, exp, log, trig)
// ============================================================================

#[rstest]
// Constants
#[case("pi()", "3.141592653589793115997963467")]
// Exponential / Logarithmic
#[case("exp(1)", "2.7182818261984928651595318263")]
#[case("ln(2.718281828459045)", "0.9999999999999999134157889712")]
#[case("log10(100)", "2")]
#[case("log10(1000)", "3")]
#[case("log10(0.01)", "-2")]
// Trigonometry (Radians)
#[case("sin(0)", "0")]
#[case("sin(pi()/2)", "1")]
#[case("cos(0)", "1")]
#[case("cos(pi())", "-1")]
#[case("tan(0)", "0")]
// Inverse Trig
#[case("asin(0)", "0")]
#[case("asin(1)", "1.5707963267948965579989817335")]
#[case("acos(1)", "0")]
#[case("acos(0)", "1.5707963267948965579989817335")]
#[case("atan(0)", "0")]
#[case("atan(1)", "0.7853981633974482789994908668")]
#[case("atan2(0, 1)", "0")]
#[case("atan2(1, 0)", "1.5707963267948965579989817335")]
#[case("atan2(1, 1)", "0.7853981633974482789994908668")]
// Conversions
#[case("degrees(pi())", "180")]
#[case("degrees(pi()/2)", "90")]
#[case("radians(180)", "3.141592653589793115997963467")]
#[case("radians(90)", "1.5707963267948965579989817335")]
fn test_math_advanced(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Invalid/Edge cases for advanced functions
// ============================================================================

#[rstest]
#[case("ln(0)", "NotApplicable('ln of non-positive number')")]
#[case("ln(-1)", "NotApplicable('ln of non-positive number')")]
#[case("log10(0)", "NotApplicable('log10 of non-positive number')")]
#[case("asin(2)", "NotApplicable('asin input out of range [-1, 1]')")]
#[case("acos(-2)", "NotApplicable('acos input out of range [-1, 1]')")]
fn test_math_advanced_invalid(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

#[test]
fn test_math_tan_near_one() {
    // tan(pi/4) should be approximately 1
    init_logger();
    assert_value!("tan(pi()/4) > 0.999999 and tan(pi()/4) < 1.000001", "true");
}

// ============================================================================
// Parameterized Math Tests - Floats and Mixed Types
// ============================================================================

#[rstest]
#[case("1.1 + 2", "3.1")]
#[case("1.1 + 2.1", "3.2")]
#[case("1.0 + 2", "3")]
#[case("-1 + 2", "1")]
#[case("-2 + 1", "-1")]
#[case("2 / 3", "0.6666666666666666666666666667")]
#[case("1 * 2 / 3 + 1 - 2", "-0.3333333333333333333333333333")]
fn test_math_floats_and_mixed(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized Math Tests - Sum Variants
// ============================================================================

#[rstest]
#[case("sum(1,2,3) + (2 * 2)", "10")]
#[case("sum([1,2,3]) + 1", "7")]
#[case("sum([1.0,2.0,3.0]) + 1", "7")]
#[case("sum([1,2.1,3]) + 1", "7.1")]
#[case("sum([duration('PT6H'),duration('PT12H')])", "PT18H")]
fn test_functions_sum_variants(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

#[test]
fn test_functions_sum_nested() {
    // This test uses a nested sum expression that's more complex
    assert_eq!(eval_field("value: sum(1,2,3 + sum(2,2 * sum(0,1,0,0))) + (2 * 2)", "value"), "14");
}

// ============================================================================
// Math Tests - Complex Discount Calculation (kept as single test due to shared setup)
// ============================================================================

#[test]
fn test_complex_discount_calculation() {
    init_logger();
    let code = r#"
        {
            func calculateDiscount(productType): {
                availableDiscounts: [0.20, 0.10, 0.16]
                activeCampaignDiscount: 0.05
                activeCampaign: "SUMMER_SALE"
                baseDiscount: availableDiscounts[productType - 1]
                return: {
                    campaign: activeCampaign
                    discount: baseDiscount + activeCampaignDiscount
                }
            }
            discount1: calculateDiscount(1)
            discount2: calculateDiscount(2)
            discount3: calculateDiscount(3)
        }
    "#;

    assert_eq!(inline(eval_field(code, "discount1")), inline("discount1: {campaign: 'SUMMER_SALE' discount: 0.25}"));
    assert_eq!(inline(eval_field(code, "discount2")), inline("discount2: {campaign: 'SUMMER_SALE' discount: 0.15}"));
    assert_eq!(inline(eval_field(code, "discount3")), inline("discount3: {campaign: 'SUMMER_SALE' discount: 0.21}"));
}

// ============================================================================
// Parameterized Math Tests - Limits and Precision
// ============================================================================

#[rstest]
// High precision division (default scale 28)
#[case("1 / 3", "0.3333333333333333333333333333")]
// Exact arithmetic (no float artifacts)
#[case("0.1 + 0.2", "0.3")]
// Max Decimal (2^96 - 1)
#[case("79228162514264337593543950335", "79228162514264337593543950335")]
// Very small number
#[case("0.0000000000000000000000000001", "0.0000000000000000000000000001")]
// Power with decimal exponent (sqrt(2))
#[case("2 ^ 0.5", "1.4142135623730951454746218583")]
// Complex calculation preserving precision
#[case("(1 / 3) * 3", "0.9999999999999999999999999999")]
fn test_math_limits(#[case] expression: &str, #[case] expected: &str) {
    init_logger();
    assert_value!(expression, expected);
}
