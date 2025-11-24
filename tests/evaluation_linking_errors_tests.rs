#[test]
fn reports_location_for_function_body_errors() {
    let code = r#"
{
    calculations: {
        func takeDate(d: date): { year: d.nonexistent }
        result: takeDate(date('2024-01-01')).year
    }
    value : calculations.result
}
"#;

    link_error_location(code, &["calculations", "takeDate", "year"], "d.nonexistent");
}

#[test]
fn reports_location_for_root_field_errors() {
    let code = r#"
{
    value: 1 + 'a'
}
"#;

    link_error_location(code, &["value"], "(1 + 'a')");
}

#[test]
fn reports_location_for_nested_object_fields() {
    let code = r#"
{
    nested: { deeper: 1 + 'a' }
}
"#;

    link_error_location(code, &["nested", "deeper"], "(1 + 'a')");
}

#[test]
fn reports_location_for_simple_field_access_error() {
    let code = r#"
{
    value: date('2024-01-01') + 'a'
}
"#;

    link_error_location(code, &["value"], "(date('2024-01-01') + 'a')");
}

#[test]
fn reports_location_for_deep_context_access() {
    let code = r#"
{
    lvl1: { lvl2: { lvl3: 1 + 'a' } }
    value: lvl1.lvl2.lvl3
}
"#;

    link_error_location(code, &["value"], "lvl1.lvl2.lvl3");
}

#[test]
fn reports_location_for_errors_inside_array_elements() {
    let code = r#"
{
    value: [{ bad: 1 + 'a' }][0].bad
}
"#;

    link_error_location(code, &["bad"], "(1 + 'a')");
}

#[test]
fn reports_location_for_if_else_body_errors() {
    let code = r#"
{
    value: if true then 1 + 'a' else 0
}
"#;

    link_error_location(code, &["value"], "if true then 1 + 'a' else 0");
}

#[test]
fn reports_location_for_loop_body_errors() {
    let code = r#"
{
    value: for x in [1, 2] return 1 + 'a'
}
"#;

    // The loop body links its return expression under "_return".
    link_error_location(code, &["_return"], "(1 + 'a')");
}

mod utilities;
pub use utilities::*;
