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
- `just wasi`: Build the WASI binary `edgerules-wasi.wasm` and run demo (prints size).
- `just core`: Build core library for wasm32-unknown-unknown.
- `just core-opt`: Minify core output to .min.wasm.
- `just demo-web`: Serve at http://localhost:8080 (expects `target/pkg-web/`).
- `just demo-node`: Run Node demo (expects `target/pkg-node/`).
- `just demo-wasi`: Run WASI demo via wasmtime.

## Coding Style & Naming Conventions

- Rust 2021 edition; 4 spaces; keep modules small and cohesive.
- Naming: modules/files `snake_case`; types/enums `CamelCase`; functions/fields `snake_case`.
- Formatting: run `cargo fmt` before commits; keep `clippy` clean.
- WASM: default features include `wasm-bindgen` and `console_error_panic_hook`.

## Testing Guidelines

- Prefer unit tests near the code (`mod tests`), with focused cases.
- Use `tests/` for integration-style, text-based fixtures; mirror feature areas (e.g., `tests/errors/*.txt`).
- Run `cargo test` locally; add tests for new behavior and bug fixes.

## Commit & Pull Request Guidelines

- Commits: short, imperative subject lines (e.g., "fix tokenizer range parsing"); group related changes; avoid "various
  updates".
- PRs: clear description, rationale, and scope; link issues; include run/build steps and demo notes (Node/Web) when
  relevant; attach screenshots for web demos.

## Security & Configuration Tips

- Validate untrusted model inputs; avoid panics in runtime paths.
- For WASI/Node/Web, prefer explicit feature flags and minimal exports to keep binaries/WASM small.
