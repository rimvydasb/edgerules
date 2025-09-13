#[test]
fn test_string_functions() {
    assert_eq!(crate::eval_value("value : 'hello'"), "'hello'");
    assert_eq!(
        crate::eval_value("value : substring(\"foobar\", 3)"),
        "'obar'"
    );
    assert_eq!(
        crate::eval_value("value : substring(\"foobar\", -3, 2)"),
        "'ba'"
    );
    assert_eq!(
        crate::eval_value("value : substring(\"abc\", 1, 2)"),
        "'ab'"
    );
    assert_eq!(crate::eval_value("value : length(\"foo\")"), "3");
    assert_eq!(crate::eval_value("value : length(\"\")"), "0");
    assert_eq!(crate::eval_value("value : toUpperCase(\"aBc4\")"), "'ABC4'");
    assert_eq!(crate::eval_value("value : toLowerCase(\"aBc4\")"), "'abc4'");
    assert_eq!(
        crate::eval_value("value : substringBefore(\"foobar\", \"bar\")"),
        "'foo'"
    );
    assert_eq!(
        crate::eval_value("value : substringAfter(\"foobar\", \"ob\")"),
        "'ar'"
    );
    assert_eq!(
        crate::eval_value("value : contains(\"foobar\", \"of\")"),
        "false"
    );
    assert_eq!(
        crate::eval_value("value : startsWith(\"foobar\", \"fo\")"),
        "true"
    );
    assert_eq!(
        crate::eval_value("value : endsWith(\"foobar\", \"r\")"),
        "true"
    );
    assert_eq!(
        crate::eval_value("value : split(\"John Doe\", \" \")"),
        "['John', 'Doe']"
    );
    assert_eq!(
        crate::eval_value("value : split(\"a-b-c\", \"-\")"),
        "['a', 'b', 'c']"
    );
    assert_eq!(
        crate::eval_value("value : regexSplit('one   two\tthree', '\\s+')"),
        "['one', 'two', 'three']"
    );
    assert_eq!(crate::eval_value("value : trim(\"  hello  \")"), "'hello'");
    assert_eq!(
        crate::eval_value("value : toBase64(\"FEEL\")"),
        "'RkVFTA=='"
    );
    assert_eq!(
        crate::eval_value("value : fromBase64(\"RkVFTA==\")"),
        "'FEEL'"
    );
    assert_eq!(
        crate::eval_value("value : replace(\"abcd\", \"ab\", \"xx\")"),
        "'xxcd'"
    );
    assert_eq!(
        crate::eval_value("value : replace(\"Abcd\", \"ab\", \"xx\", \"i\")"),
        "'xxcd'"
    );
    assert_eq!(
        crate::eval_value("value : regexReplace('Abcd', '[a-z]', 'x', 'i')"),
        "'xxxx'"
    );
    assert_eq!(
        crate::eval_value("value : regexReplace('2025-09-02', '\\d', '#')"),
        "'####-##-##'"
    );
    assert_eq!(
        crate::eval_value("value : replaceFirst(\"foo bar foo\", \"foo\", \"baz\")"),
        "'baz bar foo'"
    );
    assert_eq!(
        crate::eval_value("value : replaceLast(\"foo bar foo\", \"foo\", \"baz\")"),
        "'foo bar baz'"
    );
    assert_eq!(crate::eval_value("value : charAt(\"Abcd\", 2)"), "'c'");
    assert_eq!(crate::eval_value("value : charCodeAt(\"Abcd\", 2)"), "99");
    assert_eq!(crate::eval_value("value : indexOf(\"Abcd\", \"b\")"), "1");
    assert_eq!(
        crate::eval_value("value : lastIndexOf(\"Abcb\", \"b\")"),
        "3"
    );
    assert_eq!(
        crate::eval_value("value : fromCharCode(99, 100, 101)"),
        "'cde'"
    );
    assert_eq!(
        crate::eval_value("value : padStart(\"7\", 3, \"0\")"),
        "'007'"
    );
    assert_eq!(
        crate::eval_value("value : padEnd(\"7\", 3, \"0\")"),
        "'700'"
    );
    assert_eq!(crate::eval_value("value : repeat(\"ab\", 3)"), "'ababab'");
    assert_eq!(crate::eval_value("value : reverse(\"abc\")"), "'cba'");
    assert_eq!(
        crate::eval_value("value : sanitizeFilename(\"a/b\\\\c:d*e?fg<h>ij\")"),
        "'abcdefghij'"
    );
    assert_eq!(
        crate::eval_value("value : interpolate(\"Hi ${name}\", { name : \"Ana\" })"),
        "'Hi Ana'"
    );
}

#[test]
fn test_string_concatenation_with_plus() {
    assert_eq!(crate::eval_value("value : \"a\" + \"b\""), "'ab'");
    assert_eq!(
        crate::eval_value("value : \"a\" + \"b\" + toString(1)"),
        "'ab1'"
    );
    assert_eq!(crate::eval_value("value : toString(1) + \"a\""), "'1a'");
}

#[test]
fn test_concat_left_side_must_be_string_error() {
    crate::link_error_contains(
        "{ a: 1; result: a + \"z\" }",
        &["left side", "string", "+"],
    );
}
mod utilities;
pub use utilities::*;
