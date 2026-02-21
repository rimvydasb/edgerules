import {before, describe, it} from 'node:test';
import {strict as assert} from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

const DEFAULT_VALUES_MODEL = `
    {
        type Customer: {
            name: <string>
            income: <number, 0>
            isActive: <boolean, true>
            category: <string, "STD">
        }

        // Test 1: Cast empty object to Customer
        customerFromEmpty: {} as Customer

        // Test 2: Cast object with some values
        customerPartial: { name: "John" } as Customer

        // Test 3: Cast object with all values (defaults ignored)
        customerFull: { name: "Jane"; income: 5000; isActive: false; category: "VIP" } as Customer
        
        // Test 4: Nested defaults
        type Loan: { customer: <Customer>; amount: <number, 1000> }
        loanFromEmpty: {} as Loan
    }
`;

const MIXED_PORTABLE_MODEL = {
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
    simpleObject: {x: 1, y: "'s'"},
    nestedObject: {
        child: {
            grandchild: "'value'",
            age: 10
        }
    },

    // Lists of Objects
    objects1List: [{x: 1}, {x: 2}, {x: 3}],

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
        '@parameters': {'req': 'RequestType'},
        joined: "req.strList[0] + req.strList[1] + req.strList[2]",
        elem: "req.strList[floor(req.index)]",
        sum: "sum(req.numList)",
        // Concatenation test for quoted strings using parameters
        quoteTest: "req.quoted + req.a"
    },

    // Functions for execution tests
    hasHiddenFields: {
        '@type': 'function',
        '@parameters': {'req': 'RequestType'},
        joined: "req.strList[0] + req.strList[1] + req.strList[2]",
        return: {
            elem: "req.strList[floor(req.index)]",
            sum: "sum(req.numList)",
            // Concatenation test for quoted strings using parameters
            quoteTest: "req.quoted + req.a"
        }
    }
}

describe('Type Introspection (getType) and List Operations', () => {
    let service;

    before(() => {
        wasm.init_panic_hook();
        service = new wasm.DecisionService(MIXED_PORTABLE_MODEL);
    });

    describe('getType and get API Coverage', () => {
        it('retrieves correct types for primitive fields', () => {
            assert.equal(service.getType('aString'), 'string');
            assert.equal(service.getType('aNumber'), 'number');
            assert.equal(service.getType('pi'), 'number');
            assert.equal(service.getType('aBoolean'), 'boolean');
        });

        it('retrieves correct values for primitive fields via get', () => {
            assert.equal(service.get('aString'), "'text'");
            assert.equal(service.get('aNumber'), 42);
            assert.equal(service.get('pi'), 3.14159265359);
            assert.equal(service.get('aBoolean'), true);
        });

        it('retrieves correct types for list fields', () => {
            // String list
            assert.deepEqual(service.getType('stringList'), {type: 'list', itemType: 'string'});

            // Variable list (resolved to number)
            assert.deepEqual(service.getType('variablesList'), {type: 'list', itemType: 'number'});

            // Number list
            assert.deepEqual(service.getType('numberList'), {type: 'list', itemType: 'number'});

            // Boolean list
            assert.deepEqual(service.getType('boolList'), {type: 'list', itemType: 'boolean'});

            // Empty list
            assert.equal(service.getType('emptyList'), '[]');

            // Nested list
            assert.deepEqual(service.getType('nestedList'), {
                type: 'list',
                itemType: {type: 'list', itemType: 'number'}
            });

            // List of objects
            assert.deepEqual(service.getType('objects1List'), {
                type: 'list',
                itemType: {x: 'number'}
            });
        });

        it('retrieves correct values and schemas for list fields via get', () => {
            const stringList = service.get('stringList');
            assert.deepEqual(filterSchema(stringList), ["'a'", "'b'", "'c'"]);
            assert.deepEqual(stringList['@schema'], {type: 'list', itemType: 'string'});

            const variablesList = service.get('variablesList');
            assert.deepEqual(filterSchema(variablesList), ["xVar", "yVar", "zVar"]);
            assert.deepEqual(variablesList['@schema'], {type: 'list', itemType: 'number'});

            const objects1List = service.get('objects1List');
            assert.deepEqual(filterSchema(objects1List), [{x: 1}, {x: 2}, {x: 3}]);
            assert.deepEqual(objects1List['@schema'], {
                type: 'list',
                itemType: {x: 'number'}
            });
        });

        it('retrieves correct types for object fields', () => {
            // Simple object
            assert.deepEqual(service.getType('simpleObject'), {x: 'number', y: 'string'});

            // Nested object
            assert.deepEqual(service.getType('nestedObject'), {
                child: {grandchild: 'string', age: 'number'}
            });
        });

        it('retrieves correct values and schemas for object fields via get', () => {
            const simpleObject = service.get('simpleObject');
            assert.deepEqual(filterSchema(simpleObject), {x: 1, y: "'s'"});
            assert.deepEqual(simpleObject['@schema'], {x: 'number', y: 'string'});

            const nestedObject = service.get('nestedObject');
            assert.deepEqual(filterSchema(nestedObject), {child: {grandchild: "'value'", age: 10}});
            assert.deepEqual(nestedObject['@schema'], {
                child: {grandchild: 'string', age: 'number'}
            });
        });

        it('retrieves correct types for function', () => {
            assert.deepEqual(service.getType('main'), {
                "elem": 'string',
                "joined": 'string',
                "quoteTest": 'string',
                "sum": 'number'
            });
            assert.deepEqual(service.getType('hasHiddenFields'), {
                "joined": 'string',
                "return": {"elem": 'string', "quoteTest": 'string', "sum": 'number'}
            });

            // Checking with getType("*") bypasses function definitions as well as other definitions
            const allTypes = service.getType("*");
            assert.deepEqual(allTypes.main, undefined);
            assert.deepEqual(allTypes.hasHiddenFields, undefined);
        });

        it('retrieves correct values and schemas for function definitions via get', () => {
            const main = service.get('main');
            assert.equal(main['@type'], 'function');
            assert.deepEqual(main['@parameters'], {req: 'RequestType'});
            assert.equal(main.joined, "req.strList[0] + req.strList[1] + req.strList[2]");
            assert.deepEqual(main['@schema'], {
                "elem": 'string',
                "joined": 'string',
                "quoteTest": 'string',
                "sum": 'number'
            });

            const hasHiddenFields = service.get('hasHiddenFields');
            assert.equal(hasHiddenFields['@type'], 'function');
            assert.deepEqual(hasHiddenFields['@schema'], {
                "joined": 'string',
                "return": {"elem": 'string', "quoteTest": 'string', "sum": 'number'}
            });
        });

        it('retrieves full model with metadata via get("*")', () => {
            const model = service.get('*');

            // 1. Check top-level keys presence (Portable structure)
            assert.ok(model.RequestType, "Should have RequestType");
            assert.ok(model.main, "Should have main function");
            assert.ok(model.simpleObject, "Should have simpleObject");
            assert.ok(model.stringList, "Should have stringList");

            // 2. Check @schema existence and content
            assert.ok(model['@schema'], "Root should have @schema");
            const schema = model['@schema'];

            // Primitives
            assert.equal(schema.aString, 'string');
            assert.equal(schema.aNumber, 'number');

            // Complex types in schema
            assert.deepEqual(schema.simpleObject, {x: 'number', y: 'string'});
            assert.deepEqual(schema.stringList, {type: 'list', itemType: 'string'});

            // Functions and Types should NOT be in @schema (they are definitions, not data fields)
            assert.equal(schema.main, undefined);
            assert.equal(schema.RequestType, undefined);

            // 3. Verify function structure in get('*')
            assert.equal(model.main['@type'], 'function');
            assert.deepEqual(model.main['@parameters'], {req: 'RequestType'});
            assert.equal(model.main.sum, "sum(req.numList)");

            // 4. Verify type structure in get('*')
            assert.equal(model.RequestType['@type'], 'type');
            assert.equal(model.RequestType.index, '<number>');

            // 5. Verify nested objects and lists
            assert.deepEqual(model.nestedObject.child, {grandchild: "'value'", age: 10});
            assert.deepEqual(model.nestedList[0], [1, 2]);

            // 6. Deep comparison with original model (filtering out @schema)
            const filtered = filterSchema(model);
            assert.deepEqual(filtered, MIXED_PORTABLE_MODEL);
        });

        it('reproducers: arrays should be arrays, not strings; no unnecessary brackets', () => {
            const model = service.get('*');

            assert.ok(Array.isArray(model.objects1List), `objects1List should be an array, but got ${typeof model.objects1List}: ${model.objects1List}`);
            assert.deepEqual(model.objects1List, [{x: 1}, {x: 2}, {x: 3}]);

            assert.ok(Array.isArray(model.variablesList), "variablesList should be an array");
            assert.deepEqual(model.variablesList, ["xVar", "yVar", "zVar"]);

            assert.strictEqual(model.main.joined, "req.strList[0] + req.strList[1] + req.strList[2]");
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
            const reqElem = {...testData, index: 1};
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

        it('executes wildcard (*) to evaluate entire model', () => {
            const result = service.execute('*');
            assert.strictEqual(result.aNumber, 42);
            assert.strictEqual(result.aString, "text");
            assert.deepEqual(result.simpleObject, {x: 1, y: "s"});
            // Result of wildcard execute does NOT have @schema or @type definitions, it is purely evaluated data
            assert.equal(result['@schema'], undefined);
            assert.equal(result.RequestType, undefined);
            assert.equal(result.main, undefined);
        });
    });
});

describe('Default Values in Types', () => {
    it('applies default values when casting empty object', () => {
        const result = wasm.DecisionEngine.evaluate(DEFAULT_VALUES_MODEL, 'customerFromEmpty');
        assert.equal(result.income, 0);
        assert.equal(result.isActive, true);
        assert.equal(result.category, "STD");
    });

    it('applies default values for missing fields only', () => {
        const result = wasm.DecisionEngine.evaluate(DEFAULT_VALUES_MODEL, 'customerPartial');
        assert.equal(result.name, "John");
        assert.equal(result.income, 0);
        assert.equal(result.isActive, true);
    });

    it('does not override existing values', () => {
        const result = wasm.DecisionEngine.evaluate(DEFAULT_VALUES_MODEL, 'customerFull');
        assert.equal(result.name, "Jane");
        assert.equal(result.income, 5000);
        assert.equal(result.isActive, false);
        assert.equal(result.category, "VIP");
    });

    it('works with nested objects', () => {
        const result = wasm.DecisionEngine.evaluate(DEFAULT_VALUES_MODEL, 'loanFromEmpty');
        assert.equal(result.amount, 1000);
        assert.equal(result.customer.income, 0);
        assert.equal(result.customer.isActive, true);
    });
});

describe('User Type Modification', () => {
    let service;

    const SMALL_PORTABLE_MODEL = {
        Applicant: {
            '@type': 'type',
            name: '<string>',
            income: '<number>'
        },
        processApplicant: {
            '@type': 'function',
            '@parameters': {app: 'Applicant'},
            result: 'app.income'
        }
    };

    before(() => {
        wasm.init_panic_hook();
        service = new wasm.DecisionService(SMALL_PORTABLE_MODEL);
    });

    it('modifies a user type definition by replacing it', () => {

        const all = service.get('*');
        assert.deepEqual(all, {'@schema': {}, ...SMALL_PORTABLE_MODEL});

        // Initial check via get() which returns the Portable definition
        const initialDef = service.get('Applicant');
        assert.equal(initialDef['@type'], 'type');
        assert.equal(initialDef.income, '<number>');

        // Modify Applicant type definition: change income to <string>
        // We must provide the full type definition
        service.set('Applicant', {
            '@type': 'type',
            name: '<string>',
            income: '<string>'
        });

        // Verify with get()
        const modifiedDef = service.get('Applicant');
        assert.equal(modifiedDef.income, '<string>');

        // Verify execution reflects the change
        const res = service.execute('processApplicant', {name: 'John', income: 'High'});
        assert.equal(res.result, 'High');
    });
});

// Utilities:

// filter out '@schema' recursively from the returned model for comparison
const filterSchema = (obj) => {
    if (Array.isArray(obj)) {
        return obj.map(filterSchema);
    } else if (obj && typeof obj === 'object') {
        const newObj = {};
        for (const key in obj) {
            if (key !== '@schema') {
                newObj[key] = filterSchema(obj[key]);
            }
        }
        return newObj;
    } else {
        return obj;
    }
};

describe('Model Metadata', () => {
    it('retrieves model with metadata (@version, @model_name) via get("*")', () => {
        const metadataModel = {
            '@version': '1.2.3',
            '@model_name': 'MetadataTest',
            a: 1
        };
        const metaService = new wasm.DecisionService(metadataModel);
        const retrieved = metaService.get('*');

        assert.equal(retrieved['@version'], '1.2.3');
        assert.equal(retrieved['@model_name'], 'MetadataTest');
        assert.equal(retrieved.a, 1);
        assert.ok(retrieved['@schema']);
        assert.equal(retrieved['@schema'].a, 'number');
    });

    it('retrieves model with invocations via get("*")', () => {
        const invocationModel = {
            myFunc: {
                '@type': 'function',
                '@parameters': {x: 'number'},
                return: 'x + 1'
            },
            callFunc: {
                '@type': 'invocation',
                '@method': 'myFunc',
                '@arguments': [10]
            }
        };
        const invService = new wasm.DecisionService(invocationModel);
        const retrieved = invService.get('*');

        assert.deepEqual(retrieved.callFunc, {
            '@type': 'invocation',
            '@method': 'myFunc',
            '@arguments': [10]
        });

        // Invocation return types ARE in @schema after linking
        assert.equal(retrieved['@schema'].callFunc, 'number');
    });
});