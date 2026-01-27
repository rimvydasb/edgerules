use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::typesystem::errors::RuntimeError;
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::ValueType::{DurationType, PeriodType};
use crate::typesystem::types::{Float, Integer, SpecialValueEnum, TypedValue, ValueType};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::ops::Range;
use std::rc::Rc;
use std::string::String;
use time::{Date, Month, OffsetDateTime, Time};

use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::values::ValueEnum::{Array, BooleanValue, NumberValue, RangeValue, Reference, StringValue};

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, Eq, PartialEq)]
pub enum ValueOrSv<OkValue, SpecialValue> {
    Value(OkValue),
    Sv(SpecialValue),
}

#[allow(non_snake_case)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq, Clone)]
pub enum ValueEnum {
    /// Primitive values
    /// @Todo: move to PrimitiveValue {...} and have Primitive(PrimitiveValue) inside ValueEnum
    // @Todo: must be ValueOrSv<NumberEnum, SpecialValueEnum>, remove NumberEnum::SV
    NumberValue(NumberEnum),
    BooleanValue(bool),
    StringValue(StringEnum),
    DateValue(ValueOrSv<Date, SpecialValueEnum>),
    TimeValue(ValueOrSv<Time, SpecialValueEnum>),
    DateTimeValue(ValueOrSv<OffsetDateTime, SpecialValueEnum>),
    DurationValue(ValueOrSv<DurationValue, SpecialValueEnum>),
    PeriodValue(ValueOrSv<PeriodValue, SpecialValueEnum>),

    /// Non-primitive values
    Array(ArrayValue),

    /// If reference is provided, it is possible to update it if additional calculation is done.
    /// All context is still immutable, but for performance reasons, calculations will not be recalculated.
    Reference(Rc<RefCell<ExecutionContext>>),

    // @Todo: inclusive or exclusive range
    // @Todo: infinity or static
    // @Todo: range is not a value, it one of filter methods
    RangeValue(Range<Integer>),

    // @Todo: the type is provided using this value, but this is kind of not a value - need to rethink this
    TypeValue(ValueType),
}

#[allow(non_snake_case)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq, Clone)]
pub enum ArrayValue {
    EmptyUntyped,

    // Primitive values with homogeneous type
    PrimitivesArray { values: Vec<ValueEnum>, item_type: ValueType },

    // List of object references, representative aggregated type of all objects
    ObjectsArray { values: Vec<Rc<RefCell<ExecutionContext>>>, object_type: Rc<RefCell<ContextObject>> },
    // @Todo: support array in array - currently not supported
}

type ObjectArrayParts = (Vec<Rc<RefCell<ExecutionContext>>>, Rc<RefCell<ContextObject>>);

impl ArrayValue {
    pub fn is_empty_untyped(&self) -> bool {
        matches!(self, ArrayValue::EmptyUntyped)
    }

    pub fn len(&self) -> usize {
        match self {
            ArrayValue::EmptyUntyped => 0,
            ArrayValue::PrimitivesArray { values, .. } => values.len(),
            ArrayValue::ObjectsArray { values, .. } => values.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn list_type(&self) -> ValueType {
        match self {
            ArrayValue::EmptyUntyped => ValueType::ListType(None),
            ArrayValue::PrimitivesArray { item_type, .. } => ValueType::ListType(Some(Box::new(item_type.clone()))),
            ArrayValue::ObjectsArray { object_type, .. } => {
                ValueType::ListType(Some(Box::new(ValueType::ObjectType(Rc::clone(object_type)))))
            }
        }
    }

    pub fn item_type(&self) -> Option<ValueType> {
        match self {
            ArrayValue::EmptyUntyped => None,
            ArrayValue::PrimitivesArray { item_type, .. } => Some(item_type.clone()),
            ArrayValue::ObjectsArray { object_type, .. } => Some(ValueType::ObjectType(Rc::clone(object_type))),
        }
    }

    pub fn primitive_values(&self) -> Option<&Vec<ValueEnum>> {
        match self {
            ArrayValue::PrimitivesArray { values, .. } => Some(values),
            _ => None,
        }
    }

    pub fn primitive_values_mut(&mut self) -> Option<&mut Vec<ValueEnum>> {
        match self {
            ArrayValue::PrimitivesArray { values, .. } => Some(values),
            _ => None,
        }
    }

    pub fn clone_primitive_values(&self) -> Option<Vec<ValueEnum>> {
        match self {
            ArrayValue::PrimitivesArray { values, .. } => Some(values.clone()),
            _ => None,
        }
    }

    pub fn object_values(&self) -> Option<&Vec<Rc<RefCell<ExecutionContext>>>> {
        match self {
            ArrayValue::ObjectsArray { values, .. } => Some(values),
            _ => None,
        }
    }

    pub fn object_values_mut(&mut self) -> Option<&mut Vec<Rc<RefCell<ExecutionContext>>>> {
        match self {
            ArrayValue::ObjectsArray { values, .. } => Some(values),
            _ => None,
        }
    }

    pub fn clone_object_values(&self) -> Option<Vec<Rc<RefCell<ExecutionContext>>>> {
        match self {
            ArrayValue::ObjectsArray { values, .. } => Some(values.clone()),
            _ => None,
        }
    }

    pub fn into_primitives(self) -> Option<(Vec<ValueEnum>, ValueType)> {
        match self {
            ArrayValue::PrimitivesArray { values, item_type } => Some((values, item_type)),
            _ => None,
        }
    }

    pub fn into_objects(self) -> Option<ObjectArrayParts> {
        match self {
            ArrayValue::ObjectsArray { values, object_type } => Some((values, object_type)),
            _ => None,
        }
    }

    pub fn is_primitives(&self) -> bool {
        matches!(self, ArrayValue::PrimitivesArray { .. })
    }

    pub fn is_objects(&self) -> bool {
        matches!(self, ArrayValue::ObjectsArray { .. })
    }
}

const MONTHS_PER_YEAR: i128 = 12;
const SECONDS_PER_MINUTE: i128 = 60;
const MINUTES_PER_HOUR: i128 = 60;
const HOURS_PER_DAY: i128 = 24;
const SECONDS_PER_HOUR: i128 = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;
const SECONDS_PER_DAY: i128 = HOURS_PER_DAY * SECONDS_PER_HOUR;

fn format_time_value(time: &Time) -> String {
    let (hour, minute, second) = time.as_hms();
    format!("{:02}:{:02}:{:02}", hour, minute, second)
}

fn format_datetime_value(value: &OffsetDateTime) -> String {
    let date = value.date();
    let time = value.time();
    let (hour, minute, second) = time.as_hms();
    let month: u8 = date.month() as u8;

    // If offset is UTC, we mimic the old behavior (no offset printed) to satisfy existing tests for "local" datetimes.
    // Ideally we should print 'Z' or '+00:00', but that changes the output contract.
    // For non-UTC offsets, we append the offset.
    if value.offset().is_utc() {
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", date.year(), month, date.day(), hour, minute, second)
    } else {
        let (off_h, off_m, _) = value.offset().as_hms();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{:+03}:{:02}",
            date.year(),
            month,
            date.day(),
            hour,
            minute,
            second,
            off_h,
            off_m.abs()
        )
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, Eq, PartialEq)]
pub struct DurationValue {
    seconds: u64,
    is_negative: bool,
}

impl DurationValue {
    pub fn new(seconds: u64, is_negative: bool) -> Self {
        let neg = is_negative && seconds > 0;
        DurationValue { seconds, is_negative: neg }
    }

    pub fn zero() -> Self {
        DurationValue { seconds: 0, is_negative: false }
    }

    pub fn from_components(
        days: i64,
        hours: i64,
        minutes: i64,
        seconds: i64,
        negative: bool,
    ) -> Result<Self, RuntimeError> {
        let mut total: i128 = 0;
        total = total
            .checked_add(
                i128::from(days)
                    .checked_mul(SECONDS_PER_DAY)
                    .ok_or_else(|| RuntimeError::parsing_from_string(DurationType, 110))?,
            )
            .ok_or_else(|| RuntimeError::parsing_from_string(DurationType, 114))?;
        total = total
            .checked_add(
                i128::from(hours)
                    .checked_mul(SECONDS_PER_HOUR)
                    .ok_or_else(|| RuntimeError::parsing_from_string(DurationType, 111))?,
            )
            .ok_or_else(|| RuntimeError::parsing_from_string(DurationType, 114))?;
        total = total
            .checked_add(
                i128::from(minutes)
                    .checked_mul(SECONDS_PER_MINUTE)
                    .ok_or_else(|| RuntimeError::parsing_from_string(DurationType, 112))?,
            )
            .ok_or_else(|| RuntimeError::parsing_from_string(DurationType, 114))?;
        total = total
            .checked_add(i128::from(seconds))
            .ok_or_else(|| RuntimeError::parsing_from_string(DurationType, 113))?;

        if total < 0 {
            return RuntimeError::parsing_from_string(DurationType, 115).into();
        }

        DurationValue::from_total_seconds(total as u128, negative)
    }

    pub fn from_total_seconds(total_seconds: u128, negative: bool) -> Result<Self, RuntimeError> {
        if total_seconds > u64::MAX as u128 {
            return RuntimeError::parsing_from_string(DurationType, 113).into();
        }
        Ok(DurationValue::new(total_seconds as u64, negative))
    }

    pub fn from_signed_seconds(total_seconds: i128) -> Result<Self, RuntimeError> {
        if total_seconds == 0 {
            return Ok(DurationValue::zero());
        }
        let negative = total_seconds < 0;
        let abs = total_seconds.unsigned_abs();
        DurationValue::from_total_seconds(abs, negative)
    }

    pub fn is_zero(&self) -> bool {
        self.seconds == 0
    }

    pub fn signed_seconds(&self) -> i128 {
        let magnitude = i128::from(self.seconds);
        if self.is_negative {
            -magnitude
        } else {
            magnitude
        }
    }

    pub fn to_iso_string(&self) -> String {
        if self.is_zero() {
            return "PT0S".to_string();
        }

        let mut out = String::new();
        if self.is_negative {
            out.push('-');
        }
        out.push('P');

        let mut remaining = self.seconds;
        let days = remaining / (SECONDS_PER_DAY as u64);
        remaining %= SECONDS_PER_DAY as u64;

        if days > 0 {
            let _ = write!(out, "{}D", days);
        }

        let hours = remaining / (SECONDS_PER_HOUR as u64);
        remaining %= SECONDS_PER_HOUR as u64;
        let minutes = remaining / (SECONDS_PER_MINUTE as u64);
        let seconds = remaining % (SECONDS_PER_MINUTE as u64);

        if hours > 0 || minutes > 0 || seconds > 0 {
            out.push('T');
            if hours > 0 {
                let _ = write!(out, "{}H", hours);
            }
            if minutes > 0 {
                let _ = write!(out, "{}M", minutes);
            }
            if seconds > 0 {
                let _ = write!(out, "{}S", seconds);
            }
        }

        if out.ends_with('P') {
            out.push_str("T0S");
        }
        out
    }

    pub fn normalized_components(&self) -> (i128, i128, i128, i128) {
        let mut remaining = self.seconds;
        let days = remaining / (SECONDS_PER_DAY as u64);
        remaining %= SECONDS_PER_DAY as u64;

        let hours = remaining / (SECONDS_PER_HOUR as u64);
        remaining %= SECONDS_PER_HOUR as u64;

        let minutes = remaining / (SECONDS_PER_MINUTE as u64);
        let seconds = remaining % (SECONDS_PER_MINUTE as u64);

        let day = i128::from(days);
        let hour = i128::from(hours);
        let minute = i128::from(minutes);
        let second = i128::from(seconds);

        if self.is_negative {
            (-day, -hour, -minute, -second)
        } else {
            (day, hour, minute, second)
        }
    }

    pub fn total_seconds_signed(&self) -> i128 {
        self.signed_seconds()
    }

    pub fn total_minutes(&self) -> Float {
        Float::from(self.signed_seconds()) / Float::from(SECONDS_PER_MINUTE)
    }

    pub fn total_hours(&self) -> Float {
        Float::from(self.signed_seconds()) / Float::from(SECONDS_PER_HOUR)
    }
}

impl PartialOrd for DurationValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.signed_seconds().partial_cmp(&other.signed_seconds())
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, Eq, PartialEq)]
pub struct PeriodValue {
    months: u32,
    days: u32,
    is_negative: bool,
}

impl PeriodValue {
    pub fn new(months: u32, days: u32, is_negative: bool) -> Self {
        let neg = is_negative && (months > 0 || days > 0);
        PeriodValue { months, days, is_negative: neg }
    }

    pub fn zero() -> Self {
        PeriodValue { months: 0, days: 0, is_negative: false }
    }

    pub fn from_components(years: i32, months: i32, days: i64, negative: bool) -> Result<Self, RuntimeError> {
        if years < 0 || months < 0 || days < 0 {
            return RuntimeError::parsing_from_string(PeriodType, 104).into();
        }

        let total_months = i128::from(years)
            .checked_mul(MONTHS_PER_YEAR)
            .and_then(|v| v.checked_add(i128::from(months)))
            .ok_or_else(|| RuntimeError::parsing_from_string(PeriodType, 105))?;

        PeriodValue::from_total_parts(total_months, i128::from(days), negative)
    }

    pub fn from_total_parts(months_total: i128, days_total: i128, negative: bool) -> Result<Self, RuntimeError> {
        if months_total < 0 || days_total < 0 {
            return RuntimeError::parsing_from_string(PeriodType, 104).into();
        }

        if months_total > u32::MAX as i128 || days_total > u32::MAX as i128 {
            return RuntimeError::parsing_from_string(PeriodType, 106).into();
        }

        Ok(PeriodValue::new(months_total as u32, days_total as u32, negative))
    }

    pub fn from_signed_parts(months: i128, days: i128) -> Result<Self, RuntimeError> {
        if months == 0 && days == 0 {
            return Ok(PeriodValue::zero());
        }

        if (months > 0 && days < 0) || (months < 0 && days > 0) {
            return RuntimeError::parsing_from_string(PeriodType, 107).into();
        }

        let negative = months < 0 || days < 0;
        let months_abs = months.abs();
        let days_abs = days.abs();
        PeriodValue::from_total_parts(months_abs, days_abs, negative)
    }

    pub fn is_zero(&self) -> bool {
        self.months == 0 && self.days == 0
    }

    pub fn signed_components(&self) -> (i128, i128) {
        let months = i128::from(self.months);
        let days = i128::from(self.days);
        if self.is_negative {
            (-months, -days)
        } else {
            (months, days)
        }
    }

    pub fn to_iso_string(&self) -> String {
        if self.is_zero() {
            return "P0D".to_string();
        }

        let mut out = String::new();
        if self.is_negative {
            out.push('-');
        }
        out.push('P');

        let total_months = self.months as u128;
        let years = total_months / MONTHS_PER_YEAR as u128;
        let months = total_months % MONTHS_PER_YEAR as u128;

        if years > 0 {
            let _ = write!(out, "{}Y", years);
        }
        if months > 0 {
            let _ = write!(out, "{}M", months);
        }
        if self.days > 0 {
            let _ = write!(out, "{}D", self.days);
        }

        out
    }

    pub fn normalized_years_months(&self) -> (i128, i128) {
        let total_months = i128::from(self.months);
        let years = total_months / MONTHS_PER_YEAR;
        let months = total_months % MONTHS_PER_YEAR;

        if self.is_negative {
            (-years, -months)
        } else {
            (years, months)
        }
    }

    pub fn total_months_signed(&self) -> i128 {
        let months = i128::from(self.months);
        if self.is_negative {
            -months
        } else {
            months
        }
    }

    pub fn total_days_signed(&self) -> i128 {
        let days = i128::from(self.days);
        if self.is_negative {
            -days
        } else {
            days
        }
    }
}

pub(crate) fn number_value_from_i128(value: i128) -> ValueEnum {
    if value >= i64::MIN as i128 && value <= i64::MAX as i128 {
        ValueEnum::from(value as i64)
    } else {
        ValueEnum::NumberValue(NumberEnum::from(Float::from(value)))
    }
}

impl From<ValueEnum> for EObjectContent<ExecutionContext> {
    fn from(value: ValueEnum) -> Self {
        match value {
            Reference(reference) => EObjectContent::ObjectRef(reference),
            other => EObjectContent::ConstantValue(other),
        }
    }
}

impl From<ValueType> for EObjectContent<ContextObject> {
    fn from(value: ValueType) -> Self {
        EObjectContent::Definition(value)
    }
}

impl TypedValue for ValueEnum {
    fn get_type(&self) -> ValueType {
        match self {
            ValueEnum::NumberValue(_) => ValueType::NumberType,
            ValueEnum::BooleanValue(_) => ValueType::BooleanType,
            ValueEnum::StringValue(_) => ValueType::StringType,
            ValueEnum::Array(array) => array.list_type(),
            ValueEnum::Reference(obj) => obj.borrow().get_type(),
            ValueEnum::DateValue(_) => ValueType::DateType,
            ValueEnum::TimeValue(_) => ValueType::TimeType,
            ValueEnum::DateTimeValue(_) => ValueType::DateTimeType,
            ValueEnum::DurationValue(_) => ValueType::DurationType,
            ValueEnum::PeriodValue(_) => ValueType::PeriodType,
            ValueEnum::RangeValue(_) => ValueType::RangeType,

            // @Todo: that is incorrect, because held type is not a result of get_type. Must return specific TypeValue as a type
            ValueEnum::TypeValue(value_type) => value_type.clone(),
        }
    }
}

impl Display for ValueEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Array(array) => match array {
                ArrayValue::EmptyUntyped => write!(f, "[]"),
                ArrayValue::PrimitivesArray { values, .. } => {
                    let parts: Vec<String> = values.iter().map(|v| format!("{}", v)).collect();
                    write!(f, "[{}]", parts.join(", "))
                }
                ArrayValue::ObjectsArray { values, .. } => {
                    let parts: Vec<String> =
                        values.iter().map(|ctx| format!("{}", ValueEnum::Reference(Rc::clone(ctx)))).collect();
                    write!(f, "[{}]", parts.join(", "))
                }
            },
            NumberValue(number) => write!(f, "{}", number),
            StringValue(str) => write!(f, "{}", str),
            Reference(reference) => {
                let mut code = reference.borrow().to_code();
                if let Some(stripped) = code.strip_prefix("#child: ") {
                    code = stripped.to_string();
                }
                code = code.trim_end().to_string();
                write!(f, "{}", code)
            }
            BooleanValue(value) => {
                if *value {
                    f.write_str("true")
                } else {
                    f.write_str("false")
                }
            }
            RangeValue(range) => write!(f, "{}..{}", range.start, range.end - 1),
            ValueEnum::DateValue(date) => match date {
                ValueOrSv::Value(date) => write!(f, "{}", date),
                ValueOrSv::Sv(sv) => write!(f, "{}", sv),
            },
            ValueEnum::TimeValue(time) => match time {
                ValueOrSv::Value(time) => write!(f, "{}", format_time_value(time)),
                ValueOrSv::Sv(sv) => write!(f, "{}", sv),
            },
            ValueEnum::DateTimeValue(date_time) => match date_time {
                ValueOrSv::Value(date_time) => write!(f, "{}", format_datetime_value(date_time)),
                ValueOrSv::Sv(sv) => write!(f, "{}", sv),
            },
            ValueEnum::DurationValue(duration) => match duration {
                ValueOrSv::Value(dur) => write!(f, "{}", dur.to_iso_string()),
                ValueOrSv::Sv(sv) => write!(f, "{}", sv),
            },
            ValueEnum::PeriodValue(period) => match period {
                ValueOrSv::Value(per) => write!(f, "{}", per.to_iso_string()),
                ValueOrSv::Sv(sv) => write!(f, "{}", sv),
            },
            ValueEnum::TypeValue(type_value) => {
                write!(f, "{}", type_value)
            }
        }
    }
}

impl From<Float> for ValueEnum {
    fn from(value: Float) -> Self {
        NumberValue(NumberEnum::from(value))
    }
}

impl From<f64> for ValueEnum {
    fn from(value: f64) -> Self {
        NumberValue(NumberEnum::from(value))
    }
}

impl From<Integer> for ValueEnum {
    fn from(value: Integer) -> Self {
        NumberValue(NumberEnum::from(value))
    }
}

impl From<u8> for ValueEnum {
    fn from(value: u8) -> Self {
        NumberValue(NumberEnum::from(value as i64))
    }
}

impl From<i32> for ValueEnum {
    fn from(value: i32) -> Self {
        NumberValue(NumberEnum::from(value as i64))
    }
}

impl From<Month> for ValueEnum {
    fn from(value: Month) -> Self {
        NumberValue(NumberEnum::from(value as i64))
    }
}

impl From<DurationValue> for ValueEnum {
    fn from(value: DurationValue) -> Self {
        ValueEnum::DurationValue(ValueOrSv::Value(value))
    }
}

impl From<PeriodValue> for ValueEnum {
    fn from(value: PeriodValue) -> Self {
        ValueEnum::PeriodValue(ValueOrSv::Value(value))
    }
}
