# EdgeRules Project: Gemini Manual

This document provides a comprehensive guide for AI agents to understand, build, and contribute to the EdgeRules
project. It serves as the single source of truth for project structure, commands, and coding standards.

## Non Negotiable

- YOU ARE NOT ALLOWED TO COMMIT TO GIT! All your changes will be reviewed by a human, only human can commit the code.
- `git commit` commands are forbidden.
- `git push` commands are forbidden.
- However, you can check and compare previous versions of files using git commands.
- You must run all tests and ensure they pass before creating Pull Request.

## Coding Standards

- **Format:** Always use Markdown for documents and documentation.
- **Line Length:** 120 characters maximum.
- **Indentation:** 4 spaces (no tabs).
- **Rust Edition:** 2021
- **Naming:**
    - Crates: `kebab-case`
    - Modules/Files: `snake_case`
    - Types/Enums: `CamelCase`
    - Functions/Variables: `snake_case`
- **Variable Names:** Descriptive and meaningful. Avoid single-letter names (use `index` not `i`, `context` not `ctx`
  unless standard idiom).
- **Imports:** Group imports cleanly. Avoid long module paths in code; use `use` statements.
- `*_SPEC.md` files are used for design specifications, `*_STORY.md` files for implementation stories.

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
    - `edge-js` (JavaScript printer)
- **Rust Imports:** `edge_rules`, `edge_rules_wasi`, `edge_rules_cli`, `edge_js`
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
│   ├── edge-js/        # JavaScript printer for EdgeRules AST
│   │   └── src/lib.rs  # Library entry point
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
- **Node.js (WASM + JS Printer):** `just node-js` (Builds with `to_js` feature and runs tests)
- **WASI:** `just wasi` (Builds `edgerules-wasi.wasm` and runs a sanity check)
- **CLI (Native):** `just cli` (Builds `edgerules` binary and runs a sanity check)

### Testing

- **Rust Tests:** `just test` (Runs `cargo test --all`)
- **Node/WASM Tests:** `just test-node` (Runs Node.js tests in `tests/wasm/*.mjs` and `tests/wasm-js/*.mjs`)

### Demos & Performance

- **Basic Performance:** `just performance-basic` (Runs basic WASM benchmarks)
- **Decision Service Performance:** `just performance-ds` (Runs decision service benchmarks)

## Daily Workflow Checklist

Follow this loop for every change to ensure quality and prevent regressions:

1. **Format:** `cargo fmt`
2. **Lint:** `cargo clippy --all-targets -- -D warnings`
3. **Test:**
    - Run Rust tests: `just test`
    - If touching WASM logic: `just node` then `just test-node`
4. **Verify:** If fixing a bug, create a reproduction case in `crates/core-tests/tests/`.

## Project Priorities

1. **Small WASM Size First:** The primary goal is small WASM binary size. Performance is second.
2. **Small Stack Size Second**: Optimize for low stack usage in WASM environments.
3. **Performance Third:** Optimize for speed only after size and stack considerations are met.
4. **Maintainability** and code clarity are important but secondary to the above goals.

**Tips for keeping WASM size small:**

- Avoid unnecessary `#[derive(...)]` if not strictly needed.
- Be mindful of generic monomorphization bloat.
- Do not use `Debug` or `Display` derives for WASM target.
- Regex and base64 are not included in WASM by default and are provided by the host environment.

## Error Handling

- Use `?` for propagation.
- Avoid `unwrap()`/`expect()` in runtime code; reserve them for tests.
- Avoid `.ok()` that discards errors. Propagate errors instead.

# Instructions for Jules

**Testing notes:** you will not be able to run WASM builds due to environment limitations. However, you can run Rust
tests.