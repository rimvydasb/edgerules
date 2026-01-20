import { describe, it, before } from 'node:test';
import { strict as assert } from 'node:assert';
import wasm from '../../target/pkg-node/edge_rules.js';

describe('Math Evaluation', () => {
    before(() => {
        wasm.init_panic_hook();
    });

    it('test_complex_discount_calculation', () => {
        const code = `
        {
            func calculateDiscount(productType): {
                availableDiscounts: [0.20, 0.10, 0.11]
                activeCampaignDiscount: 0.05
                activeCampaign: "SUMMER_SALE"
                baseDiscount: availableDiscounts[productType - 1]
                return: {
                    campaign: activeCampaign
                    discount: baseDiscount + activeCampaignDiscount
                }
            }
            discount1: calculateDiscount(1)
            discount2: calculateDiscount(2)
            discount3: calculateDiscount(3)
        }
        `;

        const discount1 = wasm.DecisionEngine.evaluate(code, "discount1");
        assert.deepStrictEqual(discount1, {
            campaign: 'SUMMER_SALE',
            discount: 0.25
        });

        const discount2 = wasm.DecisionEngine.evaluate(code, "discount2");
        assert.deepStrictEqual(discount2, {
            campaign: 'SUMMER_SALE',
            discount: 0.15000000000000002
        });

        const discount3 = wasm.DecisionEngine.evaluate(code, "discount3");
        assert.deepStrictEqual(discount3, {
            campaign: 'SUMMER_SALE',
            discount: 0.16
        });
    });
});
