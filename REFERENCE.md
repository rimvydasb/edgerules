# EdgeRules Language Reference

This document describes the EdgeRules DSL as currently implemented in this repository. It reflects actual behavior in `src/` and unit tests under `tests/`.

## Overview

- **Model**: A program is a context of named fields and optional function definitions. Values are immutable and referentially transparent.
- **Assignment**: Use `:` to assign a name to an expression, e.g., `value : 2 + 2`.
- **Top level**: Provide a single object `{ ... }` or a single assignment/definition per load. Multiple structures are composed by loading more than once or by nesting inside one object.
- **Comments**: Line comments start with `//` and continue to end-of-line.

## Data Types

- **number**: Integers and reals; arithmetic supports `+ - * / ^` and unary negation.
- **string**: Single-quoted or double-quoted literal text.
- **boolean**: Boolean literals `true` and `false` and results of comparisons/logical operators.
- **list of T**: Homogeneous lists are intended; mixed types parse but type inference is basic.
- **range**: Integer ranges `a..b` (inclusive), e.g., `1..5` yields 1,2,3,4,5. Useful with `for` and built-ins like `sum`, `count`, `max`.
- **object (context)**: Nested named fields forming a context object.
- **special values**: Present in the runtime/value model: `Missing`, `NotApplicable`, `NotFound`. Users cannot assign `NotFound`; out-of-bounds access and failed lookups typically yield a special numeric/string sentinel where applicable.

## Literals & Identifiers

- **Numbers**: `123`, `0`, `12.5`, `0.5`.
- **Strings**: `'hello'`, "hello".
- **Identifiers**: `a`, `my_var2`, `alpha123`. Dotted paths for field access: `applicant.age`.

## Objects & Assignment

- **Object literal**: `{ field1 : expr; field2 : expr }`.
  - Field separators: newline or `;`. Trailing commas are not used for fields.
- **Nested objects**: Values can be objects: `a : { x : 1 }`.
- **Top-level composition**: Place all fields/defs in a single `{ ... }` or load multiple snippets into the engine. Duplicate field names within the same object are the caller’s responsibility; the last wins during builder append/merge.

## Variables & Paths

- **Path selection**: `a.b.c` selects nested fields.
- **Self-qualified paths**: Inside a context `calendar : { shift : 2; ... }`, references like `calendar.shift` within that same block resolve to the local context (self) rather than starting from root. This enables patterns like arrays of inline objects referencing siblings: `{ start : calendar.shift + 1 }`.
- **Scope resolution**: Lookup climbs outward through parent contexts up to root.

## Arrays, Filters, Ranges

- **Array literal**: `[expr1, expr2, ...]` (elements comma-separated).
- **Indexing**: `list[expr]` where `expr` evaluates to a number. Out-of-bounds returns a special `Missing` value.
- **Filtering (predicate)**: `list[ ... > 10 ]`, `list[<= 3]`, or `list[not it > 10]`.
  - `...` denotes the context item during filtering (current element).
  - `it` is an alias for the current element and can be used interchangeably with `...` (e.g., `list[not it > 10]`).
  - A predicate result produces a filtered list; a numeric result selects a single element.
  - Field selection requires an object value; select an element first if you need a field: e.g., `people[...>.age > 18][0].name` (predicate then index then select).
- **Ranges**: `a..b` creates an inclusive integer range. Example: `for n in 1..5 return n * 2` → `[2,4,6,8,10]`.

## Operators

- **Arithmetic**: `+ - * / ^`
  - Precedence: `^` > `* /` > `+ -`.
  - Unary negation supported: `-x`, `-(a + b)`.
  - Modulo `%` exists in internal enum but is not tokenized; do not use.
- **Comparators**: `=`, `<>`, `<`, `>`, `<=`, `>=`.
  - Type rules: both sides must have the same type. String comparison supports `=` and `<>`.
- **Logical**: `and`, `or`, `xor` (binary) and unary `not`.
  - Precedence (high → low): comparisons (`=`, `<>`, `<`, `>`, `<=`, `>=`) > `not` > `and`/`xor`/`or`.
  - Example: `not it > 10` parses as `not (it > 10)`. Use parentheses to make intent explicit when combining.
- **Parentheses**: `( ... )` to group expressions.

### Operator Precedence

From highest to lowest. Parentheses always take precedence to group explicitly.

| Level | Operators / forms                      | Notes / examples                             |
|---|----------------------------------------|----------------------------------------------|
| 1 | Parentheses `(...)`                     | Grouping                                      |
| 2 | Function call `f(...)`                  | `sum([1,2,3])`                                |
| 3 | Field/select/filter `.`, `[...]`        | `obj.field`, `list[... > 10]`                 |
| 4 | Unary minus `-`                         | `-(a + b)`                                    |
| 5 | Power `^`                               | `2 ^ 3`                                       |
| 6 | Multiply/Divide `* /`                   | `a * b / c`                                   |
| 7 | Add/Subtract `+ -`                      | `a + b - c`                                   |
| 8 | Comparators `= <> < > <= >=`            | `a + 1 = 3` (arithmetic before compare)       |
| 9 | Unary logical `not`                     | `not it > 10` ≡ `not (it > 10)`               |
| 10 | Logical `and`, `xor`, `or`             | Use parentheses to disambiguate if needed     |

## Control Constructs

- **If-Then-Else**: `if cond then a else b`
  - `cond` must be boolean.
  - `a` and `b` must have the same type. Example: `if age >= 18 then 'adult' else 'minor'`.
- **For-Comprehension**: `for x in source return expr`
  - Iterates lists and ranges; returns a list of mapped results.
  - Example: `for n in 1..5 return n * 3` → `[3,6,9,12,15]`.

## Functions

### Built-ins (implemented)

- `sum(...)`:
  - Multi-arg: `sum(1, 2, 3)` → number.
  - Unary over list/range/number: `sum([1,2,3])`, `sum(1..5)`, `sum(10)`.
- `max(...)`:
  - Multi-arg: `max(1, 4, 2)`.
  - Unary over list/range/number: `max([1,4,2])`, `max(1..5)`, `max(10)`.
- `count(x)`:
  - For list: element count; for range: item count; for a single number: `1`.
- `find(list, value)`:
  - Returns the first index of `value` in `list`, or `Missing` if not found.

Type validation is enforced during linking: numeric-only where applicable; `find` requires the second argument to have the list’s item type.

### User-Defined Functions

- **Definition**: `myFunc(a, b) : { result : a + b }`
  - The function body is a context. To use a computed field, select it: `myFunc(1,2).result`.
  - Parameter type annotations are currently parsed as plain identifiers; types in arguments are not yet enforced.
- **Call**: `myFunc(x, y)` returns a function context (object reference) which you typically field-select: `myFunc(x,y).result`.
- **Scoping**: Calls can occur from nested contexts; parameters are evaluated in the caller’s context.

### Annotations

- `@Service` before a function marks it as a service metaphor (parsed; no special runtime behavior).
- `@DecisionTable("first-hit"|"multi-hit")` allows defining decision tables using a function name and a rows collection. Parsing and pretty-printing exist; full linking/evaluation is not implemented.

## Expression Forms

- **Assignment**: `name : expr`
- **Object**: `{ a : 1; b : a + 2 }`
- **Array**: `[1, 2, 3]`
- **Field selection**: `obj.field`
- **Filter**: `list[... >= 10]`, `list[2]`
- **Range**: `1..5`
- **If**: `if a > b then a else b`
- **For**: `for x in list return x * 2`
- **Function call**: `sum([1,2,3])`, `max(1,2,3)`, `find(list, 3)`, `myFunc(1,2).result`

## Errors & Diagnostics

- **Parse errors**: Unexpected/missing tokens, incomplete expressions, invalid sequence elements. Examples include:
  - `Very first sequence element is missing`
  - `Filter not completed '['` / `Selection must be variable or variable path`
- **Linking errors**: Type mismatches, unresolved variables, cyclic references, missing functions. Messages include:
  - `Field X not found in Y`
  - `Types not compatible` / `Operation is not supported for different types`
  - `Field A.B appears in a cyclic reference loop`
- **Runtime errors**: Applying operations to unsupported types, accessing fields on non-objects, etc.

## Examples

- Object with references and arrays:
```edgerules
application : {
    applDate : 20230402
    applicants : [1,2,3]
    first : applicants[0]
}
```

- Boolean and logic:
```edgerules
{  
  a : true
  b : false
  allTrue  : a and not b         // true
  anyTrue  : a or b              // true
  justOne  : a xor b             // true
  negate   : not (1 = 1)         // false
  complex  : (1 < 2 and true) or (false and 2 > 3) // true
}
```

- Filters with `not` and `it` alias:
```edgerules
model : {
  nums : [1, 5, 12, 7, 15]
  small : nums[not it > 10]        // [1,5,7]
  smallCount : count(small)        // 3
  mid : nums[(it > 3) and not (it > 10)] // [5,7]
}
```

- Self-qualified references within a context:
```edgerules
calendar : {
    shift : 2
    days : [ { start : calendar.shift + 1 }, { start : calendar.shift + 31 } ]
    firstDay : days[0].start
    secondDay : days[1].start
}
```

- Complex ruleset example:
```edgerules
eligibility : {
  age    : 22
  score  : 180
  hasDebt : false

  // Eligible if (adult and high score) or (no debt and 21+)
  isAdult      : age >= 18
  highScore    : score >= 200
  conditionA   : isAdult and highScore
  conditionB   : not hasDebt and age >= 21
  result       : conditionA or conditionB   // true for the given inputs
}
```

- Loop and built-ins:
```edgerules
model : {
    sales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
    salesCount : count(sales)
    sales3(month, sales) : { result : sales[month] + sales[month + 1] + sales[month + 2] }
    acc : for m in 1..(salesCount - 2) return sales3(m, sales).result
    best : max(acc)
}
```

## Limitations (Current)

- No string operators beyond `=`/`<>` comparison; no substring/length functions.
- `%` exists in internal enums but is not tokenized; avoid using it.
- Decision tables are parsed but not linked/evaluated.
- Field selection requires an object; selecting a field directly from a filtered list is not supported without indexing.
- Function parameter type annotations are not enforced at parse/link time.

## Formatting & Display

- A pretty-printer exists for execution context; evaluation to a full context can be rendered with `to_code` used in tests. Output resembles `{ a : 1; b : a + 2 }` with nested contexts.

## Notes

- The language favors small, embeddable runtime and clear tracing over breadth of features. See `README.md` for roadmap and future FEEL coverage plans.
