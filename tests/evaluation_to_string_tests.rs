#[test]
fn test_to_string_for_various_types() {
    // string self
    assert_eq!(crate::eval_value("value : toString(\"a\")"), "'a'");
    // cascading toString
    assert_eq!(
        crate::eval_value("value : toString(toString(toString(\"a\")))"),
        "'a'"
    );

    // booleans
    assert_eq!(crate::eval_value("value : toString(true)"), "'true'");
    assert_eq!(crate::eval_value("value : toString(false)"), "'false'");

    // arrays
    assert_eq!(
        crate::eval_value("value : toString([1, 2, 3])"),
        "'[1, 2, 3]'"
    );
    // arrays of strings
    assert_eq!(
        crate::eval_value("value : toString([\"a\", \"b\"])"),
        "'['a', 'b']'"
    );

    // ranges
    assert_eq!(crate::eval_value("value : toString(1..5)"), "'1..5'");

    // date/time/datetime
    assert_eq!(
        crate::eval_value("value : toString(date('2025-09-02'))"),
        "'2025-09-02'"
    );
    // The time Display includes fractional seconds when zero as ".0"
    assert_eq!(
        crate::eval_value("value : toString(time('13:45:07'))"),
        "'13:45:07.0'"
    );
    // Note: datetime parser expects 'T' between date and time; display uses space
    assert_eq!(
        crate::eval_value("value : toString(datetime('2025-09-02T13:45:07'))"),
        "'2025-09-02 13:45:07.0'"
    );

    // durations
    assert_eq!(
        crate::eval_value("value : toString(duration('P1Y2M'))"),
        "'P1Y2M'"
    );
    assert_eq!(
        crate::eval_value("value : toString(duration('P2DT3H4M5S'))"),
        "'P2DT3H4M5S'"
    );
    assert_eq!(
        crate::eval_value("value : toString(duration('-P3D'))"),
        "'-P3D'"
    );
}

mod utilities;
pub use utilities::*;
