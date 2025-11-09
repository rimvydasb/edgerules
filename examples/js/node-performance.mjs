// benchmark.mjs
import wasm from '../../target/pkg-node/edge_rules.js';
import { argv, exit } from 'node:process';

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
// defaults: iterations=100, warmup=10
const iterations = Number(argv[2] ?? 1000);
const warmup = Number(argv[3] ?? 10);

// ------ helpers ------
const nowNs = () => process.hrtime.bigint();
const nsToMs = ns => Number(ns) / 1e6;

function quantiles(ms) {
    const a = [...ms].sort((x, y) => x - y);
    const pick = p => {
        if (a.length === 0) return NaN;
        const idx = Math.floor((a.length - 1) * p);
        return a[idx];
    };
    const sum = a.reduce((s, x) => s + x, 0);
    return {
        min: a[0],
        p50: pick(0.50),
        p95: pick(0.95),
        p99: pick(0.99),
        max: a[a.length - 1],
        avg: sum / a.length,
    };
}

// ------ single run (also proves it works) ------
{
    const t0 = nowNs();
    const result = wasm.evaluate_field(PROGRAM, "applicationResponse");
    const t1 = nowNs();
    console.log('Single run result:', result, `(${nsToMs(t1 - t0).toFixed(3)} ms)`);
}

// ------ warmup ------
for (let i = 0; i < warmup; i++) {
    wasm.evaluate_field(PROGRAM, "applicationResponse");
}

// Optionally trigger GC between warmup and measured runs for more stability
if (global.gc) { global.gc(); } // run with: node --expose-gc benchmark.mjs

// ------ measured loop ------
const samplesMs = [];
const tAll0 = nowNs();
for (let i = 0; i < iterations; i++) {
    const t0 = nowNs();
    const _ = wasm.evaluate_field(PROGRAM, "applicationResponse");
    const t1 = nowNs();
    samplesMs.push(nsToMs(t1 - t0));
}
const tAll1 = nowNs();

// ------ stats ------
const q = quantiles(samplesMs);
console.log('\nIterations:', iterations, 'Warmup:', warmup);
console.log('Total time:', nsToMs(tAll1 - tAll0).toFixed(3), 'ms');
const tpsAvg = q.avg > 0 ? 1000 / q.avg : NaN;
console.log('TPS (based on avg):', Number.isFinite(tpsAvg) ? tpsAvg.toFixed(2) : 'NaN');
console.log('Per-iter (ms):');
console.table({
    min: q.min.toFixed(3),
    p50: q.p50.toFixed(3),
    p95: q.p95.toFixed(3),
    p99: q.p99.toFixed(3),
    avg: q.avg.toFixed(3),
    max: q.max.toFixed(3),
});

// Optional memory snapshot
const m = process.memoryUsage();
console.log('Memory (MB): rss=', (m.rss/1e6).toFixed(1),
    ' heapUsed=', (m.heapUsed/1e6).toFixed(1),
    ' ext=', (m.external/1e6).toFixed(1));
