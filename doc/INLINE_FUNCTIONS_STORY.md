# EdgeRules Inline Functions and Return Scoping

## Goal
The goal is to enhance the EdgeRules language with two key features to improve developer experience and encapsulation capabilities.

## Inline Function Support

### Goal
Support for concise function definitions without requiring braces for single-expression bodies.

**Syntax:**
```edgerules
func myInline(a): a + a
```

**Behavior:**
*   The function body is a single expression.
*   Introspection (via `get` methods or Portable Format) must preserve the inline nature (returning the expression, not a wrapped object).

### Analysis

**Architecture Change:**
Instead of wrapping the expression into a `StaticObject` immediately during parsing, we will introduce a dedicated definition type. This ensures that the AST preserves the "inline" semantic, allowing introspection to show the raw expression.

1.  **New Definition Type:**
    *   Introduce `InlineFunctionDefinition` struct containing `name`, `arguments`, and `body` (of type `ExpressionEnum`).
    *   Add `InlineUserFunction(InlineFunctionDefinition)` to `DefinitionEnum` in `crates/core/src/ast/token.rs`.

2.  **Parser Changes:**
    *   **File:** `crates/core/src/tokenizer/builder.rs` (factory).
    *   Update `build_assignment` to detect when a function definition has a non-object body.
    *   If the body is not a `StaticObject`, construct an `InlineUserFunction` definition instead of wrapping it.

3.  **Trait Implementation:**
    *   Implement `UserFunction` trait for `InlineFunctionDefinition`.
    *   `create_context`: This method will perform the normalization for execution. It should create a temporary `ContextObject` containing the expression (implicitly assigned to `return` or a reserved field) so that the existing `FunctionContext` machinery can execute it.

### Tasks
- [ ] **AST Update**: Add `InlineFunctionDefinition` struct and `DefinitionEnum::InlineUserFunction`.
- [ ] **Parser Update**: Modify `build_assignment` in `tokenizer/builder.rs` to produce `InlineUserFunction` for inline bodies.
- [ ] **Introspection Testing**: Create WASM/Node.js tests to verify that `getType` on an inline function returns the expression structure, not a wrapper object.

## Optional return Body

### Goal
Support for a specific `return` field in function bodies to define the exact return value, allowing internal variables to be hidden.

**Syntax:**
```edgerules
func myFunc(a): {
    internalVar: a * 2;
    return: internalVar + 1
}
```

**Behavior:**
If `return` is present in the evaluated function context, only its value is returned. If not, the whole context is returned (legacy behavior).

### Analysis

1.  **Parser Changes:**
    *   **File:** `crates/core/src/tokenizer/parser.rs`.
    *   The literal `return` is currently a reserved keyword.
    *   Update `tokenize` to allow `return` to be treated as a field name if followed by a colon (`:`).

2.  **Runtime Evaluation:**
    *   **File:** `crates/core/src/ast/user_function_call.rs`.
    *   Update `UserFunctionCall::eval`.
    *   After evaluating the function context, inspect the result.
    *   Check for a field named `return`.
    *   **If found:** Extract and return its value.
    *   **If not found:** Return the full context object.
    *   *Note:* This logic will naturally handle `InlineUserFunction` as well, assuming its `create_context` wraps the expression in a `return` field.

3.  **Linking Logic:**
    *   **File:** `crates/core/src/ast/user_function_call.rs`.
    *   Update `UserFunctionCall::link`.
    *   The current logic likely returns the full `ObjectType` of the function body.
    *   The linker must inspect the function definition's return type (the `ObjectType` of the body).
    *   **Check:** Does the body's `ContextObject` contain a field named `return`?
    *   **If yes:** The return type of the function call is the type of that `return` field.
    *   **If no:** The return type is the `ObjectType` of the body itself (legacy behavior).

### Tasks
- [ ] **Parser Update**: Modify `parser.rs` to allow `return:` as a field key.
- [ ] **Runtime Update**: Modify `UserFunctionCall::eval` to implement the return value extraction logic.
- [ ] **Linking Update**: Modify `UserFunctionCall::link` to resolve the return type based on the presence of the `return` field in the function body context.
- [ ] **Core Testing**:
    - [ ] Test explicit return scoping (hiding internal vars).
    - [ ] Test backward compatibility (returning full objects).
    - [ ] Test nested returns.
