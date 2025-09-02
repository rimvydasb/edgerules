set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
export PATH := env_var("HOME") + "/.cargo/bin:" + env_var("PATH")

# Feature flags and target names
FEATURES := "wasm"
CRATE := "edge-rules"
BIN_WASI := "edgerules-wasi"
PROFILE := "release"

# Default wasm-pack output dir used by demos (pkg/)
wasm_bg := "pkg/edge_rules_bg.wasm"
wasm_bg_opt := "pkg/edge_rules_bg.opt.wasm"

# Split output dirs for separate web/node packages
wasm_bg_web := "pkg-web/edge_rules_bg.wasm"
wasm_bg_web_opt := "pkg-web/edge_rules_bg.opt.wasm"
wasm_bg_node := "pkg-node/edge_rules_bg.wasm"
wasm_bg_node_opt := "pkg-node/edge_rules_bg.opt.wasm"

# --- prerequisites ---
ensure:
    rustup target add wasm32-unknown-unknown
    rustup target add wasm32-wasip1
    command -v wasm-pack >/dev/null
    command -v wasm-opt >/dev/null || echo "TIP: brew install binaryen"
    command -v wasmtime >/dev/null || echo "TIP: brew install wasmtime"

# --- primary builds (shared pkg/ output; used by demos) ---
web: ensure
    rustup run stable wasm-pack build --release --target web -- --features {{FEATURES}}
    wasm-opt -Oz --strip-debug --strip-producers --dce {{wasm_bg}} -o {{wasm_bg_opt}}

node: ensure
    rustup run stable wasm-pack build --release --target nodejs -- --features {{FEATURES}}
    wasm-opt -Oz --strip-debug --strip-producers --dce {{wasm_bg}} -o {{wasm_bg_opt}}

wasi: ensure
    cargo build --release --target wasm32-wasip1 -p {{CRATE}} --bin {{BIN_WASI}}

core: ensure
    cargo build --release --target wasm32-unknown-unknown -p {{CRATE}}

core-opt: core
    wasm-opt -Oz --strip-debug --strip-producers --dce \
      target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE}}.wasm \
      -o target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE}}.min.wasm

# --- split builds (separate pkg-web/ and pkg-node/ outputs) ---
web-separate: ensure
    rustup run stable wasm-pack build --release --target web --out-dir pkg-web --out-name edge_rules -- --features {{FEATURES}}
    wasm-opt -Oz --strip-debug --strip-producers --dce {{wasm_bg_web}} -o {{wasm_bg_web_opt}}

node-separate: ensure
    rustup run stable wasm-pack build --release --target nodejs --out-dir pkg-node --out-name edge_rules -- --features {{FEATURES}}
    wasm-opt -Oz --strip-debug --strip-producers --dce {{wasm_bg_node}} -o {{wasm_bg_node_opt}}

# --- demo / test commands ---
demo-node: node
    node examples/js/node-demo.mjs

demo-web: web
    npx -y http-server -p 8080 .

demo-wasi: wasi
    wasmtime target/wasm32-wasip1/{{PROFILE}}/{{BIN_WASI}}.wasm "{ value : 2 + 2 }"

# --- dev quality-of-life ---
fmt:
    cargo fmt --all

clippy:
    cargo clippy --all-targets -- -D warnings

test:
    cargo test --all
