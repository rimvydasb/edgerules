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
            assert.deepEqual(e, {
                type: 'SchemaViolation',
                key: 'function',
                violation: 'Missing required field'
            });
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
            assert.deepEqual(e, {
                type: 'SchemaViolation',
                key: 'function',
                violation: 'Missing required field'
            });
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

    it('round-trips inline functions correctly', () => {
        const code = `{ func f(a): a + a }`;
        const service = new wasm.DecisionService(code);
        const f = service.get('f');
        
        // Inline function should be expanded to standard portable format with "return" field
        assert.strictEqual(f['@type'], 'function');
        assert.deepStrictEqual(f['@parameters'], { a: null });
        assert.strictEqual(f['return'], "a + a");

        // Round-trip: set it back and verify it still works
        service.set('f2', f);
        const result = service.execute('f2', 10);
        assert.strictEqual(result, 20);
    });

    it('supports setting return field on function body', () => {
        const code = `{ func f(a): a + a }`;
        const service = new wasm.DecisionService(code);
        
        // Current result: f(10) -> 20
        assert.strictEqual(service.execute('f', 10), 20);

        // Update return field: f(a): a * 3
        service.set('f.return', 'a * 3');
        assert.strictEqual(service.execute('f', 10), 30);

        // Add another field to body
        service.set('f.extra', 100);
        const fullF = service.get('f');
        assert.strictEqual(fullF.extra, 100);
        assert.strictEqual(fullF.return, 'a * 3');
        
        // Still returns only the return field
        assert.strictEqual(service.execute('f', 10), 30);
    });

    it('collapses return-only portable definition to inline function', () => {
        const model = {
            "f": {
                "@type": "function",
                "@parameters": { "x": null },
                "return": "x * x"
            }
        };
        const service = new wasm.DecisionService(model);
        // Verify it works
        assert.strictEqual(service.execute('f', 4), 16);
        
        // Verify it's considered inline (internal check via get)
        const f = service.get('f');
        assert.strictEqual(f.return, "x * x");
        assert.strictEqual(Object.keys(f).filter(k => !k.startsWith('@')).length, 1);
    });
});