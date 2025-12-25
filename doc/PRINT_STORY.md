# Print Story

`impl Display for Lines` already provides a printing back to EdgeRules syntax.
`serialize_` functions in WASM module already know how to return JSON portable format.
We need to create `to_js` trait similar to `Display` to convert EdgeRules to JavaScript objects.
To JavaScript feature must be enabled or disabled via Cargo features, because it is expected it will take 
a lot of code size. Exposed JavaScript must support both Node.js and browser environments.

## Implementation

### Phase 1

- [x] Create a separate workspace in `crates` called `edge-js`
- [x] The workspace will be baked in wasm and cli only if the `to_js` feature is enabled.
- [x] Define a `ToJs` trait similar to `Display` trait. `ToJs` should emit executable JavaScript source strings.
- [x] Use the latest ECMAScript features where applicable (e.g., `const`, `let`, arrow functions, template literals).
- [x] Use the latest Node.js or browser APIs (no backward compatibility required).

### Phase 2

- [x] Implement `ToJs` trait for all relevant AST nodes in the core crate.
- [x] Ensure that complex structures (like functions, objects, arrays) are correctly represented in JavaScript.
- [x] Handle edge cases such as:
  - Nested structures
  - Special characters in strings

### Phase 3

- [x] All built-in functions and standard library components should have implemented as helpers in JavaScript.
- [x] As for now, simply add `builtins.js` file as a header for all tests in `tests/wasm-js` folder.
Later on we will think how to bundle them properly with WASM or deliverable if needed.
- [x] Develop tests in `tests/wasm-js` to validate the correctness of the JavaScript output.
Feel free to add additional helpers or utilities as needed to `builtins.js` to make tests pass.
- [x] Simply include `builtins.js` where possible to make returned JavaScript executable - later we will
      refine that part.

### Phase 4

- [x] The goal is to enable printing all EdgeRules objects to JavaScript objects and make them executable
same as EdgeRules are executable in EdgeRules runtime.
- [x] Review all tests and check if all cases are covered and no boilerplate code is present.
- [x] Review the implementation and ensure correctness and eliminate warnings.
- [ ] Do not fix old bugs in `crates/core` or `crates/core-tests` unless they block the implementation.
      Ideally you should not change any code outside of `crates/edge-js` and `tests/wasm-js` folders.
- [x] Check `performance-decision-service.mjs` decision service definition. Make sure this decision service
definition can be printed to JavaScript objects and metrods are executed correctly. Implement any missing
functionality in `ToJs` trait as needed. Do this test in `tests/wasm-js/decision-service.js`.

## Testing

- [x] Add unit tests for `to_js` implementations for all relevant AST nodes.
    - [x] Add integration tests in `tests/wasm-js` where printed JavaScript objects are evaluated to ensure correctness.
- [x] Add a separate `just node-js` that builds and tests WASM with `to_js` feature enabled.

## Detected Issues

- `ToJs` rendering still defaults to stringification for some AST variants; coverage needs to be expanded.
- `tests/wasm-js` harness exists with a shared `builtins.js`, but coverage is still thin.
