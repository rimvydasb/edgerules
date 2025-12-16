# Print Story

`impl Display for Lines` already provides a printing back to EdgeRules syntax.
`serialize_` functions in WASM module already know how to return JSON portable format.
We need to create `to_js` trait similar to `Display` to convert EdgeRules to JavaScript objects.
To JavaScript feature must be enabled or disabled via Cargo features, because it is expected it will take 
a lot of code size. Exposed JavaScript must support both Node.js and browser environments.

## Implementation

### Phase 1

- Create a separate workspace in `crates` called `edge-js`
- The workspace will be baked in wasm and cli only if the `to_js` feature is enabled.
- Define a `ToJs` trait similar to `Display` trait. `ToJs` should emit executable JavaScript source strings.
- Use the latest ECMAScript features where applicable (e.g., `const`, `let`, arrow functions, template literals).
- Use the latest Node.js or browser APIs (no backward compatibility required).

### Phase 2

- Implement `ToJs` trait for all relevant AST nodes in the core crate.
- Ensure that complex structures (like functions, objects, arrays) are correctly represented in JavaScript.
- Handle edge cases such as:
  - Nested structures
  - Special characters in strings

### Phase 3

- All built-in functions and standard library components should have implemented as helpers in JavaScript.
- As for now, simply add `builtins.js` file as a header for all tests in `tests/wasm-js` folder.
Later on we will think how to bundle them properly with WASM or deliverable if needed.
- Develop tests in `tests/wasm-js` to validate the correctness of the JavaScript output.
Feel free to add additional helpers or utilities as needed to `builtins.js` to make tests pass.
- Simply include `builtins.js` where possible to make returned JavaScript executable - later we will refine that part.

### Phase 4

- The goal is to enable printing all EdgeRules objects to JavaScript objects and make them executable
same as EdgeRules are executable in EdgeRules runtime.
- Review all tests and check if all cases are covered and no boilerplate code is present.
- Review the implementation and ensure correctness and eliminate warnings.
- Do not fix old bugs in `crates/core` or `crates/core-tests` unless they block the implementation. Ideally you should
not change any code outside of `crates/edge-js` and `tests/wasm-js` folders.

## Testing

- Add unit tests for `to_js` implementations for all relevant AST nodes.
- Add integration tests in `tests/wasm-js` where printed JavaScript objects are evaluated to ensure correctness.

## Detected Issues

Print eny detected issues during the implementation here