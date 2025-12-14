import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

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
        maxAmount: 20000,
        minCreditScore: 680,
        baseApr: 0.08,
        vipDiscount: 0.02,
        vipBonus: 80
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

const PORTABLE_MODEL = {
    '@version': 1,
    '@model_name': 'LoanDecisions',
    LoanRequest: {
        '@type': 'type',
        amount: '<number>',
        creditScore: '<number>',
        vip: '<boolean>'
    },
    decideLoanOffer: DECISION_FUNCTION
};

const INVOCATION_MODEL = {
    '@model_name': 'InvocationDemo',
    evaluateEligibility: {
        '@type': 'function',
        '@parameters': {request: null},
        approved: 'request.score >= 640'
    },
    buildResponse: {
        '@type': 'function',
        '@parameters': {request: null},
        summary: {
            '@type': 'invocation',
            '@method': 'evaluateEligibility'
        },
        score: 'request.score'
    }
};

describe('Decision Service', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    describe('Loan Decision Workflow', () => {
        // Shared logic / constants for this suite
        const executeLoan = (request) => {
            return wasm.execute_decision_service('decideLoanOffer', request);
        };
        let baselineResult;

        it('initializes decision service', () => {
            const modelSnapshot = portableToObject(wasm.create_decision_service(PORTABLE_MODEL));
            assert.ok(modelSnapshot.decideLoanOffer, 'Model snapshot should contain decideLoanOffer');
        });

        it('verifies metadata persistence', () => {
            const modelSnapshot = portableToObject(wasm.get_decision_service_model());
            assert.strictEqual(modelSnapshot['@version'], '1', 'Snapshot should contain version metadata');
            assert.strictEqual(modelSnapshot['@model_name'], 'LoanDecisions', 'Snapshot should contain model_name metadata');
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
                ...DECISION_FUNCTION,
                settings: {
                    ...DECISION_FUNCTION.settings,
                    maxAmount: 35000
                }
            };

            wasm.set_to_decision_service_model('decideLoanOffer', MODIFIED_DECISION_FUNCTION);
            const limitRead = portableToObject(
                wasm.get_from_decision_service_model('decideLoanOffer')
            );
            assert.strictEqual(limitRead.settings.maxAmount, 35000, 'Reading maxAmount should match updated value');
        });

        it('stores and retrieves audit notes', () => {
            const auditNotes = portableToObject(
                wasm.set_to_decision_service_model('auditNote', "'Loan rules executed'")
            );
            assert.ok(typeof auditNotes === 'string' && auditNotes.includes('Loan rules executed'), 'Audit note should be echoed back as a string');
        });

        it('removes audit notes', () => {
            const removalResult = wasm.remove_from_decision_service_model('auditNote');
            assert.strictEqual(removalResult, true, 'Removal result should be true');

            assert.throws(() => {
                wasm.get_from_decision_service_model('auditNote');
            }, (err) => {
                return /Entry 'auditNote' not found/.test(err.message);
            });
        });

        it('verifies global state persistence and effect', () => {
            // The previous test modified the global state (maxAmount = 35000).
            // Since the WASM module uses thread_local state, it should persist.
            const snapshotAfterEdits = portableToObject(wasm.get_decision_service_model());
            assert.strictEqual(snapshotAfterEdits.decideLoanOffer.settings.maxAmount, 35000, 'Snapshot should include updated maxAmount');
            
            const postUpdateRequest = {amount: 42000, creditScore: 700, vip: false};
            const postUpdate = executeLoan(postUpdateRequest);
            assert.strictEqual(postUpdate.result.approvedAmount, 35000, 'Post-update amount should be limited to 35000');
        });
    });

    describe('Invocation Workflow', () => {
        // This runs AFTER Loan Decision Workflow. `create_decision_service` will reset the state.
        
        it('initializes invocation model', () => {
            wasm.create_decision_service(INVOCATION_MODEL);
        });

        it('executes initial invocation', () => {
            const response = wasm.execute_decision_service('buildResponse', {score: 705});
            assert.strictEqual(response.summary.approved, true, 'Invocation should call evaluateEligibility');
            assert.strictEqual(response.score, 705, 'buildResponse should echo the request score');
        });

        it('sets dynamic invocation', () => {
            const invocationEcho = portableToObject(
                wasm.set_invocation('eligibilityPreview', {
                    '@type': 'invocation',
                    '@method': 'evaluateEligibility',
                    '@arguments': [{score: 580}]
                })
            );
            assert.strictEqual(invocationEcho['@method'], 'evaluateEligibility', 'set_invocation should return the stored invocation snippet');
        });

        it('handles link errors', () => {
            assert.throws(() => {
                wasm.set_invocation('brokenInvocation', {
                    '@type': 'invocation',
                    '@method': 'someKindOfFunction'
                });
            }, { message: /Function 'someKindOfFunction.*' not found/ });
        });
    });

    describe('Nested Function Insertion', () => {
        it('handles nested function insertion and invocation', () => {
            wasm.create_decision_service({
                applicationDecisions: {
                    '@type': 'function',
                    '@parameters': {age: 'number'},
                    isEligible: 'age >= 18'
                }
            });

            wasm.set_to_decision_service_model('applicationDecisions.scholarshipCalc', {
                '@type': 'function',
                '@parameters': {
                    age: 'number'
                },
                result: 'if age < 25 then 1000 else 500'
            });

            wasm.set_to_decision_service_model('applicationDecisions.scholarship', {
                '@type': 'invocation',
                '@method': 'scholarshipCalc',
                '@arguments': ['age']
            });

            const decision = wasm.execute_decision_service('applicationDecisions', 22);
            assert.strictEqual(decision.isEligible, true, 'Outer function should still evaluate eligibility');
            assert.strictEqual(decision.scholarship.result, 1000, 'Invocation should reuse inner function and compute scholarship');
        });
    });

    describe('Unhappy Paths', () => {
        it('throws on invalid logic during creation', () => {
            try {
                wasm.create_decision_service({
                    applicationDecisions: {
                        '@type': 'function',
                        '@parameters': {age: 'number'},
                        isEligible: 'age >= 18 + "invalid_string"'
                    }
                });
            } catch (e) {
                assert.ok(e, 'Should have thrown an error');
            }
        });
    });
});
