mod utilities;
use edge_rules::runtime::edge_rules::EdgeRulesModel;
pub use utilities::*;

fn assert_type_string(lines: &[&str], expected: &str) {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRulesModel::new();
    let _ = service.load_source(&code);
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert_eq!(ty, expected);
}

fn assert_type_fields_unordered(lines: &[&str], expected_fields: &[&str]) {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRulesModel::new();
    let _ = service.load_source(&code);
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert!(ty.starts_with("Type<") && ty.ends_with('>'));
    let inner = &ty[5..ty.len() - 1];
    let mut actual: Vec<&str> = if inner.is_empty() {
        vec![]
    } else {
        inner.split(", ").collect()
    };
    let mut expected: Vec<&str> = expected_fields.to_vec();
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
    assert_value!("toString(time('12:00:00'))", "'12:00:00.0'");
    assert_value!(
        "toString(datetime('2024-06-05T07:30:00'))",
        "'2024-06-05 7:30:00.0'"
    );
    assert_value!("toString(duration('P1Y2M'))", "'P1Y2M'");
    assert_value!("toString(duration('P3DT4H5M6S'))", "'P3DT4H5M6S'");
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
        "Type<a: number, b: string, c: boolean>",
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
        "Type<a: number, b: number, c: Type<x: string, y: number>>",
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
        "Type<a: time, b: date, c: datetime, d: Type<inner: Type<z: time>>>",
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
        "Type<r: range>",
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
        "Type<a: number[], b: range, c: number[][]>",
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
        "Type<a: number, b: number, c: string>",
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
