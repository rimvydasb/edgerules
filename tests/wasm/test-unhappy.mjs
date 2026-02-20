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

        const error = getError(() => wasm.DecisionEngine.evaluate(code, 'value'));
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

        const error = getError(() => wasm.DecisionEngine.evaluate(code, 'value'));
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

        const error = getError(() => wasm.DecisionEngine.evaluate(code, 'value'));
        delete error.message;

        assert.deepEqual(error, {
            error: {
                type: 'TypesNotCompatible',
                subject: "Left side of operator '+'",
                unexpected: 'number',
                expected: ['string']
            },
            location: 'value',
            expression: "1 + 'a'",
            stage: 'linking'
        });
    });

    it('evaluate expression with field throws', () => {
        const error = getError(() => wasm.DecisionEngine.evaluate('1 + 1', 'someField'));
        assert.deepEqual(error, {
            message: "Context 'someField' not found"
        });
    });

    describe('Runtime Location Errors', () => {
        it('reports location for root field runtime error', () => {
            const code = `
            {
                value: date('invalid')
            }
            `;

            const error = getError(() => wasm.DecisionEngine.evaluate(code, 'value'));
            delete error.message;

            assert.deepEqual(error, {
                error: {
                    type: 'ValueParsingError',
                    from: 'string',
                    to: 'date',
                    code: 0,
                    message: "Failed to parse 'date' from 'string'"
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

            const error = getError(() => wasm.DecisionEngine.evaluate(code, 'value'));
            delete error.message;

            assert.deepEqual(error, {
                error: {
                    type: 'ValueParsingError',
                    from: 'string',
                    to: 'date',
                    code: 0,
                    message: "Failed to parse 'date' from 'string'"
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

            const error = getError(() => wasm.DecisionEngine.evaluate(code, 'result'));
            delete error.message;

            assert.deepEqual(error, {
                error: {
                    type: 'ValueParsingError',
                    from: 'string',
                    to: 'date',
                    code: 0,
                    message: "Failed to parse 'date' from 'string'"
                },
                location: 'source.value',
                expression: "date('invalid')",
                stage: 'runtime'
            });
        });
    });
});

describe('Abnormal Path Handling', () => {
    let service;

    before(() => {
        wasm.init_panic_hook();
        const model = {
            valid: 10,
            nested: {
                inner: 20
            }
        };
        service = new wasm.DecisionService(model);
    });

    const getError = (fn) => {
        try {
            fn();
        } catch (e) {
            return e;
        }
        assert.fail('Expected function to throw an error');
    };

    describe('service.get(path)', () => {
        it('throws on empty path', () => {
            const error = getError(() => service.get(''));
            assert.match(error.message, /Field path is empty/);
        });

        it('throws on path with empty segments (..)', () => {
            const error = getError(() => service.get('valid..path'));
            assert.match(error.message, /Invalid path 'valid..path'/);
        });

        it('throws on path starting with dot (.path)', () => {
            const error = getError(() => service.get('.valid'));
            assert.match(error.message, /Invalid path '.valid'/);
        });

        it('throws on path ending with dot (path.)', () => {
            const error = getError(() => service.get('valid.'));
            assert.match(error.message, /Invalid path 'valid.'/);
        });

        it('throws for non-existent root path', () => {
            const error = getError(() => service.get('nonexistent'));
            assert.match(error.message, /Entry 'nonexistent' not found/);
        });
        
        it('throws for non-existent nested leaf', () => {
             const error = getError(() => service.get('nested.ghost'));
             assert.match(error.message, /Entry 'nested.ghost' not found/);
        });

        it('throws for non-existent nested parent', () => {
             const error = getError(() => service.get('ghost.child'));
             assert.match(error.message, /Context 'ghost' not found/);
        });
    });

    describe('service.getType(path)', () => {
        it('throws on empty path', () => {
            const error = getError(() => service.getType(''));
            assert.match(error.message, /Field path is empty/);
        });

        it('throws on path with empty segments (..)', () => {
            const error = getError(() => service.getType('valid..path'));
            assert.match(error.message, /Invalid path 'valid..path'/);
        });

        it('throws on path starting with dot (.path)', () => {
            const error = getError(() => service.getType('.valid'));
            assert.match(error.message, /Invalid path '.valid'/);
        });

        it('throws on path ending with dot (path.)', () => {
            const error = getError(() => service.getType('valid.'));
            assert.match(error.message, /Invalid path 'valid.'/);
        });

        it('throws on non-existent path', () => {
            const error = getError(() => service.getType('nonexistent'));
            assert.match(error.message, /Entry 'nonexistent' not found/);
        });

        it('throws on non-existent nested path', () => {
            const error = getError(() => service.getType('nested.ghost'));
            assert.match(error.message, /Entry 'nested.ghost' not found/);
        });

        it('throws on non-existent parent context', () => {
            const error = getError(() => service.getType('ghost.child'));
            assert.match(error.message, /Context 'ghost' not found/);
        });
    });
});

describe('Array Access Exceptions', () => {
    let service;
    before(() => {
        wasm.init_panic_hook();
        const model = {
            list: [1, 2, 3],
            scalar: 10
        };
        service = new wasm.DecisionService(model);
    });

    const getError = (fn) => {
        try {
            fn();
        } catch (e) {
            return e;
        }
        assert.fail('Expected function to throw an error');
    };

    it('set throws on gap', () => {
        const error = getError(() => service.set('list[4]', 99));
        assert.match(error.message, /Index 4 is out of bounds for array of length 3/);
    });

    it('get throws on out of bounds', () => {
        const error = getError(() => service.get('list[3]'));
         assert.match(error.message, /Index 3 out of bounds/);
    });
    
    it('remove throws on out of bounds', () => {
        const error = getError(() => service.remove('list[3]'));
         assert.match(error.message, /Index 3 out of bounds/);
    });

    it('set throws if field is not an array', () => {
         const error = getError(() => service.set('scalar[0]', 2));
         assert.match(error.message, /Field 'scalar' is not an array/);
    });
});