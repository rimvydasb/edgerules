# Test Infrastructure Improvement Strategy

> **Role**: Senior Rust Engineer specializing in Test Infrastructure and Developer Experience (DX)  
> **Objective**: Reduce code volume, maintain 100% coverage, and optimize for "Agent Readability"

## Executive Summary

The EdgeRules test suite contains **373 tests** across **28 test files**, with existing test infrastructure including
macros (`assert_value!`, `assert_string_contains!`), helper functions, and a `TestServiceBuilder`. This document
provides a strategic roadmap for test refactoring that shrinks code size while improving maintainability for both humans
and AI coding assistants.

---

## Current State Analysis

### Strengths

1. **Existing Utilities** (`utilities.rs`):
    - `assert_value!` macro handles multiple input formats cleanly
    - `eval_field()`, `eval_value()`, `eval_all()` provide consistent evaluation patterns
    - `link_error_contains()` and `parse_error_contains()` abstract error testing

2. **Test Builder Pattern** (`edge_rules_tests.rs`):
    - `TestServiceBuilder` exists with fluent API
    - Methods: `expect_num()`, `expect_type()`, `expect_parse_error()`, `expect_link_error()`

3. **Clear Separation**:
    - Tests are organized by domain (math, logic, datetime, strings, lists, etc.)
    - Each test file includes its own `mod utilities; pub use utilities::*;` pattern

### Weaknesses

1. **Verbose Test Setups**: Many tests repeat boilerplate initialization
2. **Mixed Assertion Styles**: Some tests use raw `assert_eq!`, others use custom macros
3. **Limited Parameterization**: No `rstest` or table-driven tests for repetitive patterns
4. **Inconsistent Naming**: Variables like `tb`, `rt`, `err` lack semantic clarity
5. **Implicit AAA**: Arrange-Act-Assert phases are often blended together

---

## Recommended Patterns and Refactoring Strategies

### 1. Shrink via Abstraction (Builders & Factories)

#### Pattern 1A: Expression Test Builder

Current verbose pattern:

```rust
#[test]
fn test_math_abs() {
    init_logger();
    assert_value!("abs(10)", "10");
    assert_value!("abs(-10)", "10");
    assert_value!("abs(0)", "0");
    assert_value!("abs(-1.5)", "1.5");
}
```

**Recommended** – Use domain-specific builder that groups related cases:

```rust
#[test]
fn test_math_abs() {
    ExpressionTest::new("abs")
        .case("abs(10)", "10")
        .case("abs(-10)", "10")
        .case("abs(0)", "0")
        .case("abs(-1.5)", "1.5")
        .run_all();
}
```

**Implementation** – Add to `utilities.rs`:

```rust
pub struct ExpressionTest {
    function_name: String,
    cases: Vec<(String, String)>,
}

impl ExpressionTest {
    pub fn new(function_name: &str) -> Self {
        init_logger();
        Self { function_name: function_name.to_string(), cases: Vec::new() }
    }
    
    pub fn case(mut self, input: &str, expected: &str) -> Self {
        self.cases.push((input.to_string(), expected.to_string()));
        self
    }
    
    pub fn run_all(&self) {
        for (input, expected) in &self.cases {
            let actual = eval_value(&format!("value: {}", input));
            assert_eq!(
                actual, expected.as_str(),
                "Function {} failed for input: {}", self.function_name, input
            );
        }
    }
}
```

#### Pattern 1B: Error Expectation Builder

Current verbose pattern:

```rust
#[test]
fn test_unary_numeric_validation() {
    let numeric_funcs = ["abs", "floor", "ceiling", "trunc", "sqrt"];
    
    for func in numeric_funcs {
        let code = format!("{{ value: {}() }}", func);
        parse_error_contains(&code, &[&format!("Function '{}' got no arguments", func)]);
        
        let code = format!("{{ value: {}('abc') }}", func);
        link_error_location(
            &code,
            &["value"],
            &format!("{}('abc')", func),
            LinkingErrorEnum::TypesNotCompatible(None, ValueType::StringType, Some(vec![ValueType::NumberType])),
        );
    }
}
```

**Recommended** – Introduce validation builder:

```rust
#[test]
fn test_unary_numeric_validation() {
    UnaryFunctionValidator::for_number_functions(&["abs", "floor", "ceiling", "trunc", "sqrt"])
        .expect_parse_error_when_no_args()
        .expect_link_error_when_wrong_type("'abc'", ValueType::StringType)
        .validate();
}
```

---

### 2. Shrink via Parameterization (rstest)

#### Pattern 2A: Simple Parameterized Tests

**Add `rstest` to `Cargo.toml`**:

```toml
[dev-dependencies]
rstest = "0.24"
```

Current repetitive pattern:

```rust
#[test]
fn test_math_rounding_basic() {
    assert_value!("floor(1.1)", "1");
    assert_value!("floor(1.9)", "1");
    assert_value!("floor(-1.1)", "-2");
    assert_value!("floor(-1.9)", "-2");
    assert_value!("ceiling(1.1)", "2");
    assert_value!("ceiling(1.9)", "2");
    // ... more cases
}
```

**Recommended with rstest**:

```rust
use rstest::rstest;

#[rstest]
#[case("floor(1.1)", "1")]
#[case("floor(1.9)", "1")]
#[case("floor(-1.1)", "-2")]
#[case("floor(-1.9)", "-2")]
#[case("ceiling(1.1)", "2")]
#[case("ceiling(1.9)", "2")]
#[case("ceiling(-1.1)", "-1")]
#[case("ceiling(-1.9)", "-1")]
fn test_rounding_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_eq!(eval_value(&format!("value: {}", expression)), expected);
}
```

#### Pattern 2B: Grouped Parameterized Tests with Fixtures

```rust
use rstest::{rstest, fixture};

#[fixture]
fn math_runtime() -> EdgeRulesRuntime {
    let mut model = EdgeRulesModel::new();
    model.append_source("{ pi_val: 3.14159 }").unwrap();
    model.to_runtime().unwrap()
}

#[rstest]
#[case("abs(10)", "10")]
#[case("abs(-10)", "10")]
#[case("sqrt(4)", "2")]
fn test_math_functions(math_runtime: EdgeRulesRuntime, #[case] expr: &str, #[case] expected: &str) {
    let result = math_runtime.evaluate_expression_str(expr).unwrap();
    assert_eq!(result.to_string(), expected);
}
```

#### Pattern 2C: Parameterized Error Tests

```rust
#[rstest]
#[case("value: 10 / 0", "[runtime] Division by zero")]
#[case("value: 10 % 0", "[runtime] Division by zero")]
#[case("value: modulo(10, 0)", "[runtime] Division by zero")]
#[case("value: idiv(10, 0)", "[runtime] Division by zero")]
fn test_division_by_zero_errors(#[case] input: &str, #[case] expected_error: &str) {
    assert_string_contains!(expected_error, eval_value(input));
}
```

---

### 3. Optimize for Agents (Context & Structure)

#### Pattern 3A: Strict AAA Separation

Current blended pattern:

```rust
#[test]
fn test_complex_calculation() {
    let code = r#"{ func calc(x): { result: x * 2 }; value: calc(5).result }"#;
    assert_eq!(eval_field(code, "value"), "10");
}
```

**Recommended** – Explicit AAA with comments:

```rust
#[test]
fn test_user_function_doubles_input() {
    // Arrange
    let function_definition = "func doubleInput(x): { result: x * 2 }";
    let invocation = "value: doubleInput(5).result";
    let source_code = format!("{{ {}; {} }}", function_definition, invocation);
    
    // Act
    let evaluated_result = eval_field(&source_code, "value");
    
    // Assert
    assert_eq!(evaluated_result, "10", "doubleInput(5) should return 10");
}
```

#### Pattern 3B: Semantic Variable Naming

**Bad** (ambiguous for agents):

```rust
let tb = test_code_lines(&["func f(a): { result: a }", "value: f(35).result"]);
tb.expect_num("value", Int(35));
```

**Good** (explicit intent):

```rust
let user_function_test = test_code_lines(&[
    "func returnArgument(input): { result: input }",
    "invokedResult: returnArgument(35).result",
]);
user_function_test.expect_num("invokedResult", Int(35));
```

#### Pattern 3C: Structured Test Documentation

```rust
/// Tests that the `abs` function correctly handles positive, negative, and zero inputs.
/// 
/// # Test Cases
/// | Input      | Expected |
/// |------------|----------|
/// | `abs(10)`  | `10`     |
/// | `abs(-10)` | `10`     |
/// | `abs(0)`   | `0`      |
#[test]
fn test_abs_function_for_all_sign_types() {
    // Test implementation
}
```

---

### 4. Custom Assertions

#### Pattern 4A: Domain-Specific Assertions

**Create `test_assertions.rs`**:

```rust
/// Asserts that evaluating the given expression results in a link error
/// containing the expected message fragments.
#[macro_export]
macro_rules! assert_link_error {
    ($code:expr, $( $needle:expr ),+ $(,)?) => {{
        let needles: &[&str] = &[$($needle),+];
        link_error_contains($code, needles)
    }};
}

/// Asserts that evaluating the given expression results in a parse error
/// containing the expected message fragments.
#[macro_export]
macro_rules! assert_parse_error {
    ($code:expr, $( $needle:expr ),+ $(,)?) => {{
        let needles: &[&str] = &[$($needle),+];
        parse_error_contains($code, needles)
    }};
}

/// Asserts that a runtime error occurs at the specified location
/// with the expected expression.
#[macro_export]
macro_rules! assert_runtime_error {
    ($runtime:expr, $field:expr, $expected_location:expr, $expected_expr:expr) => {{
        let err = $runtime.evaluate_field($field).expect_err("expected runtime error");
        assert_eq!(err.location(), $expected_location, "Location mismatch");
        assert_eq!(err.expression().map(|s| s.as_str()), Some($expected_expr), "Expression mismatch");
    }};
}

/// Asserts that two code outputs are equivalent after whitespace normalization.
/// Note: `inline` function must be imported from utilities.rs
#[macro_export]
macro_rules! assert_code_eq {
    ($actual:expr, $expected:expr) => {{
        let actual_normalized = inline($actual);
        let expected_normalized = inline($expected);
        assert_eq!(
            actual_normalized, expected_normalized,
            "Code mismatch:\nActual: {}\nExpected: {}", $actual, $expected
        );
    }};
}
```

Usage example:

```rust
#[test]
fn test_cyclic_reference_detection() {
    assert_link_error!("value: value + 1", "cyclic");
    assert_link_error!(
        "{ record1: 15 + record2; record2: 7 + record3; record3: record1 * 10 }",
        "cyclic", "record1"
    );
}
```

#### Pattern 4B: Type-Specific Assertions

```rust
/// Asserts that evaluating the expression produces a number value.
#[macro_export]
macro_rules! assert_number_value {
    ($code:expr, $expected:expr) => {{
        let result = eval_value(&format!("value: {}", $code));
        assert_eq!(
            result.parse::<f64>().ok(),
            Some($expected),
            "Expected number {}, got: {}", $expected, result
        )
    }};
}

/// Asserts that evaluating the expression produces a boolean value.
#[macro_export]
macro_rules! assert_boolean_value {
    ($code:expr, $expected:expr) => {{
        let result = eval_value(&format!("value: {}", $code));
        assert_eq!(
            result,
            if $expected { "true" } else { "false" },
            "Expected boolean {}, got: {}", $expected, result
        )
    }};
}
```

---

### 5. Safety Improvements

#### Pattern 5A: Result-Based Tests

Current pattern with unwrap:

```rust
#[test]
fn test_service() {
    let mut service = EdgeRulesModel::new();
    service.append_source("value: 2 + 2").unwrap();
    let runtime = service.to_runtime().unwrap();
    let result = runtime.evaluate_field("value").unwrap();
    assert_eq!(result, ValueEnum::NumberValue(Int(4)));
}
```

**Recommended** – Return `Result`:

```rust
#[test]
fn test_service() -> Result<(), EvalError> {
    // Arrange
    let mut service = EdgeRulesModel::new();
    service.append_source("value: 2 + 2")?;
    let runtime = service.to_runtime()?;
    
    // Act
    let result = runtime.evaluate_field("value")?;
    
    // Assert
    assert_eq!(result, ValueEnum::NumberValue(Int(4)));
    Ok(())
}
```

#### Pattern 5B: Contextual Error Handling

```rust
/// Evaluates an expression and returns a descriptive error on failure.
fn evaluate_with_context(code: &str, field: &str) -> Result<String, String> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(|e| format!("Parse error for code '{code}': {e}"))?;
    let runtime = service.to_runtime().map_err(|e| format!("Link error for code '{code}': {e}"))?;
    runtime.evaluate_field(field).map(|v| v.to_string()).map_err(|e| format!("Runtime error for field '{field}': {e}"))
}

#[test]
fn test_with_better_errors() {
    let result = evaluate_with_context("value: 2 + 2", "value");
    assert_eq!(result, Ok("4".to_string()));
}
```

---

## Actionable Refactoring Plan

### Phase 1: Foundation (Week 1)

| Task                            | File                             | Priority |
|---------------------------------|----------------------------------|----------|
| Add `rstest` dependency         | `Cargo.toml`                     | High     |
| Create `test_assertions.rs`     | `crates/core-tests/tests/`       | High     |
| Enhance `ExpressionTest` builder| `utilities.rs`                   | High     |
| Add function validation builder | `utilities.rs`                   | Medium   |

### Phase 2: Math & Logic Tests (Week 2)

| Task                             | File                               | LOC Reduction |
|----------------------------------|------------------------------------|---------------|
| Parameterize rounding tests      | `evaluation_math_tests.rs`         | ~40%          |
| Parameterize division by zero    | `evaluation_math_tests.rs`         | ~30%          |
| Parameterize boolean comparisons | `evaluation_logic_tests.rs`        | ~50%          |
| Convert validation loops to rstest| `built_in_functions_validation_tests.rs` | ~40% |

### Phase 3: String & DateTime Tests (Week 3)

| Task                            | File                              | LOC Reduction |
|---------------------------------|-----------------------------------|---------------|
| Parameterize string functions   | `evaluation_string_tests.rs`      | ~35%          |
| Parameterize datetime comparisons| `evaluation_datetime_tests.rs`   | ~45%          |
| Parameterize duration/period ops| `evaluation_datetime_tests.rs`    | ~40%          |

### Phase 4: Integration & Service Tests (Week 4)

| Task                            | File                              | Focus         |
|---------------------------------|-----------------------------------|---------------|
| Apply AAA pattern strictly      | `edge_rules_tests.rs`             | Readability   |
| Improve variable naming         | `decision_service_tests.rs`       | Agent clarity |
| Convert to Result-based tests   | All files with unwrap()           | Safety        |

---

## Estimated Impact

| Metric                 | Current State | After Refactoring |
|------------------------|---------------|-------------------|
| Total test LOC         | ~4,500        | ~3,000 (-33%)     |
| Test files             | 28            | 28 (unchanged)    |
| Parameterized tests    | 0             | ~80 cases         |
| Custom assertions      | 2 macros      | 8+ macros         |
| Result-based tests     | ~30%          | ~90%              |
| AAA compliance         | ~40%          | ~95%              |

---

## Appendix: Files Requiring Changes

### High Priority (High test volume, repetitive patterns)

1. `evaluation_math_tests.rs` – 15 tests, heavy parameterization opportunity
2. `evaluation_logic_tests.rs` – 3 tests, large boolean truth tables
3. `evaluation_datetime_tests.rs` – 19 tests, temporal comparison patterns
4. `built_in_functions_validation_tests.rs` – 13 tests, loop-based validation

### Medium Priority (Moderate improvement potential)

5. `evaluation_string_tests.rs` – 4 tests, string function coverage
6. `evaluation_list_tests.rs` – 7 tests, list operation coverage
7. `evaluation_filter_tests.rs` – 5 tests, filter expression coverage
8. `evaluation_user_functions_tests.rs` – 25 tests, function call patterns

### Lower Priority (Already well-structured)

9. `edge_rules_tests.rs` – Uses TestServiceBuilder, needs naming/AAA cleanup
10. `decision_service_tests.rs` – Clean structure, needs Result-based conversion
11. `context_object_tests.rs` – Well-organized, minimal changes needed

---

## Implementation Notes for AI Agents

When implementing these refactoring patterns, AI coding assistants should:

1. **Preserve Test Intent**: Never change what the test validates, only how it's expressed
2. **Maintain Coverage**: Run `cargo test` after each refactoring batch to verify 100% pass rate
3. **Use Consistent Patterns**: Apply the same pattern across all tests of the same category
4. **Document Decisions**: Add comments explaining why a particular pattern was chosen
5. **Prefer Composition**: Build complex test helpers from simpler, reusable components

---

## Version History

| Version | Date       | Author | Changes               |
|---------|------------|--------|-----------------------|
| 1.0     | 2026-01-29 | Agent  | Initial specification |
