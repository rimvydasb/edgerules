import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

describe('Function Execution via Evaluate', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    it('validates parser support for no-arg functions', () => {
        const result = wasm.DecisionEngine.evaluate(`{ func noarg(): { a: 1 } }`);
        assert.deepStrictEqual(result, { });
    });

    it('executes a function without arguments', () => {
        const code = `{
            func myMain(): { result: 420 }
        }`;
        const result = wasm.DecisionEngine.evaluate(code, 'myMain');
        assert.deepStrictEqual(result, { result: 420 });
    });

    it('parsing check with args', () => {
        const code = `{
            func myMainWithArgs(x): { result: 420 }
        }`;
        // This validates that our evaluate logic correctly identifies the function and checking its args
        try {
            wasm.DecisionEngine.evaluate(code, 'myMainWithArgs');
            assert.fail('Should have thrown argument error');
        } catch (e) {
            assert.match(e.message, /requires arguments/);
        }
    });

    it('throws error for function with arguments', () => {
        const code = `{
            func add(a, b): { result: a + b }
        }`;
        try {
            wasm.DecisionEngine.evaluate(code, 'add');
            assert.fail('Should have thrown');
        } catch (e) {
            assert.match(e.message, /requires arguments/);
        }
    });

    it('executes a nested function without arguments', () => {
        const code = `{
            nested: {
                func myVal(): { val: 100 }
            }
        }`;

         const result = wasm.DecisionEngine.evaluate(code, 'nested.myVal');
         assert.deepStrictEqual(result, { val: 100 });
    });

    it('executes a portable function without arguments', () => {
        const model = {
            "myFunc": {
                "@type": "function",
                "@parameters": {}, 
                "result": "10 * 2"
            }
        };
        // Expects { result: 20 } because function body returns object with field "result"
        const result = wasm.DecisionEngine.evaluate(model, 'myFunc');
        assert.deepStrictEqual(result, { result: 20 });
    });
});