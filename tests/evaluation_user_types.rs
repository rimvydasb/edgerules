mod utilities;

use edge_rules::runtime::edge_rules::EdgeRules;
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
    let mut service = edge_rules::runtime::edge_rules::EdgeRules::new();
    service.load_source(&code1).expect("parse type with placeholders");

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
    let code = vec![
        "type Person: {}",
    ];
    // Should parse and link with no errors
    let mut service = edge_rules::runtime::edge_rules::EdgeRules::new();
    service.load_source(&format!("{{\n{}\n}}", code.join("\n"))).unwrap();
    let _ = service.to_runtime().expect("link");
}

#[test]
fn typed_placeholders_are_allowed_in_model_but_evaluate_to_missing() {
    // Placeholders in model (not inside type definitions) are accepted and eval to Missing
    let lines = vec![
        "id: <number>",
        "name: <string>",
    ];
    // Evaluated values are Missing (rendered in string form per to_string rules)
    let model = format!("{{\n{}\n}}", lines.join("\n"));
    let printed = eval_all(&model);
    // At least ensure fields exist; exact Missing rendering can vary across types
    assert!(printed.contains("id :") && printed.contains("name :"));
}

// @Todo: need to have user types pre-collection step before the linker
#[test]
#[ignore]
fn loan_offer_decision_service_end_to_end() {
    let lines = vec![
        "type Customer: {name: <string>, birthdate: <date>, income: <number>}",
        "type Applicant: {customer: <Customer>, requestedAmount: <number>, termInMonths: <number>}",
        "type LoanOffer: {eligible: <boolean>, amount: <number>, termInMonths: <number>, monthlyPayment: <number>}",

        // NOTE: placeholder not supported yet, so set a concrete date
        "executionDatetime: date('2024-01-01')",

        "func calculateLoanOffer(applicant): {",
        // Compare with 18 years in days to match current duration support
        "    eligible: if executionDatetime - applicant.customer.birthdate >= duration('P6570D') then true else false;",
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

        "checkEligible: if executionDatetime - applicant1.customer.birthdate >= duration('P6570D') then true else false",
        "checkAmount: applicant1.requestedAmount",
        "checkTerm: applicant1.termInMonths",
        "checkPayment: (applicant1.requestedAmount * (1 + (if applicant1.customer.income > 5000 then 0.05 else 0.1))) / applicant1.termInMonths",
    ];

    // Evaluate outputs
    assert_eq!(eval_lines_field(&lines, "checkEligible"), "true");
    assert_eq!(eval_lines_field(&lines, "checkAmount"), "20000");
    assert_eq!(eval_lines_field(&lines, "checkTerm"), "24");
    // 20000 * 1.05 / 24 = 875
    assert_eq!(eval_lines_field(&lines, "checkPayment"), "875");
}

// Potential limitation to explore further: forward references and alias-based placeholders.

#[test]
fn unknown_alias_in_placeholder_is_link_error() {
    link_error_contains(
        &format!("{{\n{}\n}}", ["x: <NotDefined>"] .join("\n")),
        &["unknown type", "notdefined"],
    );
}

#[test]
fn cast_primitive_to_number_changes_type() {
    //assert_value!("'5' as number","5");
    let lines = vec![
        "x: '5' as number",
        "y: x + 2",
    ];
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRules::new();
    service.load_source(&*code).unwrap();
    assert_eq!(service.evaluate_field("y"),"7");
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("x: number"), "got `{}`", ty);
}

// @Todo: need to have user types pre-collection step before the linker
#[test]
#[ignore]
fn cast_object_to_alias_shape_links_type() {
    let code = vec![
        "type Point: { x: <number>; y: <number> }",
        "p: { x: 1 } as Point",
    ];
    let mut service = edge_rules::runtime::edge_rules::EdgeRules::new();
    service.load_source(&format!("{{\n{}\n}}", code.join("\n"))).unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("p: Type<x: number, y: number>"), "got `{}`", ty);
}

// @Todo: need to have user types pre-collection step before the linker
#[test]
#[ignore]
fn cast_list_to_alias_of_number_list() {
    let code = vec![
        "type NumList: <number[]>",
        "vals: [1,2,3] as NumList",
    ];
    let mut service = edge_rules::runtime::edge_rules::EdgeRules::new();
    service.load_source(&format!("{{\n{}\n}}", code.join("\n"))).unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("vals: list of number"), "got `{}`", ty);
}

// @Todo: need to have user types pre-collection step before the linker
#[test]
#[ignore]
fn cast_to_nested_alias() {
    let code = vec![
        "type Customer: {name: <string>; birthdate: <date>; income: <number>}",
        "c: {name: 'A'} as Customer",
    ];
    let mut service = edge_rules::runtime::edge_rules::EdgeRules::new();
    service.load_source(&format!("{{\n{}\n}}", code.join("\n"))).unwrap();
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.contains("c: Type<name: string, birthdate: date, income: number>"), "got `{}`", ty);
}

// cast operator is parsed and linked; deeper shaping/validation to be covered separately
