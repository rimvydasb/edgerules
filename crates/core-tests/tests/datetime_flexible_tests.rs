mod utilities;
pub use utilities::*;

#[test]
fn flexible_datetime_parsing() {
    // 1. YYYY-MM-DDTHH:MM:SS (no offset) - assumes UTC
    assert_value!("datetime('2026-01-27T10:00:00')", "2026-01-27T10:00:00");
    
    // 2. YYYY-MM-DDTHH:MM (no offset) - assumes UTC, seconds 00
    assert_value!("datetime('2026-01-27T10:00')", "2026-01-27T10:00:00");
    
    // 3. YYYY-MM-DDTHH:MM:SSZ (UTC)
    // Note: our format_datetime_value currently prints without Z if it is UTC to match old behavior.
    assert_value!("datetime('2026-01-27T10:00:00Z')", "2026-01-27T10:00:00");
    
    // 4. YYYY-MM-DDTHH:MM:SS.sssZ (UTC with subseconds)
    // Subseconds might be truncated in our default formatter or printed? 
    // format_datetime_value uses default printing which doesn't seem to include subseconds in my implementation.
    // Let's check: {:02} for seconds.
    assert_value!("datetime('2026-01-27T10:00:00.123Z')", "2026-01-27T10:00:00");

    // 5. YYYY-MM-DDTHH:MM:SS+00:00 (UTC with offset)
    assert_value!("datetime('2026-01-27T10:00:00+00:00')", "2026-01-27T10:00:00");

    // 6. YYYY-MM-DDTHH:MM:SS.sss+00:00 (UTC with offset and subseconds)
    assert_value!("datetime('2026-01-27T10:00:00.123+00:00')", "2026-01-27T10:00:00");

    // 7. Non-UTC offset
    // 2026-01-27T10:00:00+02:00
    // Should be printed WITH offset: 2026-01-27T10:00:00+02:00
    assert_value!("datetime('2026-01-27T10:00:00+02:00')", "2026-01-27T10:00:00+02:00");

    // 8. Non-UTC offset with no seconds
    // 2026-01-27T10:00+02:00
    assert_value!("datetime('2026-01-27T10:00+02:00')", "2026-01-27T10:00:00+02:00");
}
