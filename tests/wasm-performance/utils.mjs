const nowNs = () => process.hrtime.bigint();
const nsToMs = ns => Number(ns) / 1e6;

const parseNumber = (value, fallback) => {
    if (value === undefined) {
        return fallback;
    }
    const parsed = Number(value);
    return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback;
};

export const parseBenchmarkArgs = (argv, defaults = {}) => {
    const {
        defaultIterations = 1000,
        defaultWarmup = 10,
    } = defaults;
    return {
        iterations: parseNumber(argv[2], defaultIterations),
        warmup: parseNumber(argv[3], defaultWarmup),
    };
};

export const runSingle = (run, label = 'Single run result') => {
    const t0 = nowNs();
    const result = run();
    const t1 = nowNs();
    console.log(`${label}:`, result, `(${nsToMs(t1 - t0).toFixed(3)} ms)`);
    return result;
};

export const warmupLoop = (warmup, run) => {
    for (let i = 0; i < warmup; i++) {
        run();
    }
};

export const maybeTriggerGc = () => {
    if (globalThis.gc) {
        globalThis.gc();
    }
};

export const measureIterations = (iterations, run) => {
    const samplesMs = [];
    const tAll0 = nowNs();
    for (let i = 0; i < iterations; i++) {
        const t0 = nowNs();
        run();
        const t1 = nowNs();
        samplesMs.push(nsToMs(t1 - t0));
    }
    const tAll1 = nowNs();
    return { samplesMs, totalNs: tAll1 - tAll0 };
};

export const quantiles = samplesMs => {
    if (samplesMs.length === 0) {
        return {
            min: NaN,
            p50: NaN,
            p95: NaN,
            p99: NaN,
            max: NaN,
            avg: NaN,
        };
    }
    const samples = [...samplesMs].sort((a, b) => a - b);
    const pick = p => {
        const idx = Math.floor((samples.length - 1) * p);
        return samples[idx];
    };
    const sum = samples.reduce((total, value) => total + value, 0);
    return {
        min: samples[0],
        p50: pick(0.50),
        p95: pick(0.95),
        p99: pick(0.99),
        max: samples[samples.length - 1],
        avg: sum / samples.length,
    };
};

export const logPerformanceSummary = ({ iterations, warmup, samplesMs, totalNs }) => {
    const q = quantiles(samplesMs);
    console.log('\nIterations:', iterations, 'Warmup:', warmup);
    console.log('Total time:', nsToMs(totalNs).toFixed(3), 'ms');
    const tpsAvg = Number.isFinite(q.avg) && q.avg > 0 ? 1000 / q.avg : NaN;
    console.log('TPS (based on avg):', Number.isFinite(tpsAvg) ? tpsAvg.toFixed(2) : 'NaN');
    console.log('Per-iter (ms):');
    console.table({
        min: Number.isFinite(q.min) ? q.min.toFixed(3) : 'NaN',
        p50: Number.isFinite(q.p50) ? q.p50.toFixed(3) : 'NaN',
        p95: Number.isFinite(q.p95) ? q.p95.toFixed(3) : 'NaN',
        p99: Number.isFinite(q.p99) ? q.p99.toFixed(3) : 'NaN',
        avg: Number.isFinite(q.avg) ? q.avg.toFixed(3) : 'NaN',
        max: Number.isFinite(q.max) ? q.max.toFixed(3) : 'NaN',
    });
};

export const logMemoryUsage = () => {
    const stats = process.memoryUsage();
    console.log('Memory (MB): rss=', (stats.rss / 1e6).toFixed(1),
        ' heapUsed=', (stats.heapUsed / 1e6).toFixed(1),
        ' ext=', (stats.external / 1e6).toFixed(1));
};
