use rstest::rstest;

mod utilities;
pub use utilities::*;

// ============================================================================
// Parameterized String Tests - Basic Functions
// ============================================================================

#[rstest]
#[case("'hello'", "'hello'")]
#[case("length('foo')", "3")]
#[case("length('')", "0")]
#[case("toUpperCase('aBc4')", "'ABC4'")]
#[case("toLowerCase('aBc4')", "'abc4'")]
#[case("trim('  hello  ')", "'hello'")]
fn test_string_basic_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Substring Operations
// ============================================================================

#[rstest]
#[case("substring('foobar', 3)", "'obar'")]
#[case("substring('foobar', -3, 2)", "'ba'")]
#[case("substring('abc', 1, 2)", "'ab'")]
#[case("substringBefore('foobar', 'bar')", "'foo'")]
#[case("substringAfter('foobar', 'ob')", "'ar'")]
fn test_string_substring_operations(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Search and Contains
// ============================================================================

#[rstest]
#[case("contains('foobar', 'of')", "false")]
#[case("startsWith('foobar', 'fo')", "true")]
#[case("endsWith('foobar', 'r')", "true")]
#[case("indexOf('Abcd', 'b')", "1")]
#[case("lastIndexOf('Abcb', 'b')", "3")]
fn test_string_search_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Split Operations
// ============================================================================

#[rstest]
#[case("split('John Doe', ' ')", "['John', 'Doe']")]
#[case("split('a-b-c', '-')", "['a', 'b', 'c']")]
#[case("regexSplit('one   two\tthree', '\\s+')", "['one', 'two', 'three']")]
fn test_string_split_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Replace Operations
// ============================================================================

#[rstest]
#[case("replace('abcd', 'ab', 'xx')", "'xxcd'")]
#[case("replace('Abcd', 'ab', 'xx', 'i')", "'xxcd'")]
#[case("regexReplace('Abcd', '[a-z]', 'x', 'i')", "'xxxx'")]
#[case("regexReplace('2025-09-02', '\\d', '#')", "'####-##-##'")]
#[case("replaceFirst('foo bar foo', 'foo', 'baz')", "'baz bar foo'")]
#[case("replaceLast('foo bar foo', 'foo', 'baz')", "'foo bar baz'")]
fn test_string_replace_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Character Operations
// ============================================================================

#[rstest]
#[case("charAt('Abcd', 2)", "'c'")]
#[case("charCodeAt('Abcd', 2)", "99")]
#[case("fromCharCode(99, 100, 101)", "'cde'")]
fn test_string_char_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Padding and Repeat
// ============================================================================

#[rstest]
#[case("padStart('7', 3, '0')", "'007'")]
#[case("padEnd('7', 3, '0')", "'700'")]
#[case("repeat('ab', 3)", "'ababab'")]
fn test_string_padding_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Encoding and Misc
// ============================================================================

#[rstest]
#[case("toBase64('FEEL')", "'RkVFTA=='")]
#[case("fromBase64('RkVFTA==')", "'FEEL'")]
#[case("reverse('abc')", "'cba'")]
#[case("sanitizeFilename('a/b\\\\c:d*e?fg<h>ij')", "'abcdefghij'")]
#[case("interpolate('Hi ${name}', { name : 'Ana' })", "'Hi Ana'")]
fn test_string_encoding_misc(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Concatenation
// ============================================================================

#[rstest]
#[case("'a' + 'b'", "'ab'")]
#[case("'a' + 'b' + toString(1)", "'ab1'")]
#[case("toString(1) + 'a'", "'1a'")]
fn test_string_concatenation_with_plus(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Equality
// ============================================================================

#[rstest]
#[case("'a' = 'a'", "true")]
#[case("'a' <> 'b'", "true")]
fn test_string_equality(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized String Tests - Invalid Comparators
// ============================================================================

#[rstest]
#[case("value: 'a' < 'b'", "Operation '<' not supported for types 'string' and 'string'")]
#[case("value: 'a' <= 'a'", "Operation '<=' not supported for types 'string' and 'string'")]
#[case("value: 'b' > 'a'", "Operation '>' not supported for types 'string' and 'string'")]
#[case("value: 'b' >= 'b'", "Operation '>=' not supported for types 'string' and 'string'")]
fn test_string_invalid_comparators(#[case] code: &str, #[case] expected_error: &str) {
    link_error_contains(code, &[expected_error]);
}

#[test]
fn test_concat_left_side_must_be_string_error() {
    link_error_contains("{ a: 1; result: a + 'z' }", &["left side", "string", "+"]);
}
