#[test]
fn test_conditionals() {
    // comparisons
    assert_value!("1 = 2", "false");
    assert_value!("1 < 2", "true");
    assert_value!("1 <= 2", "true");
    assert_value!("2 > 1", "true");
    assert_value!("2 >= 1", "true");
    assert_value!("1 = 1", "true");
    assert_value!("1 = 1 + 1", "false");

    // boolean ops with numbers in conditionals
    assert_value!("1 = 2 and 5 = 5", "false");
    assert_value!("1 + 1 = 2 and 5 = 5", "true");

    assert_value!("1 = 2 or 5 = 5", "true");
    assert_value!("1 = 2 or 5 = 5 + 1", "false");

    assert_value!("1 = 2 xor 5 = 5 + 1", "false");
    assert_value!("1 = 2 xor 5 = 4 + 1", "true");
    assert_value!("1 = 2 - 1 xor 5 = 5 + 1", "true");

    assert_value!("1 = 2 or 5 = 5 and 1 = 1", "true");
    assert_value!("1 = 2 or 5 = 5 and 1 = 1 + 1", "false");

    // if-then-else nesting
    assert_value!("if 1 > 2 then 3 else 4", "4");
    assert_value!("if 1 < 2 then 3 else 4", "3");
    assert_value!("if 1 < 2 then 3 + 1 else 5", "4");
    assert_value!("if 1 > 2 then 3 + 1 else 5 * 10", "50");
    assert_value!(
        "if 1 > 2 then 3 + 1 else (if 1 < 2 then 5 * 10 else 0)",
        "50"
    );
    assert_value!(
        "if 1 > 2 then 3 + 1 else (if 1 > 2 then 5 * 10 else 0)",
        "0"
    );
    assert_value!("if 1 < 2 then (if 5 > 2 then 5 * 10 else 0) else 1", "50");
    assert_value!(
        "(if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1",
        "51"
    );
    assert_value!(
        "1 + (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1",
        "52"
    );
    assert_value!(
        "2 * (if 1 < 2 then if 5 > 2 then 5 * 10 else 0 else 1) + 1",
        "101"
    );
}

#[test]
fn test_boolean_literals_and_logic() {
    // OR
    assert_value!("true  or true", "true");
    assert_value!("true  or false", "true");
    assert_value!("false or true", "true");
    assert_value!("false or false", "false");

    // AND
    assert_value!("true  and true", "true");
    assert_value!("true  and false", "false");
    assert_value!("false and true", "false");
    assert_value!("false and false", "false");

    // XOR
    assert_value!("true  xor true", "false");
    assert_value!("true  xor false", "true");
    assert_value!("false xor true", "true");
    assert_value!("false xor false", "false");

    // NOT
    assert_value!("not true", "false");
    assert_value!("not false", "true");
    assert_value!("not (1 = 1)", "false");
    assert_value!("not (1 = 2)", "true");

    // Mixed
    assert_value!("true and (1 < 2)", "true");
    assert_value!("(1 = 1) and false", "false");
    assert_value!("(1 = 1) or false", "true");
    assert_value!("true and not false", "true");
    assert_value!("(1 < 2) and not (2 < 1)", "true");

    // More complex
    assert_value!("(true and (1 < 2)) or (false and (3 = 4))", "true");
    assert_value!("(true xor (1 = 1 and false)) or (2 < 1)", "true");
    assert_value!("(true and true) xor (false or (1 < 1))", "true");
    assert_value!(
        "(true and (2 > 1 and (3 > 2))) and (false or (5 = 5))",
        "true"
    );
}

#[test]
fn test_constraints() {
    assert_value!("[1,2,3][...>1]", "[2, 3]");
    assert_value!("[1,2,3][...>0]", "[1, 2, 3]");
    assert_value!("[1,2,3][...>-5]", "[1, 2, 3]");
    assert_value!("[1,2,3][...<-5]", "[]");

    assert_eq!(
        eval_field(
            r#"
            {
                nums : [1, 5, 12, 7];
                filtered: nums[...>6]
            }
            "#
            .trim(),
            "filtered"
        ),
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

mod utilities;
pub use utilities::*;
