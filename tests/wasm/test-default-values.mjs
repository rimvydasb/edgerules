import {describe, it} from 'node:test';
import {strict as assert} from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

const MODEL_CODE = `
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

describe('Default Values in Types', () => {
    describe('Portable format tests', () => {
        it('applies default values when casting empty object', () => {
            const result = wasm.DecisionEngine.evaluate(MODEL_CODE, 'customerFromEmpty');
            assert.equal(result.income, 0);
            assert.equal(result.isActive, true);
            assert.equal(result.category, "STD");
        });

        it('applies default values for missing fields only', () => {
            const result = wasm.DecisionEngine.evaluate(MODEL_CODE, 'customerPartial');
            assert.equal(result.name, "John");
            assert.equal(result.income, 0);
            assert.equal(result.isActive, true);
        });

        it('does not override existing values', () => {
            const result = wasm.DecisionEngine.evaluate(MODEL_CODE, 'customerFull');
            assert.equal(result.name, "Jane");
            assert.equal(result.income, 5000);
            assert.equal(result.isActive, false);
            assert.equal(result.category, "VIP");
        });

        it('works with nested objects', () => {
            const result = wasm.DecisionEngine.evaluate(MODEL_CODE, 'loanFromEmpty');
            assert.equal(result.amount, 1000);
            assert.equal(result.customer.income, 0);
            assert.equal(result.customer.isActive, true);
        });

    });

    describe('User Type Modification', () => {
        let service;

        before(() => {
            wasm.init_panic_hook();
            const model = {
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
            service = new wasm.DecisionService(model);
        });

        it('modifies a user type definition by replacing it', () => {
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
});
