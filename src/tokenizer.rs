mod builder;
pub mod parser;
pub mod utils;

pub const C_ASSIGN: char = ':';

/// Test Cases
///
#[cfg(test)]
mod test {
    use crate::ast::utils::array_to_code_sep;
    use crate::tokenizer::parser::tokenize;
    use crate::utils::test::*;
    use log::info;

    fn is_equals(code: &str, expected: &str) {
        let result = &tokenize(&code.to_string());
        let result_line = array_to_code_sep(result.iter(), ", ");

        if result.len() > 1 {
            panic!(
                "Expected only one token, but got {}.\n text:\n {:?}\n tokens:\n {:?}",
                result.len(),
                result_line,
                result
            );
        }

        info!("{:?}", result_line);
        assert_eq!(expected.replace(" ", ""), result_line.replace(" ", ""));
    }

    #[test]
    fn test_first() {
        init_test("test_first");

        //is_equals("value : count(1,2,3) + 1", "value : count(1,2,3) + 1");
        //is_equals("value : if (1 > 2) then 1 else if (2 > 3) then 2 else 3", "value : if 1 > 2 then 1 else if 2 > 3 then 2 else 3");
    }

    #[test]
    fn test_common() {
        init_test("test_common");

        is_equals("value : sum(1,sum(7,8),3)", "value : sum(1,sum(7,8),3)");
        is_equals("value : 1 + 7 - 8 / 3 * 10", "value : 1+(7-8/3*10)");
        is_equals("value : 1 + -2", "value : 1 + -2");
        is_equals("value : -1 + 2", "value : -1 + 2");
        is_equals("value : - (-2*10)", "value:-(-(2*10))");
        is_equals(
            "value : (1 + 7 * (5 / 6 + (2-1))-1) - 8 / 3 * 10",
            "value:(1+(7*(5/6+(2-1))-1))-8/3*10",
        );
        is_equals(
            "{ record : { age : 18; value : 1 + 2 }}",
            "{record:{age:18;value:1+2}}",
        );
        is_equals("{ r : { a : 1 + 2} b : 3}", "{r:{a:1+2};b:3}");
        is_equals("{ r : { a : 1 + 2}; b : 3}", "{r:{a:1+2};b:3}"); // testing comma separator that should be OK
        is_equals(
            "{ record : { age1 : 11; variable : 100; variable : 200 }; record2 : { age2 : 22 }}",
            "{record:{age1:11;variable:200;variable:200};record2:{age2:22}}",
        );
        is_equals("value : [1,sum(9,8),3]", "value:[1,sum(9,8),3]");
        is_equals(
            "record(input) : { age : input.age } ",
            "record(input):{age:input.age}",
        );
        is_equals(
            "{doublethis(input) : { out : input * input }; result : doublethis(2).out }",
            "{doublethis(input):{out:input*input};result:doublethis(2).out}",
        );
        is_equals(
            "resultset : for x in [1,2,3] return x * 2",
            "resultset : for x in [1,2,3] return x * 2",
        );
        is_equals(
            "applicants : [{age:23},{age:34}]",
            "applicants:[{age:23},{age:34}]",
        );
        is_equals("p : [{a:1},5]", "p:[{a:1},5]");
        is_equals("myFunc(x,y,z) : {a : 1}", "myFunc(x,y,z):{a:1}");
        is_equals(
            "result : sales[month] + sales[month + 1] + sales[month + 2]",
            "result:(sales[month]+sales[(month+1)])+sales[(month+2)]",
        );

        // constraints
        is_equals("value : ...> 1 ", "value: ...> 1");

        // if, else
        is_equals(
            "value : if 1 > 2 then 1 else 2",
            "value : if 1 > 2 then 1 else 2",
        );
        is_equals(
            "value : if (1 > 2) then 1 else 2",
            "value : if 1 > 2 then 1 else 2",
        );
        is_equals(
            "value : if (1 > 2) then 1 else 2 * 5",
            "value: if 1 > 2 then 1 else 2 * 5",
        );
        is_equals(
            "value : if (1 > 2 * 9) then 1 + 1 else 2 * 5",
            "value : if 1 > 2 * 9 then 1 + 1 else 2 * 5",
        );
        is_equals(
            "value : if (1 > 2) then 1 else (if (2 > 3) then 2 else 3)",
            "value : if 1 > 2 then 1 else if 2 > 3 then 2 else 3",
        );

        // @Todo: complete, 3 + 5 is a mistake
        is_equals(
            "value : (if (1 > 2) then 1 else (if (2 > 3) then 2 else 3)) + 5",
            "value : if 1 > 2 then 1 else if 2 > 3 then 2 else 3 + 5",
        );
        is_equals(
            "value : if (1 > 2) then 1 else (if (2 > 3) then 2 else 3)",
            "value : if 1 > 2 then 1 else if 2 > 3 then 2 else 3",
        );
        is_equals(
            "value : if (1 > 2) then (if (2 > 3) then 2 else 3) else (if (2 > 3) then 2 else 3)",
            "value : if 1 > 2 then if 2 > 3 then 2 else 3 else if 2 > 3 then 2 else 3",
        );

        // string type
        is_equals("value : 'hello'", "value:'hello'");

        // @Todo: complete
        //is_equals("value : 'hello' + 'world'", "value:'hello'+'world'");

        // filtering
        is_equals("value : [1,2,3][0]", "value:[1,2,3][0]");
        is_equals("value : [1,2,3][>1]", "value:[1,2,3][...>1]");
        is_equals("value : [1,2,3][...>1]", "value:[1,2,3][...>1]");
        is_equals(
            "value : [1,2,3][...>=2 and ...<=3]",
            "value:[1,2,3][...>=2 and ...<=3]",
        );
        is_equals("value : [1,2,3][position-1]", "value:[1,2,3][(position-1)]");
        is_equals(
            "value : application.applicant[0]",
            "value:application.applicant[0]",
        );
        is_equals(
            "value : application.applicant[0].age",
            "value:application.applicant[0].age",
        );
        is_equals(
            "value : application.applicant[<=1].age",
            "value:application.applicant[...<=1].age",
        );
        is_equals(
            "value : application.applicant[sum(1,2)].age",
            "value:application.applicant[sum(1,2)].age",
        );

        // Variable and variable path:
        is_equals("value : subContext", "value : subContext");
        is_equals("value : subContext*1", "value : subContext*1");
        is_equals(
            "value : subContext.subResult",
            "value : subContext.subResult",
        );
        is_equals(
            "value : subContext.subResult+1",
            "value : subContext.subResult+1",
        );

        // Nested arrays
        is_equals("value : [1,2,3]", "value:[1,2,3]");
        is_equals(
            "value : [[key,value],[1,2],[3,4]]",
            "value : [[key,value],[1,2],[3,4]]",
        );
        is_equals(
            "value : [[key,value],[,2],[3,4]]",
            "Veryfirstsequenceelementismissing→'value'assignmentsideisnotcomplete",
        );

        // Various functions
        is_equals("value : max(1) + 1", "value : max(1) + 1");
        is_equals("value : sum(1,2) + 1", "value : sum(1,2) + 1");
        is_equals("value : sum(1,2,3) + 1", "value : sum(1,2,3) + 1");
        is_equals("value : count([1,2,3]) + 1", "value : count([1,2,3]) + 1");

        // Complex structures
        is_equals("{ p : [{a:1},5] }", "{ p : [{a:1},5] }");

        // Testing annotations:
        is_equals(
            "@Service myFunc(x,y,z) : {a : 1}",
            "@Service myFunc(x,y,z) : {a : 1}",
        );
        //is_equals("@DecisionTable myFunc(x,y) : [[x,y,z],[1,2,3]]", "@DecisionTable(\"\"first-hit\"\") myFunc(x:any,y:any) : [[x,y,z],[x=1,y=2,3]]");
    }

    #[test]
    fn test_errors() {
        init_logger();

        is_equals(
            "p : [{a:},5]",
            "'a'assignmentsideisnotcomplete→'p'assignmentsideisnotcomplete",
        );
    }

    #[test]
    fn test_range() {
        init_logger();

        is_equals("p : 1..5", "p : 1..5");
        is_equals(
            "p : for number in 1..5 return number * 2",
            "p : for number in 1..5 return number * 2",
        );
        is_equals(
            "p : for number in 1..(5+inc) return number * 3",
            "p : for number in 1..(5+inc) return number * 3",
        );
        is_equals(
            "p : for number in 1 * 0 .. 5+inc return number * 3",
            "p : for number in 1*0..(5+inc) return number * 3",
        );
    }

    #[test]
    fn test_functions() {
        init_logger();

        is_equals("p : sum(2,2 * sum(1,1))", "p : sum(2,2 * sum(1,1))");
        is_equals(
            "p : sum(2 * sum(3,3),2 * sum(1,1))",
            "p : sum(2 * sum(3,3),2 * sum(1,1))",
        );
        is_equals(
            "value : sum(1,2,3 + sum(2,2 * sum(0,0,0,0))) + (2 * 2)",
            "value:sum(1,2,(3+sum(2,2*sum(0,0,0,0))))+2*2",
        );
        is_equals("value : [1,1*sum(9,8^3)^2,3]", "value:[1,1*sum(9,8^3)^2,3]");
    }

    #[test]
    fn test_conditionals() {
        is_equals("p : 1 = a", "p : 1 = a");
        is_equals("p : 2 >= a", "p : 2 >= a");
        is_equals("p : 3 <= a", "p : 3 <= a");
        is_equals("p : 4 <> a", "p : 4 <> a");
        is_equals("p : 4 <> a + 1", "p : 4 <> (a + 1)");

        is_equals("p : a and b", "p : a and b");
        is_equals("p : a or b", "p : a or b");
        is_equals("p : a xor b", "p : a xor b");
    }
}
