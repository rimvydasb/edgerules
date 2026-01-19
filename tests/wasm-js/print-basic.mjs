import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';
import { installBuiltins } from './builtins.js';

const toJsSupported = typeof wasm.DecisionEngine?.printExpressionJs === 'function';
const describeToJs = toJsSupported ? describe : describe.skip;

const evaluate = (source) => Function(`\"use strict\"; return (${source});`)();

describeToJs('To JS printing', () => {
    before(() => {
        wasm.init_panic_hook();
        installBuiltins();
    });

    it('prints arithmetic expression', () => {
        const js = wasm.DecisionEngine.printExpressionJs('2 + 3');
        const result = evaluate(js);
        assert.strictEqual(result, 5);
    });

    it('prints filters and selections', () => {
        const js = wasm.DecisionEngine.printExpressionJs('[1,2,3][...>1].length');
        const result = evaluate(js);
        assert.strictEqual(result, 2);
    });

    it('prints context model', () => {
        const js = wasm.DecisionEngine.printModelJs(`
        {
            name: "Ada"
            age: 20 + 1
            person: { label: name; years: age }
        }
        `);
        const ctx = evaluate(js);
        assert.deepStrictEqual(ctx, { name: 'Ada', age: 21, person: { label: 'Ada', years: 21 } });
    });

    it('escapes strings correctly', () => {
        const js = wasm.DecisionEngine.printExpressionJs("'hi\\nworld\"test'");
        const result = evaluate(js);
        assert.strictEqual(result, 'hi\\nworld"test');
    });

    it('supports builtin helpers', () => {
        const js = wasm.DecisionEngine.printExpressionJs('sum([1, 2, 3, 4])');
        const result = evaluate(js);
        assert.strictEqual(result, 10);
    });
});

describeToJs('Builtins interop', () => {
    before(() => {
        wasm.init_panic_hook();
        installBuiltins();
    });

    it('computes aggregates', () => {
        const sumJs = wasm.DecisionEngine.printExpressionJs('sum([1, 2, 3, 4])');
        const meanJs = wasm.DecisionEngine.printExpressionJs('mean([1, 2, 3, 4])');
        const medianJs = wasm.DecisionEngine.printExpressionJs('median([1, 2, 3, 4])');

        assert.equal(evaluate(sumJs), 10);
        assert.equal(evaluate(meanJs), 2.5);
        assert.equal(evaluate(medianJs), 2.5);
    });

    it('handles distinct and duplicates', () => {
        const distinctJs = wasm.DecisionEngine.printExpressionJs('distinctValues([1,2,2,3,3])');
        const duplicatesJs = wasm.DecisionEngine.printExpressionJs('duplicateValues([1,2,2,3,3])');

        assert.deepStrictEqual(evaluate(distinctJs), [1, 2, 3]);
        assert.deepStrictEqual(evaluate(duplicatesJs), [2, 3]);
    });

    it('supports union and append helpers', () => {
        const unionJs = wasm.DecisionEngine.printExpressionJs('union([1,2], [2,3])');
        const appendJs = wasm.DecisionEngine.printExpressionJs('append([1], 2, [3])');

        assert.deepStrictEqual(evaluate(unionJs), [1, 2, 3]);
        assert.deepStrictEqual(evaluate(appendJs), [1, 2, 3]);
    });
});

describeToJs('Roundtrip comparison', () => {
    before(() => {
        wasm.init_panic_hook();
        installBuiltins();
    });

    const cases = [
        { src: '2 * (3 + 4)', expected: 14 },
        { src: "'hi\"there'", expected: 'hi"there' },
        { src: '[1,2,3][...>1]', expected: [2, 3] },
        { src: '{a: 1; b: a + 2}.b', expected: 3 },
        { src: 'for x in [1,2,3] return x + 1', expected: [2, 3, 4] },
    ];

    for (const { src, expected } of cases) {
        it(`matches runtime for ${src}`, () => {
            const js = wasm.DecisionEngine.printExpressionJs(src);
            const jsResult = evaluate(js);
            const runtimeResult = wasm.DecisionEngine.evaluate(src);
            assert.deepStrictEqual(jsResult, runtimeResult);
            assert.deepStrictEqual(jsResult, expected);
        });
    }
});
