#[test]
fn test_replace_case_insensitive_basic() {
    // simple ASCII case-insensitive replace
    assert_value!("replace(\"Abcd\", \"ab\", \"xx\", \"i\")", "'xxcd'");
    assert_value!("replace(\"AbCdAb\", \"ab\", \"x\", \"i\")", "'xCdx'");
}

#[test]
fn test_replace_empty_pattern_behavior() {
    // Empty pattern inserts between every boundary and ends (Rust replace semantics)
    assert_value!("replace(\"abc\", \"\", \"-\")", "'-a-b-c-'");
    // For replaceFirst/Last we keep explicit, predictable behavior
    assert_value!("replaceFirst(\"abc\", \"\", \"x\")", "'xabc'");
    assert_value!("replaceLast(\"abc\", \"\", \"x\")", "'abcx'");
}

#[test]
fn test_replace_case_insensitive_non_ascii() {
    // Unicode case-insensitive (regex path) should handle accented letters
    assert_value!("replace(\"Ábcd\", \"á\", \"x\", \"i\")", "'xbcd'");
    assert_value!("replace(\"Ää\", \"ä\", \"x\", \"i\")", "'xx'");
}

#[test]
fn period_addition_overflow_u32() {
    // 200,000,000 years * 12 = 2,400,000,000 months.
    // Fits in u32 (max 4.29B).
    // Sum = 4,800,000,000 months. Overflows u32.
    // This should fail with Error 106 (PeriodTooLarge) in the result construction.
    let evaluated = eval_all("value: period('P200000000Y') + period('P200000000Y')");
    // Message should be "Period value is too large... (Error code: 106)"
    assert!(evaluated.contains("106"), "Expected error code 106, got: {}", evaluated);
    assert!(!evaluated.contains("Period addition overflowed"), "Should not hit i128 overflow check");
}

#[test]
fn period_subtraction_overflow_u32() {
    // -200M years (negative period)
    // -2.4B months.
    // Subtract 2.4B months.
    // Result -4.8B months.
    // Abs(4.8B) > u32::MAX.
    let evaluated = eval_all("value: period('-P200000000Y') - period('P200000000Y')");
    assert!(evaluated.contains("106"), "Expected error code 106, got: {}", evaluated);
    assert!(!evaluated.contains("Period subtraction overflowed"), "Should not hit i128 overflow check");
}

mod utilities;
pub use utilities::*;
