use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::typesystem::errors::RuntimeError;
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::{Float, Integer, SpecialValueEnum, TypedValue, ValueType};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter, Write};
use std::ops::Range;
use std::rc::Rc;
use std::string::String;
use time::{Date, Month, PrimitiveDateTime, Time};

use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::values::ValueEnum::{
    Array, BooleanValue, NumberValue, RangeValue, Reference, StringValue,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueOrSv<OkValue, SpecialValue> {
    Value(OkValue),
    Sv(SpecialValue),
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Clone)]
pub enum ValueEnum {
    /// Primitive values
    /// @Todo: move to PrimitiveValue {...} and have Primitive(PrimitiveValue) inside ValueEnum
    // @Todo: must be ValueOrSv<NumberEnum, SpecialValueEnum>, remove NumberEnum::SV
    NumberValue(NumberEnum),
    BooleanValue(bool),
    StringValue(StringEnum),
    DateValue(ValueOrSv<Date, SpecialValueEnum>),
    TimeValue(ValueOrSv<Time, SpecialValueEnum>),
    DateTimeValue(ValueOrSv<PrimitiveDateTime, SpecialValueEnum>),
    DurationValue(ValueOrSv<DurationValue, SpecialValueEnum>),

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
#[derive(Debug, PartialEq, Clone)]
pub enum ArrayValue {
    EmptyUntyped,

    // Primitive values with homogeneous type
    PrimitivesArray {
        values: Vec<ValueEnum>,
        item_type: ValueType,
    },

    // List of object references, representative aggregated type of all objects
    ObjectsArray {
        values: Vec<Rc<RefCell<ExecutionContext>>>,
        object_type: Rc<RefCell<ContextObject>>,
    },
    // @Todo: support array in array - currently not supported
}

type ObjectArrayParts = (
    Vec<Rc<RefCell<ExecutionContext>>>,
    Rc<RefCell<ContextObject>>,
);

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
            ArrayValue::PrimitivesArray { item_type, .. } => {
                ValueType::ListType(Some(Box::new(item_type.clone())))
            }
            ArrayValue::ObjectsArray { object_type, .. } => ValueType::ListType(Some(Box::new(
                ValueType::ObjectType(Rc::clone(object_type)),
            ))),
        }
    }

    pub fn item_type(&self) -> Option<ValueType> {
        match self {
            ArrayValue::EmptyUntyped => None,
            ArrayValue::PrimitivesArray { item_type, .. } => Some(item_type.clone()),
            ArrayValue::ObjectsArray { object_type, .. } => {
                Some(ValueType::ObjectType(Rc::clone(object_type)))
            }
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
            ArrayValue::ObjectsArray {
                values,
                object_type,
            } => Some((values, object_type)),
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
const SECONDS_PER_FEBRUARY_NON_LEAP: i128 = 28 * SECONDS_PER_DAY;

// @Todo: it must be struct and hold only seconds and negative flag
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DurationValue {
    YearsMonths { months: i128 },
    DaysTime { seconds: i128 },
    Combined { months: i128, seconds: i128 },
}

impl DurationValue {
    pub fn ym(years: i32, months: i32, negative: bool) -> Self {
        let total_months = i128::from(years) * MONTHS_PER_YEAR + i128::from(months);
        let signed = if negative {
            -total_months.abs()
        } else {
            total_months
        };
        DurationValue::YearsMonths { months: signed }
    }

    pub fn dt(days: i64, hours: i64, minutes: i64, seconds: i64, negative: bool) -> Self {
        DurationValue::dt_from_total_seconds(
            ((((i128::from(days) * HOURS_PER_DAY) + i128::from(hours)) * MINUTES_PER_HOUR
                + i128::from(minutes))
                * SECONDS_PER_MINUTE)
                + i128::from(seconds),
            negative,
        )
    }

    pub fn dt_from_total_seconds(total_seconds: i128, negative: bool) -> Self {
        let signed = if negative {
            -total_seconds.abs()
        } else {
            total_seconds
        };
        DurationValue::DaysTime { seconds: signed }
    }

    pub fn combined(
        years: i32,
        months: i32,
        days: i64,
        hours: i64,
        minutes: i64,
        seconds: i64,
        negative: bool,
    ) -> Self {
        let months_total = i128::from(years) * MONTHS_PER_YEAR + i128::from(months);
        let seconds_total = ((((i128::from(days) * HOURS_PER_DAY) + i128::from(hours))
            * MINUTES_PER_HOUR
            + i128::from(minutes))
            * SECONDS_PER_MINUTE)
            + i128::from(seconds);

        let months_signed = if negative {
            -months_total.abs()
        } else {
            months_total
        };
        let seconds_signed = if negative {
            -seconds_total.abs()
        } else {
            seconds_total
        };

        match (months_signed, seconds_signed) {
            (0, 0) => DurationValue::DaysTime { seconds: 0 },
            (0, seconds) => DurationValue::DaysTime { seconds },
            (months, 0) => DurationValue::YearsMonths { months },
            (months, seconds) => DurationValue::Combined { months, seconds },
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            DurationValue::YearsMonths { months } => *months == 0,
            DurationValue::DaysTime { seconds } => *seconds == 0,
            DurationValue::Combined { months, seconds } => *months == 0 && *seconds == 0,
        }
    }

    pub(crate) fn components(&self) -> (i128, i128) {
        match self {
            DurationValue::YearsMonths { months } => (*months, 0),
            DurationValue::DaysTime { seconds } => (0, *seconds),
            DurationValue::Combined { months, seconds } => (*months, *seconds),
        }
    }

    pub(crate) fn from_components(
        months_total: i128,
        seconds_total: i128,
    ) -> Result<Self, RuntimeError> {
        if months_total != 0
            && seconds_total != 0
            && ((months_total > 0 && seconds_total < 0) || (months_total < 0 && seconds_total > 0))
        {
            return RuntimeError::eval_error(
                "Cannot represent duration with mixed month and day/time signs".to_string(),
            )
            .into();
        }

        match (months_total, seconds_total) {
            (0, 0) => Ok(DurationValue::DaysTime { seconds: 0 }),
            (months, 0) => Ok(DurationValue::YearsMonths { months }),
            (0, seconds) => Ok(DurationValue::DaysTime { seconds }),
            (months, seconds) => Ok(DurationValue::Combined { months, seconds }),
        }
    }

    pub fn to_iso_string(&self) -> String {
        if self.is_zero() {
            return "PT0S".to_string();
        }

        let mut out = String::new();
        if self.is_negative() {
            out.push('-');
        }
        out.push('P');

        match self {
            DurationValue::YearsMonths { months } => {
                append_months_part(months.abs(), &mut out);
            }
            DurationValue::DaysTime { seconds } => {
                append_days_time_part(seconds.abs(), &mut out);
            }
            DurationValue::Combined { months, seconds } => {
                append_months_part(months.abs(), &mut out);
                append_days_time_part(seconds.abs(), &mut out);
            }
        }

        if out.ends_with('P') {
            out.push_str("T0S");
        }

        out
    }

    fn is_negative(&self) -> bool {
        match self {
            DurationValue::YearsMonths { months } => *months < 0,
            DurationValue::DaysTime { seconds } => *seconds < 0,
            DurationValue::Combined { months, seconds } => {
                if *months != 0 {
                    *months < 0
                } else {
                    *seconds < 0
                }
            }
        }
    }
}

impl PartialOrd for DurationValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        pub type DV = DurationValue;
        match (self, other) {
            (DV::YearsMonths { months: a }, DV::YearsMonths { months: b }) => Some(a.cmp(b)),
            (DV::YearsMonths { months: a }, DV::DaysTime { seconds: b }) => {
                if (b < &SECONDS_PER_FEBRUARY_NON_LEAP && *a > 0){
                    // this is a work-around for the fact that months cannot be precisely converted to days
                    Some(Ordering::Greater)
                } else {
                    None
                }
            },
            (DV::DaysTime { seconds: a }, DV::YearsMonths { months: b }) => {
                if (a < &SECONDS_PER_FEBRUARY_NON_LEAP && *b > 0){
                    // this is a work-around for the fact that months cannot be precisely converted to days
                    Some(Ordering::Less)
                } else {
                    None
                }
            },
            (DV::DaysTime { seconds: a }, DV::DaysTime { seconds: b }) => Some(a.cmp(b)),
            (
                DV::Combined {
                    months: am,
                    seconds: as0,
                },
                DV::Combined {
                    months: bm,
                    seconds: bs,
                },
            ) => match am.cmp(bm) {
                Ordering::Equal => Some(as0.cmp(bs)),
                other => Some(other),
            },
            _ => None,
        }
    }
}

fn append_months_part(months_abs: i128, target: &mut String) {
    if months_abs == 0 {
        return;
    }

    let years = months_abs / MONTHS_PER_YEAR;
    let months = months_abs % MONTHS_PER_YEAR;

    if years != 0 {
        let _ = write!(target, "{}Y", years);
    }
    if months != 0 {
        let _ = write!(target, "{}M", months);
    }
}

fn append_days_time_part(seconds_abs: i128, target: &mut String) {
    if seconds_abs == 0 {
        return;
    }

    let days = seconds_abs / SECONDS_PER_DAY;
    let mut remainder = seconds_abs % SECONDS_PER_DAY;

    if days != 0 {
        let _ = write!(target, "{}D", days);
    }

    let hours = remainder / SECONDS_PER_HOUR;
    remainder %= SECONDS_PER_HOUR;
    let minutes = remainder / SECONDS_PER_MINUTE;
    let seconds = remainder % SECONDS_PER_MINUTE;

    if hours != 0 || minutes != 0 || seconds != 0 {
        target.push('T');
        if hours != 0 {
            let _ = write!(target, "{}H", hours);
        }
        if minutes != 0 {
            let _ = write!(target, "{}M", minutes);
        }
        if seconds != 0 {
            let _ = write!(target, "{}S", seconds);
        }
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
                    let parts: Vec<String> = values
                        .iter()
                        .map(|ctx| format!("{}", ValueEnum::Reference(Rc::clone(ctx))))
                        .collect();
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
                ValueOrSv::Value(time) => write!(f, "{}", time),
                ValueOrSv::Sv(sv) => write!(f, "{}", sv),
            },
            ValueEnum::DateTimeValue(date_time) => match date_time {
                ValueOrSv::Value(date_time) => write!(f, "{}", date_time),
                ValueOrSv::Sv(sv) => write!(f, "{}", sv),
            },
            ValueEnum::DurationValue(duration) => match duration {
                ValueOrSv::Value(dur) => write!(f, "{}", dur.to_iso_string()),
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
