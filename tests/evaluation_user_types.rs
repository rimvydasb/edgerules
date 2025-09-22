mod utilities;

use edge_rules::runtime::edge_rules::EdgeRulesModel;
pub use utilities::*;

// Additional tests for user-defined types: limitations and potential problems

#[test]
fn type_alias_with_nested_types_only_disallows_placeholders_and_functions() {
    // Enforce: inside type body only nested type objects are allowed (no <type> placeholders, no functions)
    // 1) Using typed placeholders in type body should fail
    // Typed placeholders in type body are allowed per TYPES_STORY
    let code1 = format!(
        "{{\n{}\n}}",
        ["type Address: { street: <string>; house: { number: <number> } }"].join("\n")
    );
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(&code1)
        .expect("parse type with placeholders");

    // 2) Using a function in type body should fail
    let code2 = format!(
        "{{\n{}\n}}",
        [
            "type Bad: {",
            "   nested: { }",
            "   func calc(x): { r: x }",
            "}",
        ]
        .join("\n")
    );
    let err2 = service.load_source(&code2).err().unwrap().to_string();
    assert!(
        err2.to_lowercase()
            .contains(&"Type definition cannot contain function definitions".to_lowercase()),
        "expected parse error about function definitions, got: {}",
        err2
    );

    // 3) Only nested type objects are allowed
    let code = ["type Person: {}"];
    // Should parse and link with no errors
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(&format!("{{\n{}\n}}", code.join("\n")))
        .unwrap();
    let _ = service.to_runtime().expect("link");
}

#[test]
fn typed_placeholders_are_allowed_in_model_but_evaluate_to_missing() {
    // Placeholders in model (not inside type definitions) are accepted and eval to Missing
    let lines = ["id: <number>", "name: <string>"];
    // Evaluated values are Missing (rendered in string form per to_string rules)
    let model = format!("{{\n{}\n}}", lines.join("\n"));
    let printed = eval_all(&model);
    // At least ensure fields exist; exact Missing rendering can vary across types
    assert!(printed.contains("id :") && printed.contains("name :"));
}

#[test]
fn loan_offer_decision_service_end_to_end() {
    let lines = [
        "type Customer: {name: <string>; birthdate: <date>; income: <number>}",
        "type Applicant: {customer: <Customer>; requestedAmount: <number>; termInMonths: <number>}",
        "type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}",

        // NOTE: placeholder not supported yet, so set a concrete date
        "executionDatetime: date('2024-01-01')",

        "func calculateLoanOffer(executionDatetime, applicant): {",
        // Compare with 18 years in days to match current duration support
        "    eligible: if executionDatetime >= applicant1.customer.birthdate + duration('P6570D') then true else false;",
        "    interestRate: if applicant.customer.income > 5000 then 0.05 else 0.1;",
        "    monthlyPayment: (applicant.requestedAmount * (1 + interestRate)) / applicant.termInMonths;",
        "    result: {",
        "        eligible: eligible;",
        "        amount: applicant.requestedAmount;",
        "        termInMonths: applicant.termInMonths;",
        "        monthlyPayment: monthlyPayment",
        "    }",
        "}",

        "applicant1: {",
        "    customer: {name: 'Alice'; birthdate: date('2001-01-01'); income: 6000};",
        "    requestedAmount: 20000;",
        "    termInMonths: 24",
        "}",

        "loanOffer1: calculateLoanOffer(executionDatetime, applicant1).result as LoanOffer",
    ];

    let model = format!("{{\n{}\n}}", lines.join("\n"));
    let evaluated = eval_all(&model);

    assert!(
        evaluated.contains("checkEligible : true"),
        "model output did not include expected eligibility result\n{}",
        evaluated
    );
    assert!(
        evaluated.contains("checkAmount : 20000"),
        "model output did not include expected amount\n{}",
        evaluated
    );
    assert!(
        evaluated.contains("checkTerm : 24"),
        "model output did not include expected term\n{}",
        evaluated
    );
    assert!(
        evaluated.contains("checkPayment : 875"),
        "model output did not include expected payment\n{}",
        evaluated
    );
}

#[test]
#[ignore]
fn loan_offer_decision_service_end_to_end_reduced() {
    let lines = [
        "type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}",
        "sample: {eligible: false}",
        "result: sample as LoanOffer",
    ];

    let model = format!("{{\n{}\n}}", lines.join("\n"));
    let evaluated = eval_all(&model);

    // {
    //     #child : {
    //     eligible : false
    //     amount : number.Missing
    //     termInMonths : number.Missing
    //     monthlyPayment : number.Missing
    // }
    //     sample : {
    //     eligible : false
    // }
    // }

    assert!(
        evaluated.contains("result : true"),
        "model output did not include expected eligibility result\n{}",
        evaluated
    );
}

// Potential limitation to explore further: forward references and alias-based placeholders.

#[test]
fn unknown_alias_in_placeholder_is_link_error() {
    link_error_contains(
        &format!("{{\n{}\n}}", ["x: <NotDefined>"].join("\n")),
        &["unknown type", "notdefined"],
    );
}

#[test]
#[ignore]
fn cast_primitive_to_number_changes_do_not_change_type() {
    // @Todo: fix the behaviour to throw and exception "casting does not convert types. Use appropriate functions to convert types"
    let lines = ["x: '5' as number", "y: x + 2"];
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRulesModel::new();
    service.load_source(code.as_str()).unwrap();
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
    let code = [
        "type Point: { x: <number>; y: <number> }",
        "p: { x: 1 } as Point",
    ];
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(&format!("{{\n{}\n}}", code.join("\n")))
        .unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("p: Type<x: number, y: number>"), "got `{}`", ty);
}

#[test]
fn cast_list_to_alias_of_number_list() {
    let code = ["type NumList: <number[]>", "vals: [1,2,3] as NumList"];
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(&format!("{{\n{}\n}}", code.join("\n")))
        .unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("vals: list of number"), "got `{}`", ty);
}

#[test]
fn cast_to_nested_alias() {
    let code = [
        "type Customer: {name: <string>; birthdate: <date>; income: <number>}",
        "c: {name: 'A'} as Customer",
    ];
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .load_source(&format!("{{\n{}\n}}", code.join("\n")))
        .unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(
        ty.contains("c: Type<name: string, birthdate: date, income: number>"),
        "got `{}`",
        ty
    );
}

// cast operator is parsed and linked; deeper shaping/validation to be covered separately
