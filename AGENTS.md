# EdgeRules Project:

This document provides a comprehensive guide for AI agents to understand, build, and contribute to the EdgeRules
project. It serves as the single source of truth for project structure, commands, and coding standards.

## Main Instructions

- **Format:** Always use Markdown for documents and documentation.
- **Line Length:** 120 characters maximum.
- **Indentation:** 4 spaces (no tabs).

## Project Overview

EdgeRules is a lightweight, embeddable, JSON-native business rules engine written in Rust. It is designed for
performance and safety, with a strong focus on small WASM output sizes for edge computing environments. The engine
features a custom DSL for defining rules, static typing, and aims for full DMN FEEL coverage.

### Key Artifacts

- **Public Name:** `EdgeRules`
- **Crates (Cargo):**
    - `edge-rules` (Core library)
    - `edge-rules-wasi` (WASM bindings)
    - `edge-rules-cli` (Command Line Interface)
- **Rust Imports:** `edge_rules`, `edge_rules_wasi`, `edge_rules_cli`
- **CLI Binary:** `edgerules` (alias `er`)
- **WASI Binary:** `edgerules-wasi`

## Project Structure

The project is structured as a Rust workspace with the following layout:

```text
/
├── crates/
│   ├── core/           # Core logic (AST, tokenizer, runtime, type system)
│   │   ├── src/lib.rs  # Main entry point
│   │   └── src/wasm.rs # Core WASM glue (minimal)
│   ├── cli/            # Command-line interface
│   │   ├── src/lib.rs  # Shared CLI logic
│   │   └── src/bin/    # Binary entry points (edgerules, er, edgerules-wasi)
│   ├── wasm/           # WASM bindings (wasm-bindgen) for Web/Node
│   │   └── src/lib.rs  # Public API for WASM module
│   └── core-tests/     # Comprehensive test suite for the core crate
│       └── tests/      # Integration tests and data-driven fixtures (*.txt)
├── examples/           # Demo projects
│   ├── js/             # Node.js examples
│   └── web/            # Web browser examples
├── tests/              # WASM integration tests (*.mjs)
├── Justfile            # Task runner configuration (build, test, deploy)
└── Cargo.toml          # Workspace configuration
```

## Build, Test, and Development Commands

Use `just` to run common tasks. Do not memorize long `cargo` or `wasm-pack` commands; use the definitions below.

### Setup

- `just ensure`: Ensures tools exist (wasm-pack, wasm-opt, wasmtime) and creates output directories.

### Building

- **Web (WASM):** `just web` (Builds to `target/pkg-web`, optimized)
- **Node.js (WASM):** `just node` (Builds to `target/pkg-node`, optimized)
- **WASI:** `just wasi` (Builds `edgerules-wasi.wasm` to `target/wasm32-wasip1/release/`)
- **CLI (Native):** `just cli` (Builds `edgerules` binary)
- **Debug Builds:** `just web-debug` or `just node-debug` (Includes `console_error_panic_hook` for debugging; do not
  ship).

### Testing

- **Rust Tests:** `just test` (Runs `cargo test --all`)
- **WASM Tests:** `just wasm-test` (Runs Node.js tests in `tests/wasm/*.mjs`)
- **Linters:** `just clippy` (Runs `cargo clippy --all-targets -- -D warnings`)
- **Formatting:** `just fmt` (Runs `cargo fmt --all`)

### Demos & Performance

- **Web Demo:** `just demo-web` (Serves `target/pkg-web` at localhost:8080)
- **Basic Performance:** `just performance-basic` (Runs basic WASM benchmarks)
- **Decision Service Performance:** `just performance-ds` (Runs decision service benchmarks)
- **WASI Demo:** `just demo-wasi` (Runs WASI binary via `wasmtime`)

## Daily Workflow Checklist

Follow this loop for every change to ensure quality and prevent regressions:

1. **Format:** `just fmt`
2. **Lint:** `just clippy` (Treat warnings as errors).
3. **Test:**
    - Run Rust tests: `just test`
    - If touching WASM logic: `just node` then `just wasm-test`
4. **Verify:** If fixing a bug, create a reproduction case in `crates/core-tests/tests/`.

## Coding Standards & Conventions

### Style

- **Rust Edition:** 2021
- **Indentation:** 4 spaces
- **Naming:**
    - Crates: `kebab-case`
    - Modules/Files: `snake_case`
    - Types/Enums: `CamelCase`
    - Functions/Variables: `snake_case`
- **Variable Names:** Descriptive and meaningful. Avoid single-letter names (use `index` not `i`, `context` not `ctx`
  unless standard idiom).
- **Imports:** Group imports cleanly. Avoid long module paths in code; use `use` statements.

### Architecture & performance

- **WASM Size First:** The primary goal is small WASM binary size. Performance is second.
    - Avoid unnecessary `#[derive(...)]` if not strictly needed.
    - Be mindful of generic monomorphization bloat.
    - Do not use `Debug` or `Display` derives for WASM target.
    - Regex and base64 are not included in WASM by default and are provided by the host environment.
- **Error Handling:** Use `?` for propagation. Avoid `unwrap()`/`expect()` in runtime code; reserve them for tests.
- **WASM Interop:**
    - Shape conversions live in `crates/wasm/src/wasm_convert.rs`.
    - `crates/wasm/src/lib.rs` uses a `thread_local` static `DecisionServiceController` for state management. This is a
      known architectural constraint.

## Debugging & Verification Playbook

- **Unlinked Fields:** If list or context fields report `Unlinked`, inspect `crates/core/src/ast/sequence.rs` and
  ensuring `linker::link_parts` is called.
- **WASM vs Rust:** If JS output diverges from Rust, check `crates/wasm/src/wasm_convert.rs`.
- **Test Helpers:** Use `crates/core-tests/tests/utilities.rs` (helpers like `assert_eval_all`, `link_error_contains`)
  to keep tests clean.
- **Stale Artifacts:** Always run `just node` or `just web` before running `just demo-*` commands.