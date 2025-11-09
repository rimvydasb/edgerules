import wasm from '../../target/pkg-node/edge_rules.js';

wasm.init_panic_hook();

const DECISION_FUNCTION = {
    '@type': 'function',
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

const expect = (condition, message) => {
    if (!condition) {
        throw new Error(message);
    }
};

console.log('Creating decision service example (mutable controller)...');
const modelSnapshot = portableToObject(wasm.create_decision_service(PORTABLE_MODEL));
console.log('Initial portable snapshot entries:', Object.keys(modelSnapshot));

const executeLoan = (request) => {
    const response = wasm.execute_decision_service('decideLoanOffer', request);
    console.log('execute_decision_service(decideLoanOffer)', request, '=>', response);
    return response;
};

const baselineRequest = {amount: 18000, creditScore: 760, vip: false};
const baseline = executeLoan(baselineRequest);
expect(baseline.result.approved === true, 'Baseline request should be approved');
expect(baseline.result.approvedAmount === 18000, 'Baseline amount should not be capped');

const cappedRequest = {amount: 50000, creditScore: 700, vip: false};
const capped = executeLoan(cappedRequest);
expect(capped.result.approved === true, 'High amount request should still be approved');
expect(capped.result.approvedAmount === 20000, 'High amount request must be capped by maxAmount');

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
console.log('Reading updated decision function:', limitRead.settings.maxAmount);
expect(
    limitRead.settings.maxAmount === 35000,
    'Reading maxAmount should match updated value'
);

const auditNotes = portableToObject(
    wasm.set_to_decision_service_model('auditNote', "'Loan rules executed'")
);
console.log('Audit note stored:', auditNotes);
expect(
    typeof auditNotes === 'string' && auditNotes.includes('Loan rules executed'),
    'Audit note should be echoed back as a string'
);

const removalResult = wasm.remove_from_decision_service_model('auditNote');
expect(removalResult === true, 'Removal result should be true');

let removalErrored = false;
try {
    wasm.get_from_decision_service_model('auditNote');
} catch (err) {
    removalErrored = true;
    console.log('Confirmed removal of audit note:', err.message);
}
expect(removalErrored, 'Removed entry should not be readable');

const snapshotAfterEdits = portableToObject(wasm.get_decision_service_model());
expect(
    snapshotAfterEdits.decideLoanOffer.settings.maxAmount === 35000,
    'Snapshot should include updated maxAmount'
);

const postUpdateRequest = {amount: 42000, creditScore: 700, vip: false};
const postUpdate = executeLoan(postUpdateRequest);
expect(postUpdate.result.approvedAmount === 35000, 'Post-update amount should be limited to 35000');

const vipRequest = {amount: 15000, creditScore: 620, vip: true};
const vipDecision = executeLoan(vipRequest);
expect(vipDecision.result.approved === true, 'VIP request should be approved via vip bonus');
expect(
    vipDecision.result.approvedAmount === 15000,
    'VIP request should receive the requested amount'
);
expect(
    vipDecision.result.apr < baseline.result.apr,
    'VIP discount should yield a lower APR than baseline'
);

console.log('Decision service WASM example completed without errors.');
