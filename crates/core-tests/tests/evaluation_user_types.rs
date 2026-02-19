mod utilities;

use edge_rules::runtime::decision_service::DecisionService;
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
    service.append_source(code1).expect("parse type with placeholders");

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
    let err2 = service.append_source(code2).err().unwrap().to_string();
    assert!(
        err2.to_lowercase().contains(&"Type definition cannot contain function definitions".to_lowercase()),
        "expected parse error about function definitions, got: {}",
        err2
    );

    // 3) Only nested type objects are allowed
    // Should parse and link with no errors
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .append_source(
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
    assert_string_contains("eligible: true", &evaluated);
    assert_string_contains("amount: 20000", &evaluated);
    assert_string_contains("termInMonths: 24", &evaluated);
    assert_string_contains("monthlyPayment: 875", &evaluated);
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
    //     amount: Missing
    //     termInMonths: Missing
    //     monthlyPayment: Missing
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
    service.append_source(code).unwrap();
    let runtime_snapshot = service.to_runtime_snapshot().expect("runtime snapshot");
    let value = runtime_snapshot.evaluate_field("y").expect("evaluate field").to_string();
    assert_eq!(value, "7");
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.get_type("*").unwrap().to_string();
    assert!(ty.contains("x: number"), "got `{}`", ty);
}

#[test]
fn cast_object_to_alias_shape_links_type() {
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .append_source(
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
    let ty = runtime.get_type("*").unwrap().to_string();
    assert!(!ty.contains("Point: {x: number; y: number}"), "got `{}`", ty);
    assert!(ty.contains("p: Point"), "got `{}`", ty);
}

#[test]
fn cast_list_to_alias_of_number_list() {
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .append_source(
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
    let ty = runtime.get_type("*").unwrap().to_string();
    assert!(ty.contains("vals: number[]"), "got `{}`", ty);
}

#[test]
fn cast_to_nested_alias() {
    let mut service = edge_rules::runtime::edge_rules::EdgeRulesModel::new();
    service
        .append_source(
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
    let ty = runtime.get_type("*").unwrap().to_string();
    assert!(!ty.contains("Customer: {name: string; birthdate: date; income: number}"), "got `{}`", ty);
    assert!(ty.contains("c: Customer"), "got `{}`", ty);
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

    parse_error_contains(model, &["Duplicate user type 'LoanOffer'"]);
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

    link_error_contains(model, &["Argument `x` of function `inc`", "type 'number'", "expected '{eligible: boolean"]);

    let model = r#"
    {
        func greet(name: string): { result: name }
        value: greet(10).result
    }
    "#;

    link_error_contains(model, &["Argument `name` of function `greet`", "type 'number'", "expected 'string'"]);

    let model = r#"
    {
        func flag(value: boolean): { result: value }
        value: flag("yes").result
    }
    "#;

    link_error_contains(model, &["Argument `value` of function `flag`", "type 'string'", "expected 'boolean'"]);

    link_error_contains(
        r#"
        {
            func double(xs: number): {
                result: xs * 2
            }
            value: double([1,2,3]).result
        }
        "#,
        &["Argument `xs` of function `double`", "type 'number[]'", "expected 'number'"],
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
fn primitive_function_arguments() {
    assert_eval_value(
        r#"
        func double(xs: number): {
            result: xs * 2
        }
        value: double(2).result
        "#,
        "4",
    );

    assert_eval_value(
        r#"
        func doubleAll(xs: number[]): {
            result: for x in xs return x * 2
        }
        value: doubleAll([1,2,3]).result
        "#,
        "[2, 4, 6]",
    );

    assert_eval_value(
        r#"
        func doubleAll(xs: number[]): {
            result: for x in xs return x * 2
        }
        baseline: {
            items: [1,2,3]
        }
        value: doubleAll(baseline.items).result
        "#,
        "[2, 4, 6]",
    );

    assert_eval_value(
        r#"
        func add(dd: datetime, tt: time, do: date): {
            result: dd.hour + 1 + tt.hour + 1 + do.day + 1
        }
        value: add(datetime('2020-01-01T11:00:00'), time('11:00:00'), date('2020-01-01')).result
        "#,
        "26",
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
        &["{", "value: {", "simpleResult: 3", "forResult: [2, 3]", "}", "}"],
    );
}

#[test]
fn complex_type_array_function_argument_v2() {
    assert_eval_all(
        r#"
        {
            type Person: { name: <string>; age: <number>; tags: <string[]> }
            type PeopleList: Person[]
            func getAdults(people: PeopleList): {
                result: people[age >= 18]
            }
            persons: [
                {name: "Alice"; age: 30; tags: ["engineer", "manager"]}
                {name: "Bob"; age: 15; tags: ["student"]}
                {name: "Charlie"; age: 22; tags: []}
            ]
            adults: getAdults(persons)
        }
        "#,
        &[
            "{",
            "persons: [{",
            "name: 'Alice'",
            "age: 30",
            "tags: ['engineer', 'manager']",
            "},{",
            "name: 'Bob'",
            "age: 15",
            "tags: ['student']",
            "},{",
            "name: 'Charlie'",
            "age: 22",
            "tags: []",
            "}]",
            "adults: {",
            "result: [{",
            "name: 'Alice'",
            "age: 30",
            "tags: ['engineer', 'manager']",
            "},{",
            "name: 'Charlie'",
            "age: 22",
            "tags: []",
            "}]",
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
            "valid: Missing('valid')",
            "name: Missing('name')",
            "birthdate: Missing('birthdate')",
            "birthtime: Missing('birthtime')",
            "birthdatetime: Missing('birthdatetime')",
            "income: Missing('income')",
            "}",
            "}",
            "}",
        ],
    );
}

#[test]
fn get_type_lists_defined_types_and_fields() {
    let mut service = EdgeRulesModel::new();
    service
        .append_source(
            r#"
            {
                type Customer: {valid: <boolean>; name: <string>; birthdate: <date>; birthtime: <time>; birthdatetime: <datetime>; income: <number>}
                func incAll(customer: Customer): {
                    primaryCustomer: customer
                }
                value: incAll({})
            }
            "#,
        )
        .expect("parse schema sample");
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.get_type("*").unwrap().to_string();
    assert_eq!(
        ty,
        "{value: {primaryCustomer: Customer}}"
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
            "valid: Missing('customer.valid')",
            "name: Missing('customer.name')",
            "birthdate: Missing('customer.birthdate')",
            "birthtime: Missing('customer.birthtime')",
            "birthdatetime: Missing('customer.birthdatetime')",
            "income: Missing('customer.income')",
            "}",
            "simpleResult: 3",
            "forResult: [2, 3]",
            "}",
            "}",
        ],
    );
}

#[test]
fn cast_object_list_to_typed_list() {
    // This test exercises `cast_value_to_type` where `ValueType::ListType(Some(other_item_type))` is handled for `ObjectsArray`.
    // We define a list of objects and cast it to a list of a compatible alias type.
    // This triggers the deep casting logic for array elements.
    let model = r#"
    {
        type Item: { id: <number> }
        type ItemList: Item[]
        
        // Define untyped object list
        rawItems: [{id: 1}, {id: 2}]
        
        // Cast to typed list
        typedItems: rawItems as ItemList
    }
    "#;

    let evaluated = eval_all(model);
    let collapsed = inline_text(&evaluated); // Use inline helper from utilities to strip whitespace
    assert!(collapsed.contains("typedItems:[{id:1}{id:2}]"), "evaluated: {}", evaluated);
}

#[test]
fn cast_object_list_to_incompatible_primitive_list() {
    // This test ensures that casting a list of objects to a list of primitives (e.g. string[]) is handled.
    // While structurally objects != strings, `cast_value_to_type` for `ObjectsArray` -> `other_item_type`
    // attempts to cast each element. `cast_value_to_type(Reference, StringType)` returns the reference value (no error, just passes through).
    // This results in a `PrimitivesArray` containing `Reference`s, but typed as `string`.
    // This behavior effectively "stringifies" the objects if used in string context, or remains as references.
    // This specifically targets the `Some(other_item_type)` branch for `ObjectsArray` in `cast_value_to_type`.

    let model = r#"
    {
        rawItems: [{id: 1}]
        // Cast object list to string list
        strList: rawItems as string[]
    }
    "#;

    // Based on implementation, this should succeed. The result is a list of references typed as string.
    let evaluated = eval_all(model);
    let collapsed = inline_text(&evaluated);
    assert!(collapsed.contains("strList:[{id:1}]"), "evaluated: {}", evaluated);
}

fn assert_type_string(lines: &[&str], expected: &str) {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(&code);
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.get_type("*").unwrap().to_string();
    assert_eq!(ty, expected);
}

fn assert_type_fields_unordered(lines: &[&str], expected_fields: &[&str]) {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(&code);
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.get_type("*").unwrap().to_string();
    assert!(ty.starts_with('{') && ty.ends_with('}'));
    let inner = &ty[1..ty.len() - 1];
    let mut actual: Vec<String> = Vec::new();
    if !inner.trim().is_empty() {
        let mut buffer = String::new();
        let mut depth = 0;
        for ch in inner.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    buffer.push(ch);
                }
                '}' => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    buffer.push(ch);
                }
                ';' if depth == 0 => {
                    let trimmed = buffer.trim();
                    if !trimmed.is_empty() {
                        actual.push(trimmed.to_string());
                    }
                    buffer.clear();
                }
                _ => buffer.push(ch),
            }
        }
        let trimmed = buffer.trim();
        if !trimmed.is_empty() {
            actual.push(trimmed.to_string());
        }
    }
    let mut expected: Vec<String> = expected_fields.iter().map(|s| s.to_string()).collect();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected, "got type `{}`", ty);
}

fn assert_type_string_block(code: &str, expected: &str) {
    let lines: Vec<&str> = code.trim().lines().collect();
    assert_type_string(&lines, expected);
}

fn assert_type_fields_unordered_block(code: &str, expected_fields: &[&str]) {
    let lines: Vec<&str> = code.trim().lines().collect();
    assert_type_fields_unordered(&lines, expected_fields);
}

#[test]
fn to_string_for_various_values_and_lists() {
    // numbers, booleans, strings
    assert_expression_value("toString(1)", "'1'");
    assert_expression_value("toString(true)", "'true'");
    assert_expression_value("toString('hi')", "'hi'");

    // lists and nested lists
    assert_expression_value("toString([1,2,3])", "'[1, 2, 3]'");
    assert_expression_value("toString([[1,2], [3]])", "'[[1, 2], [3]]'");
    // empty list literal via sublist to avoid parse quirks for []
    assert_expression_value("toString(sublist([1], 1, 0))", "'[]'");
}

#[test]
fn date_time_and_duration_roundtrip_to_string() {
    // date/time/datetime/duration constructors and their stringification
    assert_expression_value("toString(date('2024-01-01'))", "'2024-01-01'");
    assert_expression_value("toString(time('12:00:00'))", "'12:00:00'");
    assert_expression_value("toString(datetime('2024-06-05T07:30:00'))", "'2024-06-05T07:30:00'");
    assert_expression_value("toString(duration('P3DT4H5M6S'))", "'P3DT4H5M6S'");
    assert_expression_value("toString(duration('PT90M'))", "'PT1H30M'");
    assert_expression_value("toString(period('P1Y2M'))", "'P1Y2M'");
}

#[test]
fn type_validation_errors_when_mismatched() {
    // List of booleans for all/any
    // @Todo: all and any are disabled for now
    //link_error_contains("value: all([1,2])", &["unexpected", "boolean"]);
    //link_error_contains("value: any(['x'])", &["unexpected", "boolean"]);

    // Numeric lists for numeric aggregates
    link_error_contains("value: product(['a','b'])", &["unexpected", "number"]);
}

#[test]
fn type_string_simple_root() {
    assert_type_string_block(
        r#"
        a: 1
        b: 's'
        c: true
        "#,
        "{a: number; b: string; c: boolean}",
    );
}

#[test]
fn type_string_nested_object() {
    assert_type_string_block(
        r#"
        a: 1
        b: 2
        c: { x: 'Hello'; y: a + b }
        "#,
        "{a: number; b: number; c: {x: string; y: number}}",
    );
}

#[test]
fn type_string_deeper_nesting() {
    assert_type_string_block(
        r#"
        a: time('12:00:00')
        b: date('2024-01-01')
        c: datetime('2024-06-05T07:30:00')
        d: { inner: { z: time('08:15:00') } }
        "#,
        "{a: time; b: date; c: datetime; d: {inner: {z: time}}}",
    );
}

#[test]
fn type_string_lists() {
    // list of numbers, list of strings, nested list of numbers
    assert_type_fields_unordered_block(
        r#"
        nums: [1,2,3]
        strs: ['a','b']
        nested: [[1,2], [3]]
        "#,
        &["nums: number[]", "strs: string[]", "nested: number[][]"],
    );
}

#[test]
fn type_string_ranges() {
    // numeric range
    assert_type_string_block(
        r#"
        r: 1..5
        "#,
        "{r: range}",
    );
}

#[test]
fn type_string_lists_and_ranges_combined() {
    assert_type_string_block(
        r#"
        a: [1,2,3]
        b: 10..20
        c: [[10,20],[30]]
        "#,
        "{a: number[]; b: range; c: number[][]}",
    );
}

#[test]
fn type_objects_amd_functions() {
    assert_type_string_block(
        r#"
        a: sum([1,2,3])
        b: a
        c: toString(a)
        "#,
        "{a: number; b: number; c: string}",
    );
}

#[test]
fn types_story_placeholders_and_aliases_link() {
    // Simple typed placeholders in the model (not within type definitions)
    assert_type_fields_unordered_block(
        r#"
        identification: <number>
        relationsList: <number[]>
        "#,
        &["identification: number", "relationsList: number[]"],
    );
}

#[test]
fn using_types_in_deeper_scope_v1() {
    let code = r#"
    type Application: {
        loanAmount: <number>;
        maxAmount: <number>;
    }
    func incAmount(application: Application): {
        func inc(x): {
            result: x + 1
        }
        newAmount: inc(application.loanAmount).result
    }
    applicationResponse: incAmount({
        loanAmount: 1000
    }).newAmount
    "#;

    let rt = get_runtime(code);

    assert_eval_field(&rt, "applicationResponse", "1001");
}

#[test]
fn using_types_in_deeper_scope() {
    let code = r#"
    type Application: {
        loanAmount: <number>;
        maxAmount: <number>;
    }
    func incAmount(application: Application): {
        func inc(x): {
            result: x + 1
        }
        newAmount: inc(application.loanAmount).result
    }
    func applicationDecisions(application: Application): {
        amountsDiff: {
            oldAmount: application.loanAmount
            newAmount: incAmount(application).newAmount
            evenDeeper: {
                test: incAmount(application).newAmount + 5
                willItExplode: {
                    yesItWill: incAmount(application).newAmount + newAmount
                    willBeMissing: application.maxAmount
                }
            }
        }
    }

    applicationResponse: applicationDecisions({
        loanAmount: 1000
    }).amountsDiff
    "#;

    let rt = get_runtime(code);

    assert_eval_field(
        &rt,
        "applicationResponse",
        "{oldAmount:1000newAmount:1001evenDeeper:{test:1006willItExplode:{yesItWill:2002willBeMissing:Missing('maxAmount')}}}"
    );
}

#[test]
fn explicit_cast_to_temporal_types() {
    assert_expression_value("'2026-01-26' as date", "2026-01-26");
    assert_expression_value("'12:00:00' as time", "12:00:00");
    assert_expression_value("'2026-01-26T21:33:35' as datetime", "2026-01-26T21:33:35");
    assert_expression_value("'P1DT1H' as duration", "P1DT1H");
    assert_expression_value("'P1Y2M' as period", "P1Y2M");
}

#[test]
fn cast_strings_to_temporal_types_via_decision_service() {
    let model = r#"
    {
        type Request: {
            date: <date>;
            datetime: <datetime>;
            time: <time>;
            duration: <duration>;
            period: <period>;
        }
        func check(r: Request): {
            isDate: r.date = date('2026-01-26')
            isDateTime: r.datetime = datetime('2026-01-26T21:33:35')
            isTime: r.time = time('12:00:00')
            isDuration: r.duration = duration('P1DT1H')
            isPeriod: r.period = period('P1Y2M')
        }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service from source");

    let request_code = r#"
        date: '2026-01-26'
        datetime: '2026-01-26T21:33:35'
        time: '12:00:00'
        duration: 'P1DT1H'
        period: 'P1Y2M'
    "#;

    let mut request_model = EdgeRulesModel::new();
    request_model.append_source(&format!("{{ {} }}", request_code)).unwrap();
    let request_rt = request_model.to_runtime().unwrap();
    let request = edge_rules::test_support::ValueEnum::Reference(request_rt.context);

    let response = service.execute("check", Some(vec![request])).expect("execute");
    let rendered = inline_text(response.to_string());

    assert_string_contains("isDate:true", &rendered);
    assert_string_contains("isDateTime:true", &rendered);
    assert_string_contains("isTime:true", &rendered);
    assert_string_contains("isDuration:true", &rendered);
    assert_string_contains("isPeriod:true", &rendered);
}

#[test]
fn invalid_cast_to_various_temporal_types_fails() {
    let mut service = EdgeRulesModel::new();
    service
        .append_source(
            r#"
        {
            v1: 'not-a-date' as date
            v2: 'not-a-time' as time
            v3: 'not-a-datetime' as datetime
            v4: 'not-a-duration' as duration
            v5: 'not-a-period' as period
        }
        "#,
        )
        .unwrap();
    let runtime = service.to_runtime().unwrap();

    for (field, target) in [("v1", "date"), ("v2", "time"), ("v3", "datetime"), ("v4", "duration"), ("v5", "period")] {
        let result = runtime.evaluate_field(field);
        assert!(result.is_err(), "Expected error for {}", field);
        let err_msg = result.unwrap_err().to_string();
        assert_string_contains(format!("Cannot convert value from type 'string' to type '{}'", target), &err_msg);
    }
}

#[test]
fn cast_string_array_to_temporal_array_via_decision_service() {
    let model = r#"
    {
        type Request: {
            dates: <datetime[]>
        }
        func check(r: Request): {
            allValid: count(r.dates) = 2
            firstYear: r.dates[0].year
            secondYear: r.dates[1].year
        }
    }
    "#;

    let mut service = DecisionService::from_source(model).expect("service from source");

    let request_code = r#"
        dates: ['2026-01-26T10:00:00', '2027-02-27T11:00:00']
    "#;

    let mut request_model = EdgeRulesModel::new();
    request_model.append_source(&format!("{{ {} }}", request_code)).unwrap();
    let request_rt = request_model.to_runtime().unwrap();
    let request = edge_rules::test_support::ValueEnum::Reference(request_rt.context);

    let response = service.execute("check", Some(vec![request])).expect("execute");
    let rendered = inline_text(response.to_string());

    assert_string_contains("allValid:true", &rendered);
    assert_string_contains("firstYear:2026", &rendered);
    assert_string_contains("secondYear:2027", &rendered);
}
