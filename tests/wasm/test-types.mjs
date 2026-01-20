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
        it('retrieves correct types for primitive fields', () => {
            assert.equal(service.getType('aString'), 'string');
            assert.equal(service.getType('aNumber'), 'number');
            assert.equal(service.getType('pi'), 'number');
            assert.equal(service.getType('aBoolean'), 'boolean');
        });

        it('retrieves correct types for list fields', () => {
            // String list
            assert.deepEqual(service.getType('stringList'), { type: 'list', itemType: 'string' });
            
            // Variable list (resolved to number)
            assert.deepEqual(service.getType('variablesList'), { type: 'list', itemType: 'number' });
            
            // Number list
            assert.deepEqual(service.getType('numberList'), { type: 'list', itemType: 'number' });
            
            // Boolean list
            assert.deepEqual(service.getType('boolList'), { type: 'list', itemType: 'boolean' });
            
            // Empty list
            assert.equal(service.getType('emptyList'), '[]');
            
            // Nested list
            assert.deepEqual(service.getType('nestedList'), { 
                type: 'list', 
                itemType: { type: 'list', itemType: 'number' } 
            });
            
            // List of objects
            assert.deepEqual(service.getType('objects1List'), {
                type: 'list',
                itemType: { x: 'number' }
            });
        });

        it('retrieves correct types for object fields', () => {
            // Simple object
            assert.deepEqual(service.getType('simpleObject'), { x: 'number', y: 'string' });
            
            // Nested object
            assert.deepEqual(service.getType('nestedObject'), {
                child: { grandchild: 'string', age: 'number' }
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

        it('executes list and string operations correctly', () => {
            // Access number list from function
            const resSum = service.execute('main', testData);
            assert.strictEqual(resSum.sum, 6);

            // Evaluate joined strings
            const resJoined = service.execute('main', testData);
            assert.strictEqual(resJoined.joined, 'abc');

            // Evaluate single array element
            const reqElem = { ...testData, index: 1 };
            const resElem = service.execute('main', reqElem);
            assert.strictEqual(resElem.elem, 'b');

            // Preserve quotes in string concatenation
            const resQuote = service.execute('main', testData);
            assert.strictEqual(resQuote.quoteTest, '"To be or not to be!"!');
        });

        it('retrieves decimal values correctly', () => {
            const pi = service.get('pi');
            assert.strictEqual(pi, 3.14159265359);
        });
    });
});
