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
        delete error.message;
        
        assert.deepEqual(error, {
            error: {
                type: 'FieldNotFound',
                fields: ['object1', 'nonexistent']
            },
            location: 'calculations.calc',
            expression: 'object1.nonexistent',
            stage: 'linking'
        });
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
        delete error.message;

        assert.deepEqual(error, {
            error: {
                type: 'TypesNotCompatible',
                subject: "Left side of operator '+'",
                unexpected: 'number',
                expected: ['string']
            },
            location: 'value',
            expression: "(1 + 'a')",
            stage: 'linking'
        });
    });

    describe('Runtime Location Errors', () => {
        it('reports location for root field runtime error', () => {
            const code = `
            {
                value: date('invalid')
            }
            `;

            const error = getError(() => wasm.evaluate_field(code, 'value'));
            delete error.message;

            assert.deepEqual(error, {
                error: {
                    type: 'EvalError',
                    message: 'Invalid date string'
                },
                location: 'value',
                expression: "date('invalid')",
                stage: 'runtime'
            });
        });

        it('reports location for nested field runtime error', () => {
            const code = `
            {
                nested: { bad: date('invalid') }
                value: nested.bad
            }
            `;

            const error = getError(() => wasm.evaluate_field(code, 'value'));
            delete error.message;

            assert.deepEqual(error, {
                error: {
                    type: 'EvalError',
                    message: 'Invalid date string'
                },
                location: 'nested.bad',
                expression: "date('invalid')",
                stage: 'runtime'
            });
        });

        it('reports deep dependency chain location', () => {
            const code = `
            {
                source: {
                    value: date('invalid')
                }
                intermediate: {
                    calc: source.value
                }
                result: intermediate.calc
            }
            `;

            const error = getError(() => wasm.evaluate_field(code, 'result'));
            delete error.message;

            assert.deepEqual(error, {
                error: {
                    type: 'EvalError',
                    message: 'Invalid date string'
                },
                location: 'source.value',
                expression: "date('invalid')",
                stage: 'runtime'
            });
        });
    });
});