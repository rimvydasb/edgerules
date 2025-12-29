import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

describe('Basic Evaluation', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    it('evaluate_field simple arithmetic', () => {
        const result = wasm.DecisionEngine.evaluateField('{ x : 1; y : x + 2 }', 'y');
        assert.deepStrictEqual(result, 3);
    });

    it('evaluate_expression simple arithmetic', () => {
        const result = wasm.DecisionEngine.evaluateExpression('2 + 3');
        assert.deepStrictEqual(result, 5);
    });

    it('regexReplace', () => {
        const result = wasm.DecisionEngine.evaluateExpression(`regexReplace('Hello 123 world 456', '\\d+', 'X', 'g')`);
        assert.deepStrictEqual(result, 'Hello X world X');
    });

    it('regexSplit', () => {
        const split = wasm.DecisionEngine.evaluateExpression(`regexSplit('one   two\tthree', '\\s+')`);
        assert.deepStrictEqual(split, ['one', 'two', 'three']);
    });

    it('base64 functions', () => {
        const b64 = wasm.DecisionEngine.evaluateExpression(`toBase64('FEEL')`);
        assert.deepStrictEqual(b64, 'RkVFTA==');

        const from = wasm.DecisionEngine.evaluateExpression(`fromBase64('RkVFTA==')`);
        assert.deepStrictEqual(from, 'FEEL');
    });


    it('complex evaluation with filter', () => {
        const result = wasm.DecisionEngine.evaluateField(
            `
            {
                type Person: { name: <string>; age: <number>; tags: <string[]> }
                type PeopleList: Person[]
                func getAdults(people: PeopleList): {
                    result: people[age >= 18]
                }
                persons: [
                    {name: "Alice"; age: 30; tags: ["engineer", "manager"]}
                    {name: "Bob"; age: 15; tags: ["student"]}
                    {name: "Charlie"; age: 22; tags: []}
                ]
                adults: getAdults(persons)
            }
        `, "adults");

        assert.deepStrictEqual(result, {
            result: [
                { name: "Alice", age: 30, tags: ["engineer", "manager"] },
                { name: "Charlie", age: 22, tags: [] }
            ]
        });
    });

    it('evaluate_all', () => {
        const result = wasm.DecisionEngine.evaluateAll(`
        {
            sales: [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
            salesCount: count(sales)
            func sales3(month, sales): { 
                result: sales[month] + sales[month + 1] + sales[month + 2] 
            }
            acc: for m in 0..(salesCount - 3) return sales3(m, sales).result
            best: max(acc)
        }
        `);
        
        // In JS via wasm-bindgen, Maps are typically returned as Maps.
        // However, if it returns a plain object, handle that.
        const salesCount = result.get ? result.get('salesCount') : result.salesCount;
        const best = result.get ? result.get('best') : result.best;

        assert.strictEqual(salesCount, 12);
        assert.strictEqual(best, 94);
    });
});

