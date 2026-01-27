mod utilities;
pub use utilities::*;

#[test]
fn list_comprehension_missing_origin() {
    assert_value!(
        r#"
        data: [{amount: 1}, {amount: 2}, {}]
        value: for item in data return item.amount
        "#,
        "[1, 2, Missing('amount')]"
    );
}

#[test]
fn sum_propagates_missing_origin() {
    assert_value!(
        r#"
        data: [{amount: 1}, {amount: 2}, {}]
        value: sum(for item in data return item.amount)
        "#,
        "Missing('amount')"
    );
}

#[test]
fn max_empty_list_uses_default_origin() {
    assert_value!("sum([])", "0");
    assert_value!("max([])", "Missing('N/A')");
    assert_value!("min([])", "Missing('N/A')");
}

#[test]
fn cast_nested_object_tracks_field_path() {
    assert_eval_all(
        r#"
        type Address: { city: <string>; zip: <number> }
        type Person: { address: Address }
        value: { address: { city: 'Vilnius' } } as Person
        "#,
        &[
            "{",
            "   value: {",
            "      address: {",
            "         city: 'Vilnius'",
            "         zip: Missing('address.zip')",
            "      }",
            "   }",
            "}",
        ],
    );
}

#[test]
fn referencing_context_variable() {
    assert_value!("for item in [{a:1},{a:2},{a:3}] return item.a", "[1, 2, 3]");
    assert_value!("for item in [{a:1},{a:2},{b:3}] return item.a", "[1, 2, Missing('a')]");
}

#[test]
fn missing_is_applied_for_function_argument() {
    let model = r#"
    {
        type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}
        func inc(x: LoanOffer): { termInMonths: x.termInMonths * 2; result: x }
        value: inc({amount: 100}).result
        termInMonths: inc({amount: 100}).termInMonths
    }
    "#;

    assert_eval_all(
        model,
        &[
            "{",
            "   value: {",
            "      eligible: Missing('eligible')",
            "      amount: 100",
            "      termInMonths: Missing('termInMonths')",
            "      monthlyPayment: Missing('monthlyPayment')",
            "   }",
            "   termInMonths: Missing('termInMonths')",
            "}",
        ],
    );
}
