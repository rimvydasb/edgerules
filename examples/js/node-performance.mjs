// benchmark.mjs
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

const PROGRAM = `
{
    // Business Object Model Entities:

    type Application: {
        applicationDate: <datetime>;
        applicants: <Applicant[]>;
        propertyValue: <number>;
        loanAmount: <number>;
    }
    type Applicant: {
        name: <string>;
        birthDate: <date>;
        income: <number>;
        expense: <number>;
    }

    // Applicant Level Decisions
  
    func applicantDecisions(applicant: Applicant, applicationRecord): {        

         func CreditScore(age, income): {
            bins: [
                {name: "AGE_BIN"; score: 20; condition: if age <= 25 then score else 0}
                {name: "AGE_BIN"; score: 30; condition: if age > 25 then score else 0}
                {name: "INC_BIN"; score: 30; condition: if income >= 1500 then score else 0}                
            ]
            totalScore: sum(for bin in bins return bin.condition)
        }
      
        func EligibilityDecision(applicantRecord, creditScore): {
            rules: [
                {name: "INC_CHECK"; rule: applicantRecord.data.income > applicantRecord.data.expense * 2}
                {name: "MIN_INCOM"; rule: applicantRecord.data.income > 1000}
                {name: "AGE_CHECK"; rule: applicantRecord.age >= 18}
                {name: "SCREDIT_S"; rule: creditScore.totalScore > 10}
            ]
            firedRules: for invalid in rules[rule = false] return invalid.name
            status: if count(rules) = 0 then "ELIGIBLE" else "INELIGIBLE"
        }

        // Applicant Record
  
        applicantRecord: {
            data: applicant
            age: calendarDiff(applicant.birthDate, applicationRecord.data.applicationDate.date).years
        }
        
        // Applicant Decisions
        
        creditScore: CreditScore(12,1000)
        eligibility: EligibilityDecision(applicantRecord, creditScore)
    }

    // Application Level Decisions

    func applicationDecisions(application: Application): {

        // Application Record
      
        applicationRecord: {
            data: application            
        }
        
        // Application Decisions
        
        applicantDecisions: for app in application.applicants return applicantDecisions(app, applicationRecord)
        finalDecision: if (count(applicantDecisions[eligibility.status="INELIGIBLE"]) > 0) then "DECLINE" else "APPROVE"
    }

    // Example Input Data

    applicationResponse: applicationDecisions({ 
        applicationDate: datetime("2025-01-01T15:43:56")
        propertyValue: 100000
        loanAmount: 80000
        applicants: [
            {
                name: "John Doe"
                birthDate: date("1990-06-05")
                income: 1100
                expense: 600
            },
            {
                name: "Jane Doe"
                birthDate: date("1992-05-01")
                income: 1500
                expense: 300
            },
            {
                name: "Alababa"
                birthDate: date("1991-05-01")
                income: 200
                expense: 10
            },
            {
                name: "Alababa"
                birthDate: date("1992-09-01")
                income: 786
                expense: 786
            },
            {
                name: "Alababa"
                birthDate: date("1982-05-01")
                income: 786
                expense: 786786
            },
            {
                name: "Alababa"
                birthDate: date("1912-05-01")
                income: 786
                expense: 786786
            }
        ]
    }).finalDecision
}
`;

// ------ CLI args ------
// Usage: node benchmark.mjs [iterations] [warmup]
// defaults: iterations=1000, warmup=10
const { iterations, warmup } = parseBenchmarkArgs(argv, {
    defaultIterations: 1000,
    defaultWarmup: 10,
});

const run = () => wasm.evaluate_field(PROGRAM, "applicationResponse");

// ------ single run (also proves it works) ------
runSingle(run);

// ------ warmup ------
warmupLoop(warmup, run);

// Optionally trigger GC between warmup and measured runs for more stability
maybeTriggerGc(); // run with: node --expose-gc benchmark.mjs

// ------ measured loop ------
const { samplesMs, totalNs } = measureIterations(iterations, run);

// ------ stats ------
logPerformanceSummary({ iterations, warmup, samplesMs, totalNs });

// Optional memory snapshot
logMemoryUsage();
