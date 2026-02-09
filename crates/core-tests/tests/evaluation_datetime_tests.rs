mod utilities;
pub use utilities::*;

#[test]
fn datetime_primitives_and_components() {
    assert_expression_value("date('2017-05-03').year", "2017");
    assert_expression_value("date('2017-05-03').month", "5");
    assert_expression_value("date('2017-05-03').day", "3");

    assert_expression_value("time('12:00:00').second", "0");
    assert_expression_value("time('13:10:30').minute", "10");
    assert_expression_value("time('13:10:30').hour", "13");

    assert_expression_value("datetime('2016-12-09T15:37:00').month", "12");
    assert_expression_value("datetime('2016-12-09T15:37:00').hour", "15");
    assert_expression_value("datetime('2016-12-09T15:37:00').time", "15:37:00");
    assert_expression_value("datetime('2016-12-09T15:37:00').weekday", "5");
    assert_expression_value("datetime('2016-12-09T15:37:00').date", "2016-12-09");

    assert_expression_value("date('2018-10-11').weekday", "4");
}

#[test]
fn duration_object_properties() {
    assert_expression_value("duration('PT90M').hours", "1");
    assert_expression_value("duration('PT90M').minutes", "30");
    assert_expression_value("duration('PT90M').seconds", "0");
    assert_expression_value("duration('PT90M').totalSeconds", "5400");
    assert_expression_value("duration('PT90M').totalMinutes", "90");
    assert_expression_value("duration('PT90M').totalHours", "1.5");
    assert_expression_value("duration('P2DT3H4M5S').days", "2");
    assert_expression_value("duration('P2DT3H4M5S').hours", "3");
    assert_expression_value("duration('P2DT3H4M5S').minutes", "4");
    assert_expression_value("duration('P2DT3H4M5S').seconds", "5");
    assert_expression_value("duration('-PT90M').hours", "-1");
    assert_expression_value("duration('-PT90M').minutes", "-30");
    assert_expression_value("duration('-PT45S').seconds", "-45");
    assert_expression_value("duration('-PT45S').totalMinutes", "-0.75");
}

#[test]
fn period_object_properties() {
    assert_expression_value("period('P18M').years", "1");
    assert_expression_value("period('P18M').months", "6");
    assert_expression_value("period('P18M').days", "0");
    assert_expression_value("period('P18M').totalMonths", "18");
    assert_expression_value("period('P18M').totalDays", "0");
    assert_expression_value("period('P2Y3M4D').years", "2");
    assert_expression_value("period('P2Y3M4D').months", "3");
    assert_expression_value("period('P2Y3M4D').days", "4");
    assert_expression_value("period('-P1Y2M5D').years", "-1");
    assert_expression_value("period('-P1Y2M5D').months", "-2");
    assert_expression_value("period('-P1Y2M5D').days", "-5");
    assert_expression_value("period('-P1Y2M5D').totalMonths", "-14");
    assert_expression_value("period('-P1Y2M5D').totalDays", "-5");
}

#[test]
fn invalid_temporal_properties() {
    link_error_contains("value: duration('PT1H').years", &["duration does not have 'years'"]);
    link_error_contains("value: duration('PT1H').weekday", &["duration does not have 'weekday'"]);
    link_error_contains("value: time('13:10:30').year", &["time does not have 'year'"]);
    link_error_contains("value: period('P1Y').totalHours", &["period does not have 'totalhours'"]);
    link_error_contains("value: date('2020-01-01').hour", &["date does not have 'hour'"]);
    link_error_contains("value: datetime('2016-12-09T15:37:00').timezone", &["datetime does not have 'timezone'"]);
}

#[test]
fn datetime_additional_functions() {
    assert_expression_value("dayOfWeek(date('2025-09-02'))", "'Tuesday'");
    assert_expression_value("monthOfYear(date('2025-09-02'))", "'September'");
    assert_expression_value("lastDayOfMonth(date('2025-02-10'))", "28");
}

#[test]
fn date_comparator_operators() {
    assert_expression_value("date('2020-01-01') = date('2020-01-01')", "true");
    assert_expression_value("date('2020-01-01') <> date('2020-01-02')", "true");
    assert_expression_value("date('2020-01-01') < date('2020-01-02')", "true");
    assert_expression_value("date('2020-01-02') <= date('2020-01-02')", "true");
    assert_expression_value("date('2020-01-03') > date('2020-01-02')", "true");
    assert_expression_value("date('2020-01-03') >= date('2020-01-03')", "true");
}

#[test]
fn datetime_comparator_operators() {
    assert_expression_value("datetime('2020-01-01T10:00:00') = datetime('2020-01-01T10:00:00')", "true");
    assert_expression_value("datetime('2020-01-01T10:00:00') <> datetime('2020-01-01T12:00:00')", "true");
    assert_expression_value("datetime('2020-01-01T08:00:00') < datetime('2020-01-01T09:00:00')", "true");
    assert_expression_value("datetime('2020-01-01T09:00:00') <= datetime('2020-01-01T09:00:00')", "true");
    assert_expression_value("datetime('2020-01-01T11:00:00') > datetime('2020-01-01T09:00:00')", "true");
    assert_expression_value("datetime('2020-01-01T11:00:00') >= datetime('2020-01-01T11:00:00')", "true");
}

#[test]
fn time_comparator_operators() {
    assert_expression_value("time('09:00:00') = time('09:00:00')", "true");
    assert_expression_value("time('09:00:00') <> time('10:00:00')", "true");
    assert_expression_value("time('08:30:00') < time('09:00:00')", "true");
    assert_expression_value("time('09:00:00') <= time('09:00:00')", "true");
    assert_expression_value("time('10:00:00') > time('09:30:00')", "true");
    assert_expression_value("time('10:00:00') >= time('10:00:00')", "true");
}

#[test]
fn duration_parsing_and_operations() {
    assert_expression_value("duration('PT90M')", "PT1H30M");
    assert_expression_value("duration('P2DT3H')", "P2DT3H");
    assert_expression_value("duration('PT4H') + duration('PT30M')", "PT4H30M");
    assert_expression_value("duration('PT30M') - duration('PT45M')", "-PT15M");
    assert_expression_value("duration('PT45S') + duration('PT15S')", "PT1M");
}

#[test]
fn period_parsing_and_operations() {
    assert_expression_value("period('P18M')", "P1Y6M");
    assert_expression_value("period('P1Y6M') + period('P2M')", "P1Y8M");
    assert_expression_value("period('P6M') - period('P2M')", "P4M");
    assert_expression_value("period('P10D') + period('P5D')", "P15D");
    assert_expression_value("period('P1Y6M') - period('P8M')", "P10M");
}

#[test]
fn duration_with_temporal_values() {
    assert_expression_value("date('2017-05-03') + duration('P1D')", "2017-05-04T00:00:00");
    assert_expression_value("date('2017-05-03') - duration('P1D')", "2017-05-02T00:00:00");
    assert_expression_value("datetime('2016-12-09T15:37:00') + duration('PT23H')", "2016-12-10T14:37:00");
    assert_expression_value("datetime('2016-12-10T14:37:00') - duration('PT23H')", "2016-12-09T15:37:00");
    assert_expression_value("datetime('2020-01-02T00:00:00') - datetime('2020-01-01T08:00:00')", "PT16H");
    assert_expression_value("date('2020-01-02') - datetime('2020-01-01T12:00:00')", "PT12H");
    assert_expression_value("datetime('2020-01-01T12:00:00') - date('2020-01-01')", "PT12H");
    assert_expression_value("time('13:10:30') - duration('PT1H10M30S')", "12:00:00");
    assert_expression_value("time('13:10:30') + duration('PT50S')", "13:11:20");
    assert_expression_value("time('13:10:30') - time('12:00:00')", "PT1H10M30S");
}

#[test]
fn period_with_temporal_values() {
    assert_expression_value("date('2020-01-31') + period('P1M')", "2020-02-29");
    assert_expression_value("date('2020-02-29') - period('P1M')", "2020-01-29");
    assert_expression_value("datetime('2020-01-15T10:30:00') + period('P1Y2M')", "2021-03-15T10:30:00");
    assert_expression_value("datetime('2021-03-15T10:30:00') - period('P1Y2M')", "2020-01-15T10:30:00");
}

#[test]
fn calendar_diff_produces_period() {
    assert_expression_value("calendarDiff(date('2000-05-03'), date('2025-09-10'))", "P25Y4M7D");
    assert_expression_value("calendarDiff(date('2025-03-10'), date('2024-01-15'))", "-P1Y1M23D");
}

#[test]
fn duration_comparator_operators() {
    assert_expression_value("duration('PT3H') = duration('PT180M')", "true");
    assert_expression_value("duration('PT1H') < duration('PT2H')", "true");
    assert_expression_value("duration('P2D') >= duration('P1D')", "true");
}

#[test]
fn period_comparator_equality_only() {
    assert_expression_value("period('P1Y') = period('P12M')", "true");
    assert_expression_value("period('P1Y') <> period('P13M')", "true");
    link_error_contains("value: period('P1M') < period('P2M')", &["'<'"]);
}

#[test]
fn invalid_period_duration_combinations() {
    link_error_contains("value: period('P4D') + duration('PT5H')", &["Operator '+'"]);
    link_error_contains("value: duration('PT5H') - period('P4D')", &["Operator '-'"]);
}

#[test]
fn duration_literal_with_years_is_invalid() {
    runtime_error_contains("value: duration('P1Y')", &["[runtime] Failed to parse 'duration' from 'string'"]);
}

#[test]
fn addition_with_identical_temporal_types_is_rejected() {
    link_error_contains("value: date('2020-01-01') + date('2020-01-02')", &["Operator '+'"]);
    link_error_contains("value: datetime('2020-01-01T00:00:00') + datetime('2020-01-02T00:00:00')", &["Operator '+'"]);
    link_error_contains("value: time('12:00:00') + time('01:00:00')", &["Operator '+'"]);
}

#[test]
fn comparator_type_mismatch_is_rejected() {
    link_error_contains("value: date('2020-01-01') = time('12:00:00')", &["Comparator"]);
    link_error_contains("value: datetime('2020-01-01T00:00:00') = time('12:00:00')", &["Comparator"]);
    link_error_contains("value: duration('PT1H') = time('12:00:00')", &["Comparator"]);
}

#[test]
fn flexible_datetime_parsing() {
    // 1. YYYY-MM-DDTHH:MM:SS (no offset) - assumes UTC
    assert_expression_value("datetime('2026-01-27T10:00:00')", "2026-01-27T10:00:00");

    // 2. YYYY-MM-DDTHH:MM (no offset) - assumes UTC, seconds 00
    assert_expression_value("datetime('2026-01-27T10:00')", "2026-01-27T10:00:00");

    // 3. YYYY-MM-DDTHH:MM:SSZ (UTC)
    // Note: our format_datetime_value currently prints without Z if it is UTC to match old behavior.
    assert_expression_value("datetime('2026-01-27T10:00:00Z')", "2026-01-27T10:00:00");

    // 4. YYYY-MM-DDTHH:MM:SS.sssZ (UTC with subseconds)
    // Subseconds might be truncated in our default formatter or printed?
    // format_datetime_value uses default printing which doesn't seem to include subseconds in my implementation.
    // Let's check: {:02} for seconds.
    assert_expression_value("datetime('2026-01-27T10:00:00.123Z')", "2026-01-27T10:00:00");

    // 5. YYYY-MM-DDTHH:MM:SS+00:00 (UTC with offset)
    assert_expression_value("datetime('2026-01-27T10:00:00+00:00')", "2026-01-27T10:00:00");

    // 6. YYYY-MM-DDTHH:MM:SS.sss+00:00 (UTC with offset and subseconds)
    assert_expression_value("datetime('2026-01-27T10:00:00.123+00:00')", "2026-01-27T10:00:00");

    // 7. Non-UTC offset
    // 2026-01-27T10:00:00+02:00
    // Should be printed WITH offset: 2026-01-27T10:00:00+02:00
    assert_expression_value("datetime('2026-01-27T10:00:00+02:00')", "2026-01-27T10:00:00+02:00");

    // 8. Non-UTC offset with no seconds
    // 2026-01-27T10:00+02:00
    assert_expression_value("datetime('2026-01-27T10:00+02:00')", "2026-01-27T10:00:00+02:00");

    // 9. Subsecond inequality (verifies preservation)
    assert_expression_value("datetime('2026-01-27T10:00:00.123Z') = datetime('2026-01-27T10:00:00.456Z')", "false");
    assert_expression_value("datetime('2026-01-27T10:00:00.123Z') = datetime('2026-01-27T10:00:00Z')", "false");

    // 10. Fallback to UTC (verifies that no offset is treated as UTC/Z)
    assert_expression_value("datetime('2026-01-27T10:00:00') = datetime('2026-01-27T10:00:00Z')", "true");
}
