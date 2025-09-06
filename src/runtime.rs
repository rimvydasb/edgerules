pub mod edge_rules;
pub mod execution_context;

/// Test Cases
///
#[cfg(test)]
mod test {
    use crate::ast::token::ExpressionEnum;
    use crate::runtime::edge_rules::{expr, EvalError};
    use crate::typesystem::errors::LinkingErrorEnum::{CyclicReference, FieldNotFound};
    use crate::typesystem::types::number::NumberEnum;
    use crate::typesystem::types::number::NumberEnum::{Int, Real};
    use crate::typesystem::types::SpecialValueEnum::Missing;
    use crate::typesystem::types::ValueType::NumberType;
    use crate::typesystem::values::ValueEnum;
    use crate::typesystem::values::ValueEnum::BooleanValue;
    use crate::utils::test::*;
    use log::info;

    type V = ValueEnum;
    type E = ExpressionEnum;

    #[test]
    fn test_first() {
        init_logger();
    }

    #[test]
    fn test_common() -> Result<(), EvalError> {
        init_logger();

        info!(">>> test_common()");

        // Math
        test_code("value : 1 + 2").expect_num("value", Int(3));
        test_code("value : 1.1 + 2").expect_num("value", Real(3.1));
        test_code("value : 1.1 + 2.1").expect_num("value", Real(3.2));
        test_code("value : 1.0 + 2").expect_num("value", Real(3.0));
        test_code("value : -1 + 2").expect_num("value", Int(1));
        test_code("value : -2 + 1").expect_num("value", Int(-1));
        test_code("value : 1 * 2 + 1").expect_num("value", Int(3));

        // New API
        test_code("value : for x in [1,2,3] return x * 2")
            .expect(&mut expr("value")?, V::from(vec![2, 4, 6]));

        // for each loop
        test_code("value : for x in [1,2,3] return x * 2")
            .expect(&mut expr("value")?, V::from(vec![2, 4, 6]));
        test_code("value : for x in [1,2,3] return x * 2.0")
            .expect(&mut expr("value")?, V::from(vec![2.0, 4.0, 6.0]));
        test_code("value : for x in 1..3 return x * 2")
            .expect(&mut expr("value")?, V::from(vec![2, 4, 6]));
        test_code("value : for x in [{age:23},{age:34}] return x.age + 2")
            .expect(&mut expr("value")?, V::from(vec![25, 36]));

        test_code("value : 2 / 3").expect_num("value", Real(0.6666666666666666));
        test_code("value : 1 * 2 / 3 + 1 - 2").expect_num("value", Real(-0.33333333333333337));
        assert_eq!(1.0 * 2.0 / 3.0 + 1.0 - 2.0, -0.3333333333333335);

        test_code("{ age : 18; value : 1 + 2 }").expect_num("value", Int(3));

        // Selection
        is_variable_evaluating_to(
            "{ record : [1,2,3]; record2 : record[1]}",
            "record2",
            V::from(2),
        );
        is_variable_evaluating_to(
            "{ list : [1,2,3]; value : list[0] * list[1] + list[2]}",
            "value",
            V::from(5),
        );
        is_variable_evaluating_to(
            "{ list : [1,2,3]; value : list[0] * (list[1] + list[2] * 3)}",
            "value",
            V::from(11),
        );

        is_evaluating_to(
            "{ record : { age : 18; value : 1 + 2 }}",
            &mut ExpressionEnum::variable("record.value"),
            V::from(3),
        );

        test_code("{ record : { age : somefield; value : 1 + 2 }}")
            .expect_link_error(FieldNotFound("Root".to_string(), "somefield".to_string()));

        is_evaluating_to(
            "{ record : { age : 18; value : age + 1 }}",
            &mut ExpressionEnum::variable("record.value"),
            V::from(19),
        );

        is_evaluating_to(
            "{ record : { age : 18; value : age + 2 + addition; addition : age + 2 }}",
            &mut ExpressionEnum::variable("record.value"),
            V::from(40),
        );

        is_evaluating_to(
            "{ record : { age : 18; value : record.age + 1 }}",
            &mut ExpressionEnum::variable("record.value"),
            V::from(19),
        );

        is_evaluating_to(
            "{ record : { value : record2.age2 }; record2 : { age2 : 22 }}",
            &mut ExpressionEnum::variable("record.value"),
            V::from(22),
        );

        is_evaluating_to("{ record : { age : 18; value : age + 2 + addition; addition : age + record2.age2 }; record2 : { age2 : 22 }}",
                         &mut ExpressionEnum::variable("record.value"), V::from(60));

        is_evaluating_to(
            "{ doublethis(input) : { out : input * input }; result : doublethis(2).out }",
            &mut ExpressionEnum::variable("result"),
            V::from(4),
        );

        // @todo: this is not working yet
        //test_code("p : [{a:1},5]").expect_code("p: [{a : 1}, 5]");

        Ok(())
    }

    #[test]
    fn test_functions() -> Result<(), EvalError> {
        init_test("test_functions()");

        test_code("value : 2 * 2").expect_num("value", Int(4));
        test_code("value : sum(1,2,3) + (2 * 2)").expect_num("value", Int(10));
        test_code("value : sum(1,2,3 + sum(2,2 * sum(0,1,0,0))) + (2 * 2)")
            .expect_num("value", Int(14));
        test_code("value : count([1,2,3]) + 1").expect_num("value", Int(4));
        test_code("value : max([1,2,3]) + 1").expect_num("value", Int(4));
        test_code("value : find([1,2,3],1)").expect_num("value", Int(0));
        test_code("value : find([1,2,888],888)").expect_num("value", Int(2));
        test_code("value : find([1,2,888],999)").expect_num("value", NumberEnum::SV(Missing));

        // @Todo: check exceptions
        //test_code("value : count(1,2,3) + 1").expect_parse_error(ParseErrorEnum::FunctionWrongNumberOfArguments("".to_string(),EFunctionType::Unary,0));

        Ok(())
    }

    #[test]
    fn test_strings() -> Result<(), EvalError> {
        info!(">>> test_strings()");

        test_code("value : 'hello'").expect(&mut expr("value")?, V::from("hello"));
        //test_code("value : 'hello' + 'world'").expect(expr("value")?, V::from("helloworld"));

        Ok(())
    }

    #[test]
    fn client_functions_test() {
        init_logger();

        info!(">>> client_functions_test()");

        test_code_lines(&[
            "month : 1",
            "sales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
            "value : sales[month] + sales[month + 1] + sales[month + 2]",
        ])
        .expect_num("value", Int(35));

        test_code_lines(&[
            "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
            "salesIn3Months(month,sales) : {",
            "result : sales[month] + sales[month + 1] + sales[month + 2]",
            "}",
            "value : salesIn3Months(1,inputSales).result",
        ])
        .expect_num("value", Int(35));

        test_code_lines(&[
            "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
            "salesIn3Months(month,sales) : {",
            "result : sales[month] + sales[month + 1] + sales[month + 2]",
            "}",
            "subContext : {",
            "subResult : salesIn3Months(1,inputSales).result",
            "}",
            "value : subContext.subResult",
        ])
        .expect_num("value", Int(35));

        test_code_lines(&[
            "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
            "salesIn3Months(month,sales) : {",
            "result : sales[month] + sales[month + 1] + sales[month + 2]",
            "}",
            "bestMonths : for monthIter in 0..11 return salesIn3Months(monthIter,inputSales).result",
            "value : bestMonths[0]",
        ]).expect_num("value", Int(38));
    }

    #[test]
    fn datetime_primitives_and_components_browsing() {
        init_logger();

        // Also verify that a bound variable date exposes components
        is_lines_evaluating_to(
            vec![
                "d1 : date(\"2017-05-03\")",
                "y : d1.year",
            ],
            &mut E::variable("y"),
            V::from(2017),
        );
    }

    #[test]
    fn datetime_primitives_and_components() {
        init_logger();

        // Date components
        test_code("value : date(\"2017-05-03\").year").expect_num("value", Int(2017));
        test_code("value : date(\"2017-05-03\").month").expect_num("value", Int(5));
        test_code("value : date(\"2017-05-03\").day").expect_num("value", Int(3));

        // Time components
        test_code("value : time(\"12:00:00\").second").expect_num("value", Int(0));
        test_code("value : time(\"13:10:30\").minute").expect_num("value", Int(10));

        // Datetime components and time extraction
        test_code("value : datetime(\"2016-12-09T15:37:00\").month").expect_num("value", Int(12));
        test_code("value : datetime(\"2016-12-09T15:37:00\").hour").expect_num("value", Int(15));
        test_code("value : datetime(\"2016-12-09T15:37:00\").time").expect(
            &mut expr("value").unwrap(),
            ValueEnum::TimeValue(crate::typesystem::values::ValueOrSv::Value(
                time::Time::from_hms(15, 37, 0).unwrap(),
            )),
        );

        // Weekday (ISO Monday=1) for 2018-10-11 is Thursday=4
        test_code("value : date(\"2018-10-11\").weekday").expect_num("value", Int(4));

        // Also verify that a bound variable date exposes components
        is_lines_evaluating_to(
            vec![
                "d1 : date(\"2017-05-03\")",
                "y : d1.year",
            ],
            &mut E::variable("y"),
            V::from(2017),
        );

        // is_lines_evaluating_to(
        //     vec![
        //         "d1 : date(\"2017-05-03\")",
        //         "m : d1.month",
        //     ],
        //     &mut E::variable("m"),
        //     V::from(5),
        // );
        //
        // is_lines_evaluating_to(
        //     vec![
        //         "d1 : date(\"2017-05-03\")",
        //         "d : d1.day",
        //     ],
        //     &mut E::variable("d"),
        //     V::from(3),
        // );

    }

    #[test]
    fn datetime_comparisons_and_arithmetic() {
        init_logger();

        // Comparisons
        test_code("value : date(\"2017-05-03\") < date(\"2017-05-04\")")
            .expect(&mut expr("value").unwrap(), BooleanValue(true));

        // Subtraction to duration
        // date - date => P1D
        test_code("value : date(\"2017-05-04\") - date(\"2017-05-03\")").expect(
            &mut expr("value").unwrap(),
            ValueEnum::DurationValue(crate::typesystem::values::ValueOrSv::Value(
                crate::typesystem::values::DurationValue::dt(1, 0, 0, 0, false),
            )),
        );

        // Addition with duration days
        test_code("value : date(\"2017-05-03\") + duration(\"P1D\")").expect(
            &mut expr("value").unwrap(),
            ValueEnum::DateValue(crate::typesystem::values::ValueOrSv::Value(
                time::Date::from_calendar_date(2017, time::Month::May, 4).unwrap(),
            )),
        );

        // Years-months duration addition clamping day-of-month
        test_code("value : date(\"2018-01-31\") + duration(\"P1M\")").expect(
            &mut expr("value").unwrap(),
            ValueEnum::DateValue(crate::typesystem::values::ValueOrSv::Value(
                time::Date::from_calendar_date(2018, time::Month::February, 28).unwrap(),
            )),
        );

        // time - time => duration
        test_code("value : time(\"13:10:30\") - time(\"12:00:00\")").expect(
            &mut expr("value").unwrap(),
            ValueEnum::DurationValue(crate::typesystem::values::ValueOrSv::Value(
                crate::typesystem::values::DurationValue::dt(0, 1, 10, 30, false),
            )),
        );

        // datetime +/- PT
        test_code("value : datetime(\"2016-12-09T15:37:00\") + duration(\"PT23H\")").expect(
            &mut expr("value").unwrap(),
            ValueEnum::DateTimeValue(crate::typesystem::values::ValueOrSv::Value(
                time::PrimitiveDateTime::new(
                    time::Date::from_calendar_date(2016, time::Month::December, 10).unwrap(),
                    time::Time::from_hms(14, 37, 0).unwrap(),
                ),
            )),
        );
    }

    #[test]
    fn datetime_additional_functions() {
        init_logger();
        test_code("value : dayOfWeek(date(\"2025-09-02\"))")
            .expect(&mut expr("value").unwrap(), V::from("Tuesday"));
        test_code("value : monthOfYear(date(\"2025-09-02\"))")
            .expect(&mut expr("value").unwrap(), V::from("September"));
        test_code("value : lastDayOfMonth(date(\"2025-02-10\"))")
            .expect_num("value", Int(28));
    }

    //#[test]
    #[allow(dead_code)]
    fn tables_test() {
        init_logger();

        info!(">>> tables_test()");

        is_lines_evaluating_to(
            vec![
                "@DecisionTable",
                "simpleTable(age,score) : [",
                "[age, score, eligibility],",
                "[18, 300, 0],",
                "[22, 100, 1],",
                "[65, 200, 0]",
                "]",
                "value : simpleTable(22,100).eligibility",
            ],
            &mut E::variable("value"),
            V::from(1),
        );
    }

    #[test]
    fn test_conditionals() {
        info!(">>> test_conditionals()");

        is_this_one_evaluating_to("value : 1 = 2", BooleanValue(false));
        is_this_one_evaluating_to("value : 1 < 2", BooleanValue(true));
        is_this_one_evaluating_to("value : 1 <= 2", BooleanValue(true));
        is_this_one_evaluating_to("value : 2 > 1", BooleanValue(true));
        is_this_one_evaluating_to("value : 2 >= 1", BooleanValue(true));
        is_this_one_evaluating_to("value : 1 = 1", BooleanValue(true));
        is_this_one_evaluating_to("value : 1 = 1 + 1", BooleanValue(false));

        // @Todo: complete
        // is_evaluating_to_error("value : (1 = 1 + 1) + 2",
        //                        ExpressionEnum::variable("value"),
        //                        RuntimeError::EvalError("Operator '+' is not implemented for 'false + 2'".to_string()));

        is_this_one_evaluating_to("value : 1 = 2 and 5 = 5", BooleanValue(false));
        is_this_one_evaluating_to("value : 1 + 1 = 2 and 5 = 5", BooleanValue(true));

        is_this_one_evaluating_to("value : 1 = 2 or 5 = 5", BooleanValue(true));
        is_this_one_evaluating_to("value : 1 = 2 or 5 = 5 + 1", BooleanValue(false));

        is_this_one_evaluating_to("value : 1 = 2 xor 5 = 5 + 1", BooleanValue(false));
        is_this_one_evaluating_to("value : 1 = 2 xor 5 = 4 + 1", BooleanValue(true));
        is_this_one_evaluating_to("value : 1 = 2 - 1 xor 5 = 5 + 1", BooleanValue(true));

        is_this_one_evaluating_to("value : 1 = 2 or 5 = 5 and 1 = 1", BooleanValue(true));
        is_this_one_evaluating_to("value : 1 = 2 or 5 = 5 and 1 = 1 + 1", BooleanValue(false));

        is_this_one_evaluating_to("value : if 1 > 2 then 3 else 4", V::from(4));
        is_this_one_evaluating_to("value : if 1 < 2 then 3 else 4", V::from(3));
        is_this_one_evaluating_to("value : if 1 < 2 then 3 + 1 else 5", V::from(4));
        is_this_one_evaluating_to("value : if 1 > 2 then 3 + 1 else 5 * 10", V::from(50));
        is_this_one_evaluating_to(
            "value : if 1 > 2 then 3 + 1 else (if 1 < 2 then 5 * 10 else 0)",
            V::from(50),
        );
        is_this_one_evaluating_to(
            "value : if 1 > 2 then 3 + 1 else (if 1 > 2 then 5 * 10 else 0)",
            V::from(0),
        );
        is_this_one_evaluating_to(
            "value : if 1 < 2 then (if 5 > 2 then 5 * 10 else 0) else 1",
            V::from(50),
        );
        is_this_one_evaluating_to(
            "value : (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1",
            V::from(51),
        );
        is_this_one_evaluating_to(
            "value : 1 + (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1",
            V::from(52),
        );
        is_this_one_evaluating_to(
            "value : 2 * (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1",
            V::from(101),
        );
    }

    #[test]
    fn test_boolean_literals_and_logic() {
        use ValueEnum::BooleanValue as B;
        info!(">>> test_boolean_literals_and_logic()");

        // OR truth table
        is_this_one_evaluating_to("value : true  or true", B(true));
        is_this_one_evaluating_to("value : true  or false", B(true));
        is_this_one_evaluating_to("value : false or true", B(true));
        is_this_one_evaluating_to("value : false or false", B(false));

        // AND truth table
        is_this_one_evaluating_to("value : true  and true", B(true));
        is_this_one_evaluating_to("value : true  and false", B(false));
        is_this_one_evaluating_to("value : false and true", B(false));
        is_this_one_evaluating_to("value : false and false", B(false));

        // XOR truth table
        is_this_one_evaluating_to("value : true  xor true", B(false));
        is_this_one_evaluating_to("value : true  xor false", B(true));
        is_this_one_evaluating_to("value : false xor true", B(true));
        is_this_one_evaluating_to("value : false xor false", B(false));

        // NOT operator
        is_this_one_evaluating_to("value : not true", B(false));
        is_this_one_evaluating_to("value : not false", B(true));
        is_this_one_evaluating_to("value : not (1 = 1)", B(false));
        is_this_one_evaluating_to("value : not (1 = 2)", B(true));

        // Mixed with comparisons
        is_this_one_evaluating_to("value : true and (1 < 2)", B(true));
        is_this_one_evaluating_to("value : (1 = 1) and false", B(false));
        is_this_one_evaluating_to("value : (1 = 1) or false", B(true));
        is_this_one_evaluating_to("value : true and not false", B(true));
        is_this_one_evaluating_to("value : (1 < 2) and not (2 < 1)", B(true));

        // More complex cases simulating rulesets
        is_this_one_evaluating_to(
            "value : (true and (1 < 2)) or (false and (3 = 4))",
            B(true),
        );
        is_this_one_evaluating_to(
            "value : (true xor (1 = 1 and false)) or (2 < 1)",
            B(true),
        );
        is_this_one_evaluating_to(
            "value : (true and true) xor (false or (1 < 1))",
            B(true),
        );
        is_this_one_evaluating_to(
            "value : (true and (2 > 1 and (3 > 2))) and (false or (5 = 5))",
            B(true),
        );
    }

    #[test]
    fn test_filter_not_alias() {
        use ValueEnum::NumberValue as N;
        use crate::typesystem::types::number::NumberEnum::Int;

        // not with implicit context variable alias 'it'
        is_this_one_evaluating_to(
            "value : count([1, 5, 12, 7][not it > 10])",
            N(Int(3)),
        );

        // not with explicit context variable '...'
        is_this_one_evaluating_to(
            "value : count([1, 5, 12, 7][not ... > 10])",
            N(Int(3)),
        );

        // combine inside filter
        is_this_one_evaluating_to(
            "value : count([1, 5, 12, 7, 15][(it > 3) and not (it > 10)])",
            N(Int(2)),
        );
    }

    #[test]
    fn test_constraints() {
        info!(">>> test_constraints()");
        init_logger();
        is_this_one_evaluating_to("value : [1,2,3][...>1]", V::from(vec![2, 3]));
        is_this_one_evaluating_to("value : [1,2,3][...>0]", V::from(vec![1, 2, 3]));
        is_this_one_evaluating_to("value : [1,2,3][...>-5]", V::from(vec![1, 2, 3]));
        is_this_one_evaluating_to(
            "value : [1,2,3][...<-5]",
            ValueEnum::Array(vec![], NumberType),
        );
        is_lines_evaluating_to(
            vec![
                "nums : [1, 5, 12, 7]",
                "filtered: nums[...>6]",
            ],
            &mut E::variable("filtered"),
            V::from(vec![12, 7]),
        );
        is_lines_evaluating_to(
            vec![
                "input : {",
                "   nums : [1, 5, 12, 7]",
                "   filtered: nums[...>6]",
                "}",
            ],
            &mut E::variable("input.filtered"),
            V::from(vec![12, 7]),
        );
    }

    #[test]
    fn variable_linkin_test() {
        init_test("variable_linkin_test()");

        is_lines_evaluating_to(
            vec![
                "input : {",
                "   application: {",
                "      status: 1",
                "   }",
                "}",
                "model: {",
                "   output: input.application.status",
                "}",
            ],
            &mut E::variable("model.output"),
            V::from(1),
        );

        is_lines_evaluating_to(
            vec![
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
            &mut E::variable("model.output"),
            V::from("ok"),
        );
    }

    #[test]
    fn test_problems() {
        init_test("test_problems");

        test_code("{ record : { age : 18; value : 1 + 2 }}").expect_num("record.value", Int(3));

        test_code("value : value + 1")
            .expect_link_error(CyclicReference("Root".to_string(), "value".to_string()));

        test_code("{ record1 : 15 + record2; record2 : 7 + record3; record3 : record1 * 10 }")
            .expect_link_error(CyclicReference("Root".to_string(), "record1".to_string()));

        test_code("{ record1 : { age : 18}; record2 : record1.age + record1.age}")
            .expect_num("record2", Int(36));

        test_code("{ p : [{a:1},5] }")
            .expect_no_errors()
            .expect_code_contains("[{a : 1}, 5]");
    }
}
