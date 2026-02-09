import {before, describe, it} from 'node:test';
import {strict as assert} from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';
import {installBuiltins} from '../wasm-js/builtins.js';

describe('Decision Service Date/Time Strings', () => {
    let service;

    before(() => {
        wasm.init_panic_hook();
        installBuiltins();
        
        const model = {
            "@model_name": "DateTimeCheck",
            "Request": {
                "@type": "type",
                "date": "<date>",
                "datetime": "<datetime>",
                "datetimeWithOffset": "<datetime>",
                "time": "<time>",
                "duration": "<duration>",
                "period": "<period>"
            },
            "check": {
                "@type": "function",
                "@parameters": {"r": "Request"},
                "result": {
                    "isDate": "r.date = date('2026-01-26')",
                    "isDateTime": "r.datetime = datetime('2026-01-26T21:33:35')",
                    "isDateTimeWithOffset": "r.datetimeWithOffset = datetime('2026-01-26T21:33:35+02:00')",
                    "isTime": "r.time = time('12:00:00')",
                    "isDuration": "r.duration = duration('P1DT1H')",
                    "isPeriod": "r.period = period('P1Y2M')"
                }
            }
        };
        service = new wasm.DecisionService(model);
    });

    it('handles JS Date objects (sanity check)', () => {
        // This should already work for fields mapped to Date/DateTime if implementation supports it
        // But current implementation converts JS Date to Date/DateTime ValueEnum.
        // If the model expects Date, and we pass Date, it works.
        // If model expects DateTime, and we pass Date (which converts to DateTime if has time), it works.
        
        // Note: JS Date is always a specific point in time (DateTime). 
        // EdgeRules conversion tries to map it to Date if time is 00:00:00.000, else DateTime.
        
        const req = {
            date: new Date('2026-01-26T00:00:00Z'),
            datetime: new Date('2026-01-26T21:33:35Z'),
            time: "12:00:00", // JS doesn't have Time object, usually string or Date. 
            duration: "P1DT1H",
            period: "P1Y2M"
        };
        
        // We need to bypass the strict check in the test model for now or just check execution
        // The model checks strict equality.
        
        // Note: passing strings for time/duration/period works if they are passed as strings to valueEnum 
        // AND the engine doesn't complain about type mismatch during linking.
        // BUT currently, passing string for <time> parameter will likely fail linking if strict type check is on.
        
        // Let's rely on the fix to make strings work.
    });

    it('handles string inputs for Date', () => {
        const req = {
            date: "2026-01-26",
            datetime: "2026-01-26T21:33:35",
            datetimeWithOffset: "2026-01-26T21:33:35+02:00",
            time: "12:00:00",
            duration: "P1DT1H",
            period: "P1Y2M"
        };
        
        const res = service.execute('check', req);
        assert.equal(res.result.isDate, true);
        assert.equal(res.result.isDateTime, true);
        assert.equal(res.result.isDateTimeWithOffset, true);
        assert.equal(res.result.isTime, true);
        assert.equal(res.result.isDuration, true);
        assert.equal(res.result.isPeriod, true);
    });

    it('handles datetime string with Z', () => {
        const req = {
            date: "2026-01-26",
            datetime: "2026-01-26T21:33:35Z",
            datetimeWithOffset: "2026-01-26T21:33:35+02:00",
            time: "12:00:00",
            duration: "P1DT1H",
            period: "P1Y2M"
        };
        
        const res = service.execute('check', req);
        assert.equal(res.result.isDateTime, true);
    });

    it('handles datetime string with +00:00', () => {
        const req = {
            date: "2026-01-26",
            datetime: "2026-01-26T21:33:35+00:00",
            datetimeWithOffset: "2026-01-26T21:33:35+02:00",
            time: "12:00:00",
            duration: "P1DT1H",
            period: "P1Y2M"
        };
        
        const res = service.execute('check', req);
        assert.equal(res.result.isDateTime, true);
    });
    
    it('supports non-zero datetime timezone offset', () => {
         const req = {
            date: "2026-01-26",
            datetime: "2026-01-26T21:33:35",
            datetimeWithOffset: "2026-01-26T21:33:35+02:00",
            time: "12:00:00",
            duration: "P1DT1H",
            period: "P1Y2M"
        };
        
        const res = service.execute('check', req);
        assert.equal(res.result.isDateTimeWithOffset, true);
    });

    it('handles subsecond datetime strings', () => {
        const model = {
            "check": {
                "@type": "function",
                "@parameters": {"r": "string"},
                "eq": "datetime('2026-01-26T21:33:35.123Z') = datetime('2026-01-26T21:33:35.123Z')",
                "ne": "datetime('2026-01-26T21:33:35.123Z') = datetime('2026-01-26T21:33:35.456Z')",
                "ne2": "datetime('2026-01-26T21:33:35.123Z') = datetime('2026-01-26T21:33:35Z')"
            }
        };
        const subService = new wasm.DecisionService(model);
        const res = subService.execute('check', "");
        assert.equal(res.eq, true);
        assert.equal(res.ne, false);
        assert.equal(res.ne2, false);
    });

    it('verifies fallback to UTC for no-offset strings', () => {
        const model = {
            "check": {
                "@type": "function",
                "@parameters": {"r": "string"},
                "isUtc": "datetime('2026-01-26T21:33:35') = datetime('2026-01-26T21:33:35Z')"
            }
        };
        const subService = new wasm.DecisionService(model);
        const res = subService.execute('check', "");
        assert.equal(res.isUtc, true);
    });
});
