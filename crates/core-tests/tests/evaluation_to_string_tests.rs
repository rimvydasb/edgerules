mod utilities;
pub use utilities::*;

#[test]
fn test_to_string_for_various_types() {
    assert_expression_value("toString(\"a\")", "'a'");
    assert_expression_value("toString(toString(toString(\"a\")))", "'a'");

    assert_expression_value("toString(true)", "'true'");
    assert_expression_value("toString(false)", "'false'");

    assert_expression_value("toString([1, 2, 3])", "'[1, 2, 3]'");
    assert_expression_value("toString([\"a\", \"b\"])", "'['a', 'b']'");

    assert_expression_value("toString(1..5)", "'1..5'");

    assert_expression_value("toString(date('2025-09-02'))", "'2025-09-02'");
    assert_expression_value("toString(time('13:45:07'))", "'13:45:07'");
    assert_expression_value("toString(datetime('2025-09-02T13:45:07'))", "'2025-09-02T13:45:07'");

    assert_expression_value("toString(duration('P2DT3H4M5S'))", "'P2DT3H4M5S'");
    assert_expression_value("toString(duration('PT90M'))", "'PT1H30M'");
    assert_expression_value("toString(duration('-PT3H'))", "'-PT3H'");
    assert_expression_value("toString(period('P1Y2M'))", "'P1Y2M'");
    assert_expression_value("toString(period('-P3D'))", "'-P3D'");
}
