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

    // deep evaluation of contexts triggers all inner rules to be evaluated
    assert_eval_all(
        code,
        &[
            "{",
            "applicant:{",
            "income:1100",
            "expense:600",
            "age:22",
            "}",
            "rules:{",
            "row1:{",
            "rule:false",
            "}",
            "row2:{",
            "rule:true",
            "}",
            "row3:{",
            "rule:true",
            "}",
            "}",
            "}",
        ],
    );
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

    assert_eval_all(
        code,
        &[
            "{",
            "applicant:{",
            "income:1100",
            "expense:600",
            "age:22",
            "}",
            "rules:[{",
            "rule:false",
            "},{",
            "rule:true",
            "},{",
            "rule:true",
            "}]",
            "applicantEligibility:[{",
            "rule:true",
            "},{",
            "rule:true",
            "}]",
            "}",
        ],
    );

    assert_eq!(
        inline(eval_field(code, "applicantEligibility")),
        inline("[{rule: true}, {rule: true}]")
    );

    assert_eq!(
        inline(eval_field(code, "rules")),
        inline("[{rule:false},{rule:true},{rule:true}]")
    );

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

    assert_eval_all(
        code,
        &[
            "{",
            "applicantEligibility:[{",
            "rule:true",
            "},{",
            "rule:true",
            "},{",
            "rule:true",
            "}]",
            "}",
        ],
    );
}

mod utilities;
pub use utilities::*;
