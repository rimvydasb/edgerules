# Parser Review Specification

This is planning phase. Do not code.

- [x] Scan `builder.rs` and `parser.rs` for all `@Todo` comments.
- [x] Investigate those comments as a whole to identify any common themes and required refactoring. Find out if proposed
  refactoring is feasible and beneficial.
- [x] Maybe we need to clarify proposals and better align them with project priorities: first priority WASM size, second
  priority performance, third priority code maintainability.
- [x] Write a summary of findings and proposed actions in this PARSER_REVIEW_SPEC.md.

## Findings

The scan of `crates/core/src/tokenizer` revealed several key areas for improvement, primarily focused on safety and code
maintainability.

### 1. Safety and Stability (High Priority)

Several TODOs point to missing checks for recursion limits and nesting levels.

- `parser.rs`: Need to implement a check for `max level (255)` to prevent stack overflows.
- `parser.rs`: Need to ensure `level` is calculated correctly and all brackets are closed.
- `builder.rs`: Infinite loop prevention checks are mentioned.

### 2. Code Quality and Refactoring (Medium Priority)

- **Splitting Logic:** `build_assignment` is identified as a candidate for splitting into `build_assignment`,
  `build_type_definition_assignment`, and `build_function_assignment`. This would improve readability.
- **Deprecation:** `push_element` in `builder.rs` is marked for deprecation but is currently used in `parser.rs`.
- **Clarity:** Logic in `build_math_operator` (handling `Unparsed(_left)`) is marked as "absolutely not clear" and needs
  investigation/rewrite.

### 3. Error Handling (Low Priority for WASM Size, Medium for DX)

- Comments suggest a need for better error accumulation instead of returning the first error or pushing errors back into
  the chain.
- However, for the primary goal of **Small WASM Size**, complex error accumulation might add unnecessary bloat. We
  should implement this only if it doesn't significantly impact size.

## Next Steps

### Phase 1: Safety & Clean-up (Immediate)

- [ ] **Implement Max Nesting Level Check**: Enforce a hard limit (e.g., 255) on recursion/nesting depth in `parser.rs`
  and `builder.rs` to prevent stack overflow.
- [ ] Write additional Rust tests that provoke deep nesting to ensure the limit is respected. Write tests with wrong
  bracket sequences to ensure proper error handling.
- [ ] **Verify Bracket Balancing**: Ensure the parser correctly errors out on unbalanced brackets.
- [ ] **Review `push_element` usage**: Investigate all usages of `push_element` and replace/remove it if feasible to
  proceed with deprecation.

### Phase 2: Refactoring (Secondary)

- [ ] **Refactor `build_assignment`**: Extract `build_type_definition_assignment` from `build_assignment` in
  `builder.rs`.
- [ ] **Clarify `build_math_operator`**: Analyze the `Unparsed(_left)` logic in `builder.rs` L853 and rewrite it for
  clarity and correctness.

### Phase 3: Literal review

- [ ] Review `Literal(Cow<'static, str>)` usage - it could be that WASM size and performance can be improved by simply
  having additional enumerations instead of using Literal + Cow for everything, for example maybe we can have
  `Unparsed::RangeToken` instead of `Unparsed(Literal(RANGE_LITERAL.into()))`