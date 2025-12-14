import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

describe('Basic Evaluation', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    it('evaluate_field simple arithmetic', () => {
        const result = wasm.evaluate_field('{ x : 1; y : x + 2 }', 'y');
        assert.deepStrictEqual(result, 3);
    });

    it('evaluate_expression simple arithmetic', () => {
        const result = wasm.evaluate_expression('2 + 3');
        assert.deepStrictEqual(result, 5);
    });

    it('regexReplace', () => {
        const result = wasm.evaluate_expression(`regexReplace('Hello 123 world 456', '\\d+', 'X', 'g')`);
        assert.deepStrictEqual(result, 'Hello X world X');
    });

    it('regexSplit', () => {
        const split = wasm.evaluate_expression(`regexSplit('one   two\tthree', '\\s+')`);
        assert.deepStrictEqual(split, ['one', 'two', 'three']);
    });

    it('base64 functions', () => {
        const b64 = wasm.evaluate_expression(`toBase64('FEEL')`);
        assert.deepStrictEqual(b64, 'RkVFTA==');

        const from = wasm.evaluate_expression(`fromBase64('RkVFTA==')`);
        assert.deepStrictEqual(from, 'FEEL');
    });

    it('evaluate_method', () => {
        const result = wasm.evaluate_method(
            `{
            func personalize(customer) : {
              greeting: 'Hello ' + customer.name;
              total: customer.subtotal + customer.tax;
              vip: customer.vip
            }
          }`,
            'personalize',
            [{
                name: 'Ada',
                subtotal: 40,
                tax: 5,
                vip: true
            }],
        );
        assert.deepStrictEqual(result, {
            greeting: 'Hello Ada',
            total: 45,
            vip: true
        });
    });

    it('evaluate_method with array mapping', () => {
        const result = wasm.evaluate_method(
            `{
            type BaselineType: { items : <number[]> };
            func interpolate(baseline: BaselineType) : {
               resultset : for x in baseline.items return x * 2
            }
          }`,
            'interpolate',
            { items: [1, 2, 3, 4, 5] },
        );
        assert.deepStrictEqual(result, { resultset: [2, 4, 6, 8, 10] });
    });

    it('complex decision service logic', () => {
        const decisionServiceResponse = wasm.evaluate_method(
            `
            {
                type Customer: {name: <string>; birthdate: <date>; income: <number>}
                type Applicant: {customer: <Customer>; requestedAmount: <number>; termInMonths: <number>}
                type LoanOffer: {eligible: <boolean>; amount: <number>; termInMonths: <number>; monthlyPayment: <number>}

                func calculateLoanOffer(applicant: Applicant): {
                    // NOTE: placeholder not supported yet, so set a concrete date
                    executionDatetime: date('2024-01-01')
                
                    eligibleCalc: executionDatetime >= applicant.customer.birthdate + duration('P6570D');
                    amount: applicant.requestedAmount;
                    termInMonths: applicant.termInMonths;
                    monthlyPaymentCalc: (applicant.requestedAmount * (1 + (if applicant.customer.income > 5000 then 0.05 else 0.1))) / applicant.termInMonths
                    result: {
                        eligible: eligibleCalc;
                        amount: applicant.requestedAmount;
                        termInMonths: applicant.termInMonths;
                        monthlyPayment: monthlyPaymentCalc
                    }
                }
            }
        `, "calculateLoanOffer", {
                customer: {
                    name: 'Alice',
                    birthdate: new Date('2001-01-01'),
                    income: 6000
                },
                requestedAmount: 20000,
                termInMonths: 24
            }
        );

        assert.strictEqual(decisionServiceResponse.executionDatetime, '2024-01-01');
        assert.strictEqual(decisionServiceResponse.eligibleCalc, true);
        assert.strictEqual(decisionServiceResponse.amount, 20000);
        assert.strictEqual(decisionServiceResponse.termInMonths, 24);
        assert.strictEqual(decisionServiceResponse.monthlyPaymentCalc, 875);
        assert.deepStrictEqual(decisionServiceResponse.result, {
            eligible: true,
            amount: 20000,
            termInMonths: 24,
            monthlyPayment: 875
        });
    });

    it('complex evaluation with filter', () => {
        const result = wasm.evaluate_field(
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
        const result = wasm.evaluate_all(`
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

