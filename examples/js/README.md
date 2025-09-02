Node.js example

Prerequisites
- Node.js 18+
- wasm-pack installed (cargo install wasm-pack)

Steps
- Build the package for Node: `wasm-pack build --release --target nodejs --out-dir target/pkg-node --out-name edge_rules`
- Run the demo: `node examples/js/node-demo.mjs`

Notes
- The script imports from `../../target/pkg-node/edge_rules.js`, so run the demo from the repo root.
- Results are printed to stdout.
