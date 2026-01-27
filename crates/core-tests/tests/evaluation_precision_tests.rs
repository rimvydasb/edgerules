mod utilities;
use utilities::*;

#[test]
fn test_complex_discount_calculation() {
    init_logger();
    let code = r#"
        {
            func calculateDiscount(productType): {
                availableDiscounts: [0.20, 0.10, 0.16]
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
    "#;

    // The assertions were previously failing with floating point errors.
    // e.g. discount: 0.15000000000000002
    // Now they should be precise.
    
    // Note: rust_decimal's default Display might output "0.25" or "0.2500..." depending on scale. 
    // Usually it trims trailing zeros if created from strings, but let's see. 
    // 0.20 + 0.05 = 0.25. 
    // 0.10 + 0.05 = 0.15. 
    // 0.16 + 0.05 = 0.21.

    assert_eq!(
        inline(eval_field(code, "discount1")),
        inline("discount1: {campaign: 'SUMMER_SALE' discount: 0.25}")
    );
    assert_eq!(
        inline(eval_field(code, "discount2")),
        inline("discount2: {campaign: 'SUMMER_SALE' discount: 0.15}")
    );
    assert_eq!(
        inline(eval_field(code, "discount3")),
        inline("discount3: {campaign: 'SUMMER_SALE' discount: 0.21}")
    );
}

#[test]
fn test_simple_addition_precision() {
    init_logger();
    let code = r#"
    {
        a: 0.1
        b: 0.2
        c: a + b
    }
    "#;
    // 0.1 + 0.2 = 0.3. In f64 this is 0.30000000000000004
    assert_eq!(
        inline(eval_field(code, "c")),
        inline("0.3")
    );
}
