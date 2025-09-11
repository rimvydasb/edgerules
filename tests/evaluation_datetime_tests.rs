#[test]
fn datetime_primitives_and_components() {
    // Date components
    assert_eq!(crate::eval_value("value : date(\"2017-05-03\").year"), "2017");
    assert_eq!(crate::eval_value("value : date(\"2017-05-03\").month"), "5");
    assert_eq!(crate::eval_value("value : date(\"2017-05-03\").day"), "3");

    // Time components
    assert_eq!(crate::eval_value("value : time(\"12:00:00\").second"), "0");
    assert_eq!(crate::eval_value("value : time(\"13:10:30\").minute"), "10");

    // Datetime components and .time extraction
    assert_eq!(
        crate::eval_value("value : datetime(\"2016-12-09T15:37:00\").month"),
        "12"
    );
    assert_eq!(
        crate::eval_value("value : datetime(\"2016-12-09T15:37:00\").hour"),
        "15"
    );
    // .time string form
    assert_eq!(
        crate::eval_value("value : datetime(\"2016-12-09T15:37:00\").time"),
        "15:37:00.0"
    );

    // Weekday (ISO Monday=1) for 2018-10-11 is Thursday=4
    assert_eq!(crate::eval_value("value : date(\"2018-10-11\").weekday"), "4");

    // all date component elements
    assert_eq!(
        crate::eval_lines_field(
            &[
                "d1 : date(\"2017-05-03\")",
                "y : d1.year",
                "m : d1.month",
                "d : d1.day",
                "result : [y,m,d]",
            ],
            "result"
        ),
        "[2017, 5, 3]"
    );

    // complex browsing and type inference
    assert_eq!(
        crate::eval_lines_field(
            &[
                "d1 : date(\"2017-05-03\")",
                "d2 : date(\"2018-12-31\")",
                "y : d1.year",
                "plusOneYear : y + 1 - d2.year",
            ],
            "plusOneYear"
        ),
        "0"
    );
}

#[test]
fn datetime_comparisons_and_arithmetic() {
    // Comparisons
    assert_eq!(
        crate::eval_field(
            "value : date(\"2017-05-03\") < date(\"2017-05-04\")",
            "value"
        ),
        "true"
    );

    // date - date => P1D
    assert_eq!(
        crate::eval_field(
            "value : date(\"2017-05-04\") - date(\"2017-05-03\")",
            "value"
        ),
        "P1D"
    );

    // date + duration days
    assert_eq!(
        crate::eval_value("value : date(\"2017-05-03\") + duration(\"P1D\")"),
        "2017-05-04"
    );

    // clamp day-of-month
    assert_eq!(
        crate::eval_value("value : date(\"2018-01-31\") + duration(\"P1M\")"),
        "2018-02-28"
    );

    // time - time => PT1H10M30S
    assert_eq!(
        crate::eval_value("value : time(\"13:10:30\") - time(\"12:00:00\")"),
        "PT1H10M30S"
    );

    // datetime + PT23H
    assert_eq!(
        crate::eval_value("value : datetime(\"2016-12-09T15:37:00\") + duration(\"PT23H\")"),
        "2016-12-10 14:37:00.0"
    );
}

#[test]
fn datetime_additional_functions() {
    assert_eq!(
        crate::eval_value("value : dayOfWeek(date(\"2025-09-02\"))"),
        "'Tuesday'"
    );
    assert_eq!(
        crate::eval_value("value : monthOfYear(date(\"2025-09-02\"))"),
        "'September'"
    );
    assert_eq!(
        crate::eval_value("value : lastDayOfMonth(date(\"2025-02-10\"))"),
        "28"
    );
}
mod utilities;
pub use utilities::*;
