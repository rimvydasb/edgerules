mod utilities;
pub use utilities::*;

#[test]
fn test_indexing() {
    assert_value!("[1,2,3][1]", "2");
    assert_value!("[1,2,3][0]", "1");
    assert_value!("[1,2,3][-1]", "Missing('N/A')");
    assert_value!("[1,2,3][3]", "Missing('N/A')");
}

#[test]
fn test_constraints() {
    assert_value!("[1,2,3][...>1]", "[2, 3]");
    assert_value!("[1,2,3][...>0]", "[1, 2, 3]");
    assert_value!("[1,2,3][...>-5]", "[1, 2, 3]");
    assert_value!("[1,2,3][...<-5]", "[]");

    assert_value!(
        r#"
        nums : [1, 5, 12, 7];
        value: nums[...>6]
        "#,
        "[12, 7]"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                input : {
                    nums : [1, 5, 12, 7]
                    filtered: nums[...>6]
                }
            }
            "#
            .trim(),
            "input.filtered"
        ),
        "[12, 7]"
    );
}

#[test]
fn test_complex_constraints() {
    assert_value!("[{a: 1},{a: 2}][a > 1]", "[{a: 2}]");
    assert_value!("[{a: 1},{a: 2},{c: 2}][a > 1]", "[{a: 2}]");
    // missing fields are ignored in comparisons (treated as NotFound)
    assert_value!("[{a: 1},{a: 2},{c: 2}][a + 1 > 1]", "[{a: 1},{a: 2}]");
    // deeply nested objects are allowed
    assert_value!("[{a: {b: 1}},{a: {b: 2}}][a.b > 1]", "[{a: {b: 2}}]");
}

#[test]
fn test_filter_not_alias() {
    // implicit 'it'
    assert_value!("count([1, 5, 12, 7][not it > 10])", "3");

    // explicit '...'
    assert_value!("count([1, 5, 12, 7][not ... > 10])", "3");

    // combine inside filter
    assert_value!("count([1, 5, 12, 7, 15][(it > 3) and not (it > 10)])", "2");
}

#[test]
fn test_string_filters() {
    assert_value!("['happy','unhappy','harp'][startsWith(...,'ha')]", "['happy', 'harp']");

    // @Todo: it appears that type is a reserved word, need to make it not reserved based on context
    let model = r#"
    {
        creditLines: [
            { lineType: 'Standard'; limit: 1000 },
            { lineType: 'Premium'; limit: 5000 },
            { lineType: 'Standard Plus'; limit: 2000 }
        ];
        standardLines: creditLines[lineType = 'Standard'];
        bigStandardLines: creditLines[limit >= 1000 and startsWith(lineType, 'Standard ')];
        multiFilter: bigStandardLines[limit = 2000];
    }
    "#;

    assert_eval_all(
        model,
        &[
            "{",
            "creditLines: [{",
            "lineType: 'Standard'",
            "limit: 1000",
            "},{",
            "lineType: 'Premium'",
            "limit: 5000",
            "},{",
            "lineType: 'Standard Plus'",
            "limit: 2000",
            "}]",
            "standardLines: [{",
            "lineType: 'Standard'",
            "limit: 1000",
            "}]",
            "bigStandardLines: [{",
            "lineType: 'Standard Plus'",
            "limit: 2000",
            "}]",
            "multiFilter: [{",
            "lineType: 'Standard Plus'",
            "limit: 2000",
            "}]",
            "}",
        ],
    );
}
