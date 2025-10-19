# Date and Time Support in EdgeRules

## Priorities

- Small WebAssembly binary size
- Fast execution speed
- ISO-8601 lexical formats (subset), local-time only (offsets/time zones not supported)
- Intuitive similarity (non-contradiction) to other standards (DMN, FEEL, SQL, etc.)

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
during AST building and linking phase

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

# Implementation Notes

1. Find `DurationValue(ValueOrSv<DurationValue, SpecialValueEnum>),`
DurationValue must be struct and hold only seconds and negative flag.
During printing or serializing duration, it must be converted to ISO 8601 format
(so days and minutes must be calculated from seconds).

2. Add `PeriodValue(ValueOrSv<PeriodValue, SpecialValueEnum>),`
PeriodValue must be struct and hold months, days and negative flag.
During printing or serializing period, it must be converted to ISO 8601 format
(so years must be calculated from months).

3. There will be no such a thing as `Combined` type and exceptions must be raised
if user mixes period and duration in operations.

## Literal Duration Support (TBC, subject to change)

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