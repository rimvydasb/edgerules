mod utilities;
use edge_rules::ast::metaphors::metaphor::UserFunction;
use edge_rules::ast::token::ExpressionEnum;
use edge_rules::ast::user_function_call::UserFunctionCall;
use edge_rules::link::node_data::ContentHolder;
use edge_rules::runtime::edge_rules::{EdgeRulesModel, UserFunctionDefinition};
use edge_rules::test_support::expr;
use edge_rules::typesystem::types::TypedValue;
use edge_rules::typesystem::values::ValueEnum;
pub use utilities::*;

// Dedicated coverage for user-defined functions (custom functions)

#[test]
fn execute_no_arg_function_root_manual() {
    let code = r#"
    {
        func main(): { result: 420 }
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    let result = runtime.call_method("main", vec![]).unwrap();

    if let ValueEnum::Reference(ctx) = result {
        let borrowed = ctx.borrow();
        let val = borrowed.get("result").unwrap();
        assert_eq!(format!("{}", val), "420");
    } else {
        panic!("Expected reference result");
    }
}

#[test]
fn execute_no_arg_function_nested_manual() {
    let code = r#"
    {
        nested: {
            func getVal(): { val: 100 }
        }
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();

    let method_entry = model.get_user_function("nested.getVal").unwrap();
    let definition = method_entry.borrow().function_definition.create_context(vec![], None).unwrap();

    let runtime = model.to_runtime().unwrap();

    let mut call = UserFunctionCall::new("nested.getVal".to_string(), vec![]);
    call.return_type = Ok(definition.get_type());
    call.definition = Ok(definition);

    let result = runtime.evaluate_expression(ExpressionEnum::from(call)).unwrap();

    if let ValueEnum::Reference(ctx) = result {
        let borrowed = ctx.borrow();
        let val = borrowed.get("val").unwrap();
        assert_eq!(format!("{}", val), "100");
    } else {
        panic!("Expected reference result");
    }
}

#[test]
fn unhappy_execute_no_arg_function_with_arg() {
    let code = r#"
    {
        func main(): { result: 420 }
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // Call with 1 argument
    let args = vec![ExpressionEnum::from(1)];
    let result = runtime.call_method("main", args);

    match result {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(msg.contains("expects 0 arguments, but 1 were provided"));
        }
        Ok(_) => panic!("Should have failed"),
    }
}

#[test]
fn unhappy_execute_arg_function_with_no_arg() {
    let code = r#"
    {
        func add(a, b): { result: a + b }
    }
    "#;
    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let runtime = model.to_runtime().unwrap();

    // Call with 0 arguments
    let result = runtime.call_method("add", vec![]);

    match result {
        Err(e) => {
            let msg = format!("{}", e);
            assert!(msg.contains("expects 2 arguments, but 0 were provided"));
        }
        Ok(_) => panic!("Should have failed"),
    }
}

#[test]
fn user_function_with_list_argument_and_return_list() {
    // Map over a list inside a user function and return a new list
    assert_eval_value(
        r#"
        func doubleAll(xs): {
            result: for x in xs return x * 2
        }
        value: doubleAll([1,2,3]).result
        "#,
        "[2, 4, 6]",
    );
}

#[test]
fn context_functions_duplicate() {
    let model = r#"
    {
        ctx: {
            func calc(x): { result: x + 1 }
            func calc(x): { result: x + 2 }
        }
    }
    "#;

    parse_error_contains(model, &["duplicate function 'calc'"]);

    assert_eval_value(
        r#"
        {
            func inc(x): { result: x + 2 }
            ctx: {
                func inc(x): { result: x + 1 }
                baseline: inc(7).result
            }
            value: ctx.baseline
        }
        "#,
        "8",
    );

    let model = r#"
    {
        func inc(x): { result: x + 2 }
        ctx: {
            func inc(x): { result: x + 1 }
            inc: 777
            baseline: inc(7).result
        }
        value: ctx.baseline
    }
    "#;

    parse_error_contains(model, &["duplicate field 'inc'"]);

    assert_eval_value(
        r#"
        {
            func echo(v): { value: v + 2 }
            ctx: {
                func echo(v): { value: v + 1 }
                nested: {
                    func echo(v): { value: v }
                    fallback: echo(10).value
                }
                fallback: echo(10).value
            }
            value: ctx.fallback
        }
        "#,
        "11",
    );

    let model = r#"
    {
        ctx: {
            nested: {
                func echo(v): { value: v }
                func echo(v): { value: v + 1 }
            }
        }
    }
    "#;

    parse_error_contains(model, &["duplicate function 'echo'"]);
}

#[test]
fn user_function_with_list_stats_and_nested_access() {
    // Accept a list, compute stats in the function body, and read nested fields
    assert_eval_value(
        r#"
        func listStats(xs): {
            total: sum(xs)
            maxVal: max(xs)
            first: xs[0]
            doubled: for v in xs return v * 2
        }
        value: listStats([1,5,3]).total
        "#,
        "9",
    );

    assert_eval_value(
        r#"
        func listStats(xs): {
            total: sum(xs)
            maxVal: max(xs)
            first: xs[0]
            doubled: for v in xs return v * 2
        }
        value: listStats([1,5,3]).maxVal
        "#,
        "5",
    );

    assert_eval_value(
        r#"
        func listStats(xs): {
            total: sum(xs)
            maxVal: max(xs)
            first: xs[0]
            doubled: for v in xs return v * 2
        }
        value: listStats([9,5,3]).first
        "#,
        "9",
    );

    assert_eval_value(
        r#"
        func listStats(xs): {
            total: sum(xs)
            maxVal: max(xs)
            first: xs[0]
            doubled: for v in xs return v * 2
        }
        value: listStats([2,1]).doubled
        "#,
        "[4, 2]",
    );
}

#[test]
fn cannot_define_user_function_inside_list_literal() {
    // Defining a function inside a list should be a parse error.
    // Parse as a pure expression to ensure the function definition token appears inside the sequence.
    // Expect: "Function definition is not allowed in sequence"
    let expr_str = "[ func myFunc(a): { out: a } ]";
    match expr(expr_str) {
        Ok(_) => panic!("expected parse error, but expression parsed successfully"),
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("Function definition is not allowed in sequence"),
                "expected parse error about function definition in sequence, got: {}",
                msg
            );
        }
    }
}

#[test]
fn cannot_pass_self_context_as_any_argument() {
    // Mirror and extend the guard: cannot pass the same context object into a function defined in it
    link_error_contains(
        r#"
        calendar: {
            shift: 2
            func start1(calendar): { result: calendar.shift + 1 }
            firstDay: start1(calendar).result
        }
        "#
        .trim(),
        &["Cannot pass context `calendar` as argument to function `start1`"],
    );

    link_error_contains(
        r#"
        calendar: {
            shift: 2
            func start2(x, cal): { result: cal.shift + x }
            firstDay: start2(1, calendar).result
        }
        "#
        .trim(),
        &["Cannot pass context `calendar` as argument to function `start2`"],
    );
}

#[test]
fn can_pass_sub_context_with_other_functions_and_use_them() {
    // User can pass a sub-context that contains other fields (and even functions) to another function,
    // and still use root-level functions at the call site.
    // (1+2)+1 = 4, (2+2)+1 = 5
    assert_eval_value(
        r#"
        func inc(a): { r: a + 1 }
        func apply(list, cfg): {
            mapped: [ list[0] + cfg.shift, list[1] + cfg.shift ]
        }
        helpers: {
            shift: 2
            func dec(a): { r: a - 1 }
        }
        value: for n in apply([1,2], helpers).mapped return inc(n).r
        "#,
        "[4, 5]",
    );
}

#[test]
fn application_record_example_extended_with_lists() {
    // Extend the applicationRecord(application) pattern with a list field
    let code = r#"
    {
        input: {
            application: {
                status: 1
                scores: [10, 20, 5]
            }
        }
        model: {
            func applicationRecord(application): {
                statusFlag: if application.status = 1 then 'ok' else 'no'
                maxScore: max(application.scores)
                doubled: for s in application.scores return s * 2
            }
            output1: applicationRecord(input.application).statusFlag
            output2: applicationRecord(input.application).maxScore
            output3: applicationRecord(input.application).doubled
        }
    }
    "#;

    assert_eval_field(code, "model.output1", "'ok'");
    assert_eval_field(code, "model.output2", "20");
    assert_eval_field(code, "model.output3", "[20, 40, 10]");
}

#[test]
fn user_function_body_is_fully_evaluated() {
    init_logger();

    let code = r#"
    func testFunction(a,b,c): {
        sumAll: sum([a,b,c])
        lvl1: { result: sumAll * 2 }
        lvl2: { result: lvl1.result + 1 }
    }
    all: testFunction(1,2,3)
    output1: testFunction(1,2,3).lvl2.result
    structOutput: testFunction(1,2,3).lvl1
    structOutputValue: structOutput.result
    "#;

    assert_eval_all(
        code,
        &[
            "{",
            "   all: {",
            "      sumAll: 6",
            "      lvl1: {",
            "         result: 12",
            "      }",
            "      lvl2: {",
            "         result: 13",
            "      }",
            "   }",
            "   output1: 13",
            "   structOutput: {",
            "      result: 12",
            "   }",
            "   structOutputValue: 12",
            "}",
        ],
    );
}

#[test]
fn user_function_field_with_math_operator() {
    let code = r#"
    func testFunction(a,b,c): {
        sumAll: sum([a,b,c])
        lvl1: { result: sumAll * 2 }
        lvl2: { result: lvl1.result + 1 }
    }
    output1: testFunction(1,2,3).lvl2.result + 1
    "#;

    assert_eval_all(code, &["{", "output1: 14", "}"]);
}

#[test]
fn user_function_has_types() {
    let code = r#"
    func testFunction(a: number,b: string,c: date): {
        sumAll: a + c.month
        label: toString(a) + b
    }
    all: testFunction(1,'x', date('2023-05-03'))
    output1: testFunction(1,'x', date('2023-05-03')).sumAll
    output2: testFunction(1,'x', date('2023-05-03')).label
    "#;

    assert_eval_all(code, &["{", "all: {", "sumAll: 6", "label: '1x'", "}", "output1: 6", "output2: '1x'", "}"]);
}

#[test]
fn user_function_argument_type_mismatch_errors() {
    let model = r#"
    {
        func typed(a: number, b: string): { result: toString(a) + b }
        value: typed('oops', 'fail')
    }
    "#;

    link_error_contains(model, &["Argument `a`", "number", "string"]);
}

#[test]
fn user_function_arguments_duplicate() {
    let model = r#"
    {
        func typed(a: number, b: string, b: date): { result: toString(a) + b }
        value: typed('oops', 'fail')
    }
    "#;

    parse_error_contains(model, &["Duplicate function argument name 'b'"]);
}

#[test]
fn user_function_accepts_list_parameter_type() {
    let code = r#"
    func total(values: number[]): {
        size: count(values)
        sum: sum(values)
    }
    count: total([1,2,3]).size
    sum: total([1,2,3]).sum
    "#;

    assert_eval_all(code, &["{", "count: 3", "sum: 6", "}"]);
}

#[test]
fn user_function_list_argument_type_mismatch_errors() {
    let model = r#"
    {
        func total(values: number[]): { sum: sum(values) }
        bad: total(['a']).sum
    }
    "#;

    link_error_contains(model, &["Argument `values`", "number[]", "string"]);
}

#[test]
fn user_function_accepts_alias_and_fills_missing_fields() {
    let model = r#"
    {
        type Customer: {name: <string>; birthdate: <date>; income: <number>}
        func normalize(customer: Customer): {
            copy: customer
        }
        result: normalize({name: 'Sara'}).copy
    }
    "#;

    let evaluated = eval_all(model);

    assert_string_contains("name: 'Sara'", &evaluated);
    assert_string_contains("birthdate: Missing", &evaluated);
    assert_string_contains("income: Missing", &evaluated);
}

#[test]
fn user_function_not_found() {
    let model = "{ value: inc(1) }";
    link_error_contains(model, &["Function 'inc(...)' not found", "No metaphors in scope"]);

    let model = r#"
    {
        func addOne(value: number): { result: value + 1 }
        value: missingFunc(1)
    }
    "#;

    link_error_contains(model, &["Function 'missingFunc(...)' not found", "Known metaphors in scope", "addOne(...)"]);

    let model = r#"
    {
        deeper: { func inc(x): { result: x + 1 } }
        value: inc(1).result
    }
    "#;

    link_error_contains(model, &["Function 'inc(...)' not found", "No metaphors in scope"]);

    let model = r#"
    {
        deeper: { func inc(x): { result: x + 1 } }
        value: deeper.inc(1).result
    }
    "#;

    assert_eval_value(model, "2");
}

#[test]
fn user_function_deeper_level_call_is_allowed() {
    let model = r#"
    {
        deeper: {
            func inc(x): { result: x + 1 }
            value: inc(1).result
        }
        value: deeper.value
    }
    "#;

    assert_eval_all(model, &["{", "deeper: {", "value: 2", "}", "value: 2", "}"]);
}

#[test]
fn user_function_nesting_is_allowed_and_function_context_is_forgotten() {
    let model = r#"
    {
        deeper: {
            func inc(x): {
                func helper(y): {
                    result: y * 10
                }
                result: helper(x).result + 1
            }
            value1: inc(1).result
            value2: inc(5).result
        }
        value: deeper.value1 + deeper.value2
    }
    "#;

    assert_eval_all(model, &["{", "deeper: {", "value1: 11", "value2: 51", "}", "value: 62", "}"]);
}

#[test]
fn accessing_function_in_different_context() {
    // Function `incAmount`, that is defined in upper context, is accessible in lower context.
    let code = r#"
    type Application: {
        loanAmount: <number>;
    }
    func parentContextFunction(amount: number): {
        func sameContextFunction(x): {
            result: x + 1
        }
        newAmount: sameContextFunction(amount).result
    }

    func applicationDecisions(amount: number): {
        oldAmount: amount
        newAmount: parentContextFunction(amount).newAmount
    }

    applicationResponse: applicationDecisions(1000)
    "#;

    let rt = get_runtime(code);

    assert_eval_field(&rt, "applicationResponse", "{oldAmount:1000newAmount:1001}");
}

#[test]
fn accessing_function_in_lower_context() {
    // Function `rootFunction` and `incAmount are defined in upper context, is accessible in lower context.
    let code = r#"
    func rootFunction(z: number): {
        zap: z * 1000
    }
    func processAmount(amount: number): {
        func incAmount(x: number): {
            result: x + 1
        }
        func doubleAndIncAmount(x: number): {
            result: incAmount(x + x).result + rootFunction(1).zap
        }
        newAmount: doubleAndIncAmount(amount).result
    }
    applicationResponse: processAmount(1000)
    "#;

    let rt = get_runtime(code);

    assert_eval_field(&rt, "applicationResponse.newAmount", "3001");

    assert_eq!(
        rt.get_type("*").unwrap().to_string(),
        "{applicationResponse: {newAmount: number}}"
    );
}

#[test]
fn inline_function_executes_simple_expression() {
    assert_eval_value(
        r#"
        {
            func addOne(x): x + 1
            value: addOne(2)
        }
        "#,
        "3",
    );
}

#[test]
fn inline_functions_support_nested_calls() {
    assert_eval_value(
        r#"
        {
            func addOne(x): x + 1
            func doubleAndAddOne(y): addOne(y * 2)
            value: doubleAndAddOne(3)
        }
        "#,
        "7",
    );
}

#[test]
fn return_field_scopes_result_for_functions() {
    assert_eval_value(
        r#"
        {
            func calc(a): {
                hidden: a * 2;
                return: hidden + 1
            }
            value: calc(4)
        }
        "#,
        "9",
    );
}

#[test]
fn return_field_blocks_field_access_on_result() {
    link_error_contains(
        r#"
        {
            func calc(a): {
                hidden: a * 2;
                return: hidden + 1
            }
            value: calc(4).hidden
        }
        "#,
        &["not an object", "calc(4)"],
    );
}

#[test]
fn return_field_in_plain_context_behaves_as_normal_field() {
    assert_eval_value(
        r#"
        {
            obj: { return: 5 + 5 }
            value: obj.return
        }
        "#,
        "10",
    );
}

#[test]
fn return_only_bodies_collapse_to_inline_definition() {
    let code = r#"
    {
        func add(a): { return: a + a }
    }
    "#;

    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let method = model.get_user_function("add").unwrap();
    let borrowed = method.borrow();

    match &borrowed.function_definition {
        UserFunctionDefinition::Inline(_) => {}
        _ => panic!("expected inline function collapse for return-only body"),
    }

    let display = inline_text(format!("{}", borrowed.function_definition));
    assert_eq!(display, "add(a):a+a");
}

#[test]
fn inline_functions_support_type_annotations() {
    assert_eval_value(
        r#"
        {
            func addOne(x: number): x + 1
            value: addOne(2)
        }
        "#,
        "3",
    );
}

#[test]
fn inline_functions_with_wrong_argument_type_fail_linking() {
    link_error_contains(
        r#"
        {
            func addOne(x: number): x + 1
            value: addOne("not a number")
        }
        "#,
        &["expect", "number", "string"],
    );
}

#[test]
fn inline_functions_with_no_arguments() {
    assert_eval_value(
        r#"
        {
            func getTen(): 10
            value: getTen()
        }
        "#,
        "10",
    );
}

#[test]
fn inline_functions_shadowing_outer_variables() {
    assert_eval_value(
        r#"
        {
            x: 10
            func shadow(x): x + 1
            value: shadow(5) // Should use argument x=5, not outer x=10
        }
        "#,
        "6",
    );
}

#[test]
fn inline_functions_using_complex_expressions_as_arguments() {
    assert_eval_value(
        r#"
        {
            func add(a, b): a + b
            value: add(1 + 2, 3 * 4)
        }
        "#,
        "15",
    );
}

#[test]
fn inline_functions_recursion_detection() {
    // EdgeRules doesn't support recursion because it uses static linking
    link_error_contains(
        r#"
        {
            func rec(x): if x > 0 then rec(x - 1) else 0
            value: rec(5)
        }
        "#,
        &["Cyclic reference", "rec"],
    );
}

#[test]
fn nested_inline_functions_in_expression() {
    assert_eval_value(
        r#"
        {
            func outer(x): {
                func inner(y): x + y
                return: inner(x)
            }
            value: outer(10)
        }
        "#,
        "20",
    );
}

#[test]
fn inline_function_returning_object() {
    assert_eval_value(
        r#"
        {
            func mkObj(a, b): { res: a + b }
            value: mkObj(1, 2).res
        }
        "#,
        "3",
    );
}

#[test]
fn inline_function_with_all_supported_types() {
    assert_eval_value(
        r#"
        {
            func allTypes(n: number, s: string, b: boolean, d: date): s + toString(n) + toString(b) + toString(d.year)
            value: allTypes(1, 'val', true, date('2024-01-19'))
        }
        "#,
        "'val1true2024'",
    );
}

#[test]
fn inline_function_shadowing_it_alias() {
    assert_eval_value(
        r#"
        {
            func check(it): it + 1
            list: [1, 2, 3]
            value: list[check(it) > 2] // it in check(it) is the list element
        }
        "#,
        "[2, 3]",
    );
}
