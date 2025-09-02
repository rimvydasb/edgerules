// Run from repo root:
// 1) Build:  wasm-pack build --release --target nodejs
// 2) Run:    node examples/js/node-demo.mjs

import wasm from '../../target/pkg-node/edge_rules.js';

// Node target initializes WASM synchronously on import; no init() function.
wasm.init_panic_hook();

console.log('evaluate_value:', wasm.evaluate_value('{ value : 2 + 3 }'));
console.log('evaluate_field:', wasm.evaluate_field('{ x : 1; y : x + 2 }', 'y'));
console.log('to_trace:\n' + wasm.to_trace('{ a : 1; b : a + 2 }'));
