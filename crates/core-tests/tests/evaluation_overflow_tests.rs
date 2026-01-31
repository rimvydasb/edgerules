#[test]
fn period_addition_overflow_u32() {
    // 200,000,000 years * 12 = 2,400,000,000 months.
    // Fits in u32 (max 4.29B).
    // Sum = 4,800,000,000 months. Overflows u32.
    // This should fail with Error 106 (PeriodTooLarge) in the result construction.
    runtime_error_contains("value: period('P200000000Y') + period('P200000000Y')", &["106"]);
}

#[test]
fn period_subtraction_overflow_u32() {
    // -200M years (negative period)
    // -2.4B months.
    // Subtract 2.4B months.
    // Result -4.8B months.
    // Abs(4.8B) > u32::MAX.
    runtime_error_contains("value: period('-P200000000Y') - period('P200000000Y')", &["106"]);
}

#[test]
fn date_plus_duration_overflow() {
    // Duration approx 20,000 years in seconds.
    // 20000 * 365 * 24 * 3600 = ~6.3e11 seconds.
    // u64 max is 1.8e19. Easily fits.
    // Date 2020 + 20000 years = 22020 (exceeds 9999).
    // P20000Y is period. Duration must be seconds.
    // 20000 years in seconds: 631152000000.
    runtime_error_contains(
        "value: date('2020-01-01') + duration('PT631152000000S')",
        &["Date adjustment with duration overflowed"],
    );
}

#[test]
fn datetime_plus_duration_overflow() {
    runtime_error_contains(
        "value: datetime('2020-01-01T00:00:00') + duration('PT631152000000S')",
        &["Datetime adjustment with duration overflowed"],
    );
}

#[test]
fn date_plus_period_days_overflow() {
    // Period with 4 million days (~10,950 years).
    // 4,000,000 days.
    // Date 2020 + 10950 years = 12970. Overflow.
    runtime_error_contains(
        "value: date('2020-01-01') + period('P4000000D')",
        &["Date adjustment with period overflowed"],
    );
}

mod utilities;
pub use utilities::*;
