#[test]
fn test_string_functions() {
    assert_eq!(crate::eval_value("value : 'hello'"), "'hello'");
    // substring
    assert_eq!(crate::eval_value("value : substring(\"foobar\", 3)"), "'obar'");
    assert_eq!(crate::eval_value("value : substring(\"foobar\", -3, 2)"), "'ba'");
    assert_eq!(crate::eval_value("value : substring(\"abc\", 1, 2)"), "'ab'");

    // length
    assert_eq!(crate::eval_value("value : length(\"foo\")"), "3");
    assert_eq!(crate::eval_value("value : length(\"\")"), "0");

    // case conversion
    assert_eq!(crate::eval_value("value : toUpperCase(\"aBc4\")"), "'ABC4'");
    assert_eq!(crate::eval_value("value : toLowerCase(\"aBc4\")"), "'abc4'");

    // substringBefore/After
    assert_eq!(
        crate::eval_value("value : substringBefore(\"foobar\", \"bar\")"),
        "'foo'"
    );
    assert_eq!(
        crate::eval_value("value : substringAfter(\"foobar\", \"ob\")"),
        "'ar'"
    );

    // contains / startsWith / endsWith
    assert_eq!(crate::eval_value("value : contains(\"foobar\", \"of\")"), "false");
    assert_eq!(crate::eval_value("value : startsWith(\"foobar\", \"fo\")"), "true");
    assert_eq!(crate::eval_value("value : endsWith(\"foobar\", \"r\")"), "true");

    // split
    assert_eq!(
        crate::eval_value("value : split(\"John Doe\", \" \")"),
        "['John', 'Doe']"
    );
    assert_eq!(
        crate::eval_value("value : split(\"a-b-c\", \"-\")"),
        "['a', 'b', 'c']"
    );

    // trim
    assert_eq!(crate::eval_value("value : trim(\"  hello  \")"), "'hello'");

    // base64
    assert_eq!(crate::eval_value("value : toBase64(\"FEEL\")"), "'RkVFTA=='");
    assert_eq!(crate::eval_value("value : fromBase64(\"RkVFTA==\")"), "'FEEL'");

    // replace
    assert_eq!(
        crate::eval_value("value : replace(\"abcd\", \"ab\", \"xx\")"),
        "'xxcd'"
    );
    assert_eq!(
        crate::eval_value("value : replace(\"Abcd\", \"ab\", \"xx\", \"i\")"),
        "'xxcd'"
    );

    // charAt / charCodeAt
    assert_eq!(crate::eval_value("value : charAt(\"Abcd\", 2)"), "'c'");
    assert_eq!(crate::eval_value("value : charCodeAt(\"Abcd\", 2)"), "99");

    // indexOf / lastIndexOf
    assert_eq!(crate::eval_value("value : indexOf(\"Abcd\", \"b\")"), "1");
    assert_eq!(crate::eval_value("value : lastIndexOf(\"Abcb\", \"b\")"), "3");

    // fromCharCode
    assert_eq!(crate::eval_value("value : fromCharCode(99, 100, 101)"), "'cde'");

    // padStart / padEnd
    assert_eq!(crate::eval_value("value : padStart(\"7\", 3, \"0\")"), "'007'");
    assert_eq!(crate::eval_value("value : padEnd(\"7\", 3, \"0\")"), "'700'");

    // repeat / reverse
    assert_eq!(crate::eval_value("value : repeat(\"ab\", 3)"), "'ababab'");
    assert_eq!(crate::eval_value("value : reverse(\"abc\")"), "'cba'");

    // sanitizeFilename
    assert_eq!(
        crate::eval_value("value : sanitizeFilename(\"a/b\\\\c:d*e?fg<h>ij\")"),
        "'abcdefghij'"
    );

    // interpolate
    assert_eq!(
        crate::eval_value("value : interpolate(\"Hi ${name}\", { name : \"Ana\" })"),
        "'Hi Ana'"
    );
}