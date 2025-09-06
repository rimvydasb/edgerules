use crate::ast::token::into_valid;
use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::number::NumberEnum::SV;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::ValueType::{ListType, NumberType, RangeType};
use crate::typesystem::types::{Integer, SpecialValueEnum, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{Array, DateTimeValue, DateValue, DurationValue, NumberValue, RangeValue, StringValue, TimeValue};
use crate::typesystem::values::{DurationValue as ErDurationValue, ValueOrSv};
use time::macros::format_description;

pub fn eval_max_all(
    values: Vec<Result<ValueEnum, RuntimeError>>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    let mut maximum: Option<NumberEnum> = None;

    for value in values {
        match value? {
            NumberValue(ref number) => {
                if let Some(ref check) = maximum {
                    if check < number {
                        maximum = Some(number.clone());
                    }
                } else {
                    maximum = Some(number.clone());
                }
            }
            _ => return RuntimeError::type_not_supported(list_type).into(),
        }
    }

    if let Some(max) = maximum {
        Ok(NumberValue(max))
    } else {
        Ok(NumberValue(SV(SpecialValueEnum::Missing)))
    }
}

pub fn eval_max(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(_) => Ok(value),
        Array(values, list_type) => eval_max_all(values, list_type),
        RangeValue(range) => match range.max() {
            None => RuntimeError::eval_error(
                "Max is not implemented for this particular range".to_string(),
            )
            .into(),
            Some(max) => Ok(NumberValue(NumberEnum::from(max))),
        },
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_sum_all(
    values: Vec<Result<ValueEnum, RuntimeError>>,
    list_type: ValueType,
) -> Result<ValueEnum, RuntimeError> {
    if values.is_empty() {
        return Ok(ValueEnum::from(0));
    }

    let mut acc: NumberEnum = match values.first().unwrap() {
        Ok(NumberValue(NumberEnum::Real(_))) => NumberEnum::Real(0.0),
        Ok(NumberValue(NumberEnum::Int(_))) => NumberEnum::Int(0),
        _ => return RuntimeError::type_not_supported(list_type).into(),
    };

    for token in values {
        if let NumberValue(number) = token? {
            acc = acc + number;
        } else {
            return RuntimeError::type_not_supported(list_type).into();
        }
    }

    Ok(NumberValue(acc))
}

pub fn eval_sum(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(number) => Ok(NumberValue(number)),
        Array(items, list_type) => eval_sum_all(items, list_type),
        RangeValue(range) => Ok(ValueEnum::from(range.sum::<Integer>())),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_count(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match value {
        NumberValue(_) => Ok(NumberValue(NumberEnum::Int(1))),
        Array(items, _) => {
            let count = items.len();
            Ok(NumberValue(NumberEnum::Int(count as Integer)))
        }
        RangeValue(range) => Ok(ValueEnum::from(range.count() as Integer)),
        other => RuntimeError::type_not_supported(other.get_type()).into(),
    }
}

pub fn eval_find(maybe_array: ValueEnum, search: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let Array(values, _) = maybe_array {
        let valid = into_valid(values)?;

        let maybe_index = valid.iter().position(|value| value.eq(&search));

        match maybe_index {
            Some(index) => Ok(ValueEnum::from(index as Integer)),

            // todo: should determine the type
            None => Ok(NumberValue(SV(SpecialValueEnum::Missing))),
        }
    } else {
        RuntimeError::type_not_supported(maybe_array.get_type()).into()
    }
}

pub fn list_item_as_second_arg(left: ValueType, right: ValueType) -> Link<()> {
    let list_type = LinkingError::expect_array_type(Some("function arguments".to_string()), left)?;
    LinkingError::expect_same_types("function arguments", list_type, right)?;
    Ok(())
}

pub fn number_range_or_number_list(value_type: ValueType) -> Link<()> {
    if match &value_type {
        NumberType | RangeType => true,
        ListType(list_type) => matches!(*list_type.clone(), NumberType),
        _ => false,
    } {
        Ok(())
    } else {
        LinkingError::types_not_compatible(
            None,
            value_type,
            Some(vec![NumberType, RangeType, ListType(Box::new(NumberType))]),
        )
        .into()
    }
}

pub fn validate_multi_all_args_numbers(args: Vec<ValueType>) -> Link<()> {
    for arg in args {
        if !matches!(arg, NumberType) {
            return LinkingError::types_not_compatible(None, arg, Some(vec![NumberType])).into();
        }
    }

    Ok(())
}

pub fn return_binary_same_as_right_arg(_left: ValueType, right: ValueType) -> ValueType {
    right
}

pub fn return_uni_number(_arg: ValueType) -> ValueType {
    NumberType
}

pub fn return_multi_number() -> ValueType {
    NumberType
}

// ---------------------------------------------------------------------------------------------
// Date/Time/Duration parsing and helpers
// ---------------------------------------------------------------------------------------------

pub fn expect_string_arg(arg: ValueType) -> Link<()> {
    LinkingError::expect_type(None, arg, &[ValueType::StringType]).map(|_| ())
}

pub fn expect_date_arg(arg: ValueType) -> Link<()> {
    LinkingError::expect_type(None, arg, &[ValueType::DateType]).map(|_| ())
}

fn parse_date_iso(s: &str) -> Option<time::Date> {
    // Strict ISO 8601 date: YYYY-MM-DD
    let fmt = format_description!("[year]-[month]-[day]");
    time::Date::parse(s, &fmt).ok()
}

fn parse_time_local(s: &str) -> Option<time::Time> {
    // Local time: HH:MM:SS (24-hour)
    let fmt = format_description!("[hour]:[minute]:[second]");
    time::Time::parse(s, &fmt).ok()
}

fn parse_datetime_local(s: &str) -> Option<time::PrimitiveDateTime> {
    // Local datetime: YYYY-MM-DDTHH:MM:SS
    let fmt = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    time::PrimitiveDateTime::parse(s, &fmt).ok()
}

fn parse_duration_iso8601(s: &str) -> Option<ErDurationValue> {
    // format: [-]P[nY][nM][nD][T[nH][nM][nS]] with two disjoint kinds: Y/M or D/T
    if s.is_empty() { return None; }
    let mut negative = false;
    let mut idx = 0;
    let bytes = s.as_bytes();
    if bytes[idx] == b'-' { negative = true; idx += 1; }
    if idx >= bytes.len() || bytes[idx] != b'P' { return None; }
    idx += 1;

    let mut years: i32 = 0;
    let mut months: i32 = 0;
    let mut days: i64 = 0;
    let mut hours: i64 = 0;
    let mut minutes: i64 = 0;
    let mut seconds: i64 = 0;

    let mut in_time = false;
    let mut saw_ym = false;
    let mut saw_dt = false;

    let mut num_start = idx;
    while idx <= bytes.len() {
        if idx == bytes.len() || bytes[idx].is_ascii_alphabetic() {
            if idx == num_start { // nothing before unit
                if idx == bytes.len() { break; }
                if bytes[idx] == b'T' { in_time = true; idx += 1; num_start = idx; continue; }
                return None;
            }
            let num_str = &s[num_start..idx];
            let unit = if idx < bytes.len() { bytes[idx] as char } else { '\0' };
            match unit {
                'Y' => { years = num_str.parse::<i32>().ok()?; saw_ym = true; }
                'M' if !in_time => { months = num_str.parse::<i32>().ok()?; saw_ym = true; }
                'D' => { days = num_str.parse::<i64>().ok()?; saw_dt = true; }
                'T' => { in_time = true; num_start = idx + 1; idx += 1; continue; }
                'H' => { hours = num_str.parse::<i64>().ok()?; saw_dt = true; }
                'M' if in_time => { minutes = num_str.parse::<i64>().ok()?; saw_dt = true; }
                'S' => { seconds = num_str.parse::<i64>().ok()?; saw_dt = true; }
                '\0' => { break; }
                _ => { return None; }
            }
            idx += 1;
            num_start = idx;
            continue;
        }
        idx += 1;
    }

    if saw_ym && saw_dt { return None; }
    if saw_ym {
        return Some(ErDurationValue::ym(years, months, negative));
    }
    // default to days-time, allow PT0S when all zero
    Some(ErDurationValue::dt(days, hours, minutes, seconds, negative))
}

pub fn eval_date(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let StringValue(ref s) = value {
        if let StringEnum::String(raw) = s.clone() {
            if let Some(date) = parse_date_iso(raw.as_str()) {
                return Ok(DateValue(ValueOrSv::Value(date)));
            }
            return RuntimeError::eval_error("Invalid date string".to_string()).into();
        }
    }
    RuntimeError::type_not_supported(value.get_type()).into()
}

pub fn eval_time(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let StringValue(ref s) = value {
        if let StringEnum::String(raw) = s.clone() {
            if let Some(time) = parse_time_local(raw.as_str()) {
                return Ok(TimeValue(ValueOrSv::Value(time)));
            }
            return RuntimeError::eval_error("Invalid time string".to_string()).into();
        }
    }
    RuntimeError::type_not_supported(value.get_type()).into()
}

pub fn eval_datetime(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let StringValue(ref s) = value {
        if let StringEnum::String(raw) = s.clone() {
            if let Some(dt) = parse_datetime_local(raw.as_str()) {
                return Ok(DateTimeValue(ValueOrSv::Value(dt)));
            }
            return RuntimeError::eval_error("Invalid datetime string".to_string()).into();
        }
    }
    RuntimeError::type_not_supported(value.get_type()).into()
}

pub fn eval_duration(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let StringValue(ref s) = value {
        if let StringEnum::String(raw) = s.clone() {
            if let Some(dur) = parse_duration_iso8601(raw.as_str()) {
                return Ok(DurationValue(ValueOrSv::Value(dur)));
            }
            return RuntimeError::eval_error("Invalid duration string".to_string()).into();
        }
    }
    RuntimeError::type_not_supported(value.get_type()).into()
}

fn weekday_name_from_num(weekday: i32) -> &'static str {
    match weekday {
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        7 => "Sunday",
        _ => "",
    }
}

fn month_name_from_num(month: u8) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub(crate) fn last_day_of_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

pub fn eval_day_of_week(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let DateValue(ValueOrSv::Value(d)) = value {
        // Monday=1..Sunday=7 (ISO 8601)
        let iso = d.weekday().number_from_monday() as i32;
        Ok(StringValue(StringEnum::String(weekday_name_from_num(iso).to_string())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

pub fn eval_month_of_year(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let DateValue(ValueOrSv::Value(d)) = value {
        let m = d.month() as u8;
        Ok(StringValue(StringEnum::String(month_name_from_num(m).to_string())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

pub fn eval_last_day_of_month(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let DateValue(ValueOrSv::Value(d)) = value {
        let last = last_day_of_month(d.year(), d.month() as u8) as i64;
        Ok(NumberValue(NumberEnum::from(last)))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::typesystem::values::DurationKind;
    use super::*;

    #[test]
    fn test_parse_date_time_datetime() {
        assert!(parse_date_iso("2017-05-03").is_some());
        assert!(parse_time_local("13:10:30").is_some());
        assert!(parse_datetime_local("2017-05-03T13:10:30").is_some());
        assert!(parse_datetime_local("2017-05-03").is_none());
    }

    #[test]
    fn test_time_crate_parsing_values_and_invalids() {
        // Date exact value
        let d = parse_date_iso("2017-05-03").unwrap();
        assert_eq!(d, time::Date::from_calendar_date(2017, time::Month::May, 3).unwrap());
        // Invalid month/day
        assert!(parse_date_iso("2017-13-01").is_none());
        assert!(parse_date_iso("2017-02-30").is_none());

        // Time exact value
        let t = parse_time_local("00:10:59").unwrap();
        assert_eq!(t, time::Time::from_hms(0, 10, 59).unwrap());
        // Invalid time
        assert!(parse_time_local("24:00:00").is_none());
        assert!(parse_time_local("12:60:00").is_none());
        assert!(parse_time_local("12:00:60").is_none());

        // Datetime roundtrip
        let dt = parse_datetime_local("2016-12-09T15:37:00").unwrap();
        assert_eq!(
            dt,
            time::PrimitiveDateTime::new(
                time::Date::from_calendar_date(2016, time::Month::December, 9).unwrap(),
                time::Time::from_hms(15, 37, 0).unwrap(),
            )
        );
        // Invalid datetime
        assert!(parse_datetime_local("2016-12-09 15:37:00").is_none());
        assert!(parse_datetime_local("2016-12-32T00:00:00").is_none());
    }

    #[test]
    fn test_parse_duration_iso() {
        let d1 = parse_duration_iso8601("P1Y6M").unwrap();
        assert!(matches!(d1.kind, DurationKind::YearsMonths));
        assert_eq!(d1.years, 1);
        assert_eq!(d1.months, 6);

        let d2 = parse_duration_iso8601("PT45M").unwrap();
        assert!(matches!(d2.kind, DurationKind::DaysTime));
        assert_eq!(d2.minutes, 45);

        let d3 = parse_duration_iso8601("P2DT3H").unwrap();
        assert!(matches!(d3.kind, DurationKind::DaysTime));
        assert_eq!(d3.days, 2);
        assert_eq!(d3.hours, 3);

        let d4 = parse_duration_iso8601("-P1Y").unwrap();
        assert!(d4.negative);
        assert_eq!(d4.years, 1);

        assert!(parse_duration_iso8601("").is_none());
        assert!(parse_duration_iso8601("P").is_some()); // treated as zero
        assert!(parse_duration_iso8601("P1Y2D").is_none()); // mixed forms rejected
    }
}
