#[path = "utilities.rs"]
mod utilities;
use utilities::*;

#[test]
fn test_default_values_primitive_defaults() {
    assert_eval_value(
        "type Customer: { name: <string>; income: <number, 0>; isActive: <boolean, true>; category: <string, 'STD'> } \n c: {} as Customer \n value: c",
        "{name: Missing('name') income: 0 isActive: true category: 'STD'}"
    );
}

#[test]
fn test_default_values_partial_override() {
    assert_eval_value(
        "type Customer: { name: <string>; income: <number, 0>; isActive: <boolean, true> } \n c: { name: 'John' } as Customer \n value: c",
        "{name: 'John' income: 0 isActive: true}"
    );
}

#[test]
fn test_default_values_full_override() {
    assert_eval_value(
        "type Customer: { name: <string>; income: <number, 0>; isActive: <boolean, true> } \n c: { name: 'Jane'; income: 5000; isActive: false } as Customer \n value: c",
        "{name: 'Jane' income: 5000 isActive: false}"
    );
}

#[test]
fn test_nested_default_values() {
    assert_eval_value(
        "type Customer: { income: <number, 100> } \n type Loan: { customer: <Customer>; amount: <number, 1000> } \n l: {} as Loan \n value: l",
        "{customer: {income: 100} amount: 1000}"
    );
}

#[test]
fn test_list_default_values() {
    assert_eval_value(
        "type Group: { tags: <string[], []>; scores: <number[], [1, 2, 3]> } \n g: {} as Group \n value: g",
        "{tags: [] scores: [1, 2, 3]}",
    );
}

#[test]
fn test_type_placeholder_evaluation() {
    assert_eval_value("field: <number, 50> \n value: field", "50");
}

#[test]
fn test_default_value_type_mismatch() {
    // Number expected, string given
    parse_error_contains(
        "type Bad: { income: <number, 'foo'> }",
        &["Default value type mismatch", "expected number", "got string"],
    );

    // Boolean expected, number given
    parse_error_contains(
        "type Bad: { active: <boolean, 100> }",
        &["Default value type mismatch", "expected boolean", "got number"],
    );

    // List expected, primitive given
    parse_error_contains(
        "type Bad: { tags: <string[], 'foo'> }",
        &["Default value type mismatch", "expected list", "got string"],
    );
}
