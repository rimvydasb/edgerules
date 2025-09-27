#[test]
fn test_common() {
    // Math
    assert_value!("1 + 2", "3");
    assert_value!("1.1 + 2", "3.1");
    assert_value!("1.1 + 2.1", "3.2");
    assert_value!("1.0 + 2", "3");
    assert_value!("-1 + 2", "1");
    assert_value!("-2 + 1", "-1");
    assert_value!("1 * 2 + 1", "3");

    // for/return
    assert_value!("for x in [1,2,3] return x * 2", "[2, 4, 6]");
    assert_value!("for x in [1,2,3] return x * 2.0", "[2, 4, 6]");
    assert_value!("for x in 1..3 return x * 2", "[2, 4, 6]");

    assert_value!("for x in [{age:23},{age:34}] return x.age + 2", "[25, 36]");

    assert_value!("2 / 3", "0.6666666666666666");
    assert_value!("1 * 2 / 3 + 1 - 2", "-0.33333333333333337");
    assert_eq!(1.0 * 2.0 / 3.0 + 1.0 - 2.0, -0.3333333333333335);

    assert_eq!(eval_value("{ age : 18; value : 1 + 2 }"), "3");

    // Selection, paths
    assert_eq!(
        eval_field(
            r#"
            {
                record : [1,2,3];
                record2 : record[1]
            }
            "#
            .trim(),
            "record2"
        ),
        "2"
    );
    assert_eq!(
        eval_field(
            r#"
            {
                list : [1,2,3];
                value : list[0] * list[1] + list[2]
            }
            "#
            .trim(),
            "value"
        ),
        "5"
    );
    assert_eq!(
        eval_field(
            r#"
            {
                list : [1,2,3];
                value : list[0] * (list[1] + list[2] * 3)
            }
            "#
            .trim(),
            "value"
        ),
        "11"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                record : {
                    age : 18;
                    value : 1 + 2
                }
            }
            "#
            .trim(),
            "record.value"
        ),
        "3"
    );

    // FieldNotFound link error
    link_error_contains(
        "{ record : { age : somefield; value : 1 + 2 }}",
        &["field", "somefield"],
    );

    assert_eq!(
        eval_field(
            r#"
            {
                record : {
                    age : 18;
                    value : age + 1
                }
            }
            "#
            .trim(),
            "record.value"
        ),
        "19"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                record : {
                    age : 18;
                    value : age + 2 + addition;
                    addition : age + 2
                }
            }
            "#
            .trim(),
            "record.value"
        ),
        "40"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                record : {
                    age : 18;
                    value : record.age + 1
                }
            }
            "#
            .trim(),
            "record.value"
        ),
        "19"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                record : {
                    value : record2.age2
                };
                record2 : {
                    age2 : 22
                }
            }
            "#
            .trim(),
            "record.value"
        ),
        "22"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                record : {
                    age : 18;
                    value : age + 2 + addition;
                    addition : age + record2.age2
                };
                record2 : {
                    age2 : 22
                }
            }
            "#
            .trim(),
            "record.value"
        ),
        "60"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                func doublethis(input) : { out : input * input };
                result : doublethis(2).out
            }
            "#
            .trim(),
            "result"
        ),
        "4"
    );
}

#[test]
fn test_functions() {
    assert_value!("2 * 2", "4");
    assert_value!("sum(1,2,3) + (2 * 2)", "10");
    assert_eq!(
        eval_field(
            "value : sum(1,2,3 + sum(2,2 * sum(0,1,0,0))) + (2 * 2)",
            "value"
        ),
        "14"
    );
    assert_value!("count([1,2,3]) + 1", "4");
    assert_value!("max([1,2,3]) + 1", "4");
    assert_value!("find([1,2,3],1)", "0");
    assert_value!("find([1,2,888],888)", "2");
    assert_value!("find([1,2,888],999)", "number.Missing");
}

#[test]
fn client_functions_test() {
    // variant 1
    assert_value!(
        r#"
        month : 1
        sales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
        value : sales[month] + sales[month + 1] + sales[month + 2]
        "#,
        "35"
    );

    // variant 2
    assert_value!(
        r#"
        inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
        func salesIn3Months(month,sales) : {
            result : sales[month] + sales[month + 1] + sales[month + 2]
        }
        value : salesIn3Months(1,inputSales).result
        "#,
        "35"
    );

    // variant 3 with subContext
    assert_value!(
        r#"
        inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
        func salesIn3Months(month,sales) : {
            result : sales[month] + sales[month + 1] + sales[month + 2]
        }
        subContext : {
            subResult : salesIn3Months(1,inputSales).result
        }
        value : subContext.subResult
        "#,
        "35"
    );

    // bestMonths[0]
    assert_value!(
        r#"
        inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
        func salesIn3Months(month,sales) : {
            result : sales[month] + sales[month + 1] + sales[month + 2]
        }
        bestMonths : for monthIter in 0..11 return salesIn3Months(monthIter,inputSales).result
        value : bestMonths[0]
        "#,
        "38"
    );
}

// #[test]  // kept disabled like the original
#[allow(dead_code)]
fn tables_test() {
    assert_value!(
        r#"
        @DecisionTable
        simpleTable(age,score) : [
        [age, score, eligibility],
        [18, 300, 0],
        [22, 100, 1],
        [65, 200, 0]
        ]
        value : simpleTable(22,100).eligibility
        "#,
        "1"
    );
}

#[test]
fn test_filter_not_alias() {
    // implicit 'it'
    assert_value!("count([1, 5, 12, 7][not it > 10])", "3");

    // explicit '...'
    assert_value!("count([1, 5, 12, 7][not ... > 10])", "3");

    // combine inside filter
    assert_value!("count([1, 5, 12, 7, 15][(it > 3) and not (it > 10)])", "2");
}

#[test]
fn variable_linkin_test() {
    assert_eq!(
        eval_field(
            r#"
            {
                input : {
                    application: {
                        status: 1
                    }
                }
                model: {
                    output: input.application.status
                }
            }
            "#
            .trim(),
            "model.output"
        ),
        "1"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                input : {
                    application: {
                        status: 1
                    }
                }
                model: {
                    func applicationRecord(application): {
                        statusFlag: if application.status = 1 then 'ok' else 'no'
                    }
                    output: applicationRecord(input.application).statusFlag
                }
            }
            "#
            .trim(),
            "model.output"
        ),
        "'ok'"
    );
}

#[test]
fn order_test() {
    let result = eval_all(
        r#"
    {
        xx : yy + 1
        c1 : a1 + "c1"
        b1 : "b1"
        a1 : b1 + "a1"
        yy : 5
    }
    "#,
    );

    assert_eq!(
        to_lines(&result),
        to_lines(
            r#"
        {
           xx : 6
           c1 : 'b1a1c1'
           b1 : 'b1'
           a1 : 'b1a1'
           yy : 5
        }"#
        )
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

#[test]
fn field_self_references_test() {
    let model = r#"
    {
        ctx: { b: 1; a: a }
    }
    "#;

    link_error_contains(model, &["cyclic reference loop"]);

    let model = r#"
    {
        z: 1;
        ctx: { z: 2; d: { z: z } }
        value: ctx.d.z
    }
    "#;

    link_error_contains(model, &["cyclic reference loop"]);
}

#[test]
fn context_fields_duplicate() {
    let model = r#"
    {
        ctx: { a: 1; a: 2 }
    }
    "#;

    parse_error_contains(model, &["Duplicate field 'a'"]);

    assert_value!(
        r#"
    {
        z: 1;
        ctx: { z: 2; d: z }
        value: ctx.d
    }
    "#,
        "2"
    );

    assert_value!(
        r#"
    {
        z: 1;
        ctx: { z: 2; d: { zz: z } }
        value: ctx.d.zz
    }
    "#,
        "2"
    );

    let model = r#"
    {
        z: 1;
        ctx: { z: 2; d: { zz: z; zz: 5 } }
        value: ctx.d.zz
    }
    "#;

    parse_error_contains(model, &["Duplicate field 'zz'"]);
}

mod utilities;
pub use utilities::*;
