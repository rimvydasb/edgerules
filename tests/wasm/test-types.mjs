import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

describe('Type Introspection (getType) and List Operations', () => {
    let service;

    before(() => {
        wasm.init_panic_hook();
        const model = {
            // Primitives
            aString: "'text'",
            aNumber: 42,
            aBoolean: true,
            pi: 3.14159265359,
            
            // Quoted string tests (for getType)
            a: "'!'",
            quoted: "'\"To be or not to be!\"'",

            // Dependencies for variablesList
            xVar: 11,
            yVar: 22,
            zVar: 33,

            // Lists (Global)
            variablesList: ["xVar", "yVar", "zVar"],
            stringList: ["'a'", "'b'", "'c'"],
            numberList: [1, 2, 3],
            boolList: [true, false, true],
            
            // Edge cases for lists
            emptyList: [],
            nestedList: [[1, 2], [3, 4]],

            // Complex Types
            simpleObject: { x: 1, y: "'s'" },
            nestedObject: {
                child: {
                    grandchild: "'value'",
                    age: 10
                }
            },
            
            // Lists of Objects
            objects1List: [ { x: 1 }, { x: 2 }, { x: 3 } ],
            
            // Type definition for function argument
            RequestType: {
                '@type': 'type',
                index: '<number>',
                strList: '<string[]>',
                numList: '<number[]>',
                a: '<string>',
                quoted: '<string>'
            },

            // Functions for execution tests
            main: {
                '@type': 'function',
                '@parameters': { 'req': 'RequestType' },
                joined: "req.strList[0] + req.strList[1] + req.strList[2]",
                elem: "req.strList[floor(req.index)]",
                sum: "sum(req.numList)",
                // Concatenation test for quoted strings using parameters
                quoteTest: "req.quoted + req.a"
            }
        };
        service = new wasm.DecisionService(model);
    });

    describe('getType API Coverage', () => {
        it('retrieves type for string field', () => {
            assert.equal(service.getType('aString'), 'string');
        });

        it('retrieves type for number field', () => {
            assert.equal(service.getType('aNumber'), 'number');
        });

        it('retrieves type for decimal (pi)', () => {
            assert.equal(service.getType('pi'), 'number');
        });

        it('retrieves type for boolean field', () => {
            assert.equal(service.getType('aBoolean'), 'boolean');
        });

        it('retrieves type for list of strings', () => {
            const type = service.getType('stringList');
            assert.deepEqual(type, {
                type: 'list',
                itemType: 'string'
            });
        });

        it('retrieves type for list of variables (numbers)', () => {
            const type = service.getType('variablesList');
            assert.deepEqual(type, {
                type: 'list',
                itemType: 'number'
            });
        });

        it('retrieves type for list of numbers', () => {
            const type = service.getType('numberList');
            assert.deepEqual(type, {
                type: 'list',
                itemType: 'number'
            });
        });

        it('retrieves type for list of booleans', () => {
            const type = service.getType('boolList');
            assert.deepEqual(type, {
                type: 'list',
                itemType: 'boolean'
            });
        });

        it('retrieves type for empty list', () => {
            const type = service.getType('emptyList');
            assert.equal(type, '[]');
        });

        it('retrieves type for nested list', () => {
            const type = service.getType('nestedList');
            assert.deepEqual(type, {
                type: 'list',
                itemType: {
                    type: 'list',
                    itemType: 'number'
                }
            });
        });

        it('retrieves type for simple object', () => {
            const type = service.getType('simpleObject');
            assert.deepEqual(type, {
                x: 'number',
                y: 'string'
            });
        });

        it('retrieves type for nested object', () => {
            const type = service.getType('nestedObject');
            assert.deepEqual(type, {
                child: {
                    grandchild: 'string',
                    age: 'number'
                }
            });
        });

        it('retrieves type for list of objects', () => {
            const type = service.getType('objects1List');
            assert.deepEqual(type, {
                type: 'list',
                itemType: {
                    x: 'number'
                }
            });
        });

        it('throws error for non-existent path', () => {
            assert.throws(() => {
                service.getType('nonExistentField');
            });
        });
    });

    describe('Execution and Value Tests', () => {
        const testData = {
            index: 0,
            strList: ["a", "b", "c"],
            numList: [1, 2, 3],
            a: "!",
            quoted: "\"To be or not to be!\""
        };

        it('can access number list from function', () => {
            const response = service.execute('main', testData);
            assert.strictEqual(response.sum, 6);
        });

        it('evaluates joined strings correctly', () => {
            const response = service.execute('main', testData);
            assert.strictEqual(response.joined, 'abc');
        });

        it('evaluates single array element correctly', () => {
            const req = { ...testData, index: 1 };
            const response = service.execute('main', req);
            assert.strictEqual(response.elem, 'b'); 
        });

        it('preserves quotes in string concatenation', () => {
            const response = service.execute('main', testData);
            // "To be or not to be!" + ! -> "To be or not to be!"!
            assert.strictEqual(response.quoteTest, '"To be or not to be!"!');
        });

        it('retrieves decimal values correctly', () => {
            const pi = service.get('pi');
            assert.strictEqual(pi, 3.14159265359);
        });
    });
});