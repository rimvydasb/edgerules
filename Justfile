set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
export PATH := env_var("HOME") + "/.cargo/bin:" + env_var("PATH")

# Feature flags and target names
FEATURES := "wasm"
CRATE_CORE := "edge-rules"
CRATE_WASM := "edge-rules-wasi"
CRATE_CLI := "edge-rules-cli"
BIN_WASI := "edgerules-wasi"
BIN_NATIVE := "edgerules"
PROFILE := "release"
PKG_NAME := "edge_rules"
CORE_MANIFEST := "crates/core/Cargo.toml"
CLI_MANIFEST := "crates/cli/Cargo.toml"
WASM_MANIFEST := "crates/wasm/Cargo.toml"

# Output dirs for separate web/node packages under target/
out_web := "target/pkg-web"
out_node := "target/pkg-node"

# Debug package output dirs (keep separate to avoid shipping with debug hook)
out_web_debug := "target/pkg-web-debug"
out_node_debug := "target/pkg-node-debug"

# External examples/public destination for showcasing web builds (sibling repo)
edgerules_page_public := "../edgerules-page/public"
edgerules_docs_public := "../edgerules-docs/public"

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
    feats="{{features}}"; \
    if [ "${ENABLE_REGEX:-}" = "1" ]; then feats="$feats,regex_functions"; fi; \
    if [ "${ENABLE_BASE64:-}" = "1" ]; then feats="$feats,base64_functions"; fi; \
    echo "Using features: $feats"; cd crates/wasm && \
    wasm-pack build --release --target {{platform}} --out-dir ../../{{out_dir}} --out-name {{PKG_NAME}} -- --no-default-features --features "$feats"; \
    if [ -f ../../{{out_dir}}/{{PKG_NAME}}_bg.wasm ]; then ls -lh ../../{{out_dir}}/{{PKG_NAME}}_bg.wasm; fi; \
    if command -v wasm-opt >/dev/null; then \
      wasm-opt {{WASM_OPT_FLAGS}} ../../{{out_dir}}/{{PKG_NAME}}_bg.wasm -o ../../{{out_dir}}/{{PKG_NAME}}_bg.opt.wasm; \
      if [ -f ../../{{out_dir}}/{{PKG_NAME}}_bg.opt.wasm ]; then ls -lh ../../{{out_dir}}/{{PKG_NAME}}_bg.opt.wasm; fi; \
    else \
      echo "Skipping wasm-opt (not installed)"; \
    fi

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
    node examples/js/node-demo.mjs

wasi: ensure
    cargo build --release --target wasm32-wasip1 -p {{CRATE_CLI}} --bin {{BIN_WASI}} --manifest-path {{CLI_MANIFEST}}
    ls -lh target/wasm32-wasip1/{{PROFILE}}/{{BIN_WASI}}.wasm || true
    # Always run demo-wasi after wasi build
    wasmtime target/wasm32-wasip1/{{PROFILE}}/{{BIN_WASI}}.wasm "{ value : 2 + 2 }" || true

core: ensure
    cargo build --release --target wasm32-unknown-unknown -p {{CRATE_CORE}} --no-default-features --features wasm --manifest-path {{CORE_MANIFEST}}
    ls -lh target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE_CORE}}.wasm || true

core-opt: core
    # Apply shared size-focused flags and remove unnecessary metadata.
    wasm-opt {{WASM_OPT_FLAGS}} \
      target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE_CORE}}.wasm \
      -o target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE_CORE}}.min.wasm
    ls -lh target/wasm32-unknown-unknown/{{PROFILE}}/{{CRATE_CORE}}.min.wasm || true

# --- demo / test commands ---
performance-node: node
    node examples/js/node-performance.mjs

performance-ds: node
    node examples/js/node-ds-performance.mjs

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

test-node: node
    node --test tests/wasm/*.mjs

# --- native CLI build & quick check ---
cli:
    cargo build --release -p {{CRATE_CLI}} --bin {{BIN_NATIVE}} --manifest-path {{CLI_MANIFEST}}
    ls -lh target/{{PROFILE}}/{{BIN_NATIVE}} || true
    echo "Running arithmetic sanity check:"
    target/{{PROFILE}}/{{BIN_NATIVE}} "{ value : 2 + 3 }" || true

# --- release helpers ---
# Copies web builds into the external edgerules-page page project under public/.
# Excludes files not needed for serving (.gitignore, README.md).
release-to-page: web web-debug
    echo "Releasing to: {{edgerules_page_public}}"
    echo "Source (web): {{out_web}}" && ls -la "{{out_web}}" || true
    echo "Source (web-debug): {{out_web_debug}}" && ls -la "{{out_web_debug}}" || true
    mkdir -p "{{edgerules_page_public}}/pkg-web" "{{edgerules_page_public}}/pkg-web-debug"
    rsync -a --delete "{{out_web}}/" "{{edgerules_page_public}}/pkg-web/"
    rsync -a --delete "{{out_web_debug}}/" "{{edgerules_page_public}}/pkg-web-debug/"
    # Remove files not needed in edgerules_page
    rm -f "{{edgerules_page_public}}/pkg-web/.gitignore" \
          "{{edgerules_page_public}}/pkg-web/README.md" \
          "{{edgerules_page_public}}/pkg-web/package.json" || true
    rm -f "{{edgerules_page_public}}/pkg-web-debug/.gitignore" \
          "{{edgerules_page_public}}/pkg-web-debug/README.md" \
          "{{edgerules_page_public}}/pkg-web-debug/package.json" || true
    echo "Contents (web):" && ls -la "{{edgerules_page_public}}/pkg-web" || true
    echo "Contents (web-debug):" && ls -la "{{edgerules_page_public}}/pkg-web-debug" || true
    echo "Released web assets to: {{edgerules_page_public}}"

# Copies node builds into the external edgerules-docs page project under public/.
# Excludes files not needed for serving (.gitignore, README.md).
release-to-docs: node node-debug
    echo "Releasing to: {{edgerules_docs_public}}"
    echo "Source (node): {{out_node}}" && ls -la "{{out_node}}" || true
    echo "Source (node-debug): {{out_node_debug}}" && ls -la "{{out_node_debug}}" || true
    mkdir -p "{{edgerules_docs_public}}/pkg-node" "{{edgerules_docs_public}}/pkg-node-debug"
    rsync -a --delete "{{out_node}}/" "{{edgerules_docs_public}}/pkg-node/"
    rsync -a --delete "{{out_node_debug}}/" "{{edgerules_docs_public}}/pkg-node-debug/"
    # Remove files not needed in edgerules_page
    rm -f "{{edgerules_docs_public}}/pkg-node/.gitignore" \
          "{{edgerules_docs_public}}/pkg-node/README.md" \
          "{{edgerules_docs_public}}/pkg-node/package.json" || true
    rm -f "{{edgerules_docs_public}}/pkg-node-debug/.gitignore" \
          "{{edgerules_docs_public}}/pkg-node-debug/README.md" \
          "{{edgerules_docs_public}}/pkg-node-debug/package.json" || true
    echo "Contents (node):" && ls -la "{{edgerules_docs_public}}/pkg-node" || true
    echo "Contents (node-debug):" && ls -la "{{edgerules_docs_public}}/pkg-node-debug" || true
    echo "Released node assets to: {{edgerules_docs_public}}"
