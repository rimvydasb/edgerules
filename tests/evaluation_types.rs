mod utilities;
pub use utilities::*;
use edge_rules::runtime::edge_rules::EdgeRules;

fn assert_type_string(lines: &[&str], expected: &str) {
    let code = format!("{{\n{}\n}}", lines.join("\n"));
    let mut service = EdgeRules::new();
    let _ = service.load_source(&code);
    let runtime = service.to_runtime().expect("link");
    let ty = runtime.static_tree.borrow().to_type_string();
    assert_eq!(ty, expected);
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
    assert_value!("toString(date('2024-01-01'))", "'2024-01-01'"
    );
    assert_value!("toString(time('12:00:00'))", "'12:00:00.0'");
    assert_value!(
        "toString(datetime('2024-06-05T07:30:00'))",
        "'2024-06-05 7:30:00.0'"
    );
    assert_value!("toString(duration('P1Y2M'))", "'P1Y2M'"
    );
    assert_value!("toString(duration('P3DT4H5M6S'))", "'P3DT4H5M6S'"
    );
}

#[test]
fn type_validation_errors_when_mismatched() {
    // List of booleans for all/any
    link_error_contains("value : all([1,2])", &["unexpected", "boolean"]);
    link_error_contains("value : any(['x'])", &["unexpected", "boolean"]);

    // Numeric lists for numeric aggregates
    link_error_contains("value : product(['a','b'])", &["unexpected", "number"]);
}

#[test]
fn type_string_simple_root() {
    assert_type_string(
        &["a : 1", "b : 's'", "c : true"],
        "Type<a: number, b: string, c: boolean>",
    );
}

#[test]
fn type_string_nested_object() {
    assert_type_string(
        &[
            "a : 1",
            "b : 2",
            "c : { x : 'Hello'; y : a + b }",
        ],
        "Type<a: number, b: number, c: Type<x: string, y: number>>",
    );
}

#[test]
fn type_string_deeper_nesting() {
    assert_type_string(
        &[
            "a : time('12:00:00')",
            "b : date('2024-01-01')",
            "c : datetime('2024-06-05T07:30:00')",
            "d : { inner : { z : time('08:15:00') } }"
        ],
        "Type<a: time, b: date, c: datetime, d: Type<inner: Type<z: time>>>",
    );
}
