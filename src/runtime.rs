pub mod execution_context;
pub mod edge_rules;

/// Test Cases
///
#[cfg(test)]
mod test {
    use log::{info};
    use crate::utils::test::*;
    use crate::ast::token::{ExpressionEnum};
    use crate::runtime::edge_rules::{EvalError, expr};
    use crate::typesystem::errors::LinkingErrorEnum::{CyclicReference, FieldNotFound};
    use crate::typesystem::types::number::NumberEnum;
    use crate::typesystem::types::number::NumberEnum::{Int, Real};
    use crate::typesystem::types::SpecialValueEnum::Missing;
    use crate::typesystem::types::ValueType::NumberType;
    use crate::typesystem::values::ValueEnum;
    use crate::typesystem::values::ValueEnum::{BooleanValue};

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
        test_code("value : for x in [1,2,3] return x * 2").expect(&mut expr("value")?, V::from(vec![2, 4, 6]));

        // for each loop
        test_code("value : for x in [1,2,3] return x * 2").expect(&mut expr("value")?, V::from(vec![2, 4, 6]));
        test_code("value : for x in [1,2,3] return x * 2.0").expect(&mut expr("value")?, V::from(vec![2.0, 4.0, 6.0]));
        test_code("value : for x in 1..3 return x * 2").expect(&mut expr("value")?, V::from(vec![2, 4, 6]));
        test_code("value : for x in [{age:23},{age:34}] return x.age + 2").expect(&mut expr("value")?, V::from(vec![25, 36]));

        test_code("value : 2 / 3").expect_num("value", Real(0.6666666666666666));
        test_code("value : 1 * 2 / 3 + 1 - 2").expect_num("value", Real(-0.33333333333333337));
        assert_eq!(1.0 * 2.0 / 3.0 + 1.0 - 2.0, -0.3333333333333335);

        test_code("{ age : 18; value : 1 + 2 }").expect_num("value", Int(3));

        // Selection
        is_variable_evaluating_to("{ record : [1,2,3]; record2 : record[1]}", "record2", V::from(2));
        is_variable_evaluating_to("{ list : [1,2,3]; value : list[0] * list[1] + list[2]}", "value", V::from(5));
        is_variable_evaluating_to("{ list : [1,2,3]; value : list[0] * (list[1] + list[2] * 3)}", "value", V::from(11));

        is_evaluating_to("{ record : { age : 18; value : 1 + 2 }}",
                         &mut ExpressionEnum::variable("record.value"), V::from(3));

        test_code("{ record : { age : somefield; value : 1 + 2 }}")
            .expect_link_error(FieldNotFound("Root".to_string(), "somefield".to_string()));

        is_evaluating_to("{ record : { age : 18; value : age + 1 }}",
                         &mut ExpressionEnum::variable("record.value"), V::from(19));

        is_evaluating_to("{ record : { age : 18; value : age + 2 + addition; addition : age + 2 }}",
                         &mut ExpressionEnum::variable("record.value"), V::from(40));

        is_evaluating_to("{ record : { age : 18; value : record.age + 1 }}",
                         &mut ExpressionEnum::variable("record.value"), V::from(19));

        is_evaluating_to("{ record : { value : record2.age2 }; record2 : { age2 : 22 }}",
                         &mut ExpressionEnum::variable("record.value"), V::from(22));

        is_evaluating_to("{ record : { age : 18; value : age + 2 + addition; addition : age + record2.age2 }; record2 : { age2 : 22 }}",
                         &mut ExpressionEnum::variable("record.value"), V::from(60));


        is_evaluating_to("{ doublethis(input) : { out : input * input }; result : doublethis(2).out }",
                         &mut ExpressionEnum::variable("result"), V::from(4));


        // @todo: this is not working yet
        //test_code("p : [{a:1},5]").expect_code("p: [{a : 1}, 5]");

        Ok(())
    }

    #[test]
    fn test_functions() -> Result<(), EvalError> {
        init_test("test_functions()");

        test_code("value : 2 * 2").expect_num("value", Int(4));
        test_code("value : sum(1,2,3) + (2 * 2)").expect_num("value", Int(10));
        test_code("value : sum(1,2,3 + sum(2,2 * sum(0,1,0,0))) + (2 * 2)").expect_num("value", Int(14));
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
        ]).expect_num("value", Int(35));

        test_code_lines(&[
            "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
            "salesIn3Months(month,sales) : {",
            "result : sales[month] + sales[month + 1] + sales[month + 2]",
            "}",
            "value : salesIn3Months(1,inputSales).result",
        ]).expect_num("value", Int(35));

        test_code_lines(&[
            "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
            "salesIn3Months(month,sales) : {",
            "result : sales[month] + sales[month + 1] + sales[month + 2]",
            "}",
            "subContext : {",
            "subResult : salesIn3Months(1,inputSales).result",
            "}",
            "value : subContext.subResult",
        ]).expect_num("value", Int(35));

        test_code_lines(&[
            "inputSales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]",
            "salesIn3Months(month,sales) : {",
            "result : sales[month] + sales[month + 1] + sales[month + 2]",
            "}",
            "bestMonths : for monthIter in 0..11 return salesIn3Months(monthIter,inputSales).result",
            "value : bestMonths[0]",
        ]).expect_num("value", Int(38));
    }

    //#[test]
    #[allow(dead_code)]
    fn tables_test() {
        init_logger();

        info!(">>> tables_test()");

        is_lines_evaluating_to(vec![
            "@DecisionTable",
            "simpleTable(age,score) : [",
            "[age, score, eligibility],",
            "[18, 300, 0],",
            "[22, 100, 1],",
            "[65, 200, 0]",
            "]",
            "value : simpleTable(22,100).eligibility",
        ], &mut E::variable("value"), V::from(1));
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
        is_this_one_evaluating_to("value : if 1 > 2 then 3 + 1 else (if 1 < 2 then 5 * 10 else 0)", V::from(50));
        is_this_one_evaluating_to("value : if 1 > 2 then 3 + 1 else (if 1 > 2 then 5 * 10 else 0)", V::from(0));
        is_this_one_evaluating_to("value : if 1 < 2 then (if 5 > 2 then 5 * 10 else 0) else 1", V::from(50));
        is_this_one_evaluating_to("value : (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1", V::from(51));
        is_this_one_evaluating_to("value : 1 + (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1", V::from(52));
        is_this_one_evaluating_to("value : 2 * (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1", V::from(101));
    }

    #[test]
    fn test_constraints() {
        info!(">>> test_constraints()");
        init_logger();
        is_this_one_evaluating_to("value : [1,2,3][...>1]", ValueEnum::from(vec![2, 3]));
        is_this_one_evaluating_to("value : [1,2,3][...>0]", ValueEnum::from(vec![1, 2, 3]));
        is_this_one_evaluating_to("value : [1,2,3][...>-5]", ValueEnum::from(vec![1, 2, 3]));
        is_this_one_evaluating_to("value : [1,2,3][...<-5]", ValueEnum::Array(vec![], NumberType));
    }

    #[test]
    fn variable_linkin_test() {
        init_test("variable_linkin_test()");

        is_lines_evaluating_to(vec![
            "input : {",
            "   application: {",
            "      status: 1",
            "   }",
            "}",
            "model: {",
            "   output: input.application.status",
            "}",
        ], &mut E::variable("model.output"), V::from(1));

        is_lines_evaluating_to(vec![
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
        ], &mut E::variable("model.output"), V::from("ok"));
    }

    #[test]
    fn test_problems() {
        init_test("test_problems");

        test_code("{ record : { age : 18; value : 1 + 2 }}")
            .expect_num("record.value", Int(3));

        test_code("value : value + 1")
            .expect_link_error(CyclicReference("Root".to_string(),"value".to_string()));

        test_code("{ record1 : 15 + record2; record2 : 7 + record3; record3 : record1 * 10 }")
            .expect_link_error(CyclicReference("Root".to_string(), "record1".to_string()));

        test_code("{ record1 : { age : 18}; record2 : record1.age + record1.age}")
            .expect_num("record2", Int(36));

        test_code("{ p : [{a:1},5] }")
            .expect_no_errors()
            .expect_code_contains("[{a : 1}, 5]");
    }
}
