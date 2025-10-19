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

    assert_eq!(
        exe_field(&rt, "applicantEligibility"),
        "[{rule:false},{rule:true},{rule:true}]"
    );
}

#[test]
fn example_ruleset_collecting() {
    let code = r#"
    func eligibilityDecision(applicant): {
        rules: [
            {name: "INC_CHECK"; rule: applicant.income > applicant.expense * 2}
            {name: "MIN_INCOM"; rule: applicant.income > 1000}
            {name: "AGE_CHECK"; rule: applicant.age >= 18}
        ]
        result: {
            firedRules: for invalid in rules[rule = false] return invalid.name
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

    assert_eq!(
        exe_field(&rt, "applicantEligibility.firedRules"),
        "['INC_CHECK']"
    );
    assert_eq!(
        exe_field(&rt, "applicantEligibility.status"),
        "'INELIGIBLE'"
    );
}

#[test]
#[ignore]
fn example_variable_library() {
    init_logger();

    let code = r#"
    // Business Object Model Entities:

    type Application: {
        applicationDate: <datetime>;
        applicants: <Applicant[]>;
        propertyValue: <number>;
        loanAmount: <number>;
    }
    type Applicant: {
        name: <string>;
        birthDate: <date>;
        income: <number>;
        expense: <number>;
    }

    // All Decision Areas:

    func applicantDecisions(applicant: Applicant, application): {
        func eligibilityDecision(applicantRecord): {
            rules: [
                {name: "INC_CHECK"; rule: applicantRecord.data.income > applicantRecord.data.expense * 2}
                {name: "MIN_INCOM"; rule: applicantRecord.data.income > 1000}
                {name: "AGE_CHECK"; rule: applicantRecord.data.birthDate + period('P18Y') <= applicantRecord.checkDate}
            ]
            firedRules: for invalid in rules[rule = false] return invalid.name
            status: if count(rules) = 0 then "ELIGIBLE" else "INELIGIBLE"
        }
        applicantRecord: {
            checkDate: application.applicationDate
            data: applicant
            age: (application.applicationDate - applicant.birthDate).years
        }
        eligibility: eligibilityDecision(applicantRecord)
    }

    func applicationDecisions(application: Application): {
        applicationRecord: {
            data: application
            applicantsDecisions: for app in application.applicants return applicantDecisions(app, application).eligibility
        }
    }

    applicationResponse: applicationDecisions({
        applicationDate: date("2025-01-01")
        propertyValue: 100000
        loanAmount: 80000
        applicants: [
            {
                name: "John Doe"
                birthDate: date("1990-06-05")
                income: 1100
                expense: 600
            },
            {
                name: "Jane Doe"
                birthDate: date("1992-05-01")
                income: 1500
                expense: 300
            }
        ]
    })
    "#;

    let rt = get_runtime(code);

    // @Todo: finish writing test:
    assert_eq!(
        exe_field(
            &rt,
            "applicationResponse.applicationRecord.applicantsDecisions[0]"
        ),
        "'INELIGIBLE'"
    );
    assert_eq!(
        exe_field(
            &rt,
            "applicationResponse.applicationRecord.applicantsDecisions[1]"
        ),
        "'INELIGIBLE'"
    );
}

mod utilities;

pub use utilities::*;
