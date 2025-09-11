use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::ValueType::{DateType, StringType};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{
    DateTimeValue, DateValue, DurationValue, StringValue, TimeValue,
};
use crate::typesystem::values::ValueOrSv;
use time::macros::format_description;

pub fn expect_string_arg(arg: ValueType) -> Link<()> {
    LinkingError::expect_type(None, arg, &[StringType]).map(|_| ())
}

pub fn expect_date_arg(arg: ValueType) -> Link<()> {
    LinkingError::expect_type(None, arg, &[DateType]).map(|_| ())
}

fn parse_date_iso(s: &str) -> Option<time::Date> {
    let fmt = format_description!("[year]-[month]-[day]");
    time::Date::parse(s, &fmt).ok()
}

fn parse_time_local(s: &str) -> Option<time::Time> {
    let fmt = format_description!("[hour]:[minute]:[second]");
    time::Time::parse(s, &fmt).ok()
}

fn parse_datetime_local(s: &str) -> Option<time::PrimitiveDateTime> {
    let fmt = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    time::PrimitiveDateTime::parse(s, &fmt).ok()
}

pub fn parse_duration_iso8601(s: &str) -> Option<crate::typesystem::values::DurationValue> {
    if s.is_empty() {
        return None;
    }
    let mut negative = false;
    let mut idx = 0;
    let bytes = s.as_bytes();
    if bytes[idx] == b'-' {
        negative = true;
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] != b'P' {
        return None;
    }
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
            if idx == num_start {
                if idx == bytes.len() {
                    break;
                }
                if bytes[idx] == b'T' {
                    in_time = true;
                    idx += 1;
                    num_start = idx;
                    continue;
                }
                return None;
            }
            let num: i64 = std::str::from_utf8(&bytes[num_start..idx])
                .ok()?
                .parse()
                .ok()?;
            if idx < bytes.len() {
                match bytes[idx] {
                    b'Y' => {
                        years = num as i32;
                        saw_ym = true;
                    }
                    b'M' if !in_time => {
                        months = num as i32;
                        saw_ym = true;
                    }
                    b'D' => {
                        days = num;
                        saw_dt = true;
                    }
                    b'T' => {
                        in_time = true;
                        idx += 1;
                        num_start = idx;
                        continue;
                    }
                    b'H' => {
                        hours = num;
                    }
                    b'M' if in_time => {
                        minutes = num;
                    }
                    b'S' => {
                        seconds = num;
                    }
                    _ => return None,
                }
                idx += 1;
                num_start = idx;
                continue;
            } else {
                break;
            }
        }
        idx += 1;
    }

    if saw_ym && saw_dt {
        return None;
    }
    let v = if saw_ym {
        crate::typesystem::values::DurationValue::ym(years, months, negative)
    } else {
        crate::typesystem::values::DurationValue::dt(days, hours, minutes, seconds, negative)
    };
    Some(v)
}

pub(crate) fn last_day_of_month(year: i32, month: u8) -> u8 {
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

pub fn eval_day_of_week(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let DateValue(ValueOrSv::Value(d)) = value {
        let iso = d.weekday().number_from_monday() as i32;
        let name = match iso {
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            7 => "Sunday",
            _ => "",
        };
        Ok(StringValue(StringEnum::String(name.to_string())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

pub fn eval_month_of_year(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let DateValue(ValueOrSv::Value(d)) = value {
        let m = d.month() as u8;
        let name = match m {
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
        };
        Ok(StringValue(StringEnum::String(name.to_string())))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

pub fn eval_last_day_of_month(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let DateValue(ValueOrSv::Value(d)) = value {
        let last = last_day_of_month(d.year(), d.month() as u8) as i64;
        Ok(ValueEnum::from(last))
    } else {
        RuntimeError::type_not_supported(value.get_type()).into()
    }
}

pub fn eval_date(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let StringValue(ref s) = value {
        if let StringEnum::String(raw) = s.clone() {
            if let Some(d) = parse_date_iso(raw.as_str()) {
                return Ok(DateValue(ValueOrSv::Value(d)));
            }
            return RuntimeError::eval_error("Invalid date string".to_string()).into();
        }
    }
    RuntimeError::type_not_supported(value.get_type()).into()
}

pub fn eval_time(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let StringValue(ref s) = value {
        if let StringEnum::String(raw) = s.clone() {
            if let Some(t) = parse_time_local(raw.as_str()) {
                return Ok(TimeValue(ValueOrSv::Value(t)));
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
