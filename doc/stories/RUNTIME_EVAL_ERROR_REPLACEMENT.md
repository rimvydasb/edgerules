# Runtime EvalError Replacement Plan

This document lists all occurrences of `RuntimeError::eval_error` (which maps to `RuntimeErrorEnum::EvalError`) and
proposes specific error enums to replace them. It classifies them by whether they are true runtime errors or should be
prevented by static linking.

## Summary of Proposed Error Enums

- **`OverflowError(String)`**: For date/time/numeric overflows.
- **`OperationNotSupported(String)`**: For operators or functions applied to incompatible types (where static linking
  didn't catch it or it's data-dependent).
- **`FeatureDisabled(String)`**: For optional features like Regex or Base64.
- **`RegexError(String)`**: For invalid regex patterns at runtime.
- **`IterationError(String)`**: For failures in `for` loops (e.g., iterating over non-lists).
- **`ConversionError(String)`**: For failures in type conversions (e.g., to JS values).
- **`ContextError(String)`**: For issues accessing context variables.
- **`InternalError(String)`**: For unexpected internal states or wrapping external errors that shouldn't happen.

---

## Detailed Replacements by File

### `crates/core/src/ast/operators/math_operators.rs`

| Line | Code Context                                                    | Proposed Replacement    | Notes                                                                     |
|:-----|:----------------------------------------------------------------|:------------------------|:--------------------------------------------------------------------------|
| 324  | `Operator '{}' is not implemented for '{} ^ {}'`                | `OperationNotSupported` | **Guarded by Linking**: `StaticLink` ensures types match and are numbers. |
| 362  | `Unsupported operator '{}' for duration values`                 | `OperationNotSupported` | **Guarded by Linking**: `StaticLink` validates duration operators.        |
| 375  | `Duration addition overflowed`                                  | `OverflowError`         | **Runtime**: Depends on data.                                             |
| 378  | `Duration subtraction overflowed`                               | `OverflowError`         | **Runtime**: Depends on data.                                             |
| 393  | `Unsupported operator '{}' for period values`                   | `OperationNotSupported` | **Guarded by Linking**.                                                   |
| 406  | `Period addition overflowed`                                    | `OverflowError`         | **Runtime**.                                                              |
| 409  | `Period addition overflowed`                                    | `OverflowError`         | **Runtime**.                                                              |
| 415  | `Period subtraction overflowed`                                 | `OverflowError`         | **Runtime**.                                                              |
| 418  | `Period subtraction overflowed`                                 | `OverflowError`         | **Runtime**.                                                              |
| 432  | `Month offset is out of range...`                               | `OverflowError`         | **Runtime**.                                                              |
| 454  | `Invalid date produced by duration adjustment`                  | `OverflowError`         | **Runtime**.                                                              |
| 469  | `Second offset is out of range...`                              | `OverflowError`         | **Runtime**.                                                              |
| 475  | `Date adjustment with duration overflowed`                      | `OverflowError`         | **Runtime**.                                                              |
| 490  | `Second offset is out of range...`                              | `OverflowError`         | **Runtime**.                                                              |
| 496  | `Datetime adjustment with duration overflowed`                  | `OverflowError`         | **Runtime**.                                                              |
| 519  | `Invalid time produced by duration adjustment`                  | `OverflowError`         | **Runtime**.                                                              |
| 529  | `Day offset is out of range...`                                 | `OverflowError`         | **Runtime**.                                                              |
| 533  | `Date adjustment with period overflowed`                        | `OverflowError`         | **Runtime**.                                                              |
| 628  | `Operator '{}' is not implemented for date and duration...`     | `OperationNotSupported` | **Guarded by Linking**.                                                   |
| 650  | `Operator '{}' is not implemented for datetime and duration...` | `OperationNotSupported` | **Guarded by Linking**.                                                   |
| 670  | `Operator '{}' is not implemented for time and duration...`     | `OperationNotSupported` | **Guarded by Linking**.                                                   |
| 688  | `Date difference overflowed duration range`                     | `OverflowError`         | **Runtime**.                                                              |
| 736  | `Operator '{}' is not implemented for date and period...`       | `OperationNotSupported` | **Guarded by Linking**.                                                   |
| 757  | `Operator '{}' is not implemented for datetime and period...`   | `OperationNotSupported` | **Guarded by Linking**.                                                   |
| 774  | `Cannot apply '{}' between period and duration...`              | `OperationNotSupported` | **Guarded by Linking**.                                                   |
| 780  | `Operator '{}' is not implemented for...`                       | `OperationNotSupported` | **Guarded by Linking** (Generic fallback).                                |
| 851  | `Cannot negate '{}'`                                            | `OperationNotSupported` | **Guarded by Linking**.                                                   |

### `crates/core/src/ast/operators/comparators.rs`

| Line | Code Context                                | Proposed Replacement    | Notes                                          |
|:-----|:--------------------------------------------|:------------------------|:-----------------------------------------------|
| 351  | `Cannot compare durations 1`                | `OperationNotSupported` | **Guarded by Linking**: Checks `DurationType`. |
| 357  | `Cannot compare durations 2`                | `OperationNotSupported` | **Guarded by Linking**.                        |
| 365  | `Cannot compare durations 3`                | `OperationNotSupported` | **Guarded by Linking**.                        |
| 374  | `Cannot compare durations 4`                | `OperationNotSupported` | **Guarded by Linking**.                        |
| 383  | `Cannot compare '{}' and '{}'`              | `OperationNotSupported` | **Guarded by Linking**.                        |
| 390  | `Comparator '{}' is not implemented for...` | `OperationNotSupported` | **Guarded by Linking**.                        |

### `crates/core/src/ast/functions/function_string.rs`

| Line | Code Context                              | Proposed Replacement | Notes                                         |
|:-----|:------------------------------------------|:---------------------|:----------------------------------------------|
| 281  | `.map_err(                                | e                    | RuntimeError::eval_error(e.to_string()))`     | `RegexError` | **Runtime**: Invalid pattern. |
| 378  | `.map_err(                                | e                    | RuntimeError::eval_error(e.to_string()))`     | `RegexError` | **Runtime**: Invalid pattern. |
| 409  | `RuntimeError::eval_error(e)`             | `RegexError`         | **Runtime**: Execution failure.               |
| 422  | `regex_functions feature is disabled`     | `FeatureDisabled`    | **Runtime** (unless linking checks features). |
| 443  | `RuntimeError::eval_error(e)`             | `ConversionError`    | **Runtime**.                                  |
| 454  | `base64_functions feature is disabled`    | `FeatureDisabled`    | **Runtime**.                                  |
| 463  | `RuntimeError::eval_error(e.to_string())` | `ConversionError`    | **Runtime**.                                  |
| 479  | `RuntimeError::eval_error(e)`             | `ConversionError`    | **Runtime**.                                  |
| 490  | `base64_functions feature is disabled`    | `FeatureDisabled`    | **Runtime**.                                  |
| 515  | `RuntimeError::eval_error(e.to_string())` | `RegexError`         | **Runtime**.                                  |
| 547  | `RuntimeError::eval_error(e)`             | `RegexError`         | **Runtime**.                                  |
| 560  | `regex_functions feature is disabled`     | `FeatureDisabled`    | **Runtime**.                                  |

### `crates/core/src/ast/functions/function_numeric.rs`

| Line | Code Context                                       | Proposed Replacement    | Notes                                           |
|:-----|:---------------------------------------------------|:------------------------|:------------------------------------------------|
| 629  | `Cannot compare durations of different kinds`      | `OperationNotSupported` | **Runtime**: Data-dependent (Years vs Seconds). |
| 788  | `Max is not implemented for this particular range` | `OperationNotSupported` | **Runtime**: Empty or infinite range?           |
| 828  | `Min is not implemented for this particular range` | `OperationNotSupported` | **Runtime**.                                    |

### `crates/core/src/ast/functions/function_date.rs`

| Line | Code Context         | Proposed Replacement    | Notes                                               |
|:-----|:---------------------|:------------------------|:----------------------------------------------------|
| 288  | `calendarDiff` error | `OperationNotSupported` | **Guarded by Linking**: Checks arguments are dates. |

### `crates/core/src/ast/foreach.rs`

| Line | Code Context                                | Proposed Replacement | Notes                                           |
|:-----|:--------------------------------------------|:---------------------|:------------------------------------------------|
| 87   | `RuntimeError::eval_error(err.to_string())` | `ContextError`       | **Runtime**: OOM or internal state?             |
| 233  | `Cannot iterate {type}`                     | `IterationError`     | **Guarded by Linking**: Checks collection type. |

### `crates/core/src/ast/expression.rs`

| Line | Code Context                    | Proposed Replacement | Notes                                                                         |
|:-----|:--------------------------------|:---------------------|:------------------------------------------------------------------------------|
| 400  | `Failed to evaluate field '{}'` | `InternalError`      | **Guarded by Usage**: "ObjectField evaluation is deprecated".                 |
| 417  | `Range is not a valid number`   | `ConversionError`    | **Runtime**: `ValueType::NumberType` includes floats, but range requires Int. |

### `crates/core/src/ast/operators/logical_operators.rs`

| Line | Code Context                              | Proposed Replacement    | Notes                   |
|:-----|:------------------------------------------|:------------------------|:------------------------|
| 121  | `Operator '{}' is not implemented for...` | `OperationNotSupported` | **Guarded by Linking**. |

### `crates/core/src/ast/selections.rs`

| Line | Code Context                      | Proposed Replacement    | Notes                                              |
|:-----|:----------------------------------|:------------------------|:---------------------------------------------------|
| 148  | `Cannot select a value with '{}'` | `OperationNotSupported` | **Guarded by Linking**.                            |
| 271  | `Failed to evaluate filter...`    | `InternalError`         | Wraps inner error.                                 |
| 516  | `Failed to get field '{}'`        | `ContextError`          | **Runtime** (if field missing in runtime object?). |

### `crates/wasm/src/conversion/to_js.rs`

| Line            | Code Context                         | Proposed Replacement | Notes                    |
|:----------------|:-------------------------------------|:---------------------|:-------------------------|
| 30, 36, 42, ... | `.map_err(RuntimeError::eval_error)` | `ConversionError`    | **Runtime**: JS Interop. |
| 174             | `Failed to create Date object`       | `ConversionError`    | **Runtime**.             |
| 179             | Wrapping generic error               | `ConversionError`    | **Runtime**.             |

### `crates/core/src/typesystem/errors.rs`

| Line | Code Context                  | Proposed Replacement | Notes                                               |
|:-----|:------------------------------|:---------------------|:----------------------------------------------------|
| 600  | `From<ParseErrorEnum>`        | `ParseError`         | **Runtime**: If dynamic evaluation/parsing happens. |
| 606  | `From<DuplicateNameError>`    | `ContextError`       | **Runtime**: Context building.                      |
| 616  | `From<LinkingError>` fallback | `InternalError`      | **Should not happen**.                              |

### `crates/core/src/runtime/execution_context.rs`

| Line | Code Context                  | Proposed Replacement | Notes                         |
|:-----|:------------------------------|:---------------------|:------------------------------|
| 193  | `Cannot get context variable` | `ContextError`       | **Runtime**: Missing binding. |

### `crates/core/src/link/linker.rs`

| Line | Code Context                         | Proposed Replacement | Notes                                                      |
|:-----|:-------------------------------------|:---------------------|:-----------------------------------------------------------|
| 306  | `Definition(definition) => Err(...)` | `InternalError`      | **Guarded by Structure**: Definitions shouldn't be eval'd. |