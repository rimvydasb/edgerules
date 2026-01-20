# EdgeRules Error Codes Specification

This document defines the structured error codes used within the EdgeRules engine. These codes are designed to provide
specific, stable identifiers for various error conditions, enabling better error handling in host environments and
minimizing binary size by avoiding large error strings in the WASM build.

## Internal Integrity Errors (100-499)

Internal integrity errors indicate states that should have been prevented by the Linking phase. If encountered at
runtime, they typically signal a bug in the Linker or an inconsistency in the engine's internal state.

### Math Linking Guards (100-149)

| Code | Description                                                                   |
|:-----|:------------------------------------------------------------------------------|
| 100  | Operator '^' (Power) is not implemented for the provided numeric types.       |
| 101  | Unsupported operator applied to duration values.                              |
| 102  | Unsupported operator applied to period values.                                |
| 103  | Operator is not implemented for date and duration combinations.               |
| 104  | Operator is not implemented for datetime and duration combinations.           |
| 105  | Operator is not implemented for time and duration combinations.               |
| 106  | Operator is not implemented for date and period combinations.                 |
| 107  | Operator is not implemented for datetime and period combinations.             |
| 108  | Cannot apply operator between period and duration values.                     |
| 109  | Generic fallback: Operator is not implemented for the resolved operand types. |
| 110  | Cannot negate the provided value type.                                        |

### Comparator Linking Guards (150-159)

| Code | Description                                                                |
|:-----|:---------------------------------------------------------------------------|
| 150  | Cannot compare durations (Less than) due to internal kind mismatch.        |
| 151  | Cannot compare durations (Greater than) due to internal kind mismatch.     |
| 152  | Cannot compare durations (Less or Equal) due to internal kind mismatch.    |
| 153  | Cannot compare durations (Greater or Equal) due to internal kind mismatch. |
| 154  | Comparator is not supported for period values (only equality is allowed).  |
| 155  | Generic fallback: Not possible to compare the provided operands.           |

### Logical Operator Linking Guards (160-169)

| Code | Description                                                                             |
|:-----|:----------------------------------------------------------------------------------------|
| 160  | Logical operator (and, or, xor, not) is not implemented for the provided operand types. |

### String Function Linking Guards (200-299)

| Code | Description                                                           |
|:-----|:----------------------------------------------------------------------|
| 200  | `regex_functions` feature is disabled; cannot execute `regexSplit`.   |
| 201  | `regex_functions` feature is disabled; cannot execute `regexReplace`. |
| 202  | `base64_functions` feature is disabled; cannot execute `toBase64`.    |
| 203  | `base64_functions` feature is disabled; cannot execute `fromBase64`.  |

### Date & Time Function Linking Guards (300-399)

| Code | Description                                                             |
|:-----|:------------------------------------------------------------------------|
| 300  | `calendarDiff` expects date arguments, but received incompatible types. |

### Array & Object Linking Guards (400-499)

| Code | Description                                                            |
|:-----|:-----------------------------------------------------------------------|
| 400  | Cannot iterate through the provided type (must be a list or range).    |
| 401  | Cannot select a value using the provided index/method.                 |
| 402  | Cannot perform field selection because the data type is not an object. |
| 403  | User function call failed unexpectedly during context creation.        |

## Value Parsing Errors (Runtime)

These errors occur when the engine fails to parse a specific value type from a string representation. These are
considered true runtime errors as they depend on the input data.

| Code | Description                                                        |
|:-----|:-------------------------------------------------------------------|
| 0    | Generic parsing error from string.                                 |
| 101  | Date adjustment overflowed the supported year range.               |
| 102  | Invalid month produced during `calendarDiff` calculation.          |
| 103  | Invalid date produced during `calendarDiff` calculation.           |
| 104  | Period components must be non-negative before applying the sign.   |
| 105  | Period months overflow the supported range.                        |
| 106  | Period components overflow the supported range.                    |
| 107  | Period months and days must carry the same sign.                   |
| 110  | Duration days overflow the supported range.                        |
| 111  | Duration hours overflow the supported range.                       |
| 112  | Duration minutes overflow the supported range.                     |
| 113  | Duration seconds overflow the supported range.                     |
| 114  | Duration overflow while calculating total seconds.                 |
| 115  | Duration components must be non-negative before applying the sign. |
