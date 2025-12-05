#[test]
fn reports_location_for_object_body_errors() {
    let code = r#"
{
    object1: {
        fieldA: "a"
        fieldB: "b"
    }
    calculations: {
        calc: object1.nonexistent
    }
    value : calculations.calc
    }
"#;

    link_error_location(
        code,
        &["calculations", "calc"],
        "object1.nonexistent",
        LinkingErrorEnum::FieldNotFound("object1".to_string(), "nonexistent".to_string()),
    );
}

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

    link_error_location(
        code,
        &["calculations", "takeDate", "year"], // perfect resolution!
        "d.nonexistent", // also very good expression capture!
        LinkingErrorEnum::FieldNotFound("d".to_string(), "nonexistent".to_string()),
    );
}

#[test]
fn reports_location_for_root_field_errors() {
    let code = r#"
{
    value: 1 + 'a'
}
"#;

    link_error_location(
        code,
        &["value"],
        "(1 + 'a')",
        LinkingErrorEnum::TypesNotCompatible(
            Some("Left side of operator '+'".to_string()),
            ValueType::NumberType,
            Some(vec![ValueType::StringType]),
        ),
    );
}

#[test]
fn reports_location_for_nested_object_fields() {
    let code = r#"
{
    nested: { deeper: 1 + 'a' }
}
"#;

    link_error_location(
        code,
        &["nested", "deeper"],
        "(1 + 'a')",
        LinkingErrorEnum::TypesNotCompatible(
            Some("Left side of operator '+'".to_string()),
            ValueType::NumberType,
            Some(vec![ValueType::StringType]),
        ),
    );
}

#[test]
fn reports_location_for_simple_field_access_error() {
    let code = r#"
{
    value: date('2024-01-01') + 'a'
}
"#;

    link_error_location(
        code,
        &["value"],
        "(date('2024-01-01') + 'a')",
        LinkingErrorEnum::TypesNotCompatible(
            Some("Left side of operator '+'".to_string()),
            ValueType::DateType,
            Some(vec![ValueType::StringType]),
        ),
    );
}

#[test]
fn reports_location_for_deep_context_access() {
    let code = r#"
{
    lvl1: { lvl2: { lvl3: 1 + 'a' } }
    value: lvl1.lvl2.lvl3
}
"#;

    link_error_location(
        code,
        &["lvl1", "lvl2", "lvl3"],
        "(1 + 'a')",
        LinkingErrorEnum::TypesNotCompatible(
            Some("Left side of operator '+'".to_string()),
            ValueType::NumberType,
            Some(vec![ValueType::StringType]),
        ),
    );
}

#[test]
fn reports_location_for_errors_inside_array_elements() {
    let code = r#"
{
    value: [{ bad: 1 + 'a' }][0].bad
}
"#;

    link_error_location(
        code,
        // @Todo: this is not ideal, it should start with value and then bad
        &["bad"],
        // @Todo: not idea, brackets are not necessary
        "(1 + 'a')",
        LinkingErrorEnum::TypesNotCompatible(
            Some("Left side of operator '+'".to_string()),
            ValueType::NumberType,
            Some(vec![ValueType::StringType]),
        ),
    );
}

#[test]
fn reports_location_for_if_else_body_errors() {
    let code = r#"
{
    value: if true then 1 + 'a' else 0
}
"#;

    link_error_location(
        code,
        &["value"],
        "if true then 1 + 'a' else 0",
        LinkingErrorEnum::TypesNotCompatible(
            Some("Left side of operator '+'".to_string()),
            ValueType::NumberType,
            Some(vec![ValueType::StringType]),
        ),
    );
}

#[test]
fn reports_location_for_loop_body_errors() {
    let code = r#"
{
    value: for x in [1, 2] return 1 + 'a'
}
"#;

    // The loop body links its return expression under "_return".
    link_error_location(
        code,
        // @Todo: not ideal, it should start with value
        &["_return"],
        "(1 + 'a')",
        LinkingErrorEnum::TypesNotCompatible(
            Some("Left side of operator '+'".to_string()),
            ValueType::NumberType,
            Some(vec![ValueType::StringType]),
        ),
    );
}

mod utilities;

use edge_rules::test_support::{LinkingErrorEnum, ValueType};
pub use utilities::*;
