import {before, describe, it} from 'node:test';
import {strict as assert} from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

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

describe('DecisionService Rename', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    const getError = (fn) => {
        try {
            fn();
        } catch (e) {
            return e;
        }
        assert.fail('Expected function to throw an error');
    };

    it('renames a root field', () => {
        const service = new wasm.DecisionService({
            "foo": 10
        });

        assert.strictEqual(service.get("foo"), 10);

        service.rename("foo", "bar");

        const err = getError(() => service.get("foo"));
        assert.match(err.message || err, /Entry 'foo' not found/);

        assert.strictEqual(service.get("bar"), 10);

        assert.deepEqual(service.get("*"), {
            "bar": 10
        });
    });

    it('renames a nested field', () => {
        const service = new wasm.DecisionService({
            "applicant": {
                "age": 30
            }
        });

        assert.strictEqual(service.get("applicant.age"), 30);

        service.rename("applicant.age", "applicant.years");

        const err = getError(() => service.get("applicant.age"));
        assert.match(err.message || err, /Entry 'applicant.age' not found/);

        assert.strictEqual(service.get("applicant.years"), 30);
    });

    it('renames a function', () => {
        const service = new wasm.DecisionService({
            "calc": {
                "@type": "function",
                "@parameters": {"x": "number"},
                "res": "x * 2"
            }
        });

        const res1 = service.execute("calc", 5);
        assert.strictEqual(res1.res, 10);

        service.rename("calc", "calculate");

        const err = getError(() => service.execute("calc", 5));
        const msg = err.message || err;
        assert.ok(/Entry 'calc' not found/.test(msg) || /Function 'calc.*' not found/.test(msg), `Unexpected error: ${msg}`);

        const res2 = service.execute("calculate", 5);
        assert.strictEqual(res2.res, 10);

        assert.deepEqual(service.get("*"), {
            "calculate": {
                "@type": "function",
                "@parameters": {"x": "number"},
                "res": "x * 2"
            }
        });
    });

    it('renames a nested function', () => {
        const service = new wasm.DecisionService({
            "utils": {
                "nestedFunc": {
                    "@type": "function",
                    "@parameters": {"x": "number"},
                    "res": "x * 2"
                }
            }
        });

        // Verify initial state
        const initial = service.get("utils.nestedFunc");
        assert.ok(initial);

        // Rename
        service.rename("utils.nestedFunc", "utils.renamedFunc");

        // Verify old name gone
        const err = getError(() => service.get("utils.nestedFunc"));
        assert.match(err.message || err, /Entry 'utils.nestedFunc' not found/);

        // Verify new name present
        const renamed = service.get("utils.renamedFunc");
        assert.ok(renamed);
    });

    it('renames a type', () => {
        const service = new wasm.DecisionService({
            "MyType": {
                "@type": "type",
                "f": "<string>"
            }
        });

        const t1 = portableToObject(service.get("MyType"));
        assert.ok(t1.f);

        service.rename("MyType", "YourType");

        const err = getError(() => service.get("MyType"));
        assert.match(err.message || err, /Entry 'MyType' not found/);

        const t2 = portableToObject(service.get("YourType"));
        assert.ok(t2.f);
    });

    it('renames a nested type', () => {
        const service = new wasm.DecisionService({
            "Domain": {
                "MyType": {
                    "@type": "type",
                    "f": "<string>"
                }
            }
        });

        assert.ok(service.get("Domain.MyType"));

        service.rename("Domain.MyType", "Domain.YourType");

        const err = getError(() => service.get("Domain.MyType"));
        assert.match(err.message || err, /Entry 'Domain.MyType' not found/);

        assert.ok(service.get("Domain.YourType"));
    });

    it('renames an invocation', () => {
        const service = new wasm.DecisionService({
            "calc": {
                "@type": "function",
                "@parameters": {"x": "number"},
                "res": "x"
            },
            "call": {
                "@type": "invocation",
                "@method": "calc",
                "@arguments": [10]
            }
        });

        assert.ok(service.get("call"));

        service.rename("call", "invoke");

        const err = getError(() => service.get("call"));
        assert.match(err.message || err, /Entry 'call' not found/);

        assert.ok(service.get("invoke"));

        assert.deepEqual(service.getType("*"), {
            invoke: {
                res: 'number'
            }
        });
    });

    it('renames a context variable', () => {
        const service = new wasm.DecisionService({
            user: {
                firstName: "'John'",
                lastName: "'Doe'"
            }
        });

        assert.strictEqual(service.get("user.firstName"), "'John'");

        service.rename("user.firstName", "user.givenName");

        const err = getError(() => service.get("user.firstName"));
        assert.match(err.message || err, /Entry 'user.firstName' not found/);

        assert.strictEqual(service.get("user.givenName"), "'John'");
    });

    it('renames a context variable in root', () => {
        const service = new wasm.DecisionService({
            user: {
                firstName: "'John'"
            }
        });

        service.rename("user", "customer");

        const err = getError(() => service.get("user"));
        assert.match(err.message || err, /Entry 'user' not found/);

        assert.strictEqual(service.get("customer.firstName"), "'John'");
    });

    describe('DecisionService.rename Exceptions', () => {
        let service;

        before(() => {
            wasm.init_panic_hook();
            const model = {
                a: 10,
                b: 20,
                nested: {
                    x: 1,
                    y: 2
                }
            };
            service = new wasm.DecisionService(model);
        });

        const getError = (fn) => {
            try {
                fn();
            } catch (e) {
                return e;
            }
            assert.fail('Expected function to throw an error');
        };

        it('throws if old path does not exist', () => {
            const error = getError(() => service.rename('nonexistent', 'exists'));
            assert.match(error.message, /Entry 'nonexistent' not found/);
        });

        it('throws if new name conflicts with existing sibling', () => {
            const error = getError(() => service.rename('a', 'b'));
            assert.match(error.message, /Duplicate field 'b'/);
        });

        it('throws if new name is empty', () => {
            const error = getError(() => service.rename('a', ' '));
            assert.match(error.message, /Field path is empty/);
        });

        it('throws if renaming to different context (root vs nested)', () => {
            const error = getError(() => service.rename('a', 'nested.z'));
            assert.match(error.message, /Renaming must be within the same context/);
        });

        it('throws if renaming to different context (nested vs nested)', () => {
            const error = getError(() => service.rename('nested.x', 'nested.child.x'));
            assert.match(error.message, /Renaming must be within the same context/);
        });

        it('throws if old path nested parent not found', () => {
            const error = getError(() => service.rename('ghost.child', 'ghost.child2'));
            assert.match(error.message, /Context 'ghost' not found/);
        });

        it('throws if new name conflicts in nested context', () => {
            const error = getError(() => service.rename('nested.x', 'nested.y'));
            assert.match(error.message, /Duplicate field 'y'/);
        });
    });
});