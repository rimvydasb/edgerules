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

    let runtime = get_runtime(code);

    assert_path!(runtime, "applicant.income", "1100");
    assert_path!(runtime, "applicant.expense", "600");
    assert_path!(runtime, "applicant.age", "22");

    assert_path!(runtime, "rules.row1.rule", "false");
    assert_path!(runtime, "rules.row2.rule", "true");
    assert_path!(runtime, "rules.row3.rule", "true");
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

    let runtime = get_runtime(code);
    assert_path!(runtime, "applicant.income", "1100");
    assert_path!(runtime, "applicant.expense", "600");
    assert_path!(runtime, "applicant.age", "22");

    // Ensures array indexing resolves correctly (regression guard for RuntimeFieldNotFound).
    assert_path!(runtime, "rules[0]", "{rule:false}");
    assert_path!(runtime, "rules[0].rule", "false");
    assert_path!(runtime, "rules[1].rule", "true");
    assert_path!(runtime, "rules[2].rule", "true");

    assert_path!(runtime, "applicantEligibility[0].rule", "true");
    assert_path!(runtime, "applicantEligibility[1].rule", "true");

    let code = r#"
    func eligibilityDecision(applicant): {
        rules: [
            {rule: applicant.income > applicant.expense * 2}
            {rule: applicant.income > 1000}
            {rule: applicant.age >= 18}
        ]
    }
    applicantEligibility: eligibilityDecision({
        income: 5000
        expense: 550
        age: 22
    }).rules
    "#;

    let runtime = get_runtime(code);

    assert_path!(runtime, "applicantEligibility", "[{rule:true},{rule:true},{rule:true}]");
}

mod utilities;

pub use utilities::*;
