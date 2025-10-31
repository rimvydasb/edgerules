import wasm from '../../target/pkg-node/edge_rules.js';

// Node target initializes WASM synchronously on import; no init() function.
wasm.init_panic_hook();

console.log('evaluate_field:', wasm.evaluate_field('{ x : 1; y : x + 2 }', 'y'));

console.log('evaluate_expression:', wasm.evaluate_expression('2 + 3'));
const result = wasm.evaluate_expression(`regexReplace('Hello 123 world 456', '\\d+', 'X', 'g')`);
console.log('regexReplace:', result);
if (result !== 'Hello X world X') {
    throw new Error('regexReplace failed: ' + result);
}

const split = wasm.evaluate_expression(`regexSplit('one   two\tthree', '\\s+')`);
console.log('regexSplit:', split);
if (!Array.isArray(split) || split.join(',') !== 'one,two,three') {
    throw new Error('regexSplit failed: ' + JSON.stringify(split));
}

const b64 = wasm.evaluate_expression(`toBase64('FEEL')`);
console.log('toBase64:', b64);
if (b64 !== 'RkVFTA==') {
    throw new Error('toBase64 failed: ' + b64);
}

const from = wasm.evaluate_expression(`fromBase64('RkVFTA==')`);
console.log('fromBase64:', from);
if (from !== 'FEEL') {
    throw new Error('fromBase64 failed: ' + from);
}

const methodResult = wasm.evaluate_method(
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
console.log('evaluate_method:', methodResult);
if (
    !methodResult ||
    methodResult.greeting !== 'Hello Ada' ||
    methodResult.total !== 45 ||
    methodResult.vip !== true
) {
    throw new Error('evaluate_method failed: ' + JSON.stringify(methodResult));
}

const arrayResult = wasm.evaluate_method(
    `{
    type BaselineType: { items : <number[]> };
    func interpolate(baseline: BaselineType) : {
       resultset : for x in baseline.items return x * 2
    }
  }`,
    'interpolate',
    {items: [1, 2, 3, 4, 5]},
);
console.log('evaluate_method (interpolate):', arrayResult);
if (arrayResult === null || !Array.isArray(arrayResult.resultset) || arrayResult.resultset.join(',') !== '2,4,6,8,10') {
    throw new Error('evaluate_method failed: ' + JSON.stringify(methodResult));
}

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
)
console.log('evaluate_method (calculateLoanOffer):', decisionServiceResponse);

if (
    decisionServiceResponse === null ||
    decisionServiceResponse.executionDatetime !== '2024-01-01' ||
    decisionServiceResponse.eligibleCalc !== true ||
    decisionServiceResponse.amount !== 20000 ||
    decisionServiceResponse.termInMonths !== 24 ||
    decisionServiceResponse.monthlyPaymentCalc !== 875 ||
    decisionServiceResponse.result.eligible !== true ||
    decisionServiceResponse.result.amount !== 20000 ||
    decisionServiceResponse.result.termInMonths !== 24 ||
    decisionServiceResponse.result.monthlyPayment !== 875
) {
    throw new Error('evaluate_method failed: ' + JSON.stringify(decisionServiceResponse));
}

const complexEval = wasm.evaluate_field(
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
`, "adults")
console.log('complexEval:', complexEval);

if (JSON.stringify(complexEval) != `{"result":[{"name":"Alice","age":30,"tags":["engineer","manager"]},{"name":"Charlie","age":22,"tags":[]}]}`) {
    throw new Error('complexEval failed: ' + JSON.stringify(complexEval));
}

const eval_all = wasm.evaluate_all(`
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
console.log('eval_all:', eval_all);