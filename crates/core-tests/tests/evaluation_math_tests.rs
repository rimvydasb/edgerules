mod utilities;
pub use utilities::*;

#[test]
fn test_math_abs() {
    init_logger();
    assert_value!("abs(10)", "10");
    assert_value!("abs(-10)", "10");
    assert_value!("abs(0)", "0");
    assert_value!("abs(-1.5)", "1.5");
}

#[test]
fn test_math_rounding_basic() {
    init_logger();
    // floor
    assert_value!("floor(1.1)", "1");
    assert_value!("floor(1.9)", "1");
    assert_value!("floor(-1.1)", "-2");
    assert_value!("floor(-1.9)", "-2");

    // ceiling
    assert_value!("ceiling(1.1)", "2");
    assert_value!("ceiling(1.9)", "2");
    assert_value!("ceiling(-1.1)", "-1");
    assert_value!("ceiling(-1.9)", "-1");

    // trunc
    assert_value!("trunc(1.1)", "1");
    assert_value!("trunc(1.9)", "1");
    assert_value!("trunc(-1.1)", "-1");
    assert_value!("trunc(-1.9)", "-1");
}

#[test]
fn test_math_round() {
    init_logger();
    // Banker's rounding
    assert_value!("round(2.5)", "2");
    assert_value!("round(3.5)", "4");
    assert_value!("round(1.2)", "1");
    assert_value!("round(1.8)", "2");

    // With digits
    assert_value!("round(1.2345, 2)", "1.23");
    assert_value!("round(1.235, 2)", "1.24"); // 35 -> 4 (even?) No, 123.5 -> 124

    // Negative digits (rounding to tens, etc)
    assert_value!("round(123, -1)", "120");
    assert_value!("round(125, -1)", "120"); // 12.5 -> 12
    assert_value!("round(135, -1)", "140"); // 13.5 -> 14
}

#[test]
fn test_math_round_up() {
    init_logger();
    // Away from zero
    assert_value!("roundUp(1.1)", "2");
    assert_value!("roundUp(1.9)", "2");
    assert_value!("roundUp(-1.1)", "-2");
    assert_value!("roundUp(-1.9)", "-2");

    // With digits
    assert_value!("roundUp(1.11, 1)", "1.2");
    assert_value!("roundUp(1.19, 1)", "1.2");
    assert_value!("roundUp(-1.11, 1)", "-1.2");
}

#[test]
fn test_math_round_down() {
    init_logger();
    // Toward zero (same as trunc)
    assert_value!("roundDown(1.1)", "1");
    assert_value!("roundDown(1.9)", "1");
    assert_value!("roundDown(-1.1)", "-1");
    assert_value!("roundDown(-1.9)", "-1");

    // With digits
    assert_value!("roundDown(1.19, 1)", "1.1");
    assert_value!("roundDown(-1.19, 1)", "-1.1");
}

#[test]
fn test_math_division() {
    init_logger();
    // modulo: sign matches divisor
    assert_value!("modulo(10, 3)", "1");
    assert_value!("modulo(10, -3)", "-2");
    assert_value!("modulo(-10, 3)", "2");
    assert_value!("modulo(-10, -3)", "-1");

    // idiv: floor division
    assert_value!("idiv(10, 3)", "3");
    assert_value!("idiv(10, -3)", "-4"); // floor( -3.33) -> -4
    assert_value!("idiv(-10, 3)", "-4"); // floor( -3.33) -> -4
    assert_value!("idiv(-10, -3)", "3");
}

#[test]
fn test_math_division_by_zero() {
    init_logger();
    // Functions
    assert_string_contains!(
        "[runtime] Division by zero",
        eval_value("value : modulo(10, 0)")
    );
    assert_string_contains!(
        "[runtime] Division by zero",
        eval_value("value : idiv(10, 0)")
    );

    // Operators
    assert_string_contains!("[runtime] Division by zero", eval_value("value : 10 / 0"));
    assert_string_contains!("[runtime] Division by zero", eval_value("value : 10 % 0"));
}

#[test]
fn test_math_sqrt() {
    init_logger();
    assert_value!("sqrt(4)", "2");
    assert_value!("sqrt(2.25)", "1.5");
    assert_value!("sqrt(0)", "0");

    // Negative -> Invalid (NotApplicable)
    // We expect it to be a SpecialValue.
    // assert_value checks for exact string match of the value.
    // If it returns SV, it prints as "NotApplicable('sqrt of negative number')".
    assert_value!("sqrt(-1)", "NotApplicable('sqrt of negative number')");
}

#[test]
fn test_math_clamp() {
    init_logger();
    assert_value!("clamp(5, 0, 10)", "5");
    assert_value!("clamp(-5, 0, 10)", "0");
    assert_value!("clamp(15, 0, 10)", "10");

    assert_value!("clamp(5, 10, 0)", "0"); // if min > max? Logic: if n < min (5 < 10) -> 10. if 10 > max (10 > 0) -> 0. Returns 0.
}

#[test]
fn test_math_arithmetic() {
    init_logger();
    assert_value!("1 + 2", "3");
    assert_value!("10 - 4", "6");
    assert_value!("2 * 3", "6");
    assert_value!("10 / 2", "5");
    assert_value!("10.5 + 0.5", "11");
    // Order of ops
    assert_value!("1 + 2 * 3", "7");
    assert_value!("(1 + 2) * 3", "9");
}

#[test]
fn test_math_power() {
    init_logger();
    assert_value!("2 ^ 3", "8");
    assert_value!("4 ^ 0.5", "2");
    assert_value!("2 ^ -1", "0.5");
    assert_value!("(-2) ^ 2", "4");
    assert_value!("(-2) ^ 3", "-8");
    assert_value!("10 ^ 0", "1");
}

#[test]
fn test_math_unary_minus() {
    init_logger();
    assert_value!("-5", "-5");
    assert_value!("- -5", "5");
    assert_value!("5 + -3", "2");
    assert_value!("- (2 + 3)", "-5");
}

#[test]
fn test_math_precedence() {
    init_logger();
    // 1. Parentheses `(...)`
    assert_value!("(1 + 2) * 3", "9");

    // 2. Function call `f(...)` - highest precedence after parens
    // abs(-5) -> 5, then + 1 -> 6
    assert_value!("abs(-5) + 1", "6");
    // 2 * 5 -> 10
    assert_value!("2 * abs(-5)", "10");

    // 4. Unary minus `-`
    // -2^2 -> 4.
    // NOTE: This follows DMN FEEL (1.3+) and Excel standards where Unary Minus has HIGHER precedence
    // than Exponentiation (-2)^2 = 4.
    // This differs from Python/Bash/Written Math where it is usually -(2^2) = -4.
    assert_value!("-2 ^ 2", "4");

    // 5. Power `^`
    // 2 * 3 ^ 2 -> 2 * 9 -> 18
    assert_value!("2 * 3 ^ 2", "18");

    // 6. Multiply/Divide `* /`
    // 1 + 2 * 3 -> 7
    assert_value!("1 + 2 * 3", "7");

    // 7. Add/Subtract `+ -`
    // 10 - 5 + 2 -> 7 (left assoc)
    assert_value!("10 - 5 + 2", "7");

    // 8. Comparators `= <> < > <= >=`
    assert_value!("1 + 2 = 3", "true");
    assert_value!("2 ^ 3 > 5", "true");

    // 9. Unary logical `not`
    assert_value!("not 1 = 2", "true");
    assert_value!("not (1 = 2)", "true");

    // 10. Logical `and`, `xor`, `or`
    assert_value!("1 = 1 or 2 = 2 and 3 = 4", "true");
}

#[test]
fn test_math_rounding_edge_cases() {
    init_logger();
    // Rounding to negative digits (tens, hundreds)
    assert_value!("round(12345, -2)", "12300"); // 45 < 50
    assert_value!("round(12350, -2)", "12400"); // 50 -> even? 124 is even. 123.5 -> 124.

    // RoundUp negative numbers
    assert_value!("roundUp(-123.45, 1)", "-123.5");

    // RoundDown negative numbers (toward zero)
    assert_value!("roundDown(-123.45, 1)", "-123.4");
}

#[test]
fn test_math_advanced() {
    init_logger();

    // Constants
    assert_value!("pi()", "3.141592653589793115997963467");

    // Exponential / Logarithmic
    assert_value!("exp(1)", "2.7182818261984928651595318263");
    assert_value!("ln(2.718281828459045)", "0.9999999999999999134157889712");
    assert_value!("log10(100)", "2");
    assert_value!("log10(1000)", "3");
    assert_value!("log10(0.01)", "-2");

    // Trigonometry (Radians)
    assert_value!("sin(0)", "0");
    assert_value!("sin(pi()/2)", "1");
    assert_value!("cos(0)", "1");
    assert_value!("cos(pi())", "-1");
    assert_value!("tan(0)", "0");
    // tan(pi/4) should be 1
    assert_value!("tan(pi()/4) > 0.999999 and tan(pi()/4) < 1.000001", "true");

    // Inverse Trig
    assert_value!("asin(0)", "0");
    assert_value!("asin(1)", "1.5707963267948965579989817335"); // pi/2
    assert_value!("acos(1)", "0");
    assert_value!("acos(0)", "1.5707963267948965579989817335"); // pi/2
    assert_value!("atan(0)", "0");
    assert_value!("atan(1)", "0.7853981633974482789994908668"); // pi/4

    assert_value!("atan2(0, 1)", "0");
    assert_value!("atan2(1, 0)", "1.5707963267948965579989817335"); // pi/2
    assert_value!("atan2(1, 1)", "0.7853981633974482789994908668"); // pi/4

    // Conversions
    assert_value!("degrees(pi())", "180");
    assert_value!("degrees(pi()/2)", "90");
    assert_value!("radians(180)", "3.141592653589793115997963467");
    assert_value!("radians(90)", "1.5707963267948965579989817335");

    // Invalid/Edge cases
    assert_value!("ln(0)", "NotApplicable('ln of non-positive number')");
    assert_value!("ln(-1)", "NotApplicable('ln of non-positive number')");
    assert_value!("log10(0)", "NotApplicable('log10 of non-positive number')");
    assert_value!(
        "asin(2)",
        "NotApplicable('asin input out of range [-1, 1]')"
    );
    assert_value!(
        "acos(-2)",
        "NotApplicable('acos input out of range [-1, 1]')"
    );
}

#[test]
fn test_math_floats_and_mixed() {
    init_logger();
    assert_value!("1.1 + 2", "3.1");
    assert_value!("1.1 + 2.1", "3.2");
    assert_value!("1.0 + 2", "3");
    assert_value!("-1 + 2", "1");
    assert_value!("-2 + 1", "-1");

    assert_value!("2 / 3", "0.6666666666666666666666666667");
    assert_value!("1 * 2 / 3 + 1 - 2", "-0.3333333333333333333333333333");
}

#[test]
fn test_functions_sum_variants() {
    assert_value!("sum(1,2,3) + (2 * 2)", "10");
    assert_eq!(
        eval_field(
            "value: sum(1,2,3 + sum(2,2 * sum(0,1,0,0))) + (2 * 2)",
            "value"
        ),
        "14"
    );
    assert_value!("sum([1,2,3]) + 1", "7");
    assert_value!("sum([1.0,2.0,3.0]) + 1", "7");
    assert_value!("sum([1,2.1,3]) + 1", "7.1");
    assert_value!("sum([duration('PT6H'),duration('PT12H')])", "PT18H");
}

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

    assert_eq!(
        inline(eval_field(code, "discount1")),
        inline("discount1: {campaign: 'SUMMER_SALE' discount: 0.25}")
    );
    assert_eq!(
        inline(eval_field(code, "discount2")),
        inline("discount2: {campaign: 'SUMMER_SALE' discount: 0.15}")
    );
    assert_eq!(
        inline(eval_field(code, "discount3")),
        inline("discount3: {campaign: 'SUMMER_SALE' discount: 0.21}")
    );
}

#[test]
fn test_math_limits() {
    init_logger();

    // High precision division (default scale 28)
    assert_value!("1 / 3", "0.3333333333333333333333333333");

    // Exact arithmetic (no float artifacts)
    assert_value!("0.1 + 0.2", "0.3");

    // Max Decimal (2^96 - 1)
    assert_value!("79228162514264337593543950335", "79228162514264337593543950335");

    // Very small number
    assert_value!("0.0000000000000000000000000001", "0.0000000000000000000000000001");

    // Power with decimal exponent
    // 2^0.5 -> sqrt(2)
    assert_value!("2 ^ 0.5", "1.4142135623730951454746218583");

    // Complex calculation preserving precision
    assert_value!("(1 / 3) * 3", "0.9999999999999999999999999999");
}
