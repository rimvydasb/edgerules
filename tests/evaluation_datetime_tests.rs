#[test]
fn datetime_primitives_and_components() {
    assert_value!("date('2017-05-03').year", "2017");
    assert_value!("date('2017-05-03').month", "5");
    assert_value!("date('2017-05-03').day", "3");

    assert_value!("time('12:00:00').second", "0");
    assert_value!("time('13:10:30').minute", "10");

    assert_value!("datetime('2016-12-09T15:37:00').month", "12");
    assert_value!("datetime('2016-12-09T15:37:00').hour", "15");
    assert_value!("datetime('2016-12-09T15:37:00').time", "15:37:00.0");

    assert_value!("date('2018-10-11').weekday", "4");
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
}

#[test]
fn time_comparator_operators() {
    assert_value!("time('09:00:00') = time('09:00:00')", "true");
    assert_value!("time('09:00:00') <> time('10:00:00')", "true");
    assert_value!("time('08:30:00') < time('09:00:00')", "true");
    assert_value!("time('09:00:00') <= time('09:00:00')", "true");
    assert_value!("time('10:00:00') > time('09:30:00')", "true");
    assert_value!("time('10:00:00') >= time('10:00:00')", "true");
}

#[test]
fn duration_parsing_and_operations() {
    assert_value!("duration('PT90M')", "PT1H30M");
    assert_value!("duration('P2DT3H')", "P2DT3H");
    assert_value!("duration('PT4H') + duration('PT30M')", "PT4H30M");
    assert_value!("duration('PT30M') - duration('PT45M')", "-PT15M");
    assert_value!("duration('PT45S') + duration('PT15S')", "PT1M");
}

#[test]
fn period_parsing_and_operations() {
    assert_value!("period('P18M')", "P1Y6M");
    assert_value!("period('P1Y6M') + period('P2M')", "P1Y8M");
    assert_value!("period('P6M') - period('P2M')", "P4M");
    assert_value!("period('P10D') + period('P5D')", "P15D");
    assert_value!("period('P1Y6M') - period('P8M')", "P10M");
}

#[test]
fn duration_with_temporal_values() {
    assert_value!(
        "date('2017-05-03') + duration('P1D')",
        "2017-05-04 0:00:00.0"
    );
    assert_value!(
        "date('2017-05-03') - duration('P1D')",
        "2017-05-02 0:00:00.0"
    );
    assert_value!(
        "datetime('2016-12-09T15:37:00') + duration('PT23H')",
        "2016-12-10 14:37:00.0"
    );
    assert_value!(
        "datetime('2016-12-10T14:37:00') - duration('PT23H')",
        "2016-12-09 15:37:00.0"
    );
    assert_value!(
        "datetime('2020-01-02T00:00:00') - datetime('2020-01-01T08:00:00')",
        "PT16H"
    );
    assert_value!(
        "date('2020-01-02') - datetime('2020-01-01T12:00:00')",
        "PT12H"
    );
    assert_value!(
        "datetime('2020-01-01T12:00:00') - date('2020-01-01')",
        "PT12H"
    );
    assert_value!("time('13:10:30') - duration('PT1H10M30S')", "12:00:00.0");
    assert_value!("time('13:10:30') + duration('PT50S')", "13:11:20.0");
    assert_value!("time('13:10:30') - time('12:00:00')", "PT1H10M30S");
}

#[test]
fn period_with_temporal_values() {
    assert_value!("date('2020-01-31') + period('P1M')", "2020-02-29");
    assert_value!("date('2020-02-29') - period('P1M')", "2020-01-29");
    assert_value!(
        "datetime('2020-01-15T10:30:00') + period('P1Y2M')",
        "2021-03-15 10:30:00.0"
    );
    assert_value!(
        "datetime('2021-03-15T10:30:00') - period('P1Y2M')",
        "2020-01-15 10:30:00.0"
    );
}

#[test]
fn calendar_diff_produces_period() {
    assert_value!(
        "calendarDiff(date('2000-05-03'), date('2025-09-10'))",
        "P25Y4M7D"
    );
    assert_value!(
        "calendarDiff(date('2025-03-10'), date('2024-01-15'))",
        "-P1Y1M23D"
    );
}

#[test]
fn duration_comparator_operators() {
    assert_value!("duration('PT3H') = duration('PT180M')", "true");
    assert_value!("duration('PT1H') < duration('PT2H')", "true");
    assert_value!("duration('P2D') >= duration('P1D')", "true");
}

#[test]
fn period_comparator_equality_only() {
    assert_value!("period('P1Y') = period('P12M')", "true");
    assert_value!("period('P1Y') <> period('P13M')", "true");
    link_error_contains("value: period('P1M') < period('P2M')", &["Comparator"]);
}

#[test]
fn invalid_period_duration_combinations() {
    link_error_contains("value: period('P4D') + duration('PT5H')", &["Operator '+'"]);
    link_error_contains("value: duration('PT5H') - period('P4D')", &["Operator '-'"]);
}

#[test]
fn duration_literal_with_years_is_invalid() {
    let evaluated = eval_all("value: duration('P1Y')");
    assert_string_contains!("[runtime] Invalid duration string", &evaluated);
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
        "value: duration('PT1H') = time('12:00:00')",
        &["Comparator"],
    );
}

mod utilities;
pub use utilities::*;
