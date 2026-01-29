use rstest::rstest;

mod utilities;
pub use utilities::*;

// ============================================================================
// Parameterized DateTime Tests - Date Components
// ============================================================================

#[rstest]
#[case("date('2017-05-03').year", "2017")]
#[case("date('2017-05-03').month", "5")]
#[case("date('2017-05-03').day", "3")]
#[case("date('2018-10-11').weekday", "4")]
fn test_date_components(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Time Components
// ============================================================================

#[rstest]
#[case("time('12:00:00').second", "0")]
#[case("time('13:10:30').minute", "10")]
#[case("time('13:10:30').hour", "13")]
fn test_time_components(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - DateTime Components
// ============================================================================

#[rstest]
#[case("datetime('2016-12-09T15:37:00').month", "12")]
#[case("datetime('2016-12-09T15:37:00').hour", "15")]
#[case("datetime('2016-12-09T15:37:00').time", "15:37:00")]
#[case("datetime('2016-12-09T15:37:00').weekday", "5")]
#[case("datetime('2016-12-09T15:37:00').date", "2016-12-09")]
fn test_datetime_components(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Duration Properties
// ============================================================================

#[rstest]
#[case("duration('PT90M').hours", "1")]
#[case("duration('PT90M').minutes", "30")]
#[case("duration('PT90M').seconds", "0")]
#[case("duration('PT90M').totalSeconds", "5400")]
#[case("duration('PT90M').totalMinutes", "90")]
#[case("duration('PT90M').totalHours", "1.5")]
#[case("duration('P2DT3H4M5S').days", "2")]
#[case("duration('P2DT3H4M5S').hours", "3")]
#[case("duration('P2DT3H4M5S').minutes", "4")]
#[case("duration('P2DT3H4M5S').seconds", "5")]
#[case("duration('-PT90M').hours", "-1")]
#[case("duration('-PT90M').minutes", "-30")]
#[case("duration('-PT45S').seconds", "-45")]
#[case("duration('-PT45S').totalMinutes", "-0.75")]
fn test_duration_properties(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Period Properties
// ============================================================================

#[rstest]
#[case("period('P18M').years", "1")]
#[case("period('P18M').months", "6")]
#[case("period('P18M').days", "0")]
#[case("period('P18M').totalMonths", "18")]
#[case("period('P18M').totalDays", "0")]
#[case("period('P2Y3M4D').years", "2")]
#[case("period('P2Y3M4D').months", "3")]
#[case("period('P2Y3M4D').days", "4")]
#[case("period('-P1Y2M5D').years", "-1")]
#[case("period('-P1Y2M5D').months", "-2")]
#[case("period('-P1Y2M5D').days", "-5")]
#[case("period('-P1Y2M5D').totalMonths", "-14")]
#[case("period('-P1Y2M5D').totalDays", "-5")]
fn test_period_properties(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Invalid Temporal Properties
// ============================================================================

#[rstest]
#[case("value: duration('PT1H').years", "duration does not have 'years'")]
#[case("value: duration('PT1H').weekday", "duration does not have 'weekday'")]
#[case("value: time('13:10:30').year", "time does not have 'year'")]
#[case("value: period('P1Y').totalHours", "period does not have 'totalhours'")]
#[case("value: date('2020-01-01').hour", "date does not have 'hour'")]
#[case("value: datetime('2016-12-09T15:37:00').timezone", "datetime does not have 'timezone'")]
fn test_invalid_temporal_properties(#[case] code: &str, #[case] expected_error: &str) {
    link_error_contains(code, &[expected_error]);
}

// ============================================================================
// Parameterized DateTime Tests - Additional Functions
// ============================================================================

#[rstest]
#[case("dayOfWeek(date('2025-09-02'))", "'Tuesday'")]
#[case("monthOfYear(date('2025-09-02'))", "'September'")]
#[case("lastDayOfMonth(date('2025-02-10'))", "28")]
fn test_datetime_additional_functions(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Date Comparators
// ============================================================================

#[rstest]
#[case("date('2020-01-01') = date('2020-01-01')", "true")]
#[case("date('2020-01-01') <> date('2020-01-02')", "true")]
#[case("date('2020-01-01') < date('2020-01-02')", "true")]
#[case("date('2020-01-02') <= date('2020-01-02')", "true")]
#[case("date('2020-01-03') > date('2020-01-02')", "true")]
#[case("date('2020-01-03') >= date('2020-01-03')", "true")]
fn test_date_comparators(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - DateTime Comparators
// ============================================================================

#[rstest]
#[case("datetime('2020-01-01T10:00:00') = datetime('2020-01-01T10:00:00')", "true")]
#[case("datetime('2020-01-01T10:00:00') <> datetime('2020-01-01T12:00:00')", "true")]
#[case("datetime('2020-01-01T08:00:00') < datetime('2020-01-01T09:00:00')", "true")]
#[case("datetime('2020-01-01T09:00:00') <= datetime('2020-01-01T09:00:00')", "true")]
#[case("datetime('2020-01-01T11:00:00') > datetime('2020-01-01T09:00:00')", "true")]
#[case("datetime('2020-01-01T11:00:00') >= datetime('2020-01-01T11:00:00')", "true")]
fn test_datetime_comparators(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Time Comparators
// ============================================================================

#[rstest]
#[case("time('09:00:00') = time('09:00:00')", "true")]
#[case("time('09:00:00') <> time('10:00:00')", "true")]
#[case("time('08:30:00') < time('09:00:00')", "true")]
#[case("time('09:00:00') <= time('09:00:00')", "true")]
#[case("time('10:00:00') > time('09:30:00')", "true")]
#[case("time('10:00:00') >= time('10:00:00')", "true")]
fn test_time_comparators(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Duration Parsing and Operations
// ============================================================================

#[rstest]
#[case("duration('PT90M')", "PT1H30M")]
#[case("duration('P2DT3H')", "P2DT3H")]
#[case("duration('PT4H') + duration('PT30M')", "PT4H30M")]
#[case("duration('PT30M') - duration('PT45M')", "-PT15M")]
#[case("duration('PT45S') + duration('PT15S')", "PT1M")]
fn test_duration_parsing_and_operations(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Period Parsing and Operations
// ============================================================================

#[rstest]
#[case("period('P18M')", "P1Y6M")]
#[case("period('P1Y6M') + period('P2M')", "P1Y8M")]
#[case("period('P6M') - period('P2M')", "P4M")]
#[case("period('P10D') + period('P5D')", "P15D")]
#[case("period('P1Y6M') - period('P8M')", "P10M")]
fn test_period_parsing_and_operations(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Duration with Temporal Values
// ============================================================================

#[rstest]
#[case("date('2017-05-03') + duration('P1D')", "2017-05-04T00:00:00")]
#[case("date('2017-05-03') - duration('P1D')", "2017-05-02T00:00:00")]
#[case("datetime('2016-12-09T15:37:00') + duration('PT23H')", "2016-12-10T14:37:00")]
#[case("datetime('2016-12-10T14:37:00') - duration('PT23H')", "2016-12-09T15:37:00")]
#[case("datetime('2020-01-02T00:00:00') - datetime('2020-01-01T08:00:00')", "PT16H")]
#[case("date('2020-01-02') - datetime('2020-01-01T12:00:00')", "PT12H")]
#[case("datetime('2020-01-01T12:00:00') - date('2020-01-01')", "PT12H")]
#[case("time('13:10:30') - duration('PT1H10M30S')", "12:00:00")]
#[case("time('13:10:30') + duration('PT50S')", "13:11:20")]
#[case("time('13:10:30') - time('12:00:00')", "PT1H10M30S")]
fn test_duration_with_temporal_values(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Period with Temporal Values
// ============================================================================

#[rstest]
#[case("date('2020-01-31') + period('P1M')", "2020-02-29")]
#[case("date('2020-02-29') - period('P1M')", "2020-01-29")]
#[case("datetime('2020-01-15T10:30:00') + period('P1Y2M')", "2021-03-15T10:30:00")]
#[case("datetime('2021-03-15T10:30:00') - period('P1Y2M')", "2020-01-15T10:30:00")]
fn test_period_with_temporal_values(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Calendar Diff
// ============================================================================

#[rstest]
#[case("calendarDiff(date('2000-05-03'), date('2025-09-10'))", "P25Y4M7D")]
#[case("calendarDiff(date('2025-03-10'), date('2024-01-15'))", "-P1Y1M23D")]
fn test_calendar_diff(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Duration Comparators
// ============================================================================

#[rstest]
#[case("duration('PT3H') = duration('PT180M')", "true")]
#[case("duration('PT1H') < duration('PT2H')", "true")]
#[case("duration('P2D') >= duration('P1D')", "true")]
fn test_duration_comparators(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

// ============================================================================
// Parameterized DateTime Tests - Period Equality Only
// ============================================================================

#[rstest]
#[case("period('P1Y') = period('P12M')", "true")]
#[case("period('P1Y') <> period('P13M')", "true")]
fn test_period_equality(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}

#[test]
fn test_period_comparison_not_allowed() {
    link_error_contains("value: period('P1M') < period('P2M')", &["'<'"]);
}

// ============================================================================
// DateTime Tests - Invalid Combinations
// ============================================================================

#[rstest]
#[case("value: period('P4D') + duration('PT5H')", "Operator '+'")]
#[case("value: duration('PT5H') - period('P4D')", "Operator '-'")]
fn test_invalid_period_duration_combinations(#[case] code: &str, #[case] expected_error: &str) {
    link_error_contains(code, &[expected_error]);
}

#[test]
fn test_duration_literal_with_years_is_invalid() {
    let evaluated = eval_all("value: duration('P1Y')");
    assert_string_contains!("[runtime] Failed to parse 'duration' from 'string'", &evaluated);
}

// ============================================================================
// DateTime Tests - Addition with Identical Types Rejected
// ============================================================================

#[rstest]
#[case("value: date('2020-01-01') + date('2020-01-02')", "Operator '+'")]
#[case("value: datetime('2020-01-01T00:00:00') + datetime('2020-01-02T00:00:00')", "Operator '+'")]
#[case("value: time('12:00:00') + time('01:00:00')", "Operator '+'")]
fn test_addition_with_identical_temporal_types_rejected(#[case] code: &str, #[case] expected_error: &str) {
    link_error_contains(code, &[expected_error]);
}

// ============================================================================
// DateTime Tests - Comparator Type Mismatch
// ============================================================================

#[rstest]
#[case("value: date('2020-01-01') = time('12:00:00')", "Comparator")]
#[case("value: datetime('2020-01-01T00:00:00') = time('12:00:00')", "Comparator")]
#[case("value: duration('PT1H') = time('12:00:00')", "Comparator")]
fn test_comparator_type_mismatch_rejected(#[case] code: &str, #[case] expected_error: &str) {
    link_error_contains(code, &[expected_error]);
}

// ============================================================================
// Parameterized DateTime Tests - Flexible Parsing
// ============================================================================

#[rstest]
// No offset (assumes UTC)
#[case("datetime('2026-01-27T10:00:00')", "2026-01-27T10:00:00")]
// No seconds (assumes UTC, seconds 00)
#[case("datetime('2026-01-27T10:00')", "2026-01-27T10:00:00")]
// UTC with Z
#[case("datetime('2026-01-27T10:00:00Z')", "2026-01-27T10:00:00")]
// UTC with subseconds
#[case("datetime('2026-01-27T10:00:00.123Z')", "2026-01-27T10:00:00")]
// UTC with +00:00 offset
#[case("datetime('2026-01-27T10:00:00+00:00')", "2026-01-27T10:00:00")]
// UTC with offset and subseconds
#[case("datetime('2026-01-27T10:00:00.123+00:00')", "2026-01-27T10:00:00")]
// Non-UTC offset
#[case("datetime('2026-01-27T10:00:00+02:00')", "2026-01-27T10:00:00+02:00")]
// Non-UTC offset with no seconds
#[case("datetime('2026-01-27T10:00+02:00')", "2026-01-27T10:00:00+02:00")]
fn test_flexible_datetime_parsing(#[case] expression: &str, #[case] expected: &str) {
    assert_value!(expression, expected);
}
