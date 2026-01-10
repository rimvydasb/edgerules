# Errors Improvement

A current state is a bit messy and requires heavy refactoring:

```typescript
export interface PortableError {
    message?: string; // formatted message (might be deprecated)
    error: {
        type: string; // `FieldNotFound`, `CyclicReference`, `TypesNotCompatible`, etc.
        fields?: string[]; // for now only `FieldNotFound` and `CyclicReference` uses it (TBC, must be ordered map of key and string)
        subject?: string; // Used by `DifferentTypesDetected` and `TypesNotCompatible`, and `FieldNotFound` (deprecated, must be in fields)
        unexpected?: string; // used only for `TypesNotCompatible` (deprecated)
        expected?: string[]; // used only for `TypesNotCompatible` (deprecated)
        message?: string; // raw error message without formatting, now only `EvalError` uses it, @TBC
    };
    location: string; // Fully qualified path of the problem, e.g. "calculations.takeDate.year"
    expression: string; // Problematic expression snippet, e.g.  "d.nonexistent"
    stage: 'linking' | 'runtime';
}
```

## Next Steps

### Phase 1

- [x] Find all `RuntimeError` that throw errors regarding arguments count, types, etc. in built-in functions. These
  errors are not necessary, because
  each built-in function already has its own guard in `validation` field that prevents wrong arguments or types from
  being passed. Do following:
    - [x] Remove all redundant `RuntimeError` that duplicate validation checks already present in built-in functions.
    - [x] Ensure that all built-in functions have proper validation in `validation` field: add if missing.
    - [x] Stay with principle that `RuntimeError` should be used only for errors that cannot be detected during
      validation phase.
    - [x] Check all built-in function Rust tests and ensure that validation errors are properly caught during validation
      phase, not runtime. Add tests if missing.
    - [x] Run rust and just node tests to ensure nothing is broken (before the implementation, all tests passed).
    - [x] Check box if task is done.
    - [x] Check how built-in functions are tested. Ensure that all argument count or other built-in function validation
      tests are present and are able to catch errors in linking and not the execution. Add tests if missing.
      Ensure every built-in function validation is covered by Rust test.
- [x] Start using `ValueParsingError(ValueType, ValueType)` instead of
  `return RuntimeError::eval_error("Invalid duration string".to_string()).into();` where problems are related to value
  parsing. Find out all places where eval_error is used for value parsing problems and replace with `ValueParsingError`.
  Make sure all tests still pass. For now use canonic message for all type parsing problems in `impl Display for RuntimeErrorEnum`.
  Align tests to match canonic message.
- [ ] Eliminate all places where runtime errors are produced and the discarded with `.ok()` or similar approaches.
Start using error code in `ValueParsingError(ValueType, ValueType, u8)` - default error code for normal conversion problem is 0,
but use other error codes for `RuntimeError::eval_error("Date adjustment overflowed year range".to_string())` or
`RuntimeError::eval_error("Invalid month produced during calendarDiff".to_string())` and all similar cases. I do not need messages,
simply use proper error code and update the list for me to know under `/// ValueParsingError error codes:` - date related errors
may start after 100. If code exists and is not 0, then attach code number to the formatted error message, e.g.:
`"Failed to parse 'date' from 'string'. (Error code: 101)"`.
- [ ] Eliminate all message formatting's in all error enums and places where errors are created.

### Phase 2

- [ ] PortableError `fields` must be renamed to `args` and must be presented as ordered structure. `args` should now
  contain:

```typescript
var exceptionObject = {
    "error": {
        // type will be mapped to the exact message string in glue code based on localization
        "type": "FieldNotFound",

        // fields will be used in the message formatting
        "args": {
            "object": "d",
            "field": "nonexistent",
            // subject, unexpected, expected, etc. must be moved here as well
        }
    },
    "location": "calculations.takeDate.year",
    "expression": "d.nonexistent",
    "stage": "linking"
}
```

- [ ] Deprecate `message` that held formatted message. Messages are now formatted in glue code based on `type` and
  `args`.
- [ ] Deprecate `error.message` that held raw error message. Raw messages are not needed anymore as all data is in
  `type` and `args`.
- [ ] Deprecate `error.subject`, `error.unexpected`, `error.expected` as they are moved to `args`.
- [ ] **FINALLY**: rename `error` to `message` in PortableError, so it will be:

```typescript
var exceptionObject = {
    "message": {
        "type": "FieldNotFound",
        "args": {
            "object": "d",
            "field": "nonexistent",
        }
    },
    // location, expression, stage...
}
```