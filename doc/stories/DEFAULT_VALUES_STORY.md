# EdgeRules Default Values in Types

Support for default values in type definitions for primitive types. This allows fields to be automatically populated
with a predefined value when they are missing during evaluation or casting.

**Syntax:**

```edgerules
{
    type Customer: {name: <string>, birthdate: <date>, income: <number, 0>}
    type Applicant: {customer: <Customer>, requestedAmount: <number>, termInMonths: <number, 12>}
    type LoanOffer: {eligible: <boolean, false>, amount: <number>, termInMonths: <number>, monthlyPayment: <number>}
}
```

**Supported Default Values:**

- `number`: e.g., `<number, 0>` or `<number, -500>` that maps to `NumberEnum::Int`, `<number, 100.5>` or `<number, 0.0>`
  that maps to `NumberEnum::Real`
- `string`: e.g., `<string, "N/A">`, `<string, "unknown">`
- `boolean`: e.g., `<boolean, true>`, `<boolean, false>`
- `list`: e.g., `<string[], []>`, `<number[], [1, 2, 3]>`

> Not Supported: `date`, `time`, `datetime`, `duration`, `period`, `range` types.

> Default values must be plain values - no expressions or function calls.

> Default values for function arguments are out of scope for this phase.

## Analysis

**Architecture Change:**
The `ComplexTypeRef` enum, which represents a type reference in the AST, needs to be extended to hold an optional
default value. This default value should only be supported for primitive types (`number`, `string`, `boolean`).

1. **AST Changes:**
    * Update `ComplexTypeRef` in `crates/core/src/ast/token.rs`.
    * Change `BuiltinType(ValueType)` to `BuiltinType(ValueType, Option<ValueEnum>)`.
    * Update helper methods `undefined()`, `is_undefined()`, and `from_value_type()` to handle the new structure.
    * Update `Display` implementation to output the default value if present (e.g., `<number, 0>`).

2. **Parser Changes:**
    * Update `parse_complex_type_in_angle` in `crates/core/src/tokenizer/parser.rs`.
    * After parsing the base type name, detect an optional comma followed by a literal value.
    * Supported literals: Numbers (digits), Strings (quotes), and Booleans (`true`/`false`).
    * Implement validation: ensure the default value's type matches the declared primitive type.
    * *Restriction:* Defaults for `date`, `time`, `datetime`, `duration`, and `list` types are explicitly excluded in
      this phase.

3. **Runtime & Casting Logic:**
    * **File:** `crates/core/src/ast/expression.rs`.
    * Update `cast_value_to_type` function.
    * When a field is missing in the source object during casting, check the `TypePlaceholder` in the target type
      definition.
    * If the `ComplexTypeRef` within the placeholder contains a default value, use it instead of generating a `Missing`
      special value.
    * Update `ExpressionEnum::eval` for `TypePlaceholder` to return the default value if it exists.

4. **Linking Logic:**
    * Ensure that default values do not interfere with type resolution during the linking phase. The linked type remains
      the same; only the evaluation behavior changes when a value is absent.

## Tasks

- [ ] **AST Update**: Modify `ComplexTypeRef` to support `Option<ValueEnum>` for defaults.
- [ ] **Parser Update**: Modify `parse_complex_type_in_angle` to parse and validate default values.
- [ ] **Runtime Update**: Update `cast_value_to_type` to respect default values for missing fields.
- [ ] **Evaluation Update**: Update `TypePlaceholder` evaluation to return the default value.
- [ ] **Testing Strategy** (extend `evaluation_types.rs` tests):
    - [ ] **Rust**: Verify AST parsing of various default value combinations.
    - [ ] **Rust**: Test casting behavior where missing fields are replaced by defaults.
    - [ ] **Rust**: Test nested object casting with defaults.
    - [ ] **Rust**: Ensure invalid default types (e.g., `<number, "text">`) throw a proper parse error.
- [ ] During parsing, it is important that default value matches the declared type. If there is a mismatch,
  `WrongFormat` error should be raised with a clear message. Add Rust tests to assert it.

## Portable Support

- [ ] Ensure Portable support for default values in type definitions such as:

```json
{
  "Applicant": {
    "@type": "type",
    "name": "<string,''>",
    "income": "<number,0.0>"
  },
  "processApplicant": {
    "@type": "function",
    "@parameters": {
      "app": "Applicant"
    },
    "result": "app.income"
  }
}
```

- [ ] Add tests where user changes default values in Portable type definitions and verify correct behavior during
  casting and evaluation. For example, with WASM `set` API I change the default value of `income` from `0.0` to `1000.0`
  and verify that missing `income` fields yield `1000.0` after the change.
- [ ] Add a test where I revoke the default value by setting it to nothing in Portable and verify that missing fields
  yield `Missing` again, e.g. `"income": "<number>"`.
- [ ] Check if tasks are completed and mark them as done.
- [ ] Review implemented code once again for maintainability and performance. For example, ensure that you're not using
  single letter arguments such as `if q == '"' || q == '\'' {`