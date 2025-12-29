import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';
import { installBuiltins } from './builtins.js';

const evaluate = (source) => Function(`\"use strict\"; return (${source});`)();

const toJsSupported =
    typeof wasm.DecisionEngine?.printModelJs === 'function' && typeof wasm.DecisionEngine?.printExpressionJs === 'function';
const describeToJs = toJsSupported ? describe : describe.skip;

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
                { name: 'John', age: 30, income: 2000, expense: 500 },
                { name: 'Jane', age: 17, income: 900, expense: 400 },
            ],
        });

        assert.equal(decision.result, 'DECLINE');
    });
});
