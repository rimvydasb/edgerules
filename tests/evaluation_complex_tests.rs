#[test]
fn example_context_deep_evaluation() {
    let code = r#"
    applicant: {
        income: 1100
        expense: 600
        age: 22
    }
    rules: {
        row1: {rule: applicant.income > applicant.expense * 2}
        row2: {rule: applicant.income > 1000}
        row3: {rule: applicant.age >= 18}
    }
    "#;

    let rt = get_runtime(code);

    assert_eq!(exe_field(&rt, "applicant.income"), "1100");
    assert_eq!(exe_field(&rt, "applicant.expense"), "600");
    assert_eq!(exe_field(&rt, "applicant.age"), "22");

    assert_eq!(exe_field(&rt, "rules.row1.rule"), "false");
    assert_eq!(exe_field(&rt, "rules.row2.rule"), "true");
    assert_eq!(exe_field(&rt, "rules.row3.rule"), "true");
}

#[test]
fn example_ruleset_deep_evaluation() {
    let code = r#"
    applicant: {
        income: 1100
        expense: 600
        age: 22
    }
    rules: [
        {rule: applicant.income > applicant.expense * 2}
        {rule: applicant.income > 1000}
        {rule: applicant.age >= 18}
    ]
    applicantEligibility: rules[rule = true]
    "#;

    let rt = get_runtime(code);
    assert_eq!(exe_field(&rt, "applicant.income"), "1100");
    assert_eq!(exe_field(&rt, "applicant.expense"), "600");
    assert_eq!(exe_field(&rt, "applicant.age"), "22");

    // Ensures array indexing resolves correctly (regression guard for RuntimeFieldNotFound).
    assert_eq!(exe_field(&rt, "rules[0]"), "{rule:false}");
    assert_eq!(exe_field(&rt, "rules[0].rule"), "false");
    assert_eq!(exe_field(&rt, "rules[1].rule"), "true");
    assert_eq!(exe_field(&rt, "rules[2].rule"), "true");

    assert_eq!(exe_field(&rt, "applicantEligibility[0].rule"), "true");
    assert_eq!(exe_field(&rt, "applicantEligibility[1].rule"), "true");

    let code = r#"
    func eligibilityDecision(applicant): {
        rules: [
            {rule: applicant.income > applicant.expense * 2}
            {rule: applicant.income > 1000}
            {rule: applicant.age >= 18}
        ]
    }
    applicantEligibility: eligibilityDecision({
        income: 1100
        expense: 600
        age: 22
    }).rules
    "#;

    let rt = get_runtime(code);

    assert_eq!(exe_field(&rt, "applicantEligibility"), "[{rule:false},{rule:true},{rule:true}]");
}

#[test]
fn example_ruleset_collecting() {

    let code = r#"
    func eligibilityDecision(applicant): {
        rules: [
            {name: "INC_CHECK"; rule: applicant.income > applicant.expense * 2}
            {name: "MIN_INCOM"; rule: applicant.income > 1000}
            {name: "AGE_CHECK"; rule: applicant.age >= 18}
        ][rule = false]
        result: {
            firedRules: for invalid in rules return invalid.name
            status: if count(rules) = 0 then "ELIGIBLE" else "INELIGIBLE"
        }
    }
    applicantEligibility: eligibilityDecision({
        income: 1100
        expense: 600
        age: 22
    }).result
    "#;

    let rt = get_runtime(code);

    assert_eq!(exe_field(&rt, "applicantEligibility.firedRules"), "['INC_CHECK']");
    assert_eq!(exe_field(&rt, "applicantEligibility.status"), "'INELIGIBLE'");
}

mod utilities;

pub use utilities::*;
