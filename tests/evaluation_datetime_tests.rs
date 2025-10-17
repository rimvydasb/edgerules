#[test]
fn datetime_primitives_and_components() {
    // Date components
    assert_value!("date('2017-05-03').year", "2017");
    assert_value!("date('2017-05-03').month", "5");
    assert_value!("date('2017-05-03').day", "3");

    // Time components
    assert_value!("time('12:00:00').second", "0");
    assert_value!("time('13:10:30').minute", "10");

    // Datetime components and .time extraction
    assert_value!("datetime('2016-12-09T15:37:00').month", "12");
    assert_value!("datetime('2016-12-09T15:37:00').hour", "15");
    // .time string form
    assert_value!("datetime('2016-12-09T15:37:00').time", "15:37:00.0");

    // Weekday (ISO Monday=1) for 2018-10-11 is Thursday=4
    assert_value!("date('2018-10-11').weekday", "4");

    // all date component elements
    assert_eq!(
        eval_field(
            r#"
            {
                d1: date('2017-05-03');
                y: d1.year;
                m: d1.month;
                d: d1.day;
                result: [y,m,d]
            }
            "#
            .trim(),
            "result"
        ),
        "[2017, 5, 3]"
    );

    // complex browsing and type inference
    assert_eq!(
        eval_field(
            r#"
            {
                d1: date('2017-05-03');
                d2: date('2018-12-31');
                y: d1.year;
                plusOneYear: y + 1 - d2.year
            }
            "#
            .trim(),
            "plusOneYear"
        ),
        "0"
    );
}

#[test]
fn datetime_comparisons_and_arithmetic() {
    // Comparisons
    assert_eq!(
        crate::eval_field("value: date('2017-05-03') < date('2017-05-04')", "value"),
        "true"
    );

    // date - date => P1D
    assert_eq!(
        crate::eval_field("value: date('2017-05-04') - date('2017-05-03')", "value"),
        "P1D"
    );

    // date + duration days
    assert_value!("date('2017-05-03') + duration(\"P1D\")", "2017-05-04");

    // clamp day-of-month
    assert_value!("date('2018-01-31') + duration(\"P1M\")", "2018-02-28");

    // time - time => PT1H10M30S
    assert_value!("time('13:10:30') - time('12:00:00')", "PT1H10M30S");

    // datetime + PT23H
    assert_value!(
        "datetime('2016-12-09T15:37:00') + duration(\"PT23H\")",
        "2016-12-10 14:37:00.0"
    );
}

#[test]
fn datetime_additional_functions() {
    assert_value!("dayOfWeek(date('2025-09-02'))", "'Tuesday'");
    assert_value!("monthOfYear(date('2025-09-02'))", "'September'");
    assert_value!("lastDayOfMonth(date('2025-02-10'))", "28");
}

#[test]
fn date_comparator_operators() {
    assert_value!("date('2020-01-01') = date('2020-01-01')", "true");
    assert_value!("date('2020-01-01') <> date('2020-01-02')", "true");
    assert_value!("date('2020-01-01') < date('2020-01-02')", "true");
    assert_value!("date('2020-01-02') <= date('2020-01-02')", "true");
    assert_value!("date('2020-01-03') > date('2020-01-02')", "true");
    assert_value!("date('2020-01-03') >= date('2020-01-03')", "true");
    assert_value!("date('2020-01-01') > date('2020-01-02')", "false");
}

#[test]
fn datetime_comparator_operators() {
    assert_value!(
        "datetime('2020-01-01T10:00:00') = datetime('2020-01-01T10:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T10:00:00') <> datetime('2020-01-01T12:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T08:00:00') < datetime('2020-01-01T09:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T09:00:00') <= datetime('2020-01-01T09:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T11:00:00') > datetime('2020-01-01T09:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T11:00:00') >= datetime('2020-01-01T11:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T11:00:00') < datetime('2020-01-01T09:00:00')",
        "false"
    );
}

#[test]
fn time_comparator_operators() {
    assert_value!("time('09:00:00') = time('09:00:00')", "true");
    assert_value!("time('09:00:00') <> time('10:00:00')", "true");
    assert_value!("time('08:30:00') < time('09:00:00')", "true");
    assert_value!("time('09:00:00') <= time('09:00:00')", "true");
    assert_value!("time('10:00:00') > time('09:30:00')", "true");
    assert_value!("time('10:00:00') >= time('10:00:00')", "true");
    assert_value!("time('08:30:00') > time('09:00:00')", "false");
}

#[test]
fn duration_comparator_operators() {
    assert_value!("duration(\"P3D\") = duration(\"P3D\")", "true");
    assert_value!("duration(\"P3D\") <> duration(\"P4D\")", "true");
    assert_value!("duration(\"P1D\") < duration(\"P2D\")", "true");
    assert_value!("duration(\"P1D\") <= duration(\"P1D\")", "true");
    assert_value!("duration(\"P2D\") > duration(\"P1D\")", "true");
    assert_value!("duration(\"P2D\") >= duration(\"P3D\")", "false");
    assert_value!("duration(\"P1Y\") < duration(\"P2Y\")", "true");
    assert_value!("duration(\"P18M\") >= duration(\"P1Y6M\")", "true");
    assert_value!("duration(\"P1Y\") >= duration(\"P13M\")", "false");
}

#[test]
fn addition_with_identical_temporal_types_is_rejected() {
    link_error_contains(
        "value: date('2020-01-01') + date('2020-01-02')",
        &["Operator '+'"],
    );
    link_error_contains(
        "value: datetime('2020-01-01T00:00:00') + datetime('2020-01-02T00:00:00')",
        &["Operator '+'"],
    );
    link_error_contains(
        "value: time('12:00:00') + time('01:00:00')",
        &["Operator '+'"],
    );
}

#[test]
fn comparator_type_mismatch_is_rejected() {
    link_error_contains(
        "value: date('2020-01-01') = time('12:00:00')",
        &["Comparator"],
    );
    link_error_contains(
        "value: datetime('2020-01-01T00:00:00') = time('12:00:00')",
        &["Comparator"],
    );
    link_error_contains(
        "value: duration(\"PT1H\") = time('12:00:00')",
        &["Comparator"],
    );
}

#[test]
fn duration_duration_arithmetic() {
    assert_value!("duration(\"P1Y\") + duration(\"P6M\")", "P1Y6M");
    assert_value!("duration(\"P2Y\") - duration(\"P6M\")", "P1Y6M");
    assert_value!("duration(\"PT4H\") + duration(\"PT30M\")", "PT4H30M");
    assert_value!("duration(\"P2DT3H\") - duration(\"PT4H\")", "P1DT23H");
    assert_value!("duration(\"PT30M\") - duration(\"PT45M\")", "-PT15M");
    assert_value!("duration(\"P1Y\") + duration(\"PT12H\")", "P1YT12H");
    assert_value!("duration(\"PT12H\") + duration(\"P1Y\")", "P1YT12H");
    assert_value!("duration(\"P1Y2MT3H4M5S\")", "P1Y2MT3H4M5S");
    assert_value!(
        "duration(\"P1Y6M\") + duration(\"P5DT4H20M10S\")",
        "P1Y6M5DT4H20M10S"
    );
}

#[test]
fn double_quoted_literals_are_supported() {
    assert_value!("date(\"2020-01-05\")", "2020-01-05");
    assert_value!("datetime(\"2020-01-05T01:02:03\")", "2020-01-051:02:03.0");
}

#[test]
fn duration_arithmetic_with_comparators() {
    assert_value!(
        "date('2020-01-01') + duration(\"P1D\") = date('2020-01-02')",
        "true"
    );
    assert_value!(
        "date('2020-01-01') + duration(\"P1D\") <> date('2020-01-03')",
        "true"
    );
    assert_value!(
        "date('2020-01-01') + duration(\"P1D\") < date('2020-01-04')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T12:00:00') + duration(\"PT6H\") <= datetime('2020-01-01T18:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-02T00:00:00') - duration(\"PT30M\") > datetime('2019-12-31T23:00:00')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-02T00:00:00') - duration(\"PT30M\") >= datetime('2020-01-01T23:30:00')",
        "true"
    );
}

#[test]
fn date_duration_subtraction() {
    assert_value!("date('2024-03-10') - duration(\"P2D\")", "2024-03-08");
    assert_value!("date('2024-03-10') - duration(\"P1M\")", "2024-02-10");
}

#[test]
fn time_duration_arithmetic() {
    assert_value!("time('12:30:15') + duration(\"PT1H\")", "13:30:15.0");
    assert_value!("time('12:30:15') - duration(\"PT1H\")", "11:30:15.0");
    assert_value!("time('12:30:15') - time('10:15:10')", "PT2H15M5S");
}

#[test]
fn combined_duration_arithmetic() {
    assert_value!("duration(\"P1Y6M5DT4H20M10S\")", "P1Y6M5DT4H20M10S");
    assert_value!(
        "duration(\"P1Y6M5DT4H20M10S\") - duration(\"P6M\")",
        "P1Y5DT4H20M10S"
    );
    assert_value!(
        "date('2020-01-01') + duration(\"P1Y6M5DT4H20M10S\")",
        "2021-07-06"
    );
    assert_value!(
        "datetime('2020-01-01T00:00:00') + duration(\"P1Y6M5DT4H20M10S\")",
        "2021-07-064:20:10.0"
    );
    assert_value!(
        "datetime('2021-07-06T04:20:10') - duration(\"P1Y6M5DT4H20M10S\")",
        "2020-01-010:00:00.0"
    );
}

#[test]
fn date_datetime_interactions() {
    assert_value!(
        "value: date('2024-03-10') + datetime('1999-07-04T05:30:00')",
        "2024-03-10 5:30:00.0"
    );
    assert_value!(
        "value: datetime('1999-07-04T05:30:00') + date('2024-03-10')",
        "2024-03-10 5:30:00.0"
    );
    assert_value!(
        "date('2024-03-10') - datetime('2024-03-09T22:45:00')",
        "PT1H15M"
    );
    assert_value!(
        "datetime('2024-03-10T22:45:00') - date('2024-03-09')",
        "P1DT22H45M"
    );
}

#[test]
fn date_datetime_comparator_operators() {
    assert_value!(
        "date('2020-01-01') = datetime('2020-01-01T00:00:00')",
        "true"
    );
    assert_value!(
        "date('2020-01-01') <> datetime('2020-01-02T00:00:00')",
        "true"
    );
    assert_value!(
        "date('2020-01-01') < datetime('2020-01-01T12:00:00')",
        "true"
    );
    assert_value!(
        "date('2020-01-01') <= datetime('2020-01-01T00:00:00')",
        "true"
    );
    assert_value!(
        "date('2020-01-02') > datetime('2020-01-01T23:59:59')",
        "true"
    );
    assert_value!(
        "date('2020-01-02') >= datetime('2020-01-02T00:00:00')",
        "true"
    );
    assert_value!(
        "date('2020-01-03') > datetime('2020-01-04T00:00:00')",
        "false"
    );
    assert_value!(
        "datetime('2020-01-01T12:00:00') > date('2020-01-01')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T00:00:00') = date('2020-01-01')",
        "true"
    );
    assert_value!(
        "datetime('2020-01-01T00:00:00') < date('2020-01-02')",
        "true"
    );
}

#[test]
fn comparator_if_else_blocks() {
    assert_value!(
        "if date('2020-01-05') >= date('2020-01-01') then 'after' else 'before'",
        "'after'"
    );
    assert_value!(
        "if datetime('2020-01-01T18:00:00') > datetime('2020-01-01T12:00:00') then 'evening' else 'noon'",
        "'evening'"
    );
    assert_value!(
        "if date('2020-01-01') + duration(\"P2D\") = date('2020-01-03') then 'match' else 'mismatch'",
        "'match'"
    );
    assert_value!(
        "if date('2020-01-01') + duration(\"P2D\") <> date('2020-01-04') then 'still match' else 'unexpected'",
        "'still match'"
    );
}

#[test]
fn duration_offset_age_gate_if_else() {
    assert_eq!(
        eval_field(
            r#"
            {
                executionDatetime: date('2024-01-01');
                applicantBirthdate: date('2005-01-01');
                eligible: if executionDatetime >= applicantBirthdate + duration("P6570D") then true else false
            }
            "#
            .trim(),
            "eligible"
        ),
        "true"
    );

    assert_eq!(
        eval_field(
            r#"
            {
                executionDatetime: date('2022-12-27');
                applicantBirthdate: date('2005-01-01');
                eligible: if executionDatetime >= applicantBirthdate + duration("P6570D") then true else false
            }
            "#
            .trim(),
            "eligible"
        ),
        "false"
    );
}

mod utilities;
pub use utilities::*;
