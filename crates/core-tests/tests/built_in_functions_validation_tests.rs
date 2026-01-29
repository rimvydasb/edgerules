use edge_rules::test_support::{LinkingErrorEnum, ValueType};
use rstest::rstest;

mod utilities;
pub use utilities::*;

// -------------------------------------------------------------------------------------------------
// Parameterized Unary Numeric Functions Validation
// -------------------------------------------------------------------------------------------------

/// Test that unary numeric functions require at least one argument
#[rstest]
#[case("abs")]
#[case("floor")]
#[case("ceiling")]
#[case("trunc")]
#[case("sqrt")]
#[case("ln")]
#[case("log10")]
#[case("exp")]
#[case("degrees")]
#[case("radians")]
#[case("sin")]
#[case("cos")]
#[case("tan")]
#[case("asin")]
#[case("acos")]
#[case("atan")]
fn test_unary_numeric_no_args(#[case] func: &str) {
    let code = format!("{{ value: {}() }}", func);
    parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
}

/// Test that unary numeric functions reject string arguments
#[rstest]
#[case("abs")]
#[case("floor")]
#[case("ceiling")]
#[case("trunc")]
#[case("sqrt")]
#[case("ln")]
#[case("log10")]
#[case("exp")]
#[case("degrees")]
#[case("radians")]
#[case("sin")]
#[case("cos")]
#[case("tan")]
#[case("asin")]
#[case("acos")]
#[case("atan")]
fn test_unary_numeric_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}('abc') }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}('abc')", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::NumberType])),
    );
}

// -------------------------------------------------------------------------------------------------
// Parameterized Unary String Functions Validation
// -------------------------------------------------------------------------------------------------

/// Test that unary string functions require at least one argument
#[rstest]
#[case("length")]
#[case("toUpperCase")]
#[case("toLowerCase")]
#[case("trim")]
#[case("toBase64")]
#[case("fromBase64")]
#[case("sanitizeFilename")]
fn test_unary_string_no_args(#[case] func: &str) {
    let code = format!("{{ value: {}() }}", func);
    parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
}

/// Test that unary string functions reject number arguments
#[rstest]
#[case("length")]
#[case("toUpperCase")]
#[case("toLowerCase")]
#[case("trim")]
#[case("toBase64")]
#[case("fromBase64")]
#[case("sanitizeFilename")]
fn test_unary_string_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}(123) }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}(123)", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::NumberType, Some(vec![ValueType::StringType])),
    );
}

// -------------------------------------------------------------------------------------------------
// Parameterized Unary List Functions Validation
// -------------------------------------------------------------------------------------------------

/// Test that numeric list functions require at least one argument
#[rstest]
#[case("product")]
#[case("mean")]
#[case("median")]
#[case("stddev")]
fn test_unary_numeric_list_no_args(#[case] func: &str) {
    let code = format!("{{ value: {}() }}", func);
    parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
}

/// Test that numeric list functions reject string arguments
#[rstest]
#[case("product")]
#[case("mean")]
#[case("median")]
#[case("stddev")]
fn test_unary_numeric_list_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}('abc') }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}('abc')", func),
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::StringType,
            Some(vec![ValueType::ListType(Some(Box::new(ValueType::NumberType)))]),
        ),
    );
}

/// Test that boolean list functions require at least one argument
#[rstest]
#[case("all")]
#[case("any")]
fn test_unary_boolean_list_no_args(#[case] func: &str) {
    let code = format!("{{ value: {}() }}", func);
    parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
}

/// Test that boolean list functions reject string arguments
#[rstest]
#[case("all")]
#[case("any")]
fn test_unary_boolean_list_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}('abc') }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}('abc')", func),
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::StringType,
            Some(vec![ValueType::ListType(Some(Box::new(ValueType::BooleanType)))]),
        ),
    );
}

/// Test that generic list functions require at least one argument
#[rstest]
#[case("count")]
#[case("mode")]
#[case("distinctValues")]
#[case("duplicateValues")]
#[case("flatten")]
#[case("isEmpty")]
#[case("sort")]
#[case("sortDescending")]
#[case("reverse")]
fn test_unary_generic_list_no_args(#[case] func: &str) {
    let code = format!("{{ value: {}() }}", func);
    parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
}

#[test]
fn test_count_wrong_type() {
    let code = "{ value: count('abc') }";
    link_error_location(
        code,
        &["value"],
        "count('abc')",
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::StringType,
            Some(vec![
                ValueType::NumberType,
                ValueType::RangeType,
                ValueType::ListType(Some(Box::new(ValueType::NumberType))),
            ]),
        ),
    );
}

/// Test that generic list functions (except count and reverse) reject string arguments
#[rstest]
#[case("mode")]
#[case("distinctValues")]
#[case("duplicateValues")]
#[case("flatten")]
#[case("isEmpty")]
#[case("sort")]
#[case("sortDescending")]
fn test_unary_generic_list_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}('abc') }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}('abc')", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::ListType(None)])),
    );
}

// -------------------------------------------------------------------------------------------------
// Parameterized Unary Date/Time Functions Validation
// -------------------------------------------------------------------------------------------------

/// Test that all date/time functions require at least one argument
#[rstest]
#[case("date")]
#[case("time")]
#[case("datetime")]
#[case("duration")]
#[case("period")]
#[case("dayOfWeek")]
#[case("monthOfYear")]
#[case("lastDayOfMonth")]
fn test_unary_date_no_args(#[case] func: &str) {
    let code = format!("{{ value: {}() }}", func);
    parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
}

/// Test that parse functions (date, time, datetime, duration, period) reject number arguments
#[rstest]
#[case("date")]
#[case("time")]
#[case("datetime")]
#[case("duration")]
#[case("period")]
fn test_unary_date_parse_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}(123) }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}(123)", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::NumberType, Some(vec![ValueType::StringType])),
    );
}

/// Test that helper functions (dayOfWeek, monthOfYear, lastDayOfMonth) reject number arguments
#[rstest]
#[case("dayOfWeek")]
#[case("monthOfYear")]
#[case("lastDayOfMonth")]
fn test_unary_date_helper_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}(123) }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}(123)", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::NumberType, Some(vec![ValueType::DateType])),
    );
}

// -------------------------------------------------------------------------------------------------
// Parameterized Binary Functions Validation
// -------------------------------------------------------------------------------------------------

/// Test that binary math functions require 2 arguments
#[rstest]
#[case("modulo")]
#[case("idiv")]
#[case("atan2")]
fn test_binary_math_missing_arg(#[case] func: &str) {
    let code = format!("{{ value: {}(1) }}", func);
    parse_error_contains(&code, &[&format!("Binary function '{}' expected 2 arguments, but got 1", func)]);
}

/// Test that binary math functions reject string as second argument
#[rstest]
#[case("modulo")]
#[case("idiv")]
#[case("atan2")]
fn test_binary_math_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}(1, 'a') }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}(1,'a')", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::NumberType])),
    );
}

/// Test that binary string functions require 2 arguments
#[rstest]
#[case("startsWith")]
#[case("endsWith")]
#[case("split")]
#[case("regexSplit")]
#[case("substringBefore")]
#[case("substringAfter")]
fn test_binary_string_missing_arg(#[case] func: &str) {
    let code = format!("{{ value: {}(1) }}", func);
    parse_error_contains(&code, &[&format!("Binary function '{}' expected 2 arguments, but got 1", func)]);
}

/// Test that binary string functions reject number as second argument
#[rstest]
#[case("startsWith")]
#[case("endsWith")]
#[case("split")]
#[case("regexSplit")]
#[case("substringBefore")]
#[case("substringAfter")]
fn test_binary_string_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}('a', 1) }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}('a',1)", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::NumberType, Some(vec![ValueType::StringType])),
    );
}

/// Test that string+number binary functions require 2 arguments
#[rstest]
#[case("charAt")]
#[case("charCodeAt")]
#[case("repeat")]
fn test_binary_string_num_missing_arg(#[case] func: &str) {
    let code = format!("{{ value: {}(1) }}", func);
    parse_error_contains(&code, &[&format!("Binary function '{}' expected 2 arguments, but got 1", func)]);
}

/// Test that string+number binary functions reject string as second argument
#[rstest]
#[case("charAt")]
#[case("charCodeAt")]
#[case("repeat")]
fn test_binary_string_num_wrong_type(#[case] func: &str) {
    let code = format!("{{ value: {}('a', 'b') }}", func);
    link_error_location(
        &code,
        &["value"],
        &format!("{}('a','b')", func),
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::NumberType])),
    );
}

#[test]
fn test_last_index_of_validation() {
    let code = "{ value: lastIndexOf(1) }";
    parse_error_contains(code, &["Binary function 'lastIndexOf' expected 2 arguments, but got 1"]);

    let code = "{ value: lastIndexOf(1, 'a') }";
    link_error_location(
        code,
        &["value"],
        "lastIndexOf(1,'a')",
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::NumberType, Some(vec![ValueType::StringType])),
    );
}

#[test]
fn test_calendar_diff_validation() {
    let code = "{ value: calendarDiff(1) }";
    parse_error_contains(code, &["Binary function 'calendarDiff' expected 2 arguments"]);

    let code = "{ value: calendarDiff(date('2021-01-01'), 1) }";
    link_error_location(
        code,
        &["value"],
        "calendarDiff(date('2021-01-01'),1)",
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::NumberType, Some(vec![ValueType::DateType])),
    );
}

// -------------------------------------------------------------------------------------------------
// Parameterized Extrema/Sum Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_min_max_wrong_type() {
    for func in ["min", "max"] {
        let code = format!("{{ value: {}(1, 'a') }}", func);
        link_error_location(
            &code,
            &["value"],
            &format!("{}(1, 'a')", func),
            LinkingErrorEnum::TypesNotCompatible(
                None,
                ValueType::StringType,
                Some(vec![
                    ValueType::NumberType,
                    ValueType::DateType,
                    ValueType::TimeType,
                    ValueType::DateTimeType,
                    ValueType::DurationType,
                ]),
            ),
        );
    }
}

#[test]
fn test_sum_wrong_type() {
    let code = "{ value: sum(1, 'a') }";
    link_error_location(
        code,
        &["value"],
        "sum(1, 'a')",
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::StringType,
            Some(vec![ValueType::NumberType, ValueType::DurationType]),
        ),
    );
}

#[test]
fn test_pi_validation() {
    let code = "{ value: pi(1) }";
    link_error_location(
        code,
        &["value"],
        "pi(1)",
        LinkingErrorEnum::OtherLinkingError("Expects 0 arguments".to_string()),
    );
}

#[test]
fn test_other_string_multi_validation() {
    // regexReplace (3 or 4 args)
    let code = "{ value: regexReplace('a', 'b') }";
    link_error_location(
        code,
        &["value"],
        "regexReplace('a', 'b')",
        LinkingErrorEnum::OtherLinkingError("replace expects 3 or 4 arguments".to_string()),
    );

    // replaceFirst (3 args)
    let code = "{ value: replaceFirst('a', 'b') }";
    link_error_location(
        code,
        &["value"],
        "replaceFirst('a', 'b')",
        LinkingErrorEnum::OtherLinkingError("replace expects 3 or 4 arguments".to_string()),
    );

    // replaceLast (3 args)
    let code = "{ value: replaceLast('a', 'b') }";
    link_error_location(
        code,
        &["value"],
        "replaceLast('a', 'b')",
        LinkingErrorEnum::OtherLinkingError("replace expects 3 or 4 arguments".to_string()),
    );

    // fromCharCode (args must be numbers)
    let code = "{ value: fromCharCode('a') }";
    link_error_location(
        code,
        &["value"],
        "fromCharCode('a')",
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::NumberType])),
    );
}

// -------------------------------------------------------------------------------------------------
// Numeric Functions Validation (Multi)
// -------------------------------------------------------------------------------------------------

#[test]
fn test_round_validation() {
    // 0 args
    let code = "{ value: round() }";
    link_error_location(
        code,
        &["value"],
        "round()",
        LinkingErrorEnum::OtherLinkingError("round functions expect 1 or 2 arguments".to_string()),
    );

    // 3 args
    let code = "{ value: round(1, 2, 3) }";
    link_error_location(
        code,
        &["value"],
        "round(1, 2, 3)",
        LinkingErrorEnum::OtherLinkingError("round functions expect 1 or 2 arguments".to_string()),
    );

    // Wrong type (arg 1)
    let code = "{ value: round('a') }";
    link_error_location(
        code,
        &["value"],
        "round('a')",
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::NumberType])),
    );
}

#[test]
fn test_round_up_validation() {
    let code = "{ value: roundUp() }";
    link_error_location(
        code,
        &["value"],
        "roundUp()",
        LinkingErrorEnum::OtherLinkingError("round functions expect 1 or 2 arguments".to_string()),
    );
}

#[test]
fn test_round_down_validation() {
    let code = "{ value: roundDown(1, 2, 3) }";
    link_error_location(
        code,
        &["value"],
        "roundDown(1, 2, 3)",
        LinkingErrorEnum::OtherLinkingError("round functions expect 1 or 2 arguments".to_string()),
    );
}

#[test]
fn test_clamp_validation() {
    // 2 args
    let code = "{ value: clamp(1, 2) }";
    link_error_location(
        code,
        &["value"],
        "clamp(1, 2)",
        LinkingErrorEnum::OtherLinkingError("clamp expects 3 arguments".to_string()),
    );

    // 4 args
    let code = "{ value: clamp(1, 2, 3, 4) }";
    link_error_location(
        code,
        &["value"],
        "clamp(1, 2, 3, 4)",
        LinkingErrorEnum::OtherLinkingError("clamp expects 3 arguments".to_string()),
    );

    // Wrong type
    let code = "{ value: clamp(1, 'min', 3) }";
    link_error_location(
        code,
        &["value"],
        "clamp(1, 'min', 3)",
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::NumberType])),
    );
}

// -------------------------------------------------------------------------------------------------
// String Functions Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_substring_validation() {
    // 1 arg
    let code = "{ value: substring('abc') }";
    link_error_location(
        code,
        &["value"],
        "substring('abc')",
        LinkingErrorEnum::OtherLinkingError("substring expects 2 or 3 arguments".to_string()),
    );

    // 4 args
    let code = "{ value: substring('abc', 0, 1, 2) }";
    link_error_location(
        code,
        &["value"],
        "substring('abc', 0, 1, 2)",
        LinkingErrorEnum::OtherLinkingError("substring expects 2 or 3 arguments".to_string()),
    );

    // Wrong type (arg 2 must be number)
    let code = "{ value: substring('abc', '0') }";
    link_error_location(
        code,
        &["value"],
        "substring('abc', '0')",
        LinkingErrorEnum::TypesNotCompatible(
            Some("arg2".to_string()),
            ValueType::StringType,
            Some(vec![ValueType::NumberType]),
        ),
    );
}

#[test]
fn test_replace_validation() {
    // 2 args
    let code = "{ value: replace('abc', 'a') }";
    link_error_location(
        code,
        &["value"],
        "replace('abc', 'a')",
        LinkingErrorEnum::OtherLinkingError("replace expects 3 or 4 arguments".to_string()),
    );

    // 5 args
    let code = "{ value: replace('abc', 'a', 'b', 'i', 'x') }";
    link_error_location(
        code,
        &["value"],
        "replace('abc', 'a', 'b', 'i', 'x')",
        LinkingErrorEnum::OtherLinkingError("replace expects 3 or 4 arguments".to_string()),
    );
}

#[test]
fn test_pad_validation() {
    // padStart 2 args
    let code = "{ value: padStart('abc', 5) }";
    link_error_location(
        code,
        &["value"],
        "padStart('abc', 5)",
        LinkingErrorEnum::OtherLinkingError("padStart/padEnd expects 3 arguments".to_string()),
    );

    // padEnd 4 args
    let code = "{ value: padEnd('abc', 5, ' ', 'x') }";
    link_error_location(
        code,
        &["value"],
        "padEnd('abc', 5, ' ', 'x')",
        LinkingErrorEnum::OtherLinkingError("padStart/padEnd expects 3 arguments".to_string()),
    );
}

// -------------------------------------------------------------------------------------------------
// List Functions Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_sublist_validation() {
    // 1 arg
    let code = "{ value: sublist([1,2]) }";
    link_error_location(
        code,
        &["value"],
        "sublist([1, 2])",
        LinkingErrorEnum::OtherLinkingError("sublist expects 2 or 3 arguments".to_string()),
    );

    // 4 args
    let code = "{ value: sublist([1,2], 1, 1, 1) }";
    link_error_location(
        code,
        &["value"],
        "sublist([1, 2], 1, 1, 1)",
        LinkingErrorEnum::OtherLinkingError("sublist expects 2 or 3 arguments".to_string()),
    );
}

#[test]
fn test_append_validation() {
    // 0 args
    let code = "{ value: append() }";
    link_error_location(
        code,
        &["value"],
        "append()",
        LinkingErrorEnum::OtherLinkingError("append expects at least 1 argument".to_string()),
    );

    // Type mismatch (append must match list type)
    let code = "{ value: append([1, 2], 'a') }";
    link_error_location(
        code,
        &["value"],
        "append([1, 2], 'a')",
        LinkingErrorEnum::DifferentTypesDetected(
            Some("append".to_string()),
            ValueType::NumberType,
            ValueType::StringType,
        ),
    );
}

#[test]
fn test_join_validation() {
    // 0 args
    let code = "{ value: join() }";
    link_error_location(
        code,
        &["value"],
        "join()",
        LinkingErrorEnum::OtherLinkingError("join expects at least 1 argument".to_string()),
    );

    // Wrong type (arg 1 must be list of strings)
    // Actually join expects list of strings as first arg.
    let code = "{ value: join([1, 2]) }";
    link_error_location(
        code,
        &["value"],
        "join([1, 2])",
        LinkingErrorEnum::TypesNotCompatible(None, ValueType::NumberType, Some(vec![ValueType::StringType])),
    );
}

#[test]
fn test_insert_before_validation() {
    // 2 args
    let code = "{ value: insertBefore([1], 1) }";
    link_error_location(
        code,
        &["value"],
        "insertBefore([1], 1)",
        LinkingErrorEnum::OtherLinkingError("insertBefore expects 3 arguments".to_string()),
    );
}

#[test]
fn test_union_validation() {
    // 0 args
    let code = "{ value: union() }";
    link_error_location(
        code,
        &["value"],
        "union()",
        LinkingErrorEnum::OtherLinkingError("union expects at least 1 argument".to_string()),
    );

    // Type mismatch (arg 2 must match arg 1)
    let code = "{ value: union([1], ['a']) }";
    link_error_location(
        code,
        &["value"],
        "union([1], ['a'])",
        LinkingErrorEnum::DifferentTypesDetected(
            Some("union".to_string()),
            ValueType::NumberType,
            ValueType::StringType,
        ),
    );
}

#[test]
fn test_concatenate_validation() {
    // 0 args
    let code = "{ value: concatenate() }";
    link_error_location(
        code,
        &["value"],
        "concatenate()",
        LinkingErrorEnum::OtherLinkingError("concatenate expects at least 1 argument".to_string()),
    );
}
