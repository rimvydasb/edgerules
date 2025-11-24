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

mod utilities;
pub use utilities::*;
