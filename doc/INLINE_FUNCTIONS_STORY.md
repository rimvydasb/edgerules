# EdgeRules Inline Functions and Return Scoping

## Inline Function Support

Support for concise function definitions without requiring braces for single-expression bodies.

**Syntax:**

```edgerules
{
    func myInline(a): a + a
    myResult: myInline(2)
}
```

**outputs**

```json
{
  "myResult": 4
}
```

### Analysis

**Architecture Change:**
Instead of wrapping the expression into a `StaticObject` immediately during parsing, we will introduce a dedicated
definition type. This ensures that the AST preserves the "inline" semantic, allowing introspection to show the raw
expression.

1. **New Definition Type:**
    * Introduce `InlineFunctionDefinition` struct containing `name`, `arguments`, and `body` (of type `ExpressionEnum`).
    * Add `InlineUserFunction(InlineFunctionDefinition)` to `DefinitionEnum` in `crates/core/src/ast/token.rs`.

2. **Parser Changes:**
    * **File:** `crates/core/src/tokenizer/builder.rs` (factory).
    * Update `build_assignment` to detect when a function definition has a non-object body.
    * If the body is not a `StaticObject`, construct an `InlineUserFunction` definition instead of wrapping it.

3. **Trait Implementation:**
    * Implement `UserFunction` trait for `InlineFunctionDefinition`.
    * `create_context`: This method will perform the normalization for execution. It should create a temporary
      `ContextObject` containing the expression (implicitly assigned to `return` or a reserved field) so that the
      existing `FunctionContext` machinery can execute it.

4. **Portable API & Serialization:**
    * **Constraint**: `PortableFunctionDefinition` structure (JSON) cannot be arbitrarily changed.
    * **Export (Serialization)**: When `InlineFunctionDefinition` is serialized to Portable Format, it must be *
      *expanded** to a structure compliant with the Portable schema (more info: EDGE_RULES_API_SPEC.md)
        * The mapping is: `InlineUserFunction(expr)` -> `PortableFunctionDefinition` where the body contains a key
          `return` mapping to `expr`.
        * Example: `func f(a): a+a` -> `{ "f": { "@type": "function", "@parameters": {"a": null}, "return": "a+a" } }`.
    * **Import (Deserialization)**: When loading from Portable Format, we optimize for the internal AST representation.
        * If a function body contains **only** a `return` field (and no other fields), it should be deserialized into an
          `InlineUserFunction` (collapsed representation).
        * If it contains other fields, it remains a standard `UserFunction`.

### Tasks

- [ ] **AST Update**: Add `InlineFunctionDefinition` struct and `DefinitionEnum::InlineUserFunction`.
- [ ] **Parser Update**: Modify `build_assignment` in `tokenizer/builder.rs` to produce `InlineUserFunction` for inline
  bodies.
- [ ] **Serialization Update**: Implement `to_portable` for `InlineUserFunction` (expanding to `{return: ...}`) and
  update `from_portable` to detect return-only bodies (collapsing to `InlineUserFunction`).
- [ ] **Testing Strategy**:
    - [ ] **Rust**: Verify AST construction for inline syntax.
    - [ ] **Rust**: Verify execution of inline functions (correct wrapping).
    - [ ] **WASM/JS**: Test Round-trip Serialization: `Inline -> Portable (expanded) -> Inline`.
    - [ ] **WASM/JS**: Test that `getFunction` returns the expanded Portable definition for inline functions (Rule: API
      returns Portable structure).
    - [ ] **WASM/JS**: Test that importing a "return-only" Portable definition behaves as an inline function.
    - [ ] **Rust**: Add nested execution tests as below:

```edgerules
{
    func addOne(x): x + 1
    func doubleAndAddOne(y): addOne(y * 2)
    result: doubleAndAddOne(3)  // Expected: 7
}
```

- [ ] Check tasks if completed.
- [ ] Explorer edge cases and check if all tests pass.

**Edge cases to consider:**

1. Nested inline functions.
2. Inline functions used for arguments to call other functions: `func applyTwice(x): addOne(addOne(x))`
3. Inline functions with no arguments.

## Optional return Body

Support for a specific `return` field in function bodies to define the exact return value, allowing internal variables
to be hidden.

**Syntax:**

```edgerules
func myFunc(a): {
    internalVar: a * 2;
    return: internalVar + 1
}
```

**Behavior:**
If `return` is present in the evaluated function context, only its value is returned. If not, the whole context is
returned. User defined functions with return or without remain fully compatible and actual.

### Analysis

1. **Parser Changes:**
    * **File:** `crates/core/src/tokenizer/parser.rs`.
    * The literal `return` is currently a reserved keyword.
    * Update `tokenize` to allow `return` to be treated as a field name if followed by a colon (`:`).

2. **Runtime Evaluation:**
    * **File:** `crates/core/src/ast/user_function_call.rs`.
    * Update `UserFunctionCall::eval`.
    * After evaluating the function context, inspect the result.
    * Check for a field named `return`.
    * **If found:** Extract and return its value.
    * **If not found:** Return the full context object.
    * *Note:* This logic will naturally handle `InlineUserFunction` as well, assuming its `create_context` wraps the
      expression in a `return` field.

3. **Linking Logic:**
    * **File:** `crates/core/src/ast/user_function_call.rs`.
    * Update `UserFunctionCall::link`.
    * The linker must inspect the function definition's return type (the `ObjectType` of the body).
    * **Check:** Does the body's `ContextObject` contain a field named `return`?
    * **If yes:** The return type of the function call is the type of that `return` field.
    * **If no:** The return type is the `ObjectType` of the body itself (legacy behavior).

### Tasks

- [ ] **Parser Update**: Modify `parser.rs` to allow `return:` as a field key.
- [ ] **Runtime Update**: Modify `UserFunctionCall::eval` to implement the return value extraction logic.
- [ ] **Linking Update**: Modify `UserFunctionCall::link` to resolve the return type based on the presence of the
  `return` field in the function body context.
- [ ] **Core Testing**:
    - [ ] Test explicit return scoping (hiding internal vars).
    - [ ] Test backward compatibility (returning full objects).
    - [ ] Test nested returns.
- [ ] If function has only `return` field, ensure it will collapse to `InlineUserFunction` during parsing and AST
  building. You must test it in Rust with `obj.borrow().to_string()` where `obj` is `ContextObject`. This to string
  method prints the whole function body so: `func f(a): { return: a + a }` must print as `func f(a): a + a`.