import {before, describe, it} from 'node:test';
import {strict as assert} from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The following tests will read portable folder and load JSON file as Project to test.
 * EdgeRules code is under source, tests to be executed are under tests.
 * If EdgeRules project has `decisionService`, the decision service must be created, otherwise DecisionEngine can be used.
 */
describe('DecisionService Portable Format', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    const runPortableTest = (filename) => {
        const filePath = path.join('tests/wasm/portable', filename);
        const project = JSON.parse(fs.readFileSync(filePath, 'utf8'));

        if (project.decisionService && project.decisionService.entryPoints) {
            // Decision Service mode
            const service = new wasm.DecisionService(project.source);
            // In these examples, we use the first entry point for all tests
            const entryPoint = project.decisionService.entryPoints[0].methodName;

            for (const test of project.tests) {
                const result = service.execute(entryPoint, test.input);
                for (const [key, expectedValue] of Object.entries(test.expected)) {
                    assert.deepEqual(result[key], expectedValue, `Test ${test.id} failed for key ${key}`);
                }
            }
        } else {
            // Decision Engine mode (Workbook)
            for (const test of project.tests) {
                const input = { ...project.source, ...test.input };
                const result = wasm.DecisionEngine.evaluate(input);
                for (const [key, expectedValue] of Object.entries(test.expected)) {
                    assert.deepEqual(result[key], expectedValue, `Test ${test.id} failed for key ${key}`);
                }
            }
        }
    };

    it('testing example-commerce.json', () => {
        runPortableTest('example-commerce.json');
    });

    it('example-insurance.json', () => {
        runPortableTest('example-insurance.json');
    });

    it('example-loan.json', () => {
        runPortableTest('example-loan.json');
    });
});

