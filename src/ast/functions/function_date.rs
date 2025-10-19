use crate::ast::Link;
use crate::typesystem::errors::{LinkingError, RuntimeError};
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::ValueType::{DateType, StringType};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::{
    DateTimeValue, DateValue, DurationValue as DurationVariant, PeriodValue as PeriodVariant,
    StringValue, TimeValue,
};
use crate::typesystem::values::ValueOrSv;
use crate::typesystem::values::{DurationValue, PeriodValue};
use time::macros::format_description;
use time::Month;

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

pub fn parse_duration_iso8601(s: &str) -> Option<DurationValue> {
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

    let mut days: i64 = 0;
    let mut hours: i64 = 0;
    let mut minutes: i64 = 0;
    let mut seconds: i64 = 0;

    let mut in_time = false;
    let mut saw_any = false;

    let mut num_start = idx;
    while idx <= bytes.len() {
        if idx == bytes.len() || bytes[idx].is_ascii_alphabetic() {
            if idx == num_start {
                if idx == bytes.len() {
                    break;
                }
                if bytes[idx] == b'T' {
                    if in_time {
                        return None;
                    }
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
            if num < 0 {
                return None;
            }
            if idx < bytes.len() {
                match bytes[idx] {
                    b'Y' => return None,
                    b'M' if !in_time => return None,
                    b'D' => {
                        days = num;
                        saw_any = true;
                    }
                    b'T' => {
                        if in_time {
                            return None;
                        }
                        in_time = true;
                        idx += 1;
                        num_start = idx;
                        continue;
                    }
                    b'H' => {
                        hours = num;
                        saw_any = true;
                    }
                    b'M' if in_time => {
                        minutes = num;
                        saw_any = true;
                    }
                    b'S' => {
                        seconds = num;
                        saw_any = true;
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

    if !saw_any {
        return None;
    }

    DurationValue::from_components(days, hours, minutes, seconds, negative).ok()
}

pub fn parse_period_iso8601(s: &str) -> Option<PeriodValue> {
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

    let mut num_start = idx;
    let mut saw_any = false;

    while idx <= bytes.len() {
        if idx == bytes.len() || bytes[idx].is_ascii_alphabetic() {
            if idx == num_start {
                if idx == bytes.len() {
                    break;
                }
                return None;
            }
            let num: i64 = std::str::from_utf8(&bytes[num_start..idx])
                .ok()?
                .parse()
                .ok()?;
            if num < 0 {
                return None;
            }

            if idx < bytes.len() {
                match bytes[idx] {
                    b'Y' => {
                        years = num as i32;
                        saw_any = true;
                    }
                    b'M' => {
                        months = num as i32;
                        saw_any = true;
                    }
                    b'D' => {
                        days = num;
                        saw_any = true;
                    }
                    b'T' | b'H' | b'S' => return None,
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

    if !saw_any {
        return None;
    }

    PeriodValue::from_components(years, months, days, negative).ok()
}

fn shift_date_by_months_safe(
    date: time::Date,
    months_delta: i64,
) -> Result<time::Date, RuntimeError> {
    if months_delta == 0 {
        return Ok(date);
    }

    let year = i64::from(date.year());
    let mut month = i64::from(date.month() as i32);
    month += months_delta;

    let mut new_year = year + (month - 1) / 12;
    let mut new_month = (month - 1) % 12 + 1;
    if new_month <= 0 {
        new_year -= 1;
        new_month += 12;
    }

    if new_year < i32::MIN as i64 || new_year > i32::MAX as i64 {
        return RuntimeError::eval_error("Date adjustment overflowed year range".to_string())
            .into();
    }

    let new_month_u8 = new_month as u8;
    let day = date.day();
    let last = last_day_of_month(new_year as i32, new_month_u8);
    let new_day = if day > last { last } else { day };

    time::Date::from_calendar_date(
        new_year as i32,
        Month::try_from(new_month_u8).map_err(|_| {
            RuntimeError::eval_error("Invalid month produced during calendarDiff".to_string())
        })?,
        new_day,
    )
    .map_err(|_| RuntimeError::eval_error("Invalid date produced during calendarDiff".to_string()))
}

pub fn validate_binary_date_date(left: ValueType, right: ValueType) -> Link<()> {
    expect_date_arg(left)?;
    expect_date_arg(right)
}

pub fn return_period_type_binary(_: ValueType, _: ValueType) -> ValueType {
    ValueType::PeriodType
}

pub fn eval_calendar_diff(left: ValueEnum, right: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    match (left, right) {
        (DateValue(ValueOrSv::Value(start)), DateValue(ValueOrSv::Value(end))) => {
            let (negative, earlier, later) = if start <= end {
                (false, start, end)
            } else {
                (true, end, start)
            };

            let mut months_total = i64::from(later.year() - earlier.year()) * 12
                + i64::from(later.month() as i32 - earlier.month() as i32);
            if months_total < 0 {
                months_total = 0;
            }

            let mut anchor = shift_date_by_months_safe(earlier, months_total)?;
            if anchor > later && months_total > 0 {
                months_total -= 1;
                anchor = shift_date_by_months_safe(earlier, months_total)?;
            }

            let day_diff = (later - anchor).whole_days();
            let period = PeriodValue::from_total_parts(
                i128::from(months_total),
                i128::from(day_diff),
                negative,
            )?;
            Ok(PeriodVariant(ValueOrSv::Value(period)))
        }
        (left_value, right_value) => RuntimeError::eval_error(format!(
            "calendarDiff expects date arguments, received '{}' and '{}'",
            left_value.get_type(),
            right_value.get_type()
        ))
        .into(),
    }
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
                return Ok(DurationVariant(ValueOrSv::Value(dur)));
            }
            return RuntimeError::eval_error("Invalid duration string".to_string()).into();
        }
    }
    RuntimeError::type_not_supported(value.get_type()).into()
}

pub fn eval_period(value: ValueEnum) -> Result<ValueEnum, RuntimeError> {
    if let StringValue(ref s) = value {
        if let StringEnum::String(raw) = s.clone() {
            if let Some(per) = parse_period_iso8601(raw.as_str()) {
                return Ok(PeriodVariant(ValueOrSv::Value(per)));
            }
            return RuntimeError::eval_error("Invalid period string".to_string()).into();
        }
    }
    RuntimeError::type_not_supported(value.get_type()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_period_months_only() {
        assert!(parse_period_iso8601("P1M").is_some());
        assert!(parse_period_iso8601("P18M").is_some());
        assert!(parse_period_iso8601("P1Y2M3D").is_some());
    }
}
