import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

describe('Unhappy Paths & Error Handling', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    const getError = (fn) => {
        try {
            fn();
        } catch (e) {
            if (typeof e === 'string') {
                // In WASM bindgen, sometimes errors are just strings if not properly converted
                // But we expect a JSON string or object for structured errors
                // If it's a JSON string, parse it.
                try {
                    return JSON.parse(e);
                } catch {
                    return e;
                }
            }
            return e;
        }
        assert.fail('Expected function to throw an error');
    };

    it('reports location for object body errors', () => {
        const code = `
        {
            object1: {
                fieldA: "a"
                fieldB: "b"
            }
            calculations: {
                calc: object1.nonexistent
            }
            value : calculations.calc
        }
        `;

        const error = getError(() => wasm.evaluate_field(code, 'value'));
        
        // Assert structure based on ERRORS_STORY.md
        // We expect structured error object
        assert.equal(error.stage, 'linking');
        assert.deepEqual(error.error, {
            type: 'FieldNotFound',
            fields: ['object1', 'nonexistent']
        });
        // location is expected to be a string joined by dots in JSON/JS representation according to the story example
        // "location": "calculations.takeDate.year"
        // In the rust test: &["calculations", "calc"]
        assert.equal(error.location, 'calculations.calc');
        assert.equal(error.expression, 'object1.nonexistent');
    });

    it('reports location for function body errors', () => {
        const code = `
        {
            calculations: {
                func takeDate(d: date): { year: d.nonexistent }
                result: takeDate(date('2024-01-01')).year
            }
            value : calculations.result
        }
        `;

        const error = getError(() => wasm.evaluate_field(code, 'value'));
        delete error.message;

        assert.deepEqual(error, {
            error: {
                type: 'FieldNotFound',
                fields: ['d', 'nonexistent']
            },
            location: 'calculations.takeDate.year',
            expression: 'd.nonexistent',
            stage: 'linking'
        });
    });

    it('reports location for root field errors', () => {
        const code = `
        {
            value: 1 + 'a'
        }
        `;

        const error = getError(() => wasm.evaluate_field(code, 'value'));

        assert.equal(error.stage, 'linking');
        assert.equal(error.error.type, 'TypesNotCompatible');
        assert.equal(error.location, 'value');
        assert.equal(error.expression, "(1 + 'a')");
    });
});