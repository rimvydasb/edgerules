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
mod utilities;
pub use utilities::*;
