import {before, describe, it} from 'node:test';
import {strict as assert} from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';
import {installBuiltins} from '../wasm-js/builtins.js';

const evaluate = (source) => Function(`"use strict"; return (${source});`)();

const toJsSupported =
    typeof wasm.DecisionEngine?.printModelJs === 'function' && typeof wasm.DecisionEngine?.printExpressionJs === 'function';
const describeToJs = toJsSupported ? describe : describe.skip;

// Helper to convert JS Maps (which wasm-bindgen returns for HashMaps) to plain Objects recursively
// This matches the helper in examples/js/node-ds-demo.mjs
const portableToObject = (value) => {
    if (value instanceof Map) {
        const result = {};
        for (const [key, inner] of value.entries()) {
            result[key] = portableToObject(inner);
        }
        return result;
    }

    if (Array.isArray(value)) {
        return value.map(portableToObject);
    }

    return value;
};

const DECISION_FUNCTION = {
    '@type': 'function',
    '@description': 'Main decision function',
    '@parameters': {
        request: 'LoanRequest'
    },
    settings: {
        maxAmount: 20000, minCreditScore: 680, baseApr: 0.08, vipDiscount: 0.02, vipBonus: 80
    },
    requestedAmount: 'request.amount',
    maxFinance: 'min(request.amount, settings.maxAmount)',
    scoreValue: 'request.creditScore + (if request.vip then settings.vipBonus else 0)',
    eligible: 'scoreValue >= settings.minCreditScore',
    calculatedApr: `settings.baseApr - (if request.vip then settings.vipDiscount else 0) - (if request.creditScore >= 750 then 0.01 else 0)`,
    result: {
        approved: 'eligible',
        approvedAmount: 'if eligible then maxFinance else 0',
        apr: 'calculatedApr',
        approvalScore: 'scoreValue'
    }
};

const DECISION_FUNCTION_SCHEMA = {
    settings: {
        maxAmount: "number", minCreditScore: "number", baseApr: "number", vipDiscount: "number", vipBonus: "number"
    },
    requestedAmount: "number",
    maxFinance: "number",
    scoreValue: "number",
    eligible: "boolean",
    calculatedApr: "number",
    result: {approved: "boolean", approvedAmount: "number", apr: "number", approvalScore: "number"}
}

const PORTABLE_MODEL = {
    '@version': '1', '@model_name': 'LoanDecisions', LoanRequest: {
        '@type': 'type', amount: '<number>', creditScore: '<number>', vip: '<boolean>'
    }, LoanRequestAlias: {
        '@type': 'type', '@ref': '<LoanRequest>'
    }, decideLoanOffer: DECISION_FUNCTION
};

const MODEL_SCHEMA = {
    LoanRequest: {
        amount: "number",
        creditScore: "number",
        vip: "boolean"
    },
    LoanRequestAlias: {
        amount: "number",
        creditScore: "number",
        vip: "boolean"
    },
    decideLoanOffer: DECISION_FUNCTION_SCHEMA
}

const INVOCATION_MODEL = {
    '@model_name': 'InvocationDemo',
    InvocationRequest: {
        '@type': 'type',
        score: '<number>'
    },
    evaluateEligibility: {
        '@type': 'function',
        '@parameters': {request: null},
        approved: 'request.score >= 640'
    },
    buildResponse: {
        '@type': 'function',
        '@parameters': {request: 'InvocationRequest'},
        summary: {
            '@type': 'invocation',
            '@method': 'evaluateEligibility',
            '@arguments': ['request']
        },
        score: 'request.score'
    }
};

describeToJs('Decision service printing', () => {
    before(() => {
        wasm.init_panic_hook();
        installBuiltins();
    });

    it('prints and executes a decision service model', () => {
        const modelCode = `
        {
            type Applicant: { name: <string>; age: <number>; income: <number>; expense: <number> }
            type Application: { applicants: <Applicant[]> }
            func CreditScore(age: <number>, income: <number>): {
                totalScore: (if age <= 25 then 20 else 30) + (if income >= 1500 then 30 else 0)
            }
            func EligibilityDecision(applicantRecord, creditScore): {
                rules: [
                    { name: "INC_CHECK"; rule: applicantRecord.income > applicantRecord.expense * 2 }
                    { name: "AGE_CHECK"; rule: applicantRecord.age >= 18 }
                    { name: "SCREDIT_S"; rule: creditScore.totalScore > 10 }
                ]
                firedRules: for invalid in rules[rule = false] return invalid.name
                status: if count(firedRules) = 0 then "ELIGIBLE" else "INELIGIBLE"
            }
            func applicantDecisions(applicant: Applicant): {
                applicantRecord: applicant
                creditScore: CreditScore(applicant.age, applicant.income)
                eligibility: EligibilityDecision(applicantRecord, creditScore)
            }
            func applicationDecisions(application: Application): {
                applicantDecisions: for app in application.applicants return applicantDecisions(app)
                finalDecision: if (count(applicantDecisions[eligibility.status="INELIGIBLE"]) > 0) then "DECLINE" else "APPROVE"
                result: finalDecision
            }
        }
        `;

        const js = wasm.DecisionEngine.printModelJs(modelCode);
        const model = evaluate(js);

        const decision = model.applicationDecisions({
            applicants: [
                {name: 'John', age: 30, income: 2000, expense: 500},
                {name: 'Jane', age: 17, income: 900, expense: 400},
            ],
        });

        assert.equal(decision.result, 'DECLINE');
    });
});

describe('DecisionService CRUD', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    it('Array CRUD basic operations', () => {
        const model = {
            rules: [
                {id: 1, action: "'A'"},
                {id: 2, action: "'B'"},
                {id: 3, action: "'C'"}
            ],
            decision: 'rules[0].action'
        };

        const service = new wasm.DecisionService(model);

        // GET
        const first = service.get('rules[0]');
        assert.deepEqual(first, {id: 1, action: "'A'"});

        // SET Overwrite
        service.set('rules[1]', {id: 99, action: "'Z'"});
        const second = service.get('rules[1]');
        assert.deepEqual(second, {id: 99, action: "'Z'"});
        // Ensure no shift
        assert.deepEqual(service.get('rules[0]'), {id: 1, action: "'A'"});
        assert.deepEqual(service.get('rules[2]'), {id: 3, action: "'C'"});

        // SET Append
        service.set('rules[3]', {id: 4, action: "'D'"});
        assert.deepEqual(service.get('rules[3]'), {id: 4, action: "'D'"});

        // REMOVE
        // Remove index 1, index 2 & 3 should shift down
        service.remove('rules[1]');
        // Old rules[2] is now rules[1]
        assert.deepEqual(service.get('rules[1]'), {id: 3, action: "'C'"});
        // Old rules[3] is now rules[2]
        assert.deepEqual(service.get('rules[2]'), {id: 4, action: "'D'"});

        // Verify length (attempting to get index 3 should fail)
        assert.throws(() => service.get('rules[3]'), {message: /Index 3 out of bounds/});
    });
});

describe('Variable Library Complex Test', () => {
    let service;

    const inputData = {
        applicationDate: new Date("2025-01-01"),
        propertyValue: 100000,
        loanAmount: 80000,
        applicants: [
            {
                name: 'John Doe',
                birthDate: new Date("1990-06-05"),
                income: 1100,
                expense: 600
            },
            {
                name: 'Jane Doe',
                birthDate: new Date("1992-05-01"),
                income: 1500,
                expense: 300
            }
        ]
    };

    before(() => {
        wasm.init_panic_hook();
        const model = {
            "@model_name": "ApplicantCheck",
            "Applicant": {
                "@type": "type",
                "name": "<string>",
                "birthDate": "<date>",
                "income": "<number>",
                "expense": "<number>"
            },
            "Application": {
                "@type": "type",
                "applicationDate": "<datetime>",
                "applicants": "<Applicant[]>",
                "propertyValue": "<number>",
                "loanAmount": "<number>"
            },
            "applicantDecisions": {
                "@type": "function",
                "@parameters": {"applicant": "Applicant", "application": "Application"},
                "applicantRecord": {
                    "checkDate": "application.applicationDate",
                    "data": "applicant",
                    "age": "application.applicationDate - applicant.birthDate"
                },
                "eligibilityDecision": {
                    "@type": "function",
                    "@parameters": {"applicantRecord": null},
                    "rules": [
                        {
                            "name": "'INC_CHECK'",
                            "rule": "applicantRecord.data.income > applicantRecord.data.expense * 2"
                        },
                        {"name": "'MIN_INCOM'", "rule": "applicantRecord.data.income > 1000"},
                        {
                            "name": "'AGE_CHECK'",
                            "rule": "applicantRecord.data.birthDate + period('P18Y') <= applicantRecord.checkDate"
                        }
                    ],
                    "firedRules": "for invalid in rules[rule = false] return invalid.name",
                    "status": "if count(firedRules) = 0 then 'ELIGIBLE' else 'INELIGIBLE'"
                },
                "eligibility": "eligibilityDecision(applicantRecord)"
            },
            "applicationDecisions": {
                "@type": "function",
                "@parameters": {"application": "Application"},
                "applicationRecord": {
                    "data": "application",
                    "applicantsDecisions": "for app in application.applicants return applicantDecisions(app, application).eligibility"
                }
            }
        };
        service = new wasm.DecisionService(model);
    });

    const evalField = () => {
        // Execute applicationDecisions
        const res = service.execute('applicationDecisions', inputData);
        // Navigate to result
        return res.applicationRecord.applicantsDecisions;
    };

    it('evaluates initial state correctly', () => {
        const res = evalField();

        // John
        assert.equal(res[0].status, 'INELIGIBLE');
        assert.deepEqual(res[0].firedRules, ['INC_CHECK']);

        // Jane
        assert.deepEqual(res[1].firedRules, []);
        assert.equal(res[1].status, 'ELIGIBLE');
    });

    it('modifies rules via array CRUD', () => {
        // Path to rules: applicantDecisions.eligibilityDecision.rules

        // 1. Get current rule 0
        const rule0 = service.get('applicantDecisions.eligibilityDecision.rules[0]');
        assert.equal(rule0.name, "'INC_CHECK'");

        // 2. Modify rule 0 to be always true (income > 0)
        service.set('applicantDecisions.eligibilityDecision.rules[0]', {
            name: "'INC_CHECK'",
            rule: "applicantRecord.data.income > 0"
        });

        // 3. Evaluate again. John should now pass INC_CHECK.
        const res = evalField();
        assert.deepEqual(res[0].firedRules, []);
        assert.equal(res[0].status, 'ELIGIBLE');

        // 4. Add a new failing rule for everyone
        service.set('applicantDecisions.eligibilityDecision.rules[3]', {
            name: "'FAIL_ALL'",
            rule: "false"
        });

        const res2 = evalField();
        // Both should fail now
        assert.equal(res2[0].status, 'INELIGIBLE');
        assert.deepEqual(res2[0].firedRules, ['FAIL_ALL']);
        assert.equal(res2[1].status, 'INELIGIBLE');
        assert.deepEqual(res2[1].firedRules, ['FAIL_ALL']);

        // 5. Remove the failing rule (index 3)
        service.remove('applicantDecisions.eligibilityDecision.rules[3]');

        const res3 = evalField();
        // Back to ELIGIBLE
        assert.equal(res3[0].status, 'ELIGIBLE');
        assert.equal(res3[1].status, 'ELIGIBLE');
    });
});

describe('Decision Service', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    describe('Loan Decision Workflow', () => {
        // Shared logic / constants for this suite
        let service;
        const executeLoan = (request) => {
            return service.execute('decideLoanOffer', request);
        };
        let baselineResult;

        it('initializes decision service', () => {
            service = new wasm.DecisionService(PORTABLE_MODEL);
            const modelSnapshot = portableToObject(service.get('*'));
            assert.ok(modelSnapshot.decideLoanOffer, 'Model snapshot should contain decideLoanOffer');
        });

        it('verifies metadata persistence', () => {
            const modelSnapshot = portableToObject(service.get('*'));
            assert.strictEqual(modelSnapshot['@version'], '1', 'Snapshot should contain version metadata');
            assert.strictEqual(modelSnapshot['@model_name'], 'LoanDecisions', 'Snapshot should contain model_name metadata');

            // Function metadata
            assert.strictEqual(modelSnapshot.decideLoanOffer['@type'], 'function', 'Function should have @type');
            assert.ok(modelSnapshot.decideLoanOffer['@parameters'], 'Function should have @parameters');

            // Type definition metadata
            assert.strictEqual(modelSnapshot.LoanRequest['@type'], 'type', 'Type definition should have @type');

            // Type reference metadata (testing the fix)
            assert.strictEqual(modelSnapshot.LoanRequestAlias['@type'], 'type', 'Type alias should have @type');
            assert.strictEqual(modelSnapshot.LoanRequestAlias['@ref'], '<LoanRequest>', 'Type alias should have @ref');
        });

        it('evaluates baseline request', () => {
            const baselineRequest = {amount: 18000, creditScore: 760, vip: false};
            baselineResult = executeLoan(baselineRequest);
            assert.strictEqual(baselineResult.result.approved, true, 'Baseline request should be approved');
            assert.strictEqual(baselineResult.result.approvedAmount, 18000, 'Baseline amount should not be capped');
        });

        it('evaluates capped request', () => {
            const cappedRequest = {amount: 50000, creditScore: 700, vip: false};
            const capped = executeLoan(cappedRequest);
            assert.strictEqual(capped.result.approved, true, 'High amount request should still be approved');
            assert.strictEqual(capped.result.approvedAmount, 20000, 'High amount request must be capped by maxAmount');
        });

        it('modifies decision function', () => {
            const MODIFIED_DECISION_FUNCTION = {
                ...DECISION_FUNCTION, settings: {
                    ...DECISION_FUNCTION.settings, maxAmount: 35000
                }
            };

            service.set('decideLoanOffer', MODIFIED_DECISION_FUNCTION);
            const limitRead = portableToObject(service.get('decideLoanOffer'));
            assert.strictEqual(limitRead.settings.maxAmount, 35000, 'Reading maxAmount should match updated value');
        });

        it('stores and retrieves audit notes', () => {
            const auditNotes = portableToObject(service.set('auditNote', "'Loan rules executed'"));
            assert.ok(typeof auditNotes === 'string' && auditNotes.includes('Loan rules executed'), 'Audit note should be echoed back as a string');
        });

        it('removes audit notes', () => {
            const removalResult = service.remove('auditNote');
            assert.strictEqual(removalResult, true, 'Removal result should be true');

            assert.throws(() => {
                service.get('auditNote');
            }, (err) => {
                return /Entry 'auditNote' not found/.test(err.message);
            });
        });

        it('verifies global state persistence and effect', () => {
            // The previous test modified the global state (maxAmount = 35000).
            // Since the WASM module uses thread_local state, it should persist.
            const snapshotAfterEdits = portableToObject(service.get('*'));
            assert.strictEqual(snapshotAfterEdits.decideLoanOffer.settings.maxAmount, 35000, 'Snapshot should include updated maxAmount');

            const postUpdateRequest = {amount: 42000, creditScore: 700, vip: false};
            const postUpdate = executeLoan(postUpdateRequest);
            assert.strictEqual(postUpdate.result.approvedAmount, 35000, 'Post-update amount should be limited to 35000');
        });

        it('retrieves expression types', () => {
            // auditNote was removed in a previous test, so this should throw
            assert.throws(() => {
                service.getType('auditNote');
            }, (err) => {
                return /Entry 'auditNote' not found/.test(err.message);
            });

            // Verify wildcard type retrieval
            const actualWildcardSchema = service.getType('*');
            // console.log("Actual Wildcard Schema:", JSON.stringify(actualWildcardSchema, null, 2));
            assert.deepEqual(actualWildcardSchema, MODEL_SCHEMA);
        });
    });

    describe('Invocation Workflow', () => {
        // This runs AFTER Loan Decision Workflow. `create_decision_service` will reset the state.
        let service;

        it('initializes invocation model', () => {
            service = new wasm.DecisionService(INVOCATION_MODEL);
        });

        it('executes initial invocation', () => {
            const response = service.execute('buildResponse', {score: 705});
            assert.strictEqual(response.summary.approved, true, 'Invocation should call evaluateEligibility');
            assert.strictEqual(response.score, 705, 'buildResponse should echo the request score');
        });

        it('sets dynamic invocation', () => {
            const invocationEcho = portableToObject(service.set('eligibilityPreview', {
                '@type': 'invocation', '@method': 'evaluateEligibility', '@arguments': [{score: 580}]
            }));
            assert.strictEqual(invocationEcho['@method'], 'evaluateEligibility', 'set_to_decision_service_model should return the stored invocation snippet');
            assert.strictEqual(invocationEcho['@type'], 'invocation', 'set_to_decision_service_model should return the stored invocation type');
        });

        it('handles link errors', () => {
            assert.throws(() => {
                service.set('brokenInvocation', {
                    '@type': 'invocation', '@method': 'someKindOfFunction'
                });
            }, {message: /Function 'someKindOfFunction.*' not found/});
        });
    });

    describe('Nested Function Insertion', () => {
        it('handles nested function insertion and invocation', () => {
            const service = new wasm.DecisionService({
                applicationDecisions: {
                    '@type': 'function', '@parameters': {age: 'number'}, isEligible: 'age >= 18'
                }
            });

            service.set('applicationDecisions.scholarshipCalc', {
                '@type': 'function', '@parameters': {
                    age: 'number'
                }, result: 'if age < 25 then 1000 else 500'
            });

            service.set('applicationDecisions.scholarship', {
                '@type': 'invocation', '@method': 'scholarshipCalc', '@arguments': ['age']
            });

            const decision = service.execute('applicationDecisions', 22);
            assert.strictEqual(decision.isEligible, true, 'Outer function should still evaluate eligibility');
            assert.strictEqual(decision.scholarship.result, 1000, 'Invocation should reuse inner function and compute scholarship');
        });

        it('handles no-arg function invocation internally', () => {
            const service = new wasm.DecisionService({
                simpleCheck: {
                    '@type': 'function', '@parameters': {req: 'number'}, // Must have 1 arg to be executable by DecisionService
                    staticVal: {
                        '@type': 'invocation', '@method': 'getConstant', '@arguments': []
                    },
                    getConstant: {
                        '@type': 'function', '@parameters': {}, return: 42
                    },
                    return: {
                        val: 'staticVal'
                    }
                }
            });

            const result = service.execute('simpleCheck', 1);
            assert.strictEqual(result.val, 42, 'Should be able to invoke no-arg function with empty arguments array internally');
        });

        it('defaults to no-arg function invocation when arguments are missing', () => {
            const service = new wasm.DecisionService({
                simpleCheck: {
                    '@type': 'function', '@parameters': {req: 'number'},
                    staticVal: {
                        '@type': 'invocation', '@method': 'getConstant'
                        // No @arguments provided -> defaults to []
                    },
                    getConstant: {
                        '@type': 'function', '@parameters': {}, return: 100
                    },
                    return: {
                        val: 'staticVal'
                    }
                }
            });

            const result = service.execute('simpleCheck', 1);
            assert.strictEqual(result.val, 100, 'Should invoke no-arg function when arguments are missing');
        });
    });

    describe('Unhappy Paths', () => {
        it('throws on invalid logic during creation', () => {
            assert.throws(() => {
                new wasm.DecisionService({
                    applicationDecisions: {
                        // should trigger linking error due to invalid expression
                        '@type': 'function',
                        '@parameters': {age: '<number>'},
                        isEligible: 'age >= 18 + "invalid_string"'
                    }
                });
            });
        });
    });

    describe('EdgeRules Language DSL to Portable Conversion', () => {
        it('initializes from EdgeRules Language DSL and exports to EdgeRules Portable', () => {
            const code = `
            {
                taxRate: 0.21
                price: 100
                total: price * (1 + taxRate)
            }
            `;
            const service = new wasm.DecisionService(code);
            const portableModel = portableToObject(service.get('*'));

            assert.equal(portableModel.taxRate, 0.21);
            assert.equal(portableModel.price, 100);
            assert.ok(typeof portableModel.total === 'string', 'total should be an expression string');
            assert.ok(portableModel.total.includes('price * (1 + taxRate)'), 'total expression should be preserved');
        });

        it('unhappy test for EdgeRules Language DSL and exports to EdgeRules Portable', () => {
            const code = `
            {
                taxRate: 0.+....21
                price: 100
                total: price * (1 + taxRate)
            }
            `;
            assert.throws(() => {
                new wasm.DecisionService(code);
            }, (err) => {
                const msg = err && (err.message || String(err));
                return /assignment side is not complete/.test(msg);
            });
        });
    });
});
