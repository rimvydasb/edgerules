#[test]
fn test_replace_case_insensitive_basic() {
    // simple ASCII case-insensitive replace
    assert_eq!(crate::eval_value("value : replace(\"Abcd\", \"ab\", \"xx\", \"i\")"), "'xxcd'");
    assert_eq!(
        crate::eval_value("value : replace(\"AbCdAb\", \"ab\", \"x\", \"i\")"),
        "'xCdx'"
    );
}

#[test]
fn test_replace_empty_pattern_behavior() {
    // Empty pattern inserts between every boundary and ends (Rust replace semantics)
    assert_eq!(
        crate::eval_value("value : replace(\"abc\", \"\", \"-\")"),
        "'-a-b-c-'"
    );
    // For replaceFirst/Last we keep explicit, predictable behavior
    assert_eq!(
        crate::eval_value("value : replaceFirst(\"abc\", \"\", \"x\")"),
        "'xabc'"
    );
    assert_eq!(
        crate::eval_value("value : replaceLast(\"abc\", \"\", \"x\")"),
        "'abcx'"
    );
}

#[test]
fn test_replace_case_insensitive_non_ascii() {
    // Unicode case-insensitive (regex path) should handle accented letters
    assert_eq!(
        crate::eval_value("value : replace(\"Ábcd\", \"á\", \"x\", \"i\")"),
        "'xbcd'"
    );
    assert_eq!(
        crate::eval_value("value : replace(\"Ää\", \"ä\", \"x\", \"i\")"),
        "'xx'"
    );
}

mod utilities;
pub use utilities::*;

