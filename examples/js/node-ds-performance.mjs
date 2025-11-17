import wasm from '../../target/pkg-node/edge_rules.js';
import { argv } from 'node:process';
import {
    parseBenchmarkArgs,
    runSingle,
    warmupLoop,
    maybeTriggerGc,
    measureIterations,
    logPerformanceSummary,
    logMemoryUsage,
} from './utils.mjs';

wasm.init_panic_hook();

// ------ build portable decision-service model ------
const PORTABLE_MODEL = {
    '@version': 1,
    '@model_name': 'LoanApplicationDecisions',

    // Type definitions
    Application: {
        '@type': 'type',
        applicationDate: '<datetime>',
        applicants: '<Applicant[]>',
        propertyValue: '<number>',
        loanAmount: '<number>'
    },

    Applicant: {
        '@type': 'type',
        name: '<string>',
        birthDate: '<date>',
        income: '<number>',
        expense: '<number>'
    },

    // Nested function: applicantDecisions
    applicantDecisions: {
        '@type': 'function',
        '@parameters': {
            applicant: 'Applicant',
            applicationRecord: null
        },

        // Nested function: CreditScore
        CreditScore: {
            '@type': 'function',
            '@parameters': {
                age: 'number',
                income: 'number'
            },
            bins: [
                {name: '"AGE_BIN"', score: 20, condition: 'if age <= 25 then score else 0'},
                {name: '"AGE_BIN"', score: 30, condition: 'if age > 25 then score else 0'},
                {name: '"INC_BIN"', score: 30, condition: 'if income >= 1500 then score else 0'}
            ],
            totalScore: 'sum(for bin in bins return bin.condition)'
        },

        // Nested function: EligibilityDecision
        EligibilityDecision: {
            '@type': 'function',
            '@parameters': {
                applicantRecord: null,
                creditScore: null
            },
            rules: [
                {name: '"INC_CHECK"', rule: 'applicantRecord.data.income > applicantRecord.data.expense * 2'},
                {name: '"MIN_INCOM"', rule: 'applicantRecord.data.income > 1000'},
                {name: '"AGE_CHECK"', rule: 'applicantRecord.age >= 18'},
                {name: '"SCREDIT_S"', rule: 'creditScore.totalScore > 10'}
            ],
            firedRules: 'for invalid in rules[rule = false] return invalid.name',
            status: 'if count(firedRules) = 0 then "ELIGIBLE" else "INELIGIBLE"'
        },

        // Applicant record
        applicantRecord: {
            data: 'applicant',
            age: 'calendarDiff(applicant.birthDate, applicationRecord.data.applicationDate.date).years'
        },

        // Applicant decisions
        creditScore: 'CreditScore(applicantRecord.age, applicantRecord.data.income)',
        eligibility: 'EligibilityDecision(applicantRecord, creditScore)'
    },

    // Main decision function
    applicationDecisions: {
        '@type': 'function',
        '@parameters': {
            application: 'Application'
        },

        // Application record
        applicationRecord: {
            data: 'application'
        },

        // Application decisions
        applicantDecisions: 'for app in application.applicants return applicantDecisions(app, applicationRecord)',
        finalDecision: 'if (count(applicantDecisions[eligibility.status="INELIGIBLE"]) > 0) then "DECLINE" else "APPROVE"',

        result: 'finalDecision'
    }
};

// create decision service in WASM
wasm.create_decision_service(PORTABLE_MODEL);

// Example input data (passed as runtime request)
const REQUEST = {
    applicationDate: new Date("2025-01-01T15:43:56"),
    propertyValue: 100000,
    loanAmount: 80000,
    applicants: [
        {
            name: "John Doe",
            birthDate: new Date("1990-06-05"),
            income: 1100,
            expense: 600
        },
        {
            name: "Jane Doe",
            birthDate: new Date("1992-05-01"),
            income: 1500,
            expense: 300
        },
        {
            name: "Alababa",
            birthDate: new Date("1991-05-01"),
            income: 200,
            expense: 10
        },
        {
            name: "Alababa",
            birthDate: new Date("1992-09-01"),
            income: 786,
            expense: 786
        },
        {
            name: "Alababa",
            birthDate: new Date("1982-05-01"),
            income: 786,
            expense: 786786
        },
        {
            name: "Alababa",
            birthDate: new Date("1912-05-01"),
            income: 786,
            expense: 786786
        }
    ]
};

// ------ CLI args ------
// Usage: node node-ds-performance.mjs [iterations] [warmup]
// defaults: iterations=1000, warmup=10
const { iterations, warmup } = parseBenchmarkArgs(argv, {
    defaultIterations: 1000,
    defaultWarmup: 10,
});

const run = () => wasm.execute_decision_service('applicationDecisions', REQUEST);

// ------ single run (sanity) ------
runSingle(run);

// ------ warmup ------
warmupLoop(warmup, run);

// Optionally trigger GC between warmup and measured runs for more stability
maybeTriggerGc(); // run with: node --expose-gc node-ds-performance.mjs

// ------ measured loop (measure only execute_decision_service) ------
const { samplesMs, totalNs } = measureIterations(iterations, run);

// ------ stats ------
logPerformanceSummary({ iterations, warmup, samplesMs, totalNs });

// Optional memory snapshot
logMemoryUsage();
