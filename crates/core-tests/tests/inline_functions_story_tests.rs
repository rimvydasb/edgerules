mod utilities;

use edge_rules::ast::metaphors::functions::UserFunctionDefinition;
use edge_rules::runtime::edge_rules::EdgeRulesModel;
pub use utilities::*;

#[test]
fn inline_function_executes_simple_expression() {
    assert_value!(
        r#"
        {
            func addOne(x): x + 1
            value: addOne(2)
        }
        "#,
        "3"
    );
}

#[test]
fn inline_functions_support_nested_calls() {
    assert_value!(
        r#"
        {
            func addOne(x): x + 1
            func doubleAndAddOne(y): addOne(y * 2)
            value: doubleAndAddOne(3)
        }
        "#,
        "7"
    );
}

#[test]
fn return_field_scopes_result_for_functions() {
    assert_value!(
        r#"
        {
            func calc(a): {
                hidden: a * 2;
                return: hidden + 1
            }
            value: calc(4)
        }
        "#,
        "9"
    );
}

#[test]
fn return_field_blocks_field_access_on_result() {
    link_error_contains(
        r#"
        {
            func calc(a): {
                hidden: a * 2;
                return: hidden + 1
            }
            value: calc(4).hidden
        }
        "#,
        &["not an object", "calc(4)"],
    );
}

#[test]
fn return_field_in_plain_context_behaves_as_normal_field() {
    assert_value!(
        r#"
        {
            obj: { return: 5 + 5 }
            value: obj.return
        }
        "#,
        "10"
    );
}

#[test]
fn return_only_bodies_collapse_to_inline_definition() {
    let code = r#"
    {
        func add(a): { return: a + a }
    }
    "#;

    let mut model = EdgeRulesModel::new();
    model.append_source(code).unwrap();
    let method = model.get_user_function("add").unwrap();
    let borrowed = method.borrow();

    match &borrowed.function_definition {
        UserFunctionDefinition::Inline(_) => {}
        _ => panic!("expected inline function collapse for return-only body"),
    }

    let display = inline(format!("{}", borrowed.function_definition));
    assert_eq!(display, "add(a):a+a");
}

#[test]
fn inline_functions_support_type_annotations() {
    assert_value!(
        r#"
        {
            func addOne(x: number): x + 1
            value: addOne(2)
        }
        "#,
        "3"
    );
}

#[test]
fn inline_functions_with_wrong_argument_type_fail_linking() {
    link_error_contains(
        r#"
        {
            func addOne(x: number): x + 1
            value: addOne("not a number")
        }
        "#,
        &["expect", "number", "string"],
    );
}

#[test]
fn inline_functions_with_no_arguments() {
    assert_value!(
        r#"
        {
            func getTen(): 10
            value: getTen()
        }
        "#,
        "10"
    );
}

#[test]
fn inline_functions_shadowing_outer_variables() {
    assert_value!(
        r#"
        {
            x: 10
            func shadow(x): x + 1
            value: shadow(5) // Should use argument x=5, not outer x=10
        }
        "#,
        "6"
    );
}

#[test]
fn inline_functions_using_complex_expressions_as_arguments() {
    assert_value!(
        r#"
        {
            func add(a, b): a + b
            value: add(1 + 2, 3 * 4)
        }
        "#,
        "15"
    );
}

#[test]
fn inline_functions_recursion_detection() {
    // EdgeRules doesn't support recursion because it uses static linking
    link_error_contains(
        r#"
        {
            func rec(x): if x > 0 then rec(x - 1) else 0
            value: rec(5)
        }
        "#,
        &["Cyclic reference", "rec"],
    );
}

#[test]
fn nested_inline_functions_in_expression() {
    assert_value!(
        r#"
        {
            func outer(x): {
                func inner(y): x + y
                return: inner(x)
            }
            value: outer(10)
        }
        "#,
        "20"
    );
}

#[test]
fn inline_function_returning_object() {
    assert_value!(
        r#"
        {
            func mkObj(a, b): { res: a + b }
            value: mkObj(1, 2).res
        }
        "#,
        "3"
    );
}

#[test]
fn inline_function_with_all_supported_types() {
    assert_value!(
        r#"
        {
            func allTypes(n: number, s: string, b: boolean, d: date): s + toString(n) + toString(b) + toString(d.year)
            value: allTypes(1, 'val', true, date('2024-01-19'))
        }
        "#,
        "'val1true2024'"
    );
}

#[test]
fn inline_function_shadowing_it_alias() {
    assert_value!(
        r#"
        {
            func check(it): it + 1
            list: [1, 2, 3]
            value: list[check(it) > 2] // it in check(it) is the list element
        }
        "#,
        "[2, 3]"
    );
}
