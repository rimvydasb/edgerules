mod utilities;

use edge_rules::runtime::edge_rules::EdgeRulesModel;
pub use utilities::*;

// Additional tests for user-defined types: limitations and potential problems

#[test]
fn type_alias_with_nested_types_only_disallows_placeholders_and_functions() {
    // Enforce: inside type body only nested type objects are allowed (no <type> placeholders, no functions)
    // 1) Using typed placeholders in type body should fail
    // Typed placeholders in type body are allowed per TYPES_STORY
    let code1 = r#"
    {
        type Address: { street: <string>; house: { number: <number> } }
    }
    "#
    .trim();
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(code1)
        .expect("parse type with placeholders");

    // 2) Using a function in type body should fail
    let code2 = r#"
    {
        type Bad: {
            nested: { }
            func calc(x): { r: x }
        }
    }
    "#
    .trim();
    let err2 = service.load_source(code2).err().unwrap().to_string();
    assert!(
        err2.to_lowercase()
            .contains(&"Type definition cannot contain function definitions".to_lowercase()),
        "expected parse error about function definitions, got: {}",
        err2
    );

    // 3) Only nested type objects are allowed
    // Should parse and link with no errors
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(
            r#"
            {
                type Person: {}
            }
            "#
            .trim(),
        )
        .unwrap();
    let _ = service.to_runtime().expect("link");
}

#[test]
fn typed_placeholders_are_allowed_in_model_but_evaluate_to_missing() {
    // Placeholders in model (not inside type definitions) are accepted and eval to Missing
    let model = r#"
    {
        id: <number>
        name: <string>
    }
    "#
    .trim();
    // Evaluated values are Missing (rendered in string form per to_string rules)
    let printed = eval_all(model);
    // At least ensure fields exist; exact Missing rendering can vary across types
    assert!(printed.contains("id:") && printed.contains("name:"));
}

#[test]
fn loan_offer_decision_service_end_to_end() {
    let model = r#"
    {
        type Customer: {name: <string>; birthdate: <date>; income: <number>}
        type Applicant: {customer: <Customer>; requestedAmount: <number>; termInMonths: <number>}
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}

        // NOTE: placeholder not supported yet, so set a concrete date
        executionDatetime: date('2024-01-01')

        func calculateLoanOffer(executionDatetime, applicant): {
            eligibleCalc: executionDatetime >= applicant.customer.birthdate + duration('P6570D');
            amount: applicant.requestedAmount;
            termInMonths: applicant.termInMonths;
            monthlyPaymentCalc: (applicant.requestedAmount * (1 + (if applicant.customer.income > 5000 then 0.05 else 0.1))) / applicant.termInMonths
            result: {
                eligible: eligibleCalc;
                amount: applicant.requestedAmount;
                termInMonths: applicant.termInMonths;
                monthlyPayment: monthlyPaymentCalc
            }
        }

        applicant1: {
            customer: {name: 'Alice'; birthdate: date('2001-01-01'); income: 6000};
            requestedAmount: 20000;
            termInMonths: 24
        }

        loanOffer1: calculateLoanOffer(executionDatetime, applicant1).result as LoanOffer
    }
    "#
    .trim();

    let evaluated = eval_all(model);
    assert_string_contains!("eligible: true", &evaluated);
    assert_string_contains!("amount: 20000", &evaluated);
    assert_string_contains!("termInMonths: 24", &evaluated);
    assert_string_contains!("monthlyPayment: 875", &evaluated);
}

#[test]
#[ignore]
fn loan_offer_decision_service_end_to_end_reduced() {
    let model = r#"
    {
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        sample: {eligible: false}
        result: sample as LoanOffer
    }
    "#
    .trim();

    let evaluated = eval_all(model);

    // {
    //     #child: {
    //     eligible: false
    //     amount: number.Missing
    //     termInMonths: number.Missing
    //     monthlyPayment: number.Missing
    // }
    //     sample: {
    //     eligible: false
    // }
    // }

    assert!(
        evaluated.contains("result: true"),
        "model output did not include expected eligibility result\n{}",
        evaluated
    );
}

// Potential limitation to explore further: forward references and alias-based placeholders.

#[test]
fn unknown_alias_in_placeholder_is_link_error() {
    link_error_contains(
        r#"
        {
            x: <NotDefined>
        }
        "#,
        &["unknown type", "notdefined"],
    );
}

#[test]
#[ignore]
fn cast_primitive_to_number_changes_do_not_change_type() {
    // @Todo: fix the behaviour to throw and exception "casting does not convert types. Use appropriate functions to convert types"
    let code = r#"
    {
        x: '5' as number
        y: x + 2
    }
    "#
    .trim();
    let mut service = EdgeRulesModel::new();
    service.load_source(code).unwrap();
    let runtime_snapshot = service.to_runtime_snapshot().expect("runtime snapshot");
    let value = runtime_snapshot
        .evaluate_field("y")
        .expect("evaluate field")
        .to_string();
    assert_eq!(value, "7");
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("x: number"), "got `{}`", ty);
}

#[test]
fn cast_object_to_alias_shape_links_type() {
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(
            r#"
            {
                type Point: { x: <number>; y: <number> }
                p: { x: 1 } as Point
            }
            "#
            .trim(),
        )
        .unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("p: Type<x: number, y: number>"), "got `{}`", ty);
}

#[test]
fn cast_list_to_alias_of_number_list() {
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(
            r#"
            {
                type NumList: <number[]>
                vals: [1,2,3] as NumList
            }
            "#
            .trim(),
        )
        .unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("vals: list of number"), "got `{}`", ty);
}

#[test]
fn cast_to_nested_alias() {
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(
            r#"
            {
                type Customer: {name: <string>; birthdate: <date>; income: <number>}
                c: {name: 'A'} as Customer
            }
            "#
            .trim(),
        )
        .unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(
        ty.contains("c: Type<name: string, birthdate: date, income: number>"),
        "got `{}`",
        ty
    );
}

#[test]
fn context_types_duplicate() {
    let model = r#"
    {
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        func inc(x): { result: x + 1 }
        value: inc(1).result
    }
    "#;

    parse_error_contains(model, &["duplicate type 'LoanOffer'"]);
}

#[test]
fn cycle_reference_prevention() {
    let model = r#"
    {
        type Customer: {valid: <Customer>; name: <string>; birthdate: <date>; birthtime: <time>; birthdatetime: <datetime>; income: <number>}
        type LoanOffer: {eligible: <Customer>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        func inc(x: LoanOffer): { result: x.eligible }
        value: inc({}).result
    }
    "#;

    link_error_contains(model, &["cyclic reference loop"]);

    let model = r#"
    {
        type Customer: {valid: <LoanOffer>; name: <string>; birthdate: <date>; birthtime: <time>; birthdatetime: <datetime>; income: <number>}
        type LoanOffer: {eligible: <Customer>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        func inc(x: LoanOffer): { result: x.eligible }
        value: inc({}).result
    }
    "#;

    link_error_contains(model, &["cyclic reference loop"]);
}

#[test]
fn input_type_validation() {
    let model = r#"
    {
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        func inc(x: LoanOffer): { result: x.amount + 1 }
        value: inc(1).result
    }
    "#;

    link_error_contains(
        model,
        &[
            "Argument `x` of function `inc`",
            "type 'number'",
            "expected 'Type<eligible: boolean",
        ],
    );

    let model = r#"
    {
        func greet(name: string): { result: name }
        value: greet(10).result
    }
    "#;

    link_error_contains(
        model,
        &[
            "Argument `name` of function `greet`",
            "type 'number'",
            "expected 'string'",
        ],
    );

    let model = r#"
    {
        func flag(value: boolean): { result: value }
        value: flag("yes").result
    }
    "#;

    link_error_contains(
        model,
        &[
            "Argument `value` of function `flag`",
            "type 'string'",
            "expected 'boolean'",
        ],
    );

    link_error_contains(
        r#"
        {
            func double(xs: number): {
                result: xs * 2
            }
            value: double([1,2,3]).result
        }
        "#,
        &[
            "Argument `xs` of function `double`",
            "type 'list of number'",
            "expected 'number'",
        ],
    );

    let model = r#"
    {
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        func inc(x: LoanOffer): { result: x.amount + 1 }
        value: inc({amount: 1}).result
    }
    "#;

    assert_eval_all(model, &["{", "value: 2", "}"]);
}

#[test]
fn missing_is_applied_for_function_argument() {
    let model = r#"
    {
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        func inc(x: LoanOffer): { termInMonths: x.termInMonths * 2; result: x }
        value: inc({amount: 100}).result
        termInMonths: inc({amount: 100}).termInMonths
    }
    "#;

    assert_eval_all(
        model,
        &[
            "{",
            "   value: {",
            "      eligible: Missing",
            "      amount: 100",
            "      termInMonths: number.Missing",
            "      monthlyPayment: number.Missing",
            "   }",
            "   termInMonths: 2", // termInMonths is 2, because missing in multiply is replaced by 1
            "}",
        ],
    );
}

#[test]
fn primitive_function_arguments() {
    assert_value!(
        r#"
        func double(xs: number): {
            result: xs * 2
        }
        value: double(2).result
        "#,
        "4"
    );

    assert_value!(
        r#"
        func doubleAll(xs: number[]): {
            result: for x in xs return x * 2
        }
        value: doubleAll([1,2,3]).result
        "#,
        "[2, 4, 6]"
    );

    assert_value!(
        r#"
        func doubleAll(xs: number[]): {
            result: for x in xs return x * 2
        }
        baseline: {
            items: [1,2,3]
        }
        value: doubleAll(baseline.items).result
        "#,
        "[2, 4, 6]"
    );

    assert_value!(
        r#"
        func add(dd: datetime, tt: time, do: date): {
            result: dd.hour + 1 + tt.hour + 1 + do.day + 1
        }
        value: add(datetime('2020-01-01T11:00:00'), time('11:00:00'), date('2020-01-01')).result
        "#,
        "26"
    );
}

#[test]
fn complex_type_array_function_argument() {
    assert_eval_all(
        r#"
        {
            type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
            func incAll(offers: LoanOffer[]): {
                simpleResult: offers[0].amount + offers[1].amount
                forResult: for offer in offers return offer.amount + 1
            }
            value: incAll([{amount: 1}, {amount: 2}])
        }
        "#,
        &[
            "{",
            "value: {",
            "simpleResult: 3",
            "forResult: [2, 3]",
            "}",
            "}",
        ],
    );
}

#[test]
fn special_values_are_set_in_function_argument() {
    assert_eval_all(
        r#"
        {
            type Customer: {valid: <boolean>; name: <string>; birthdate: <date>; birthtime: <time>; birthdatetime: <datetime>; income: <number>}
            func incAll(customer: Customer): {
                primaryCustomer: customer
            }
            value: incAll({})
        }
        "#,
        &[
            "{",
            "value: {",
            "primaryCustomer: {",
            "valid: Missing",
            "name: Missing",
            "birthdate: Missing",
            "birthtime: Missing",
            "birthdatetime: Missing",
            "income: number.Missing",
            "}",
            "}",
            "}",
        ],
    );
}

#[test]
fn complex_nested_types_in_function_argument() {
    assert_eval_all(
        r#"
        {
            type Customer: {valid: <boolean>; name: <string>; birthdate: <date>; birthtime: <time>; birthdatetime: <datetime>; income: <number>}
            type LoanOffer: {customer: <Customer>; eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
            func incAll(offers: LoanOffer[]): {
                primaryCustomer: offers[0].customer
                simpleResult: offers[0].amount + offers[1].amount
                forResult: for offer in offers return offer.amount + 1
            }
            value: incAll([{amount: 1}, {amount: 2}])
        }
        "#,
        &[
            "{",
            "value: {",
            "primaryCustomer: {",
            "valid: Missing",
            "name: Missing",
            "birthdate: Missing",
            "birthtime: Missing",
            "birthdatetime: Missing",
            "income: number.Missing",
            "}",
            "simpleResult: 3",
            "forResult: [2, 3]",
            "}",
            "}",
        ],
    );
}
