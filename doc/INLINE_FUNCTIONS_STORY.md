# EdgeRules Inline Functions and Return Scoping

## Goal
The goal is to enhance the EdgeRules language with two key features to improve developer experience and encapsulation capabilities:

1.  **Inline Functions**: Support for concise function definitions without requiring braces for single-expression bodies.
    *   Syntax: `func myInline(a): a + a`
    *   Behavior: Implicitly wraps the expression in a context with a `return` field.

2.  **Explicit Return Scoping**: Support for a specific `return` field in function bodies to define the exact return value, allowing internal variables to be hidden.
    *   Syntax:
        ```
        func myFunc(a): {
            internalVar: a * 2;
            return: internalVar + 1
        }
        ```
    *   Behavior: If `return` is present, only its value is returned. If not, the whole context is returned (legacy behavior).

## Analysis

### 1. Tokenizer & Parser Changes
**File:** `crates/core/src/tokenizer/parser.rs`

*   **Current State:** The tokenizer treats `return` strictly as a keyword (likely for `for .. in .. return`), which might prevent it from being used as a standard object key (field name).
*   **Required Change:**
    *   Modify the `tokenize` function.
    *   When the literal `return` is encountered, check the lookahead character.
    *   If it is followed by a colon (`:`), treat it as a valid identifier/field name.
    *   Otherwise, maintain its behavior as a reserved keyword.

### 2. AST Building (Implicit Return)
**File:** `crates/core/src/tokenizer/builder.rs` (specifically `factory::build_assignment`)

*   **Current State:** Function definitions likely expect a `StaticObject` (wrapped in `{}`) as the body.
*   **Required Change:**
    *   Update `build_assignment` where `DefinitionEnum::UserFunction` is constructed.
    *   Accept *any* expression on the right-hand side of the definition.
    *   Check if the expression is a `StaticObject`.
        *   **Yes:** Use it as the function body (existing behavior).
        *   **No:** Wrap the expression in a new `ContextObject` structure equivalent to `{ return: <expression> }`.

### 3. Runtime Evaluation (Return Scoping)
**File:** `crates/core/src/ast/user_function_call.rs`

*   **Current State:** The `eval` method for `UserFunctionCall` executes the function context and returns the resulting `Reference` (the full context object).
*   **Required Change:**
    *   Modify `eval` to inspect the evaluated context.
    *   Check if a field named `return` exists in the execution context.
    *   **If found:** Evaluate and return the value of the `return` field.
    *   **If not found:** Return the entire context object (preserving backward compatibility).

## Tasks

- [ ] **Parser Update**: Modify `crates/core/src/tokenizer/parser.rs` to allow `return` as a field name when followed by `:`.
- [ ] **Builder Update**: Modify `crates/core/src/tokenizer/builder.rs` to implement implicit `{ return: ... }` wrapping for inline function bodies.
- [ ] **Runtime Update**: Modify `crates/core/src/ast/user_function_call.rs` to implement the return value extraction logic.
- [ ] **Testing**:
    - [ ] Create `crates/core-tests/tests/inline_functions_tests.rs`.
    - [ ] Test inline function definition: `func add(a, b): a + b`.
    - [ ] Test explicit return scoping: `func calc(x): { tmp: x*2; return: tmp+1 }` (ensure `tmp` is not returned).
    - [ ] Test backward compatibility: `func old(x): { val: x }` (ensure `{ val: x }` is returned).
    - [ ] Test nested returns: `func complex(x): { return: { inner: x } }`.
- [ ] **Documentation**: Update relevant documentation to reflect the new syntax and behavior.
