# Date and Time Support in EdgeRules

## Summary

- ISO-8601 lexical formats (subset), local-time only (time zone offsets are supported, but time zones not supported)
- Timezone offsets support with `OffsetDateTime`
- Intuitive similarity (non-contradiction) to other standards (DMN, FEEL, SQL, etc.)

### Not Implemented Yet

Context timezone configuration. As a default, it is UTC, that means all dates passed without timezone offsets will be
treated as UTC. For example, date("2024-06-15") will be treated as 2024-06-15T00:00:00Z.

## date(string), time(string), datetime(string) functions

| ISO Format                 | String Value        | EdgeRules Expression            |
|----------------------------|---------------------|---------------------------------|
| Date (YYYY-MM-DD)          | 2017-05-03          | date("2017-05-03")              |
| Time (local time hh:mm:ss) | 13:10:30            | time("13:10:30")                |
| Date-Time (local time)     | 2017-05-03T13:10:30 | datetime("2017-05-03T13:10:30") |

> When comparing or subtracting date and datetime values, the time component of date
> is midnight (00:00:00), so date("2017-05-03") is equivalent to datetime("2017-05-03T00:00:00").

### Date, time component values extracted by dot notation:

- date("2017-05-03").year = 2017 (number)
- time("12:00:00").second = 0 (number)
- datetime("2016-12-09T15:37:00").month = 12 (number)
- datetime("2016-12-09T15:37:00").hour = 15 (number)
- datetime("2016-12-09T15:37:00").time = time("15:37:00")
- date("2018-10-11").weekday = 4 (number) // ISO-8601: Monday=1…Sunday=7

## Object Properties

### date object properties

year (number), month (number), day (number), weekday (number, ISO-8601: Monday=1…Sunday=7)

### time object properties

hour (number), minute (number), second (number)

### datetime object properties

year (number), month (number), day (number), weekday (number, ISO-8601: Monday=1…Sunday=7),
hour (number), minute (number), second (number),
time (time), date (date)

### duration object properties

days (number), hours (number), minutes (number), seconds (number),
totalSeconds (number), totalMinutes (number), totalHours (number)

### period object properties

days (number), months (number), years (number),
totalMonths (number), totalDays (number)

### normalization and clarifications

- period or duration properties will return normalized values:
    - period("P18M").years = 1
    - period("P18M").months = 6
    - duration("PT90M").hours = 1
    - duration("PT90M").minutes = 30
- period does not have totalYears, because existing property years is already normalized
- duration does not have totalDays, because existing property days is already normalized
- period and duration negative values are represented by negative properties (e.g. duration("-PT90M").minutes = -30)

## ISO 8601 Duration Support

EdgeRules supports durations in the ISO 8601 format. The duration format is `P[n]Y[n]M[n]DT[n]H[n]M[n]S`, examples:
duration("P4D"), duration("PT90M"), period("P18Y6M").

### Period

> Period will be Java Period compatible and will be able to carry
> years, months, days components only.

- period("P1Y6M")     // 1 year 6 months
- period("P6M1D")     // 6 months, 1 day
- period("P10D")      // 10 days
- period("-P1Y")      // negative 1 year (allowed)

### Duration

- duration("PT45M")   // 45 minutes (days–time)
- duration("P2DT3H")  // 2 days 3 hours (days–time)
- duration("-PT30S")  // negative 30 seconds (allowed)

### Period vs Duration

Is invalid and will produce runtime error while parsing duration strings:

> **Todo:** at this time, it is not possible to have linking error, because strings are not investigated
> during AST building and linking phase

```edgerules
value1: period("P18YT12H")    // invalid
value2: duration("P18YT12H")  // invalid
```

Is invalid and will produce linking error:

```edgerules
value1: period("P4D") + duration("PT5H")  // invalid
value2: duration("PT5H") - period("P4D")  // invalid
```

## Operations

```edgerules
// All comparison operators are supported for date, time, and datetime values:
date("2017-05-03") < date("2017-05-04")  // true

// Subtraction operator is supported for date, time, and datetime values:
date("2017-05-04") - date("2017-05-03")  // duration("P1D")

// Subtraction operation for real period to calculate birthdays, anniversaries, etc.:
calendarDiff(date("2000-05-03"), date("2025-09-10"))  // period("P25Y4M7D")

// Addition operator is supported between date/datetime and duration values:
date("2017-05-03") + duration("P1D")      // datetime("2017-05-04T00:00:00")

// Period and Duration types will be normalized during operations:
toString(period("P18M"))        // "P1Y6M"
toString(duration("PT90M"))     // "PT1H30M"
```

## Math Operations Patterns

- [date/datetime] - [date/datetime]  => duration
- [time] - [time]  => duration
- [date/datetime/time] + [duration]  => [datetime/datetime/time] // adding duration to date results to datetime
- [date/datetime/time] - [duration]  => [datetime/datetime/time] // subtracting duration from date results to datetime
- [duration] + [duration]  => [duration]
- [duration] - [duration]  => [duration]

- [date/datetime/time] + [date/datetime/time]  => exception
- [period] +/- [duration]  => exception
- [date/datetime] - [time]  => exception

- [period] + [period]  => [period]
- [period] - [period]  => [period]
- calendarDiff([date], [date])  => [period]

- [date/datetime] +/- [period]  => [date/datetime]

## Comparator Operations Patterns

- [date/datetime] </<=/=/>=/> [date/datetime]  => boolean
- [duration] </<=/=/>=/> [duration]  => boolean
- [period] = [period]  => boolean
- [time] </<=/=/>=/> [time]  => boolean

- [date/datetime/time] </<=/=/>=/> [period/duration]  => exception
- [period] </<=/>=/> [period]  => exception // period (Y/M/D) has no total order
- [period] </<=/=/>=/> [duration]  => exception
- [date/datetime] </<=/=/>=/> [time]  => exception

> period("P1Y") = period("P12M")  // true

## Restrictions:

- No leap seconds
- No timezones will be supported
- No time offsets in time and date values for this implementation - only local time will be supported
- No support for today() or now() functions, because EdgeRules are designed to be deterministic and work on the edge
  environment without access to a real-time clock
- No nanoseconds or milliseconds support in this implementation, smallest unit is seconds
- Only english month and day names are supported

## Additional functions:

```edgerules
duration("PT90M")                   // Parsed as duration of 1 hour 30 minutes (days–time)
period("P18Y6M")                    // Parsed as period of 18 years 6 months (years–months)
dayOfWeek(date("2025-09-02"))       // "Tuesday" (string)
monthOfYear(date("2025-09-02"))     // "September" (string)
lastDayOfMonth(date("2025-02-10"))  // 28 (number)
calendarDiff(date("2024-01-15"), date("2025-03-10")) // period("P1Y1M23D")
```

# Not Implemented:

## Literal Duration Support (is not implemented, @TBD)

You can also create durations using a literal syntax:

```edgerules
1 years 6 months    // 1 year 6 months (years–months)
6 months            // 6 months (years–months)
45 minutes          // 45 minutes (days–time)
2 days 3 hours      // 2 days 3 hours (days–time)
-1 years            // negative 1 year (allowed)
```

## Todo

- Literal support for durations (years, months, days, hours, minutes, seconds)
- More date/time functions (e.g., adding support for week of year, quarter of year, etc.)

# Next Steps

Review task description below in `doc/DATETIME_SPEC.md`: most of the tasks may be already done.

Implement JSON date and datetime support for EdgeRules Portable format by doing following tasks in `DATETIME_SPEC.md`.
Currently, as per `test-decision-service.mjs`, Portable works well with JavaScript Date objects, but lacks support for
ISO date and datetime strings.

- [x] Add minimal test in `test-decision-service.mjs` that explores JavaScript Date and Date with time support without
  zones. Later this test will be used for the development.
    - [x] Minimal decision service that is created with constants that have date and datetime values - use JavaScript
      Date objects.
    - [x] Minimal decision service request that sends date and datetime values as JavaScript Date objects.
    - [x] Run this test, it should work as per current implementation.
- [x] Add another test that instead of JavaScript Date, strings are used. We must support:
    - [x] date as `YYYY-MM-DD`
    - [x] datetime as `YYYY-MM-DDTHH:MM:SS`
    - [x] Treat `2026-01-26T21:33:35Z`, `2026-01-26T21:33:35.000Z` or `2026-01-26T21:33:35+00:00` as
      `YYYY-MM-DDTHH:MM:SS`. Write test cases for all of these.
    - [x] Timezones are not supported, so any offset or Z must be ignored. For other timezones, error must be raised.
    - [x] Time offsets are not supported, so `+00:00` can be ignored, but `+02:00` must raise error.
- [x] Implement support in EdgeRules Portable for date and datetime strings as per above specification.
  Implementation should be straightforward: if field expects date or datetime, and string is provided, then simply use
  `parse_date_iso` or `parse_datetime_local` as implemented.
- [x] Use `ParseErrorEnum::CannotConvertValue(ValueType, ValueType)` for parsing problems. Map parsing errors to
  `PortableError` to properly display. Write tests for invalid date/datetime/duration/time strings to ensure proper
  error handling.
- [x] Implement support for duration and period strings in EdgeRules Portable format so user will be able to provide
  duration and period values as strings.
- [x] Implement support for time strings in EdgeRules Portable format so user will be able to provide time values as
  strings.
- [x] Add tests for duration, period, and time strings in EdgeRules Portable format.
- [x] Make sure tests are passing: Rust and JavaScript.
- [x] Review the code based on project priorities: Small WASM Size First, Small Stack Size Second, Performance Third,
  Maintainability
- [x] Mark tasks that are completed.

- [ ] Update `parse_datetime_local` to be `parse_datetime_flexible` that will support (write Rust tests for it):
    - [ ] `YYYY-MM-DDTHH:MM:SS` (no offset) - existing
    - [ ] `YYYY-MM-DDTHH:MM` (no offset) - new
    - [ ] `YYYY-MM-DDTHH:MM:SSZ` (UTC) - new
    - [ ] `YYYY-MM-DDTHH:MM:SS.sssZ` (UTC with subseconds) - new
    - [ ] `YYYY-MM-DDTHH:MM:SS+00:00` (UTC with offset) - new
    - [ ] `YYYY-MM-DDTHH:MM:SS.sss+00:00` (UTC with offset and subseconds) - new

```rust
use time::{OffsetDateTime, PrimitiveDateTime, macros::format_description};
use time::format_description::well_known::Rfc3339;

pub fn parse_datetime_flexible(s: &str) -> Option<OffsetDateTime> {
    // 1. Try standard RFC 3339 (Handles "Z", "+02:00", and variable subseconds)
    // This is the fastest and most common path for JSON.
    if let Ok(odt) = OffsetDateTime::parse(s, &Rfc3339) {
        return Some(odt);
    }

    // 2. Try Date + Time with offset but NO seconds (e.g., 2026-01-27T10:00+02:00)
    let fmt_no_sec_offset = format_description!("[year]-[month]-[day]T[hour]:[minute][offset_hour]:[offset_minute]");
    if let Ok(odt) = OffsetDateTime::parse(s, &fmt_no_sec_offset) {
        return Some(odt);
    }

    // 3. Try Primitive (No Offset) - Fallback to UTC
    // We try with seconds first, then without.
    let fmt_prim = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    let fmt_prim_no_sec = format_description!("[year]-[month]-[day]T[hour]:[minute]");

    if let Ok(dt) = PrimitiveDateTime::parse(s, &fmt_prim) {
        return Some(dt.assume_utc());
    }

    if let Ok(dt) = PrimitiveDateTime::parse(s, &fmt_prim_no_sec) {
        return Some(dt.assume_utc());
    }

    None
}
```

- [ ] Start using `OffsetDateTime` instead of `PrimitiveDateTime` internally in EdgeRules engine for datetime values.
  This will allow future support for timezones if needed.
- [ ] Update documentation in `DATETIME_SPEC.md` to reflect any changes made during implementation.
- [ ] Perform code review and testing to ensure stability and correctness of the implementation.