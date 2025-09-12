// Run from repo root:
// 1) Build:  wasm-pack build --release --target nodejs
// 2) Run:    node examples/js/node-demo.mjs

import wasm from '../../target/pkg-node/edge_rules.js';

// Node target initializes WASM synchronously on import; no init() function.
wasm.init_panic_hook();

console.log('evaluate_expression:', wasm.evaluate_expression('2 + 3'));
console.log('evaluate_field:', wasm.evaluate_field('{ x : 1; y : x + 2 }', 'y'));
const result = wasm.evaluate_expression(`regexReplace('Hello 123 world 456', '\\d+', 'X', 'g')`);
console.log('regexReplace:', result);
if (result !== `'Hello X world X'`) {
  throw new Error('regexReplace failed: ' + result);
}
