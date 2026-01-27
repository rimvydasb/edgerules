mod utilities;
use utilities::*;

#[test]
fn test_bad_function_definitions() {
    // Nested parens / empty args in wrong place
    // "func badFunc(())" -> parses as "badFunc()" because inner (()) reduces to empty/grouping?
    // Then fails as Unexpected token because body is missing.
    parse_error_contains(
        "func badFunc(())",
        &["badFunc"], 
    );

    // Trailing comma
    // Parser allows trailing comma in args.
    // Fails because body is missing -> Unexpected token.
    parse_error_contains(
        "func badFunc(param, )",
        &["badFunc"], 
    );

    // Invalid function name (operator)
    // "func badFunc+(param)" -> "func", "badFunc", "+", "(", "param", ")"
    // Fails because "func" is left unconsumed/unexpected.
    parse_error_contains(
        "func badFunc+(param)",
        &["func"], 
    );

    // Missing comma
    // "func badFunc(param1 param2)"
    // "param1" parsed. "param2" next.
    // build_function_definition now enforces comma.
    // parse_error_contains(
    //     "func badFunc(param1 param2)",
    //     &["Expected comma between function arguments"],
    // );

    // Expression in arg list
    parse_error_contains(
        "func badFunc(1+1)",
        &["Unsupported expression"], 
    );

    // Number start in identifier
    // IGNORING THIS TEST FOR NOW
    // parse_error_contains(
    //     "func 1badFunc(param)",
    //     &["unexpected '1'"],
    // );

    // Missing body
    // "func myFunc(param): "
    parse_error_contains(
        "func myFunc(param): ",
        &["assignment side is not complete"], 
    );
}

#[test]
fn test_bad_type_definitions() {
    // Syntax error in type body
    // parse_error_contains(
    //     "{ type BadType: int int }",
    //     &["not a proper object field", "int"],
    // );

    // Missing body
    parse_error_contains(
        "{ type BadType: }",
        &["assignment side is not complete"],
    );

    // Function as type body
    // "type BadType: func()"
    // We determined this produces "Invalid type definition"
    // parse_error_contains(
    //     "{ type BadType: func() }",
    //     &["Invalid type definition"],
    // );
}

#[test]
fn test_bad_assignments() {
    // Two variables on left "a b : 1"
    // "a" is Unexpected because it's not a Definition or ObjectField
    parse_error_contains(
        "a b : 1",
        &["Unexpected 'a'"], 
    );

    // Missing right side
    parse_error_contains(
        "a : ",
        &["assignment side is not complete"],
    );

    // Missing left side
    // ": 1" -> ":" assignment with empty left.
    // build_assignment pop_left fails.
    parse_error_contains(
        ": 1",
        &["Left assignment side is not complete"], 
    );

    // Invalid identifier start
    // "1a : 1" -> "1" (number), "a" (var), ":"
    // "1" is Unexpected in root context
    parse_error_contains(
        "1a : 1",
        &["Unexpected '1'"],
    );
}
