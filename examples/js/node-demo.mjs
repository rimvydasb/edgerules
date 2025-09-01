// Run from repo root:
// 1) Build:  wasm-pack build --release --target nodejs
// 2) Run:    node examples/js/node-demo.mjs

import init, { evaluate_value, evaluate_field, to_trace, init_panic_hook } from '../../pkg/edge_rules.js';

await init();
init_panic_hook();

console.log('evaluate_value:', await evaluate_value('{ value : 2 + 3 }'));
console.log('evaluate_field:', await evaluate_field('{ x : 1; y : x + 2 }', 'y'));
console.log('to_trace:\n' + to_trace('{ a : 1; b : a + 2 }'));
