#[test]
fn example_ruleset() {

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

    assert_eval_all(code, &[
        "{", "applicant:{", "income:1100", "expense:600", "age:22", "}",
        "rules:[{", "rule:false", "},{", "rule:true", "},{", "rule:true", "}]", "applicantEligibility:[{", "rule:true", "},{", "rule:true", "}]", "}"
    ]);

    assert_eq!(inline(eval_field( code, "applicantEligibility")),inline("[{rule: true}, {rule: true}]"));

    // @Todo: This fails because contexts in rules are not triggered, needs to be fixed
    assert_eq!(inline(eval_field( code, "rules")),inline("[{rule: true}, {rule: true}]"));

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

    assert_eval_all(code, &["{", "output1: 14", "}"]);

}

mod utilities;
pub use utilities::*;
