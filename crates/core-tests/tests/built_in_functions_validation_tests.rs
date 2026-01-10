mod utilities;
use edge_rules::test_support::{LinkingErrorEnum, ValueType};
pub use utilities::*;

// -------------------------------------------------------------------------------------------------
// Unary Numeric Functions Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_unary_numeric_validation() {
    let numeric_funcs = [
        "abs", "floor", "ceiling", "trunc", "sqrt", "ln", "log10", "exp",
        "degrees", "radians", "sin", "cos", "tan", "asin", "acos", "atan"
    ];

    for func in numeric_funcs {
        // 0 args -> Parse Error
        let code = format!("{{ value: {}() }}", func);
        parse_error_contains(&code, &[
            &format!("Function '{}' got no arguments", func)
        ]);

        // Wrong type -> Link Error
        let code = format!("{{ value: {}('abc') }}", func);
        link_error_location(
            &code,
            &["value"],
            &format!("{}('abc')", func),
            LinkingErrorEnum::TypesNotCompatible(
                None,
                ValueType::StringType,
                Some(vec![ValueType::NumberType]),
            ),
        );
    }
}

// -------------------------------------------------------------------------------------------------
// Unary String Functions Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_unary_string_validation() {
    let string_funcs = [
        "length", "toUpperCase", "toLowerCase", "trim",
        "toBase64", "fromBase64", "sanitizeFilename"
    ];

    for func in string_funcs {
        // 0 args -> Parse Error
        let code = format!("{{ value: {}() }}", func);
        parse_error_contains(&code, &[
            &format!("Function '{}' got no arguments", func)
        ]);

        // Wrong type -> Link Error
        let code = format!("{{ value: {}(123) }}", func);
        link_error_location(
            &code,
            &["value"],
            &format!("{}(123)", func),
            LinkingErrorEnum::TypesNotCompatible(
                None,
                ValueType::NumberType,
                Some(vec![ValueType::StringType]),
            ),
        );
    }
}

// -------------------------------------------------------------------------------------------------
// Unary List Functions Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_unary_list_validation() {
    // List functions that expect list of numbers
    let numeric_list_funcs = ["product", "mean", "median", "stddev"];
    for func in numeric_list_funcs {
        let code = format!("{{ value: {}() }}", func);
        parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);

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

    // List functions that expect list of booleans
    let bool_list_funcs = ["all", "any"];
    for func in bool_list_funcs {
        let code = format!("{{ value: {}() }}", func);
        parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);

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

    // Generic list functions
    let list_funcs = ["count", "mode", "distinctValues", "duplicateValues", "flatten", "isEmpty", "sort", "sortDescending", "reverse"];
    for func in list_funcs {
        let code = format!("{{ value: {}() }}", func);
        parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);

        // count is special, accepts range or number too
        if func == "count" {
             let code = format!("{{ value: {}('abc') }}", func);
             link_error_location(
                &code,
                &["value"],
                &format!("{}('abc')", func),
                LinkingErrorEnum::TypesNotCompatible(
                    None,
                    ValueType::StringType,
                    Some(vec![ValueType::NumberType, ValueType::RangeType, ValueType::ListType(Some(Box::new(ValueType::NumberType)))]),
                )
            );
        } else {
            let code = format!("{{ value: {}('abc') }}", func);
            // reverse accepts strings too
            if func != "reverse" {
                 link_error_location(
                    &code,
                    &["value"],
                    &format!("{}('abc')", func),
                    LinkingErrorEnum::TypesNotCompatible(
                        None,
                        ValueType::StringType,
                        Some(vec![ValueType::ListType(None)]),
                    ),
                );
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Unary Date/Time Functions Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_unary_date_validation() {
    let date_funcs = [
        "date", "time", "datetime", "duration", "period",
        "dayOfWeek", "monthOfYear", "lastDayOfMonth"
    ];

    for func in date_funcs {
        // 0 args -> Parse Error
        let code = format!("{{ value: {}() }}", func);
        parse_error_contains(&code, &[
            &format!("Function '{}' got no arguments", func)
        ]);

        // Wrong type -> Link Error
        // Parsers expect string
        let parse_funcs = ["date", "time", "datetime", "duration", "period"];
        if parse_funcs.contains(&func) {
             let code = format!("{{ value: {}(123) }}", func);
             link_error_location(
                &code,
                &["value"],
                &format!("{}(123)", func),
                LinkingErrorEnum::TypesNotCompatible(
                    None,
                    ValueType::NumberType,
                    Some(vec![ValueType::StringType]),
                ),
            );
        } else {
            // Helpers expect date
             let code = format!("{{ value: {}(123) }}", func);
             link_error_location(
                &code,
                &["value"],
                &format!("{}(123)", func),
                LinkingErrorEnum::TypesNotCompatible(
                    None,
                    ValueType::NumberType,
                    Some(vec![ValueType::DateType]),
                ),
            );
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Binary Functions Validation
// -------------------------------------------------------------------------------------------------

#[test]
fn test_binary_validation() {
    // Math
    let math_funcs = ["modulo", "idiv", "atan2"];
    for func in math_funcs {
        let code = format!("{{ value: {}(1) }}", func);
        parse_error_contains(&code, &[&format!("Binary function '{}' expected 2 arguments, but got 1", func)]);

        let code = format!("{{ value: {}(1, 'a') }}", func);
        link_error_location(
            &code,
            &["value"],
            &format!("{}(1,'a')", func),
            LinkingErrorEnum::TypesNotCompatible(
                None,
                ValueType::StringType,
                Some(vec![ValueType::NumberType]),
            ),
        );
    }

    // String
    let string_funcs = ["startsWith", "endsWith", "split", "regexSplit", "substringBefore", "substringAfter"];
    for func in string_funcs {
        let code = format!("{{ value: {}(1) }}", func);
        parse_error_contains(&code, &[&format!("Binary function '{}' expected 2 arguments, but got 1", func)]);

        let code = format!("{{ value: {}('a', 1) }}", func);
        link_error_location(
            &code,
            &["value"],
            &format!("{}('a',1)", func),
            LinkingErrorEnum::TypesNotCompatible(
                None,
                ValueType::NumberType,
                Some(vec![ValueType::StringType]),
            ),
        );
    }

    // String + Number
    let string_num_funcs = ["charAt", "charCodeAt", "repeat"];
    for func in string_num_funcs {
        let code = format!("{{ value: {}(1) }}", func);
        parse_error_contains(&code, &[&format!("Binary function '{}' expected 2 arguments, but got 1", func)]);

        let code = format!("{{ value: {}('a', 'b') }}", func);
        link_error_location(
            &code,
            &["value"],
            &format!("{}('a','b')", func),
            LinkingErrorEnum::TypesNotCompatible(
                None,
                ValueType::StringType,
                Some(vec![ValueType::NumberType]),
            ),
        );
    }

    // Mixed
    let mixed_funcs = ["indexOf", "lastIndexOf"];
    for func in mixed_funcs {
        // ... (indexOf is special, can be list or string)
        if func == "lastIndexOf" {
             let code = format!("{{ value: {}(1) }}", func);
             parse_error_contains(&code, &[&format!("Binary function '{}' expected 2 arguments, but got 1", func)]);
             let code = format!("{{ value: {}(1, 'a') }}", func);
             link_error_location(
                &code,
                &["value"],
                &format!("{}(1,'a')", func),
                LinkingErrorEnum::TypesNotCompatible(
                    None,
                    ValueType::NumberType,
                    Some(vec![ValueType::StringType]),
                ),
            );
        }
    }

    // Date
    let code = "{ value: calendarDiff(1) }";
    parse_error_contains(&code, &["Binary function 'calendarDiff' expected 2 arguments"]);
    
    let code = "{ value: calendarDiff(date('2021-01-01'), 1) }";
    link_error_location(
        &code,
        &["value"],
        "calendarDiff(date('2021-01-01'),1)",
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::NumberType,
            Some(vec![ValueType::DateType]),
        ),
    );
}

// -------------------------------------------------------------------------------------------------
// Extrema/Sum Validation (Multi)
// -------------------------------------------------------------------------------------------------

#[test]
fn test_extrema_validation() {
    let funcs = ["min", "max", "sum"];
    for func in funcs {
        // Type mismatch (mixed types)
        let code = format!("{{ value: {}(1, 'a') }}", func);
        if func == "sum" {
             link_error_location(
                &code,
                &["value"],
                &format!("{}(1, 'a')", func),
                LinkingErrorEnum::TypesNotCompatible(
                    None,
                    ValueType::StringType,
                    Some(vec![ValueType::NumberType, ValueType::DurationType]),
                ),
            );
        } else {
             link_error_location(
                &code,
                &["value"],
                &format!("{}(1, 'a')", func),
                LinkingErrorEnum::TypesNotCompatible(
                    None,
                    ValueType::StringType,
                    Some(vec![ValueType::NumberType, ValueType::DateType, ValueType::TimeType, ValueType::DateTimeType, ValueType::DurationType]),
                ),
            );
        }
    }
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
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::StringType,
            Some(vec![ValueType::NumberType]),
        ),
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
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::StringType,
            Some(vec![ValueType::NumberType]),
        ),
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
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::StringType,
            Some(vec![ValueType::NumberType]),
        ),
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
        LinkingErrorEnum::TypesNotCompatible(
            None,
            ValueType::NumberType,
            Some(vec![ValueType::StringType]),
        ),
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
