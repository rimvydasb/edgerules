# Date and Time Support in EdgeRules

## Priorities

- Small WebAssembly binary size
- Fast execution speed

## date(string), time(string), datetime(string) functions

| ISO Format                 | String Value        | EdgeRules Expression            |
|----------------------------|---------------------|---------------------------------|
| Date (YYYY-MM-DD)          | 2017-05-03          | date("2017-05-03")              |
| Time (local time hh:mm:ss) | 13:10:30            | time("13:10:30")                |
| Date-Time (local time)     | 2017-05-03T13:10:30 | datetime("2017-05-03T13:10:30") |

### Date, time component values extracted by dot notation:

- date("2017-05-03").year = 2017 (number)
- time("12:00:00").second = 0 (number)
- datetime("2016-12-09T15:37:00").month = 12 (number)
- datetime("2016-12-09T15:37:00").hour = 15 (number)
- datetime("2016-12-09T15:37:00").time = time("15:37:00")
- datetime("2018-10-11").weekday = 4 (number)

## ISO 8601 Duration Support

EdgeRules supports durations in the ISO 8601 format. The duration format is `PxDTxHxMxS`, examples:
duration("P4D"), duration("PT90M"), duration("P1DT6H")

- duration("P1Y6M")   // 1 year 6 months (years–months)
- duration("P6M")     // 6 months (years–months)
- duration("PT45M")   // 45 minutes (days–time)
- duration("P2DT3H")  // 2 days 3 hours (days–time)
- duration("-P1Y")    // negative 1 year (allowed)

## Operations

```edgerules
// All comparison operators are supported for date, time, and datetime values:
date("2017-05-03") < date("2017-05-04")  // true

// Subtraction operator is supported for date, time, and datetime values:
date("2017-05-04") - date("2017-05-03")  // duration("P1D")

// Addition operator is supported between date/datetime and duration values:
date("2017-05-03") + duration("P1D")      // date("2017-05-04")
```

## Restrictions:

- No leap seconds
- No timezones will be supported
- No time offsets in time and date values for this implementation - only local time will be supported
- No support for today() or now() functions, because EdgeRules are designed to be deterministic and work on the edge
  environment without access to a real-time clock
- No nanoseconds or milliseconds support in this implementation, smallest unit is seconds
- Only english month and day names are supported in additional functions

## Additional functions:

```edgerules
dayOfWeek(date("2025-09-02"))      // "Tuesday" (string)
monthOfYear(date("2025-09-02"))    // "September" (string)
lastDayOfMonth(date("2025-02-10")) // 28 (number)
```

## (#NEW) Literal Duration Support

You can also create durations using a literal syntax:

```edgerules
1 years 6 months    // 1 year 6 months (years–months)
6 months            // 6 months (years–months)
45 minutes          // 45 minutes (days–time)
2 days 3 hours      // 2 days 3 hours (days–time)
-1 years            // negative 1 year (allowed)
```