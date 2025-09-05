set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
export PATH := env_var("HOME") + "/.cargo/bin:" + env_var("PATH")

# Feature flags and target names
FEATURES := "wasm"
CRATE := "edge-rules"
BIN_WASI := "edgerules-wasi"
PROFILE := "release"

# Output dirs for separate web/node packages under target/
out_web := "target/pkg-web"
out_node := "target/pkg-node"
wasm_bg_web := out_web + "/edge_rules_bg.wasm"
wasm_bg_web_opt := out_web + "/edge_rules_bg.opt.wasm"
wasm_bg_node := out_node + "/edge_rules_bg.wasm"
wasm_bg_node_opt := out_node + "/edge_rules_bg.opt.wasm"

# Shared wasm-opt flags to minimize output size. -Oz enables aggressive size optimizations, mutable globals unlock
# further reductions across supported runtimes, and strip options remove debug metadata, DWARF sections, producers,
# and function names. DCE drops unreachable code.
WASM_OPT_FLAGS := "-Oz --enable-mutable-globals --strip-dwarf --strip-function-names --strip-debug --strip-producers --dce"

# --- prerequisites ---
ensure:
    rustup target add wasm32-unknown-unknown
    rustup target add wasm32-wasip1
    command -v wasm-pack >/dev/null
    command -v wasm-opt >/dev/null || echo "TIP: brew install binaryen"
    command -v wasmtime >/dev/null || echo "TIP: brew install wasmtime"
    mkdir -p {{out_web}} {{out_node}}

# --- primary builds (separate outputs under target/) ---
web: ensure
    rustup run stable wasm-pack build --release --target web --out-dir {{out_web}} --out-name edge_rules -- --features {{FEATURES}}
    test -f {{wasm_bg_web}} && ls -lh {{wasm_bg_web}} || true
    # Apply shared size-focused flags and remove unnecessary metadata.
    wasm-opt {{WASM_OPT_FLAGS}} {{wasm_bg_web}} -o {{wasm_bg_web_opt}}
    test -f {{wasm_bg_web_opt}} && ls -lh {{wasm_bg_web_opt}} || true

node: ensure
    rustup run stable wasm-pack build --release --target nodejs --out-dir {{out_node}} --out-name edge_rules -- --features {{FEATURES}}
    test -f {{wasm_bg_node}} && ls -lh {{wasm_bg_node}} || true
    # Apply shared size-focused flags and remove unnecessary metadata.
    wasm-opt {{WASM_OPT_FLAGS}} {{wasm_bg_node}} -o {{wasm_bg_node_opt}}
    test -f {{wasm_bg_node_opt}} && ls -lh {{wasm_bg_node_opt}} || true

wasi: ensure
    cargo build --release --target wasm32-wasip1 -p {{CRATE}} --bin {{BIN_WASI}}
    ls -lh target/wasm32-wasip1/{{PROFILE}}/{{BIN_WASI}}.wasm || true
    # Always run demo-wasi after wasi build
    wasmtime target/wasm32-wasip1/{{PROFILE}}/{{BIN_WASI}}.wasm "{ value : 2 + 2 }" || true

core: ensure
    cargo build --release --target wasm32-unknown-unknown -p {{CRATE}}
    ls -lh target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE}}.wasm || true

core-opt: core
    # Apply shared size-focused flags and remove unnecessary metadata.
    wasm-opt {{WASM_OPT_FLAGS}} \
      target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE}}.wasm \
      -o target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE}}.min.wasm
    ls -lh target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE}}.min.wasm || true

# --- demo / test commands ---
demo-node: node
    node examples/js/node-demo.mjs

demo-web: web
    npx -y http-server -p 8080 .

demo-wasi: wasi

# --- dev quality-of-life ---
fmt:
    cargo fmt --all

clippy:
    cargo clippy --all-targets -- -D warnings

test:
    cargo test --all
