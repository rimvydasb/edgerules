# Repository Guidelines

## Project Naming

- Public name: `EdgeRules`
- Crates (Cargo): edge-rules, edge-rules-wasi, edge-rules-cli
- Rust imports: edge_rules, edge_rules_wasi
- CLI binary: edgerules (with alias er)
- WASM file: edgerules-wasi.wasm

## Description

A lightweight, embeddable rules engine for edge environments, supporting a custom DSL for defining rules and conditions.

## Naming Conventions

- Cargo / crates.io → kebab-case (Rust convention).
- In-code imports → snake_case (automatic).
- User-facing binaries / artifacts → no hyphen (clean UX).
- 4 spaces for indentation; Rust 2021 edition.
- Line length: 120 chars

## Project Structure & Module Organization

- Core library: `src/` with modules like `ast/`, `tokenizer/`, `runtime/`, `typesystem/`, and `link/`. Entry points:
  `src/lib.rs`, WASM glue: `src/wasm.rs`.
- CLI (WASI): `src/bin/edgerules-wasi.rs`.
- Tests: unit tests co-located under `#[cfg(test)]`; data-driven fixtures in `tests/` (e.g., `tests/invalid/*.txt`,
  `tests/errors/*.txt`).
- Demos: `examples/js` and `examples/web` for Node/Web usage.
- Build config: `Cargo.toml` (crate `edge_rules`), `package.json` for WASM demos.

## Build, Test, and Development Commands

- `just ensure`: Ensures targets and tools exist (wasm-pack, wasm-opt, wasmtime).
- `just web`: Build WASM for web into `target/pkg-web/` and optimize (prints sizes).
- `just node`: Build WASM for Node into `target/pkg-node/` and optimize (prints sizes).
- `just web-debug`: Build WASM for web with panic hook enabled into `target/pkg-web-debug/` and optimize.
- `just node-debug`: Build WASM for Node with panic hook enabled into `target/pkg-node-debug/` and optimize.
- `just wasi`: Build the WASI binary `edgerules-wasi.wasm` and run demo (prints size).
- `just cli`: Build the native CLI `edgerules`, print its size, and run a quick arithmetic check.
- `just core`: Build core library for wasm32-unknown-unknown.
- `just core-opt`: Minify core output to .min.wasm.
- `just demo-web`: Serve at http://localhost:8080 (expects `target/pkg-web/`).
- `just performance-node`: Run Node performance benchmarks (expects `target/pkg-node/`).
- `just demo-wasi`: Run WASI demo via wasmtime.
- `just wasm-test`: Run WASM tests in Node.

### Daily Workflow Checklist

Follow this loop for every change:

1. `cargo fmt`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. Reproduce the scenario you are touching:
   - Rust: `cargo test <suite>` / `cargo test <path>::<name>`
   - WASM demos: rebuild first (`just node` or `just web`), then run `just performance-node` / `just demo-web`
   - WASM tests: `just wasm-test`
4. If something fails, use the **Debugging & Verification Playbook** below before guessing.

## Coding Style & Naming Conventions

- Rust 2021 edition; 4 spaces; keep modules small and cohesive.
- Avoid single letter ot two letter variable names.
- Avoid unclear abbreviations, instead use: `cfg` → `config`, `ctx` → `context`.
- Avoid casting with "as", instead use ::from - if "from" does not exist, implement it.
- Avoid long namespaces; use `use` to import types/functions.
- Use `?` for error propagation; avoid `unwrap()`/`expect()` in runtime paths except test code.
- Naming: modules/files `snake_case`; types/enums `CamelCase`; functions/fields `snake_case`.
- Formatting: run `cargo fmt` before commits; keep `clippy` clean.
- WASM features: `wasm` is the lean baseline; `wasm_debug` enables `console_error_panic_hook` for better panic traces in dev. Use `web-debug`/`node-debug` to build debug artifacts in separate folders to avoid shipping debug hooks.
- Treat clippy warnings as hard errors—keep builds clean by default.

## Code Review Guidelines

The project goal is small WASM size first, performance second.
When reviewing code, consider the following:
- Check lifetime clarity
- Detect boilerplate that can be abstracted
- Ensure error handling is idiomatic
- Check maintainability and readability
- Check the ownership model
- Remove unnecessary derive annotations `#[derive(...)]` that bloat binary size
- Check if `mut` can be avoided
- Ensure no unused imports or needless borrows exist (clippy clean)

## Debugging & Verification Playbook

- Run the equivalent Rust test whenever a WASM demo breaks; most demos mirror helpers in `tests/`.
- When list or context fields report `Unlinked`, inspect `src/ast/sequence.rs` and related linking helpers to ensure inline objects are linked with `linker::link_parts`.
- Use `tests/utilities.rs` helpers (`assert_eval_all`, `link_error_contains`, etc.) so expectations stay centralized and easy to diff.
- Refresh `target/pkg-*` before demos: `just node` or `just web` rebuilds the WASM bundle; demos reuse those folders.
- Shape conversions live in `src/wasm/wasm_convert.rs`; check there when JS output diverges from Rust results.

## WASM Bridge Checklist

1. Confirm the snippet passes `cargo test` (see **Reference Tests** below).
2. Rebuild the target package (`just node`, `just web`).
3. Ensure `CollectionExpression::link` handles nested static objects (arrays of contexts).
4. Verify conversions in `wasm_convert.rs` match runtime expectations.
5. Keep `src/wasm.rs` imports minimal to avoid `wasm-pack` warnings.

## Testing Guidelines

- Prefer unit tests near the code (`mod tests`), with focused cases.
- Use `tests/` for integration-style, text-based fixtures; mirror feature areas (e.g., `tests/errors/*.txt`).
- Run `cargo test` locally; add tests for new behavior and bug fixes.

## Main Considerations

- Project priority is small WASM size
- Second priority is performance

## Common Pitfalls

- Empty arrays and inline objects must be linked—update collection linking when you touch `CollectionExpression`.
- Running `just demo-*` without re-running `just node` / `just web` uses stale WASM artifacts.
- Clippy runs with `-D warnings`; unused imports or needless borrows must be resolved immediately.

## Reference Tests

- Array casting & special values: `tests/evaluation_user_types.rs::complex_type_array_function_argument_v2`
- Nested typed arguments: `tests/evaluation_user_types.rs::complex_nested_types_in_function_argument`
- User function behaviour: `tests/evaluation_user_functions_tests.rs`
- Tokenizer fuzz cases mirroring demos: `src/tokenizer.rs::test_fuzzy_code`

# Quality Assurance

It is possible to generate solutions to the problematic problems captured in complex examples.
Run the command below and investigate [EXAMPLES-output.md](tests/EXAMPLES-output.md)
to see results.

### Optional Function Groups for WASM

Enable for Web/Node via Just tasks:

- `ENABLE_REGEX=1 just web` to link regex functions and bake into package.
- `ENABLE_BASE64=1 just node` to link base64 functions and bake into package.
- Both: `ENABLE_REGEX=1 ENABLE_BASE64=1 just web`.

Or with Cargo directly:

- `cargo build --target wasm32-unknown-unknown --no-default-features --features "wasm,regex_functions,base64_functions"`

```bash
cargo run --bin generate-examples
```
