# Unary Tests Story

## Goals

Provide a capability to define unary tests for decision table cells, filters and lists of constraints.

```edgerules
{
    ageCheck: ... >= 18
    scoreCheck: [700..800]
    complexCheck: ... >= 100 or ... = 0
    
    // executed as functions:
    isAdult: ageCheck(20)               // true
    isHighScore: scoreCheck(750)        // true
    isTrueComplex: complexCheck(100)    // true
}
```

**Clarifications:**

- `...` is Ellipsis / Placeholder, and it means "The gap goes here". Both unary tests expressions are supported:
    - With placeholder (e.g., `... >= 18`)
    - Without placeholder (e.g., `<= 65`) - in this case parser
      can assume that `...` is appended or prepended depending on the operator position. However,
      this is valid `>= 100 or ... = 0`, but `>= 100 or = 0` is invalid.

- `..` is Range operator.

- **Unary Test Definition** is an expression that contains at least one placeholder `...` or is a Range Check.
  Unary test definitions are not executed immediately, they are just definitions same as `func` or `type` definitions.

- **Unary Test Execution** is done by calling the Unary Test Definition as a function with a single argument.
  Only one argument is supported, which is mapped to the placeholder `...` during execution or used within Range Check.
  Each unary test execution returns a `boolean` value.

- **Range Check Definition** is a special syntax for defining unary tests that check if a value falls within a specified
  range.

- **Range Check Execution** is done by calling the Range Check Definition as a function with a single argument.

## Known Limitations

1. `in` operator is not supported at all for now.

## Range Checks as Unary Tests

- [ ] Support for range check syntax: `[start..end]`, `(start..end]`, etc.
- [ ] Validation of range check boundaries and types: as of now only numbers are supported.

**Good limitations:**

- Range Checks can only be defined for numbers.
- Range Checks definition must contain both start and end boundaries and `..` operator.
- No infinite range checks (e.g., `[..100]` or `[50..`), user should use standard unary tests for that.
- A single expression can only contain one range check definition. For range check definition boundary is a first
  character `[` or `(` and last the last character  `]` or `)`.
- No support for negation of range checks (e.g., `not [start..end]`). This is done to simplify Range Check definition
  parsing.
- Unary Test must contain `...` to be recognized as Unary Test Definition (except for Range Checks when other rules
  apply).
- No support for simple unary tests without placeholders (e.g., `<= 65`, `= "Active"`), must be defined with `...`
  placeholder.

**Parser rules:**

- If expression contains `..` (two dots) it is treated as Range Check Definition, that must also have boundaries as
  first and last characters of the expression.
- If expression contains `...` (ellipsis) it is treated as Unary Test Definition.

**Examples:**

```edgerules
{
    scoreCheck1: [700..800]
    scoreCheck2: (600..700]
    nestedRanges: {
        rangeA: [10..20]
        rangeB: (30..40)
    }
    listOfRanges: [
        [1..10],
        (20..30]
    ]
    
    // executed as functions:
    isInRange1: scoreCheck1(750)        // true
    isInRange2: scoreCheck2(600)        // true
    isInNestedRangeA: nestedRanges.rangeA(15)  // true
    isInListRange1: listOfRanges[0](5)  // true
    allListTest: for test in listOfRanges return test(5)  // [true, false]
}
```

> Do not confuse Range Checks with Range Expressions (e.g., `a..b`), which produce lists of values!
> In this example `p : for number in 1..(5+inc) return number * 3` a range expression is used that does not have
> anything common with Range Checks used as unary tests!

> Do not confuse include range check `[start..end]` with list literal syntax `[ ... ]`.
> Parser must successfully identify `..` as Range Check operator and not treat the whole expression as a List literal.

## Simple Unary Tests

- [ ] Support for simple unary tests with placeholders (e.g., `... >= 18`, `... = "Active"`)
- [ ] Support for combining multiple unary tests with `and` / `or` operators.
- [ ] Validation of unary test expressions and types.
- [ ] Support for executing unary tests as functions: `ageCheck(20)`, `statusCheck("Active")`

**Examples:**

```edgerules
{
    ageCheck: ... >= 18
    statusCheck: ... = "Active"
    complexCheck: ... >= 100 or ... = 0
    nestedChecks: {
        checkA: ... < 50 and ... > 10
        checkB: not (... = 0)
    }
    listOfChecks: [
        ... <> "Inactive",
        ... <= 100,
        ... = "Pending"
    ]
    withBuiltIn: contains(["ACTIVE", "PENDING"], ...)
    
    // executed as functions:
    isAdult: ageCheck(20)               // true
    isActive: statusCheck("Active")     // true
    isTrueComplex: complexCheck(100)    // true
    isInNestedA: nestedChecks.checkA(30)  // true
    isInListCheck1: listOfChecks[0]("Active")  // true
    allListTest: for test in listOfChecks return test("Active")  // [true, true, false]
    withBuiltInTest: withBuiltIn("ACTIVE")  // true
}
```

## Clarifications

1. `start < ... <= end` and `(start..end]` are equivalent.

## Technical Implementation Notes

- [ ] Add `UnaryOrRangeTestDefinition` variant to `ExpressionEnum` to make unary tests first class citizens in the
  expression AST. This definition holds `EvaluatableExpression`.
- [ ] Make sure `UnaryOrRangeTestDefinition` can be created in any context and lists. However, lists must stay
  homogeneous so if a list contains unary tests, then all items must be unary tests.
- [ ] Same as `FunctionDefinition`, create `RangeCheckDefinition` that has name, start and end boundaries, and start and
  end ranges. Implement `EvaluatableExpression` for it. `StaticLink` link method always returns boolean type.
- [ ] Same as `FunctionDefinition`, create `UnaryTestDefinition` that has name, parameter type as `ComplexTypeRef` and
  `ExpressionEnum` as a body. Implement `EvaluatableExpression` for it. `StaticLink` link method always returns boolean
  type.
- [ ] For `UnaryTestDefinition` parameter type must be derived from the expression body. If multiple placeholders are
  used, then parameter type must be validated to be the same for all placeholders.
- [ ] Extend `UserFunctionCall` linking to support `UnaryTestDefinition` and `RangeCheckDefinition`.
  If `UserFunctionCall` has a single argument, then extend the definition resolution to also check for
  `UnaryTestDefinition` and `RangeCheckDefinition` with the same name.
- [ ] Implement dynamic unnamed calls of unary tests from the lists such as `listOfChecks[0]("Active")`.
  During linking, make sure that list of items type is `UnaryOrRangeTestDefinition` to allow dynamic calls.
  Note that lists must be homogeneous.

# Story Review

TBC

