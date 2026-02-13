# Test Infrastructure Improvement Strategy

> **Role**: Senior Rust Engineer specializing in Test Infrastructure and Developer Experience (DX)  
> **Objective**: Reduce code volume, maintain 100% coverage, and optimize for "Agent Readability"

## Executive Summary

The EdgeRules test suite contains **373 tests** across **28 test files**, with existing test infrastructure including
macros (`assert_value!`, `assert_string_contains!`), helper functions, and a `TestServiceBuilder`. This document
provides a strategic roadmap for test refactoring that shrinks code size while improving maintainability for both humans
and AI coding assistants.

## Issues

The "bad ergonomics" happen because when a custom macro panics, IntelliJ sees the panic occurring inside the macro
definition, not at the line where you called it. This breaks the "Go to Failure" functionality and inline highlighting.
For this reason do not use custom macros that panic in tests and use helper functions with `#[track_caller]`. If
required, develop additional helper functions.

## Bad Practices

**do not use #[rstest]**

**rstest** creates very bad ergonomics in IntelliJ because when a test fails inside a parameterized test, IntelliJ
cannot point to the line where the test was called with specific parameters. Instead, it points to the line inside
the **rstest** macro definition.

## Good Practices

**create service from source directly and expect success**

`let mut service = DecisionService::from_source(model).expect("service from source");`

**execute field by field if required**

```rust
let rt = get_runtime(code);
assert_eq!(exe_field(&rt, "applicationResponse.newAmount"), "3001");
```

**use helper functions**

```rust
#[test]
fn test_problems() {
    // nested value
    assert_eq!(eval_field("{ record: { age: 18; value: 1 + 2 }}", "record.value"), "3");

    // cyclic link errors
    link_error_contains("value: value + 1", &["cyclics"]);
}
```

**use moder features of Rust test framework**

`#[track_caller]`

## Utilities

`link_error_contains(code: &str, needles: &[&str])` - asserts that evaluating the code produces link errors containing
all the needles.

`parse_error_contains(code: &str, needles: &[&str])` - asserts that parsing the code produces parse errors containing
all the needles.

`runtime_error_contains(code: &str, needles: &[&str])` - asserts that evaluating the code produces runtime errors
containing all the needles.

`assert_eval_field(code: &str, field: &str, expected: &str)` - asserts that evaluating the code produces the expected
value for the specified field.

`assert_eval_value(code: &str, expected: &str)` - asserts that evaluating the code is an object, has a field `value`,
and that field equals the expected value.

`assert_expression_value(expression: &str, expected: &str)` - asserts that expression evaluation produces the expected
value.
Used to test single line expressions.

`expression_value_contains(expression: &str, needles: &[&str])` - asserts that expression evaluation produces a
value that contains the expected substring. Used to test single line expressions.

`assert_eval_all(code: &str, expected_lines: &[&str])` - asserts that evaluating the code produces the expected full
object. Used to test full object equality.

## Next Steps

### Phase 1

- [x] Develop `assert_eval_field` and start using it where `assert_eq!(eval_field(...` is used.

```rust
//assert_eq!(eval_field("{ record: { age: 18; value: 1 + 2 }}", "record.value"), "3");
assert_eval_field("{ record: { age: 18; value: 1 + 2 }}", "record.value", "3");
```

- [x] All test utilities must accept `EdgeRulesRuntime` as well, so if the runtime is already created, it can be
  reused. Use best rust practices and implement `impl From<&str> for EdgeRulesRuntime {...`, then `assert_eval_field`
  can accept both. This probably does not make sense for `parse_error_contains` and `link_error_contains` because they
  are
  used before runtime creation.
- [x] Annotate utilities with `#[track_caller]` to improve error reporting.
- [x] Convert `assert_string_contains` to utility function with `#[track_caller]`.

### Phase 2

- [x] Eliminate `assert_value` because it creates very bad ergonomics in IntelliJ and is already too complex. Use
  developed utilities instead.
- [x] Eliminate `exe_field` and use `assert_eval_field` instead.
- [x] Instead of `eval_all`, consider using `assert_expression_value`, `expression_value_contains` or `assert_eval_all`
  where appropriate.
- [x] Find `assert!(evaluated.contains(...` and use `expression_value_contains` instead.

### Phase 3

- [x] Check `edge_rules_tests.rs` for non-tests (utilities) such as `TestServiceBuilder` and move them to `utilities.rs`
- [x] Check `test_utils.rs` and move relevant utilities to `utilities.rs`
- [x] Run all tests and check them if passes