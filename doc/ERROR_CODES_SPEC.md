# Error Codes

## Value Parsing Errors


/// ValueParsingError error codes:
/// 0 - Generic parsing error
/// 101 - Date adjustment overflowed year range
/// 102 - Invalid month produced during calendarDiff
/// 103 - Invalid date produced during calendarDiff
/// 104 - Period components must be non-negative before applying the sign
/// 105 - Period months overflow the supported range
/// 106 - Period components overflow the supported range
/// 107 - Period months and days must carry the same sign
/// 110 - Duration days overflow the supported range
/// 111 - Duration hours overflow the supported range
/// 112 - Duration minutes overflow the supported range
/// 113 - Duration seconds overflow the supported range
/// 114 - Duration overflow while calculating seconds
/// 115 - Duration components must be non-negative before applying the sign


## Internal Integrity Errors

    // Code 100-199: math linking guards
    // Code 200-299: string linking guards
    // Code 300-399: date, time, duration linking guards
    // Code 400-499: array and object linking guards
    // Codes:
    // 100 - Operator '^' is not implemented for operands
    // 101 - Unsupported operator for duration values
    // 102 - Unsupported operator for period values
    // 103 - Operator is not implemented for date and duration values
    // 104 - Operator is not implemented for datetime and duration values
    // 105 - Operator is not implemented for time and duration values
    // 106 - Operator is not implemented for date and period values
    // 107 - Operator is not implemented for datetime and period values
    // 108 - Cannot apply operator between period and duration values
    // 109 - Operator is not implemented for operands
    // 110 - Cannot negate value
    // 150 - Cannot compare durations (Less)
    // 151 - Cannot compare durations (Greater)
    // 152 - Cannot compare durations (LessEquals)
    // 153 - Cannot compare durations (GreaterEquals)
    // 154 - Comparator is not supported for period values
    // 155 - Not possible to compare operands
    // 160 - Logical operator is not implemented for operands
    // 200 - regex_functions feature is disabled (split)
    // 201 - regex_functions feature is disabled (replace)
    // 202 - base64_functions feature is disabled (to_base64)
    // 203 - base64_functions feature is disabled (from_base64)
    // 300 - calendarDiff expects date arguments
    // 400 - Cannot iterate
    // 401 - Cannot select a value
    // 402 - Cannot select because data type is not an object
    // 403 - User function call failed unexpectedly