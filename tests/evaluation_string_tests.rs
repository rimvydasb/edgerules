#[test]
fn test_string_functions() {
    assert_value!("'hello'", "'hello'");
    assert_value!("substring('foobar', 3)", "'obar'");
    assert_value!("substring('foobar', -3, 2)", "'ba'");
    assert_value!("substring('abc', 1, 2)", "'ab'");
    assert_value!("length('foo')", "3");
    assert_value!("length('')", "0");
    assert_value!("toUpperCase('aBc4')", "'ABC4'");
    assert_value!("toLowerCase('aBc4')", "'abc4'");
    assert_value!("substringBefore('foobar', 'bar')", "'foo'");
    assert_value!("substringAfter('foobar', 'ob')", "'ar'");
    assert_value!("contains('foobar', 'of')", "false");
    assert_value!("startsWith('foobar', 'fo')", "true");
    assert_value!("endsWith('foobar', 'r')", "true");
    assert_value!("split('John Doe', ' ')", "['John', 'Doe']");
    assert_value!("split('a-b-c', '-')", "['a', 'b', 'c']");
    assert_value!(
        "regexSplit('one   two\tthree', '\\s+')",
        "['one', 'two', 'three']"
    );
    assert_value!("trim('  hello  ')", "'hello'");
    assert_value!("toBase64('FEEL')", "'RkVFTA=='");
    assert_value!("fromBase64('RkVFTA==')", "'FEEL'");
    assert_value!("replace('abcd', 'ab', 'xx')", "'xxcd'");
    assert_value!("replace('Abcd', 'ab', 'xx', 'i')", "'xxcd'");
    assert_value!("regexReplace('Abcd', '[a-z]', 'x', 'i')", "'xxxx'");
    assert_value!("regexReplace('2025-09-02', '\\d', '#')", "'####-##-##'");
    assert_value!(
        "replaceFirst('foo bar foo', 'foo', 'baz')",
        "'baz bar foo'"
    );
    assert_value!(
        "replaceLast('foo bar foo', 'foo', 'baz')",
        "'foo bar baz'"
    );
    assert_value!("charAt('Abcd', 2)", "'c'");
    assert_value!("charCodeAt('Abcd', 2)", "99");
    assert_value!("indexOf('Abcd', 'b')", "1");
    assert_value!("lastIndexOf('Abcb', 'b')", "3");
    assert_value!("fromCharCode(99, 100, 101)", "'cde'");
    assert_value!("padStart('7', 3, '0')", "'007'");
    assert_value!("padEnd('7', 3, '0')", "'700'");
    assert_value!("repeat('ab', 3)", "'ababab'");
    assert_value!("reverse('abc')", "'cba'");
    assert_value!("sanitizeFilename('a/b\\\\c:d*e?fg<h>ij')", "'abcdefghij'");
    assert_value!(
        "interpolate('Hi ${name}', { name : 'Ana' })",
        "'Hi Ana'"
    );
}

#[test]
fn test_string_concatenation_with_plus() {
    assert_value!("'a' + 'b'", "'ab'");
    assert_value!("'a' + 'b' + toString(1)", "'ab1'");
    assert_value!("toString(1) + 'a'", "'1a'");
}

#[test]
fn test_string_logic() {
    assert_value!("'a' = 'a'", "true");
    assert_value!("'a' <> 'b'", "true");

    // parse_error_contains(r#"{
    //     value: "'a' < 'b'"
    // }"#, &["duplicate function 'calc'"]);
    //
    // assert_value!("'a' < 'b'", "true");
    // assert_value!("'a' <= 'a'", "true");
    // assert_value!("'b' > 'a'", "true");
    // assert_value!("'b' >= 'b'", "true");
}

#[test]
fn test_concat_left_side_must_be_string_error() {
    crate::link_error_contains("{ a: 1; result: a + 'z' }", &["left side", "string", "+"]);
}
mod utilities;
pub use utilities::*;
