# Ideal Errors Story

There are various problems related to errors in EdgeRules.
Most of them already marked with `@todo` in the code.
This story gradually unfolds the ideal error handling strategy for EdgeRules.

## Introduction

1. `ParseErrors` that must fail source loading - no linking or even execution must be started. Prefixed with `[parse]`
2. `LinkingError` that must fail linking - no execution is started. Prefixed with `[link]`
3. `RuntimeError` that must fail execution. Prefixed with `[run]`

## Tasks

Example bad code:

```edgerules
{
    calculations: {
        func takeDate(d: date): { year: d.nonexistent }
        result: takeDate(date('2024-01-01')).year
    }
    value : calculations.result
}
```

This story will focus on improving `LinkingError` to provide in-structure location and expression:

**Updated GeneralStackedError:**
```json
{
  "stage": "linking",
  "error": {
    "type": "FieldNotFound",
    "fields": ["date", "nonexistent"]
  },
  "location": "calculations.takeDate.year",
  "expression": "d.nonexistent"
}
```

Currently, `LinkingError` is able to collect `context`m that is a call trace instead of in-structure location.
Old style below:

**Old (current) GeneralStackedError:**
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

Examples above are illustrative only - we're targeting Rust structures, not JSON.
Later on we will work with WASM, and we will produce JSON errors for the clients, but this
is out of scope for this story.

Investigate what is needed to build in-structure location instead of call trace context
and specify below in ERRORS_STORY.md. Produced specification will be used to implement in-structure location handling.

# In-structure location specification

**Extend GeneralStackedError**

- `location: Vec<String>` field that will represent in-structure location.
- `expression: String` field that will represent the expression that caused the error (Optional).
- `stage` field that will represent the stage of the error: `linking`, `runtime` - it can be Rust enum.

**In-structure location**

- Current call-trace contexts are built with `with_context` calls during linking. They rely on `NodeDataEnum::to_string()` which emits `Root.calculations.#child` etc. (`#child` comes from `NodeDataEnum::Internal` that is used for function bodies). The trace mixes "what we were doing" with "where this expression lives", producing the verbose context array shown above.
- To build an in-structure location we need a deterministic path inside the model (root → child → expression field). That path must not depend on where the linker recursed from, only on the owning AST nodes.
- Each `ContextObject` already knows its parent via `NodeDataEnum::get_parent()` and (for normal children) its assigned field name via `get_assigned_to_field()`. Function bodies and type objects, however, are marked as `Internal` and lose the name (`#child`), so we must carry an explicit alias for them:
  - When registering a user function (`ContextObject::add_user_function` / `FunctionDefinition`), store the function name alongside the body (e.g., in `NodeDataEnum::Internal(String, Weak<...>)` or a sibling field) so the body can report `"takeDate"` instead of `#child`.
  - Do the same for inline/internal contexts (type objects, foreach/if bodies if they use `Internal`) so every `ContextObject` can tell "what name am I under in my parent".
- With that alias in place, add a helper to compute `Vec<String>` location segments from any node:
  - Start from the failing expression’s context and field; push the field name that is being linked/evaluated (`year` in the example).
  - Walk parents via `get_parent()`, prepending each known alias/field name; stop at root. The result for the example becomes `["calculations", "takeDate", "year"]`.
  - For expressions that live in the root scope, the location is just the field being linked (`value` or similar).
- When producing a `LinkingError` (or wrapping it via `with_context`), populate the new `location` and `expression` fields instead of stacking human strings:
  - `location`: the vector built above.
  - `expression`: `Display` of the expression being linked that triggered the error (`d.nonexistent` in the example).
  - `stage`: `linking`.
- Runtime errors should follow the same shape, but build the location from the execution context (stack of `ExecutionContext` parents + current field).
- The old `context: Vec<String>` stack can be kept temporarily for debugging but should be deprecated once all call sites populate the structured fields.

**In-structure location testing**

Similar to `evaluation_common_tests.rs`, create `evaluation_linking_errors_tests.rs` that will 
contain tests that will verify that produced linking errors contain correct in-structure location and expression fields.
Similar to `link_error_contains` a required helpers can be created to verify produced errors and `location`.

Add multiple tests that will cover various location scenarios:
- Simple field access errors
- Function call errors
- Nested function calls
- Errors inside deep contexts
- Errors inside array elements
- Errors inside function bodies
- Errors inside if-else bodies
- Errors inside loop bodies
- Errors in the root scope
- Combination of the above

**Next:** do the same with runtime errors in `evaluation_runtime_errors_tests.rs`.

## Current implementation notes

- `GeneralStackedError` now stores `stage`, `location`, `expression`, and `message` alongside the underlying error enum. Linking/runtime constructors populate these fields automatically.
- `NodeDataEnum::Internal` carries an optional alias (function/type name) so in-structure paths can be reconstructed instead of `#child`.
- `link_parts` decorates linking failures with `location` and `expression` derived from the owning context/field. `context` strings remain only for legacy compatibility.
- Added `tests/evaluation_linking_errors_tests.rs` to assert locations and expressions for root, nested objects, and function-body errors.
