mod utilities;
pub use utilities::*;

#[test]
fn test_string_functions() {
    assert_expression_value("'hello'", "'hello'");
    assert_expression_value("substring('foobar', 3)", "'obar'");
    assert_expression_value("substring('foobar', -3, 2)", "'ba'");
    assert_expression_value("substring('abc', 1, 2)", "'ab'");
    assert_expression_value("length('foo')", "3");
    assert_expression_value("length('')", "0");
    assert_expression_value("toUpperCase('aBc4')", "'ABC4'");
    assert_expression_value("toLowerCase('aBc4')", "'abc4'");
    assert_expression_value("substringBefore('foobar', 'bar')", "'foo'");
    assert_expression_value("substringAfter('foobar', 'ob')", "'ar'");
    assert_expression_value("contains('foobar', 'of')", "false");
    assert_expression_value("startsWith('foobar', 'fo')", "true");
    assert_expression_value("endsWith('foobar', 'r')", "true");
    assert_expression_value("split('John Doe', ' ')", "['John', 'Doe']");
    assert_expression_value("split('a-b-c', '-')", "['a', 'b', 'c']");
    assert_expression_value("regexSplit('one   two\tthree', '\\s+')", "['one', 'two', 'three']");
    assert_expression_value("trim('  hello  ')", "'hello'");
    assert_expression_value("toBase64('FEEL')", "'RkVFTA=='");
    assert_expression_value("fromBase64('RkVFTA==')", "'FEEL'");
    assert_expression_value("replace('abcd', 'ab', 'xx')", "'xxcd'");
    assert_expression_value("replace('Abcd', 'ab', 'xx', 'i')", "'xxcd'");
    assert_expression_value("regexReplace('Abcd', '[a-z]', 'x', 'i')", "'xxxx'");
    assert_expression_value("regexReplace('2025-09-02', '\\d', '#')", "'####-##-##'");
    assert_expression_value("replaceFirst('foo bar foo', 'foo', 'baz')", "'baz bar foo'");
    assert_expression_value("replaceLast('foo bar foo', 'foo', 'baz')", "'foo bar baz'");
    assert_expression_value("charAt('Abcd', 2)", "'c'");
    assert_expression_value("charCodeAt('Abcd', 2)", "99");
    assert_expression_value("indexOf('Abcd', 'b')", "1");
    assert_expression_value("lastIndexOf('Abcb', 'b')", "3");
    assert_expression_value("fromCharCode(99, 100, 101)", "'cde'");
    assert_expression_value("padStart('7', 3, '0')", "'007'");
    assert_expression_value("padEnd('7', 3, '0')", "'700'");
    assert_expression_value("repeat('ab', 3)", "'ababab'");
    assert_expression_value("reverse('abc')", "'cba'");
    assert_expression_value("sanitizeFilename('a/b\\\\c:d*e?fg<h>ij')", "'abcdefghij'");
    assert_expression_value("interpolate('Hi ${name}', { name : 'Ana' })", "'Hi Ana'");
}

#[test]
fn test_string_concatenation_with_plus() {
    assert_expression_value("'a' + 'b'", "'ab'");
    assert_expression_value("'a' + 'b' + toString(1)", "'ab1'");
    assert_expression_value("toString(1) + 'a'", "'1a'");
}

#[test]
fn test_string_logic() {
    assert_expression_value("'a' = 'a'", "true");
    assert_expression_value("'a' <> 'b'", "true");

    link_error_contains("value: 'a' < 'b'", &["Operation '<' not supported for types 'string' and 'string'"]);

    link_error_contains("value: 'a' <= 'a'", &["Operation '<=' not supported for types 'string' and 'string'"]);

    link_error_contains("value: 'b' > 'a'", &["Operation '>' not supported for types 'string' and 'string'"]);

    link_error_contains("value: 'b' >= 'b'", &["Operation '>=' not supported for types 'string' and 'string'"]);
}

#[test]
fn test_concat_left_side_must_be_string_error() {
    link_error_contains("{ a: 1; result: a + 'z' }", &["left side", "string", "+"]);
}
