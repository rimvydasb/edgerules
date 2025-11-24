# Ideal Errors Story

There are various problems related to errors in EdgeRules.
Most of them already marked with `@todo` in the code.
This story gradually unfolds the ideal error handling strategy for EdgeRules.

## Introduction

1. `ParseErrors` that must fail source loading - no linking or even execution must be started. Prefixed with `[parse]`
2. `LinkingError` that must fail linking - no execution is started. Prefixed with `[link]`
3. `RuntimeError` that must fail execution. Prefixed with `[run]`

## Tasks

Example error:

```edgerules
{
    calculations: {
        func takeDate(d: date): { year: d.nonexistent }
        result: takeDate(date('2024-01-01')).year
    }
    value : calculations.result
}
```

Will produce linking error:

```json
{
  "stage": "linking",
  "error": {
    "type": "FieldNotFound",
    "data": ["date", "nonexistent"]
  },
  "message": "Field 'nonexistent' not found in date",
  "location": "calculations.takeDate.year",
  "expression": "d.nonexistent"
}
```

Currently, `LinkingError` is able to collect `context`m that is a call trace instead of in-structure location:

```json
{
  "error": {
    "type": "FieldNotFound",
    "fields": ["date", "nonexistent"]
  },
  "context": [
    "Error in: `Root.calculations.#child`\nTrace: `d.nonexistent`",
    "++ while linking expression field Root.calculations.#child.year",
    "Error in: `Root.calculations`\nTrace: `takeDate(date('2024-01-01'))`",
    "While looking at source 'calculations'",
    "Error in: `Root.calculations`\nTrace: `takeDate(date('2024-01-01')).year`",
    "Error in: `Root`\nTrace: `calculations.result`",
    "++ while linking expression field Root.value"
  ]
}
```

Investigate what is needed to build in-structure location instead of call trace context
and specify below. Produced specification will be used to implement in-structure location handling.

# In-structure location specification

**Extend GeneralStackedError**

- `location: Vec<String>` field that will represent in-structure location.
- `expression: String` field that will represent the expression that caused the error (Optional).
- `stage` field that will represent the stage of the error: `linking`, `runtime`.
- `message` that is Display representation of the error enum.

**In-structure location**