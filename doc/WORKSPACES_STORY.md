Workspace conversion goals
- Keep the current public shape (EdgeRules library + CLI + WASM) but split into focused crates to make IDEs happier, shrink WASM outputs, and avoid pulling unnecessary dependencies into native builds.
- Preserve optional function groups (regex/base64) and the wasm_debug toggle so existing demos keep working.
- Maintain Just targets so every current build/demo/release flow remains available after the split.

Action plan (no refactors yet)
- [x] Create a workspace Cargo.toml at the repo root with resolver = "2", members crates/core, crates/cli, crates/wasm, and shared [workspace.dependencies] for common dev tools.
- [x] Carve out crates/core: move library sources except CLI/WASM glue; keep the public API surface used in README examples and tests; expose feature flags wasm, wasm_debug, regex_functions, base64_functions with minimal defaults.
- [x] Carve out crates/cli: depend on core with native-facing features; host binaries edgerules and er; integration test harness (assert_cmd) still TODO.
- [x] Carve out crates/wasm: depend on core with wasm/wasm_debug; host wasm.rs/wasm_convert and related glue; set crate-type = ["cdylib"]; keep JS shims minimal for size.
- [x] Relink tests and fixtures: core integration tests live in crates/core-tests; wasm smoke tests now live under crates/wasm/tests (host + wasm32 cfg); examples/js and examples/web still need verification against the new wasm package output.
- [ ] Update README snippets and AGENTS/AI-USE addenda references if paths or commands change.

Justfile alignment for full functionality
- [x] Introduce workspace-aware variables (paths to crates/core, crates/cli, crates/wasm) and reuse current feature toggles (ENABLE_REGEX, ENABLE_BASE64, wasm_debug) without changing behavior.
- [x] Point wasm pack builders (web/node/web-debug/node-debug) to crates/wasm with the same out dirs and wasm-opt flow; ensure PKG_NAME stays edge_rules.
- [x] Point native builds (cli) to crates/cli while keeping binary names and the arithmetic sanity check.
- [x] Point wasi/core/core-opt targets to crates/wasm or crates/core as appropriate, preserving optimization flags and output locations.
- [x] Keep demo/release tasks (demo-node, demo-web, demo-wasi, release-to-page, release-to-docs) wired to the new outputs; confirm rsync paths remain correct.
- [x] Retain fmt/clippy/test tasks at the workspace level (`cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all`); AGENTS loop update still pending.
