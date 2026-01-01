
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
    // -2^2 -> -4 (standard math convention: -(2^2)) or (-2)^2 -> 4?
    // In many languages -2^2 is -4, but let's check our parser.
    // If Unary is tighter than Power, it's (-2)^2 = 4.
    // If Power is tighter than Unary, it's -(2^2) = -4.
    // Based on table: Unary (4) > Power (5). So (-2)^2 = 4.
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
    assert_value!("1 = 1 or 2 = 2 and 3 = 4", "true"); // or has lower prec? Usually AND > OR.
    // If AND > OR: true OR (true AND false) -> true OR false -> true.
    // If OR > AND: (true OR true) AND false -> true AND false -> false.
    // Let's verify AND/OR precedence if implicit.
    // Typically AND binds tighter.
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
