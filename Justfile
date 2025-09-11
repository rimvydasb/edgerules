set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
export PATH := env_var("HOME") + "/.cargo/bin:" + env_var("PATH")

# Feature flags and target names
FEATURES := "wasm"
CRATE := "edge-rules"
BIN_WASI := "edgerules-wasi"
BIN_NATIVE := "edgerules"
PROFILE := "release"
PKG_NAME := "edge_rules"

# Output dirs for separate web/node packages under target/
out_web := "target/pkg-web"
out_node := "target/pkg-node"

# Debug package output dirs (keep separate to avoid shipping with debug hook)
out_web_debug := "target/pkg-web-debug"
out_node_debug := "target/pkg-node-debug"

# External examples/public destination for showcasing web builds (sibling repo)
examples_public := "../edgerules-page/public"

# Shared wasm-opt flags to minimize output size. -Oz enables aggressive size optimizations, mutable globals unlock
# further reductions across supported runtimes, and strip options remove debug metadata, DWARF sections, producers,
# and function names. DCE drops unreachable code.
WASM_OPT_FLAGS := "-Oz --enable-mutable-globals --strip-dwarf --strip-debug --strip-producers --dce"

# --- prerequisites ---
ensure:
    rustup target add wasm32-unknown-unknown
    rustup target add wasm32-wasip1
    command -v wasm-pack >/dev/null
    command -v wasm-opt >/dev/null || echo "TIP: brew install binaryen"
    command -v wasmtime >/dev/null || echo "TIP: brew install wasmtime"
    mkdir -p {{out_web}} {{out_node}} {{out_web_debug}} {{out_node_debug}}

# --- shared builder for web/node ---
build-pkg platform out_dir features:
    # Toggle heavy features for WASM builds via env vars:
    #   ENABLE_REGEX=1 to include regex-based functions (split, replace)
    #   ENABLE_BASE64=1 to include base64 functions (toBase64, fromBase64)
    feats="{{features}}"
    if [ "${ENABLE_REGEX:-}" = "1" ]; then feats="$feats,regex_functions"; fi
    if [ "${ENABLE_BASE64:-}" = "1" ]; then feats="$feats,base64_functions"; fi
    echo "Using features: $feats"
    rustup run stable wasm-pack build --release --target {{platform}} --out-dir {{out_dir}} --out-name {{PKG_NAME}} -- --no-default-features --features "$feats"
    test -f {{out_dir}}/{{PKG_NAME}}_bg.wasm && ls -lh {{out_dir}}/{{PKG_NAME}}_bg.wasm || true
    # Apply shared  - command can be "-focused flags and remove unnecessary metadata.
    wasm-opt {{WASM_OPT_FLAGS}} {{out_dir}}/{{PKG_NAME}}_bg.wasm -o {{out_dir}}/{{PKG_NAME}}_bg.opt.wasm
    test -f {{out_dir}}/{{PKG_NAME}}_bg.opt.wasm && ls -lh {{out_dir}}/{{PKG_NAME}}_bg.opt.wasm || true

# --- primary builds (separate outputs under target/) ---
web: ensure
    just build-pkg web {{out_web}} {{FEATURES}}

node: ensure
    just build-pkg nodejs {{out_node}} {{FEATURES}}

# Debug builds with console_error_panic_hook enabled
web-debug: ensure
    just build-pkg web {{out_web_debug}} wasm_debug

node-debug: ensure
    just build-pkg nodejs {{out_node_debug}} wasm_debug

wasi: ensure
    cargo build --release --target wasm32-wasip1 -p {{CRATE}} --bin {{BIN_WASI}}
    ls -lh target/wasm32-wasip1/{{PROFILE}}/{{BIN_WASI}}.wasm || true
    # Always run demo-wasi after wasi build
    wasmtime target/wasm32-wasip1/{{PROFILE}}/{{BIN_WASI}}.wasm "{ value : 2 + 2 }" || true

core: ensure
    cargo build --release --target wasm32-unknown-unknown -p {{CRATE}} --no-default-features --features wasm
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

# --- native CLI build & quick check ---
cli:
    cargo build --release -p {{CRATE}} --bin {{BIN_NATIVE}}
    ls -lh target/{{PROFILE}}/{{BIN_NATIVE}} || true
    echo "Running arithmetic sanity check:"
    target/{{PROFILE}}/{{BIN_NATIVE}} "{ value : 2 + 3 }" || true

# --- release helpers ---
# Copies web builds into the external examples page project under public/.
# Excludes files not needed for serving (.gitignore, README.md).
release-to-examples: web web-debug
    echo "Releasing to: {{examples_public}}"
    echo "Source (web): {{out_web}}" && ls -la "{{out_web}}" || true
    echo "Source (web-debug): {{out_web_debug}}" && ls -la "{{out_web_debug}}" || true
    mkdir -p "{{examples_public}}/pkg-web" "{{examples_public}}/pkg-web-debug"
    rsync -a --delete "{{out_web}}/" "{{examples_public}}/pkg-web/"
    rsync -a --delete "{{out_web_debug}}/" "{{examples_public}}/pkg-web-debug/"
    # Remove files not needed in examples
    rm -f "{{examples_public}}/pkg-web/.gitignore" \
          "{{examples_public}}/pkg-web/README.md" \
          "{{examples_public}}/pkg-web/package.json" || true
    rm -f "{{examples_public}}/pkg-web-debug/.gitignore" \
          "{{examples_public}}/pkg-web-debug/README.md" \
          "{{examples_public}}/pkg-web-debug/package.json" || true
    echo "Contents (web):" && ls -la "{{examples_public}}/pkg-web" || true
    echo "Contents (web-debug):" && ls -la "{{examples_public}}/pkg-web-debug" || true
    echo "Released web assets to: {{examples_public}}"
