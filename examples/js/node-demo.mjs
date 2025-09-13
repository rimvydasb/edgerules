import wasm from '../../target/pkg-node/edge_rules.js';

// Node target initializes WASM synchronously on import; no init() function.
wasm.init_panic_hook();

console.log('evaluate_field:', wasm.evaluate_field('{ x : 1; y : x + 2 }', 'y'));

console.log('evaluate_expression:', wasm.evaluate_expression('2 + 3'));
const result = wasm.evaluate_expression(`regexReplace('Hello 123 world 456', '\\d+', 'X', 'g')`);
console.log('regexReplace:', result);
if (result !== `'Hello X world X'`) {
  throw new Error('regexReplace failed: ' + result);
}

const split = wasm.evaluate_expression(`regexSplit('one   two\tthree', '\\s+')`);
console.log('regexSplit:', split);
if (split !== `['one', 'two', 'three']`) {
  throw new Error('regexSplit failed: ' + split);
}

const b64 = wasm.evaluate_expression(`toBase64('FEEL')`);
console.log('toBase64:', b64);
if (b64 !== `'RkVFTA=='`) {
  throw new Error('toBase64 failed: ' + b64);
}

const from = wasm.evaluate_expression(`fromBase64('RkVFTA==')`);
console.log('fromBase64:', from);
if (from !== `'FEEL'`) {
  throw new Error('fromBase64 failed: ' + from);
}
