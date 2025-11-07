mod utilities;
use edge_rules::runtime::{edge_rules::EdgeRulesModel, ToSchema};
pub use utilities::*;

fn assert_type_string(lines: &[&str], expected: &str) {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(&code);
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_schema();
    assert_eq!(ty, expected);
}

fn assert_type_fields_unordered(lines: &[&str], expected_fields: &[&str]) {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRulesModel::new();
    let _ = service.append_source(&code);
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_schema();
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
    assert_value!("toString(1)", "'1'");
    assert_value!("toString(true)", "'true'");
    assert_value!("toString('hi')", "'hi'");

    // lists and nested lists
    assert_value!("toString([1,2,3])", "'[1, 2, 3]'");
    assert_value!("toString([[1,2], [3]])", "'[[1, 2], [3]]'");
    // empty list literal via sublist to avoid parse quirks for []
    assert_value!("toString(sublist([1], 1, 0))", "'[]'");
}

#[test]
fn date_time_and_duration_roundtrip_to_string() {
    // date/time/datetime/duration constructors and their stringification
    assert_value!("toString(date('2024-01-01'))", "'2024-01-01'");
    assert_value!("toString(time('12:00:00'))", "'12:00:00'");
    assert_value!(
        "toString(datetime('2024-06-05T07:30:00'))",
        "'2024-06-05T07:30:00'"
    );
    assert_value!("toString(duration('P3DT4H5M6S'))", "'P3DT4H5M6S'");
    assert_value!("toString(duration('PT90M'))", "'PT1H30M'");
    assert_value!("toString(period('P1Y2M'))", "'P1Y2M'");
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

    assert_eq!(exe_field(&rt, "applicationResponse"), "1001");
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

    assert_eq!(
        exe_field(&rt, "applicationResponse"),
        "applicationResponse:{oldAmount:1000newAmount:1001evenDeeper:{test:1006willItExplode:{yesItWill:2002willBeMissing:Missing('maxAmount')}}}"
    );
}
