mod utilities;
pub use utilities::*;

// Dedicated coverage for user-defined functions (custom functions)

#[test]
fn user_function_with_list_argument_and_return_list() {
    // Map over a list inside a user function and return a new list
    assert_value!(
        r#"
        func doubleAll(xs) : {
            result : for x in xs return x * 2
        }
        value : doubleAll([1,2,3]).result
        "#,
        "[2, 4, 6]"
    );
}

#[test]
fn user_function_with_list_stats_and_nested_access() {
    // Accept a list, compute stats in the function body, and read nested fields
    let out_sum = eval_lines_field(
        &[
            "func listStats(xs) : {",
            "  total : sum(xs)",
            "  maxVal : max(xs)",
            "  first : xs[0]",
            "  doubled : for v in xs return v * 2",
            "}",
            "value : listStats([1,5,3]).total",
        ],
        "value",
    );
    assert_eq!(out_sum, "9");

    let out_max = eval_lines_field(
        &[
            "func listStats(xs) : {",
            "  total : sum(xs)",
            "  maxVal : max(xs)",
            "  first : xs[0]",
            "  doubled : for v in xs return v * 2",
            "}",
            "value : listStats([1,5,3]).maxVal",
        ],
        "value",
    );
    assert_eq!(out_max, "5");

    let out_first = eval_lines_field(
        &[
            "func listStats(xs) : {",
            "  total : sum(xs)",
            "  maxVal : max(xs)",
            "  first : xs[0]",
            "  doubled : for v in xs return v * 2",
            "}",
            "value : listStats([9,5,3]).first",
        ],
        "value",
    );
    assert_eq!(out_first, "9");

    let out_doubled = eval_lines_field(
        &[
            "func listStats(xs) : {",
            "  total : sum(xs)",
            "  maxVal : max(xs)",
            "  first : xs[0]",
            "  doubled : for v in xs return v * 2",
            "}",
            "value : listStats([2,1]).doubled",
        ],
        "value",
    );
    assert_eq!(out_doubled, "[4, 2]");
}

#[test]
fn cannot_define_user_function_inside_list_literal() {
    // Defining a function inside a list should be a parse error.
    // Parse as a pure expression to ensure the function definition token appears inside the sequence.
    // Expect: "Function definition is not allowed in sequence"
    let expr_str = "[ func myFunc(a) : { out : a } ]";
    match edge_rules::runtime::edge_rules::expr(expr_str) {
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
        &[
            "calendar : {",
            "  shift : 2",
            "  func start1(calendar) : { result : calendar.shift + 1 }",
            "  firstDay : start1(calendar).result",
            "}",
        ]
        .join("\n"),
        &["Cannot pass context `calendar` as argument to function `start1`"],
    );

    link_error_contains(
        &[
            "calendar : {",
            "  shift : 2",
            "  func start2(x, cal) : { result : cal.shift + x }",
            "  firstDay : start2(1, calendar).result",
            "}",
        ]
        .join("\n"),
        &["Cannot pass context `calendar` as argument to function `start2`"],
    );
}

#[test]
fn can_pass_sub_context_with_other_functions_and_use_them() {
    // User can pass a sub-context that contains other fields (and even functions) to another function,
    // and still use root-level functions at the call site.
    let out = eval_lines_field(
        &[
            // Helper function in the root scope
            "func inc(a) : { r : a + 1 }",
            // A function that takes a sub-context and a list and returns a mapped list (no nested calls inside)
            "func apply(list, cfg) : {",
            "  mapped : [ list[0] + cfg.shift, list[1] + cfg.shift ]",
            "}",
            // Build a sub-context with data and an extra (unused here) function
            "helpers : {",
            "  shift : 2",
            "  func dec(a) : { r : a - 1 }",
            "}",
            // Use root-level function `inc` on the results of `apply(...)`
            "value : for n in apply([1,2], helpers).mapped return inc(n).r",
        ],
        "value",
    );
    // (1+2)+1 = 4, (2+2)+1 = 5
    assert_eq!(out, "[4, 5]");
}

#[test]
fn application_record_example_extended_with_lists() {
    // Extend the applicationRecord(application) pattern with a list field
    let lines = vec![
        "input : {",
        "   application: {",
        "      status: 1",
        "      scores: [10, 20, 5]",
        "   }",
        "}",
        "model: {",
        "   func applicationRecord(application): {",
        "      statusFlag: if application.status = 1 then 'ok' else 'no'",
        "      maxScore: max(application.scores)",
        "      doubled: for s in application.scores return s * 2",
        "   }",
        "   output1: applicationRecord(input.application).statusFlag",
        "   output2: applicationRecord(input.application).maxScore",
        "   output3: applicationRecord(input.application).doubled",
        "}",
    ];

    assert_eq!(eval_lines_field(&lines, "model.output1"), "'ok'");
    assert_eq!(eval_lines_field(&lines, "model.output2"), "20");
    assert_eq!(eval_lines_field(&lines, "model.output3"), "[20, 40, 10]");
}

#[test]
fn user_function_body_is_fully_evaluated() {
    let lines = vec![
        "func testFunction(a,b,c): {",
        "   sumAll: sum([a,b,c])",
        "   lvl1: { result: sumAll * 2 }",
        "   lvl2: { result: lvl1.result + 1 }",
        "}",
        "all: testFunction(1,2,3)",
        "output1: testFunction(1,2,3).lvl2.result",
        "structOutput: testFunction(1,2,3).lvl1",
        "structOutputValue: structOutput.result",
    ];

    assert_eval_all(
        &lines,
        &[
            "{",
            "   {",
            "      sumAll : 6",
            "      lvl1 : {",
            "         result : 12",
            "      }",
            "      lvl2 : {",
            "         result : 13",
            "      }",
            "   }",
            "   output1 : 13",
            "   lvl1 : {",
            "      result : 12",
            "   }",
            "   structOutputValue : 12",
            "}",
        ],
    );
}

#[test]
fn user_function_field_with_math_operator() {
    let lines = vec![
        "func testFunction(a,b,c): {",
        "   sumAll: sum([a,b,c])",
        "   lvl1: { result: sumAll * 2 }",
        "   lvl2: { result: lvl1.result + 1 }",
        "}",
        "output1: testFunction(1,2,3).lvl2.result + 1",
    ];

    assert_eq!(eval_lines_field(&lines, "output1"), "14");
    assert_eval_all(&lines, &["{", "   output1 : 14", "}"]);
}

#[test]
fn user_function_has_types() {
    let lines = vec![
        "func testFunction(a: number,b: string,c: date): {",
        "   sumAll: a + c.month",
        "   label: toString(a) + b",
        "}",
        "all: testFunction(1,'x', date('2023-05-03'))",
        "output1: testFunction(1,'x', date('2023-05-03')).sumAll",
        "output2: testFunction(1,'x', date('2023-05-03')).label",
    ];

    assert_eval_all(
        &lines,
        &[
            "{",
            "   {",
            "      sumAll : 6",
            "      label : '1x'",
            "   }",
            "   output1 : 6",
            "   output2 : '1x'",
            "}",
        ],
    );
}

#[test]
fn user_function_argument_type_mismatch_errors() {
    let model = format!(
        "{{\n{}\n}}",
        [
            "func typed(a: number, b: string): { result: toString(a) + b }",
            "value: typed('oops', 'fail')",
        ]
        .join("\n"),
    );

    link_error_contains(&model, &["Argument `a`", "number", "string"]);
}

#[test]
fn user_function_accepts_list_parameter_type() {
    let lines = vec![
        "func total(values: number[]): {",
        "   size: count(values)",
        "   sum: sum(values)",
        "}",
        "count: total([1,2,3]).size",
        "sum: total([1,2,3]).sum",
    ];

    assert_eval_all(&lines, &["{", "   count : 3", "   sum : 6", "}"]);
}

#[test]
fn user_function_list_argument_type_mismatch_errors() {
    let model = format!(
        "{{\n{}\n}}",
        [
            "func total(values: number[]): { sum: sum(values) }",
            "bad: total(['a']).sum",
        ]
        .join("\n"),
    );

    link_error_contains(&model, &["Argument `values`", "list of number", "string"]);
}

#[test]
fn user_function_accepts_alias_and_fills_missing_fields() {
    let lines = vec![
        "type Customer: {name: <string>; birthdate: <date>; income: <number>}",
        "func normalize(customer: Customer): {",
        "   copy: customer",
        "}",
        "result: normalize({name: 'Sara'}).copy",
    ];

    let model = format!("{{\n{}\n}}", lines.join("\n"));
    let evaluated = eval_all(&model);

    assert_string_contains!("name : 'Sara'", &evaluated);
    assert_string_contains!("birthdate : Missing", &evaluated);
    assert_string_contains!("income : number.Missing", &evaluated);
}
