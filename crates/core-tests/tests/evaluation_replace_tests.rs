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

mod utilities;
pub use utilities::*;
