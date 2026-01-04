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

- `...` is Ellipsis / Placeholder, and it means "The gap goes here". Both unary tests expressions are supported:
    - With placeholder (e.g., `... >= 18`)
    - Without placeholder (e.g., `<= 65`) - in this case parser
      can assume that `...` is appended or prepended depending on the operator position. However,
      this is valid `>= 100 or ... = 0`, but `>= 100 or = 0` is invalid.

- `..` is Range operator.

## Range Checks as Unary Tests

- [ ] Support for range check syntax: `[start..end]`, `(start..end]`, etc.
- [ ] Validation of range check boundaries and types: as of now only numbers are supported.

**Good limitations:**

- Range Checks can only be defined for numbers.
- No infinite range checks (e.g., `[..100]` or `[50..`), user should use standard unary tests for that.
- A single expression can only contain one range check definition. For range check definition boundary is a first
  character `[` or `(` and last the last character  `]` or `)`.

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
> In this example `p : for number in 1..(5+inc) return number * 3` a range expression is used that does not have anything
> common with Range Checks used as unary tests!

## Simple Unary Tests

- [ ] Support for simple unary tests with placeholders (e.g., `... >= 18`, `... = "Active"`)
- [ ] Support for simple unary tests without placeholders (e.g., `<= 65`, `= "Active"`)
- [ ] Support for combining multiple unary tests with `and` / `or` operators.
- [ ] Validation of unary test expressions and types.
- [ ] Support for executing unary tests as functions: `ageCheck(20)`, `statusCheck("Active")`

**Examples:**

```edgerules
{
    ageCheck: ... >= 18
    statusCheck: = "Active"
    complexCheck: ... >= 100 or ... = 0
    nestedChecks: {
        checkA: ... < 50 and ... > 10
        checkB: not (... = 0)
    }
    listOfChecks: [
        ... <> "Inactive",
        ... <= 100,
        = "Pending"
    ]
    
    // executed as functions:
    isAdult: ageCheck(20)               // true
    isActive: statusCheck("Active")     // true
    isTrueComplex: complexCheck(100)    // true
    isInNestedA: nestedChecks.checkA(30)  // true
    isInListCheck1: listOfChecks[0]("Active")  // true
    allListTest: for test in listOfChecks return test("Active")  // [true, true, false]
}
```