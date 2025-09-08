use edge_rules::runtime::edge_rules::EdgeRules;

fn eval_all(code: &str) -> String {
    let service = EdgeRules::new();
    service.evaluate_all(code)
}

fn eval_field(code: &str, field: &str) -> String {
    let mut service = EdgeRules::new();
    let _ = service.load_source(code);
    service.evaluate_field(field)
}

fn eval_value(code: &str) -> String {
    eval_field(code, "value")
}

fn eval_lines_field(lines: &[&str], field: &str) -> String {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    eval_field(&code, field)
}

/// For tests that must assert link errors (e.g., cyclic/self ref, missing field).
fn link_error_contains(code: &str, needles: &[&str]) {
    let mut service = EdgeRules::new();
    let _ = service.load_source(code);
    let err = service.to_runtime().err().map(|e| e.to_string()).unwrap();
    let lower = err.to_lowercase();
    for n in needles {
        assert!(
            lower.contains(&n.to_lowercase()),
            "expected error to contain `{n}`, got: {err}"
        );
    }
}

#[test]
fn test_first() {
    // old test only initialized logger; keep a no-op to preserve intent
    assert!(true);
}

#[test]
fn test_common() {
    // Math
    assert_eq!(eval_value("value : 1 + 2"), "3");
    assert_eq!(eval_value("value : 1.1 + 2"), "3.1");
    assert_eq!(eval_value("value : 1.1 + 2.1"), "3.2");
    assert_eq!(eval_value("value : 1.0 + 2"), "3");
    assert_eq!(eval_value("value : -1 + 2"), "1");
    assert_eq!(eval_value("value : -2 + 1"), "-1");
    assert_eq!(eval_value("value : 1 * 2 + 1"), "3");

    // for/return
    assert_eq!(
        eval_value("value : for x in [1,2,3] return x * 2"),
        "[2, 4, 6]"
    );
    assert_eq!(
        eval_value("value : for x in [1,2,3] return x * 2.0"),
        "[2, 4, 6]"
    );
    assert_eq!(
        eval_value("value : for x in 1..3 return x * 2"),
        "[2, 4, 6]"
    );

    assert_eq!(
        eval_value("value : for x in [{age:23},{age:34}] return x.age + 2"),
        "[25, 36]"
    );

    assert_eq!(eval_value("value : 2 / 3"), "0.6666666666666666");
    assert_eq!(
        eval_value("value : 1 * 2 / 3 + 1 - 2"),
        "-0.33333333333333337"
    );
    assert_eq!(1.0 * 2.0 / 3.0 + 1.0 - 2.0, -0.3333333333333335);

    assert_eq!(eval_value("{ age : 18; value : 1 + 2 }"), "3");

    // Selection, paths
    assert_eq!(
        eval_field("{ record : [1,2,3]; record2 : record[1]}", "record2"),
        "2"
    );
    assert_eq!(
        eval_field(
            "{ list : [1,2,3]; value : list[0] * list[1] + list[2]}",
            "value"
        ),
        "5"
    );
    assert_eq!(
        eval_field(
            "{ list : [1,2,3]; value : list[0] * (list[1] + list[2] * 3)}",
            "value"
        ),
        "11"
    );

    assert_eq!(
        eval_field("{ record : { age : 18; value : 1 + 2 }}", "record.value"),
        "3"
    );

    // FieldNotFound link error
    link_error_contains(
        "{ record : { age : somefield; value : 1 + 2 }}",
        &["field", "somefield"],
    );

    assert_eq!(
        eval_field("{ record : { age : 18; value : age + 1 }}", "record.value"),
        "19"
    );

    assert_eq!(
        eval_field(
            "{ record : { age : 18; value : age + 2 + addition; addition : age + 2 }}",
            "record.value"
        ),
        "40"
    );

    assert_eq!(
        eval_field(
            "{ record : { age : 18; value : record.age + 1 }}",
            "record.value"
        ),
        "19"
    );

    assert_eq!(
        eval_field(
            "{ record : { value : record2.age2 }; record2 : { age2 : 22 }}",
            "record.value"
        ),
        "22"
    );

    assert_eq!(eval_field("{ record : { age : 18; value : age + 2 + addition; addition : age + record2.age2 }; record2 : { age2 : 22 }}", "record.value"), "60");

    assert_eq!(
        eval_field(
            "{ doublethis(input) : { out : input * input }; result : doublethis(2).out }",
            "result"
        ),
        "4"
    );
}

#[test]
fn test_functions() {
    assert_eq!(eval_value("value : 2 * 2"), "4");
    assert_eq!(eval_value("value : sum(1,2,3) + (2 * 2)"), "10");
    assert_eq!(
        eval_field(
            "value : sum(1,2,3 + sum(2,2 * sum(0,1,0,0))) + (2 * 2)",
            "value"
        ),
        "14"
    );
    assert_eq!(eval_value("value : count([1,2,3]) + 1"), "4");
    assert_eq!(eval_value("value : max([1,2,3]) + 1"), "4");
    assert_eq!(eval_value("value : find([1,2,3],1)"), "0");
    assert_eq!(eval_value("value : find([1,2,888],888)"), "2");
    assert_eq!(eval_value("value : find([1,2,888],999)"), "number.Missing");
}

#[test]
fn client_functions_test() {
    // variant 1
    assert_eq!(
        eval_lines_field(
            &[
                "month : 1",
                "sales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
                "value : sales[month] + sales[month + 1] + sales[month + 2]",
            ],
            "value"
        ),
        "35"
    );

    // variant 2
    assert_eq!(
        eval_lines_field(
            &[
                "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
                "salesIn3Months(month,sales) : {",
                "result : sales[month] + sales[month + 1] + sales[month + 2]",
                "}",
                "value : salesIn3Months(1,inputSales).result",
            ],
            "value"
        ),
        "35"
    );

    // variant 3 with subContext
    assert_eq!(
        eval_lines_field(
            &[
                "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
                "salesIn3Months(month,sales) : {",
                "result : sales[month] + sales[month + 1] + sales[month + 2]",
                "}",
                "subContext : {",
                "subResult : salesIn3Months(1,inputSales).result",
                "}",
                "value : subContext.subResult",
            ],
            "value"
        ),
        "35"
    );

    // bestMonths[0]
    assert_eq!(
            eval_lines_field(
                &[
                    "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
                    "salesIn3Months(month,sales) : {",
                    "result : sales[month] + sales[month + 1] + sales[month + 2]",
                    "}",
                    "bestMonths : for monthIter in 0..11 return salesIn3Months(monthIter,inputSales).result",
                    "value : bestMonths[0]",
                ],
                "value"
            ),
            "38"
        );
}

#[test]
fn datetime_primitives_and_components() {
    // Date components
    assert_eq!(eval_value("value : date(\"2017-05-03\").year"), "2017");
    assert_eq!(eval_value("value : date(\"2017-05-03\").month"), "5");
    assert_eq!(eval_value("value : date(\"2017-05-03\").day"), "3");

    // Time components
    assert_eq!(eval_value("value : time(\"12:00:00\").second"), "0");
    assert_eq!(eval_value("value : time(\"13:10:30\").minute"), "10");

    // Datetime components and .time extraction
    assert_eq!(
        eval_value("value : datetime(\"2016-12-09T15:37:00\").month"),
        "12"
    );
    assert_eq!(
        eval_value("value : datetime(\"2016-12-09T15:37:00\").hour"),
        "15"
    );
    // .time string form
    assert_eq!(
        eval_value("value : datetime(\"2016-12-09T15:37:00\").time"),
        "15:37:00.0"
    );

    // Weekday (ISO Monday=1) for 2018-10-11 is Thursday=4
    assert_eq!(eval_value("value : date(\"2018-10-11\").weekday"), "4");

    // all date component elements
    assert_eq!(
        eval_lines_field(
            &[
                "d1 : date(\"2017-05-03\")",
                "y : d1.year",
                "m : d1.month",
                "d : d1.day",
                "result : [y,m,d]",
            ],
            "result"
        ),
        "[2017, 5, 3]"
    );

    // complex browsing and type inference
    assert_eq!(
        eval_lines_field(
            &[
                "d1 : date(\"2017-05-03\")",
                "d2 : date(\"2018-12-31\")",
                "y : d1.year",
                "plusOneYear : y + 1 - d2.year",
            ],
            "plusOneYear"
        ),
        "0"
    );
}

#[test]
fn datetime_comparisons_and_arithmetic() {
    // Comparisons
    assert_eq!(
        eval_field(
            "value : date(\"2017-05-03\") < date(\"2017-05-04\")",
            "value"
        ),
        "true"
    );

    // date - date => P1D
    assert_eq!(
        eval_field(
            "value : date(\"2017-05-04\") - date(\"2017-05-03\")",
            "value"
        ),
        "P1D"
    );

    // date + duration days
    assert_eq!(
        eval_value("value : date(\"2017-05-03\") + duration(\"P1D\")"),
        "2017-05-04"
    );

    // clamp day-of-month
    assert_eq!(
        eval_value("value : date(\"2018-01-31\") + duration(\"P1M\")"),
        "2018-02-28"
    );

    // time - time => PT1H10M30S
    assert_eq!(
        eval_value("value : time(\"13:10:30\") - time(\"12:00:00\")"),
        "PT1H10M30S"
    );

    // datetime + PT23H
    assert_eq!(
        eval_value("value : datetime(\"2016-12-09T15:37:00\") + duration(\"PT23H\")"),
        "2016-12-10 14:37:00.0"
    );
}

#[test]
fn datetime_additional_functions() {
    assert_eq!(
        eval_value("value : dayOfWeek(date(\"2025-09-02\"))"),
        "'Tuesday'"
    );
    assert_eq!(
        eval_value("value : monthOfYear(date(\"2025-09-02\"))"),
        "'September'"
    );
    assert_eq!(
        eval_value("value : lastDayOfMonth(date(\"2025-02-10\"))"),
        "28"
    );
}

// #[test]  // kept disabled like the original
#[allow(dead_code)]
fn tables_test() {
    let out = eval_lines_field(
        &[
            "@DecisionTable",
            "simpleTable(age,score) : [",
            "[age, score, eligibility],",
            "[18, 300, 0],",
            "[22, 100, 1],",
            "[65, 200, 0]",
            "]",
            "value : simpleTable(22,100).eligibility",
        ],
        "value",
    );
    assert_eq!(out, "1");
}

#[test]
fn test_conditionals() {
    // comparisons
    assert_eq!(eval_value("value : 1 = 2"), "false");
    assert_eq!(eval_value("value : 1 < 2"), "true");
    assert_eq!(eval_value("value : 1 <= 2"), "true");
    assert_eq!(eval_value("value : 2 > 1"), "true");
    assert_eq!(eval_value("value : 2 >= 1"), "true");
    assert_eq!(eval_value("value : 1 = 1"), "true");
    assert_eq!(eval_value("value : 1 = 1 + 1"), "false");

    // boolean ops with numbers in conditionals
    assert_eq!(eval_value("value : 1 = 2 and 5 = 5"), "false");
    assert_eq!(eval_value("value : 1 + 1 = 2 and 5 = 5"), "true");

    assert_eq!(eval_value("value : 1 = 2 or 5 = 5"), "true");
    assert_eq!(eval_value("value : 1 = 2 or 5 = 5 + 1"), "false");

    assert_eq!(eval_value("value : 1 = 2 xor 5 = 5 + 1"), "false");
    assert_eq!(eval_value("value : 1 = 2 xor 5 = 4 + 1"), "true");
    assert_eq!(eval_value("value : 1 = 2 - 1 xor 5 = 5 + 1"), "true");

    assert_eq!(eval_value("value : 1 = 2 or 5 = 5 and 1 = 1"), "true");
    assert_eq!(eval_value("value : 1 = 2 or 5 = 5 and 1 = 1 + 1"), "false");

    // if-then-else nesting
    assert_eq!(eval_value("value : if 1 > 2 then 3 else 4"), "4");
    assert_eq!(eval_value("value : if 1 < 2 then 3 else 4"), "3");
    assert_eq!(eval_value("value : if 1 < 2 then 3 + 1 else 5"), "4");
    assert_eq!(eval_value("value : if 1 > 2 then 3 + 1 else 5 * 10"), "50");
    assert_eq!(
        eval_value("value : if 1 > 2 then 3 + 1 else (if 1 < 2 then 5 * 10 else 0)"),
        "50"
    );
    assert_eq!(
        eval_value("value : if 1 > 2 then 3 + 1 else (if 1 > 2 then 5 * 10 else 0)"),
        "0"
    );
    assert_eq!(
        eval_value("value : if 1 < 2 then (if 5 > 2 then 5 * 10 else 0) else 1"),
        "50"
    );
    assert_eq!(
        eval_value("value : (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1"),
        "51"
    );
    assert_eq!(
        eval_value("value : 1 + (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1"),
        "52"
    );
    assert_eq!(
        eval_value("value : 2 * (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1"),
        "101"
    );
}

#[test]
fn test_boolean_literals_and_logic() {
    // OR
    assert_eq!(eval_value("value : true  or true"), "true");
    assert_eq!(eval_value("value : true  or false"), "true");
    assert_eq!(eval_value("value : false or true"), "true");
    assert_eq!(eval_value("value : false or false"), "false");

    // AND
    assert_eq!(eval_value("value : true  and true"), "true");
    assert_eq!(eval_value("value : true  and false"), "false");
    assert_eq!(eval_value("value : false and true"), "false");
    assert_eq!(eval_value("value : false and false"), "false");

    // XOR
    assert_eq!(eval_value("value : true  xor true"), "false");
    assert_eq!(eval_value("value : true  xor false"), "true");
    assert_eq!(eval_value("value : false xor true"), "true");
    assert_eq!(eval_value("value : false xor false"), "false");

    // NOT
    assert_eq!(eval_value("value : not true"), "false");
    assert_eq!(eval_value("value : not false"), "true");
    assert_eq!(eval_value("value : not (1 = 1)"), "false");
    assert_eq!(eval_value("value : not (1 = 2)"), "true");

    // Mixed
    assert_eq!(eval_value("value : true and (1 < 2)"), "true");
    assert_eq!(eval_value("value : (1 = 1) and false"), "false");
    assert_eq!(eval_value("value : (1 = 1) or false"), "true");
    assert_eq!(eval_value("value : true and not false"), "true");
    assert_eq!(eval_value("value : (1 < 2) and not (2 < 1)"), "true");

    // More complex
    assert_eq!(
        eval_value("value : (true and (1 < 2)) or (false and (3 = 4))"),
        "true"
    );
    assert_eq!(
        eval_value("value : (true xor (1 = 1 and false)) or (2 < 1)"),
        "true"
    );
    assert_eq!(
        eval_value("value : (true and true) xor (false or (1 < 1))"),
        "true"
    );
    assert_eq!(
        eval_value("value : (true and (2 > 1 and (3 > 2))) and (false or (5 = 5))"),
        "true"
    );
}

#[test]
fn test_string_functions() {
    assert_eq!(eval_value("value : 'hello'"), "'hello'");
    // substring
    assert_eq!(eval_value("value : substring(\"foobar\", 3)"), "'obar'");
    assert_eq!(eval_value("value : substring(\"foobar\", -3, 2)"), "'ba'");
    assert_eq!(eval_value("value : substring(\"abc\", 1, 2)"), "'ab'");

    // length
    assert_eq!(eval_value("value : length(\"foo\")"), "3");
    assert_eq!(eval_value("value : length(\"\")"), "0");

    // case conversion
    assert_eq!(eval_value("value : toUpperCase(\"aBc4\")"), "'ABC4'");
    assert_eq!(eval_value("value : toLowerCase(\"aBc4\")"), "'abc4'");

    // substringBefore/After
    assert_eq!(
        eval_value("value : substringBefore(\"foobar\", \"bar\")"),
        "'foo'"
    );
    assert_eq!(
        eval_value("value : substringAfter(\"foobar\", \"ob\")"),
        "'ar'"
    );

    // contains / startsWith / endsWith
    assert_eq!(eval_value("value : contains(\"foobar\", \"of\")"), "false");
    assert_eq!(eval_value("value : startsWith(\"foobar\", \"fo\")"), "true");
    assert_eq!(eval_value("value : endsWith(\"foobar\", \"r\")"), "true");

    // split
    assert_eq!(
        eval_value("value : split(\"John Doe\", \" \")"),
        "['John', 'Doe']"
    );
    assert_eq!(
        eval_value("value : split(\"a-b-c\", \"-\")"),
        "['a', 'b', 'c']"
    );

    // trim
    assert_eq!(eval_value("value : trim(\"  hello  \")"), "'hello'");

    // base64
    assert_eq!(eval_value("value : toBase64(\"FEEL\")"), "'RkVFTA=='");
    assert_eq!(eval_value("value : fromBase64(\"RkVFTA==\")"), "'FEEL'");

    // replace
    assert_eq!(
        eval_value("value : replace(\"abcd\", \"ab\", \"xx\")"),
        "'xxcd'"
    );
    assert_eq!(
        eval_value("value : replace(\"Abcd\", \"ab\", \"xx\", \"i\")"),
        "'xxcd'"
    );

    // charAt / charCodeAt
    assert_eq!(eval_value("value : charAt(\"Abcd\", 2)"), "'c'");
    assert_eq!(eval_value("value : charCodeAt(\"Abcd\", 2)"), "99");

    // indexOf / lastIndexOf
    assert_eq!(eval_value("value : indexOf(\"Abcd\", \"b\")"), "1");
    assert_eq!(eval_value("value : lastIndexOf(\"Abcb\", \"b\")"), "3");

    // fromCharCode
    assert_eq!(eval_value("value : fromCharCode(99, 100, 101)"), "'cde'");

    // padStart / padEnd
    assert_eq!(eval_value("value : padStart(\"7\", 3, \"0\")"), "'007'");
    assert_eq!(eval_value("value : padEnd(\"7\", 3, \"0\")"), "'700'");

    // repeat / reverse
    assert_eq!(eval_value("value : repeat(\"ab\", 3)"), "'ababab'");
    assert_eq!(eval_value("value : reverse(\"abc\")"), "'cba'");

    // sanitizeFilename
    assert_eq!(
        eval_value("value : sanitizeFilename(\"a/b\\\\c:d*e?fg<h>ij\")"),
        "'abcdefghij'"
    );

    // interpolate
    assert_eq!(
        eval_value("value : interpolate(\"Hi ${name}\", { name : \"Ana\" })"),
        "'Hi Ana'"
    );
}

#[test]
fn test_filter_not_alias() {
    // implicit 'it'
    assert_eq!(eval_value("value : count([1, 5, 12, 7][not it > 10])"), "3");

    // explicit '...'
    assert_eq!(
        eval_value("value : count([1, 5, 12, 7][not ... > 10])"),
        "3"
    );

    // combine inside filter
    assert_eq!(
        eval_value("value : count([1, 5, 12, 7, 15][(it > 3) and not (it > 10)])"),
        "2"
    );
}

#[test]
fn test_constraints() {
    assert_eq!(eval_value("value : [1,2,3][...>1]"), "[2, 3]");
    assert_eq!(eval_value("value : [1,2,3][...>0]"), "[1, 2, 3]");
    assert_eq!(eval_value("value : [1,2,3][...>-5]"), "[1, 2, 3]");
    assert_eq!(eval_value("value : [1,2,3][...<-5]"), "[]");

    assert_eq!(
        eval_lines_field(
            &["nums : [1, 5, 12, 7]", "filtered: nums[...>6]"],
            "filtered"
        ),
        "[12, 7]"
    );

    assert_eq!(
        eval_lines_field(
            &[
                "input : {",
                "   nums : [1, 5, 12, 7]",
                "   filtered: nums[...>6]",
                "}",
            ],
            "input.filtered"
        ),
        "[12, 7]"
    );
}

#[test]
fn variable_linkin_test() {
    assert_eq!(
        eval_lines_field(
            &[
                "input : {",
                "   application: {",
                "      status: 1",
                "   }",
                "}",
                "model: {",
                "   output: input.application.status",
                "}",
            ],
            "model.output"
        ),
        "1"
    );

    assert_eq!(
        eval_lines_field(
            &[
                "input : {",
                "   application: {",
                "      status: 1",
                "   }",
                "}",
                "model: {",
                "   applicationRecord(application): {",
                "      statusFlag: if application.status = 1 then 'ok' else 'no'",
                "   }",
                "   output: applicationRecord(input.application).statusFlag",
                "}",
            ],
            "model.output"
        ),
        "'ok'"
    );
}

#[test]
fn test_problems() {
    // nested value
    assert_eq!(
        eval_field("{ record : { age : 18; value : 1 + 2 }}", "record.value"),
        "3"
    );

    // cyclic link errors
    link_error_contains("value : value + 1", &["cyclic"]);
    link_error_contains(
        "{ record1 : 15 + record2; record2 : 7 + record3; record3 : record1 * 10 }",
        &["cyclic", "record1"],
    );

    // simple arithmetic across fields
    assert_eq!(
        eval_field(
            "{ record1 : { age : 18}; record2 : record1.age + record1.age}",
            "record2"
        ),
        "36"
    );

    // pretty-print containment check (keep the intent)
    let printed = eval_all("{ p : [{a:1},5] }");
    assert!(
        printed.contains("[{a : 1}, 5]") || printed.contains("[{a: 1}, 5]"),
        "expected pretty output to contain normalized array of objects, got: {printed}"
    );
}
