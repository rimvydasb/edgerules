use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::{Float, Integer, SpecialValueEnum, TypedValue, ValueType};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DurationKind {
    YearsMonths,
    DaysTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurationValue {
    pub negative: bool,
    pub kind: DurationKind,
    pub years: i32,
    pub months: i32,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
}

impl DurationValue {
    pub fn ym(years: i32, months: i32, negative: bool) -> Self {
        DurationValue {
            negative,
            kind: DurationKind::YearsMonths,
            years,
            months,
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }

    pub fn dt(days: i64, hours: i64, minutes: i64, seconds: i64, negative: bool) -> Self {
        DurationValue {
            negative,
            kind: DurationKind::DaysTime,
            years: 0,
            months: 0,
            days,
            hours,
            minutes,
            seconds,
        }
    }

    pub fn is_zero(&self) -> bool {
        match self.kind {
            DurationKind::YearsMonths => self.years == 0 && self.months == 0,
            DurationKind::DaysTime => {
                self.days == 0 && self.hours == 0 && self.minutes == 0 && self.seconds == 0
            }
        }
    }

    pub(crate) fn signed_months(&self) -> i128 {
        let total_months = (self.years as i128) * 12 + self.months as i128;
        if self.negative {
            -total_months
        } else {
            total_months
        }
    }

    pub(crate) fn signed_seconds(&self) -> i128 {
        let total_seconds =
            ((((self.days as i128 * 24) + self.hours as i128) * 60 + self.minutes as i128) * 60)
                + self.seconds as i128;
        if self.negative {
            -total_seconds
        } else {
            total_seconds
        }
    }

    pub(crate) fn from_total_months(total: i128) -> Self {
        let negative = total < 0;
        let abs_total = total.abs();
        let years = (abs_total / 12) as i32;
        let months = (abs_total % 12) as i32;
        DurationValue::ym(years, months, negative)
    }

    pub(crate) fn from_total_seconds(total: i128) -> Self {
        let negative = total < 0;
        let abs_total = total.abs();
        let days = (abs_total / 86_400) as i64;
        let mut remainder = abs_total % 86_400;
        let hours = (remainder / 3_600) as i64;
        remainder %= 3_600;
        let minutes = (remainder / 60) as i64;
        let seconds = (remainder % 60) as i64;
        DurationValue::dt(days, hours, minutes, seconds, negative)
    }
}

impl PartialOrd for DurationValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.kind != other.kind {
            return None;
        }

        match self.kind {
            DurationKind::YearsMonths => Some(self.signed_months().cmp(&other.signed_months())),
            DurationKind::DaysTime => Some(self.signed_seconds().cmp(&other.signed_seconds())),
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
                ValueOrSv::Value(dur) => {
                    // Build minimal ISO-8601 style string
                    let mut s = std::string::String::new();
                    if dur.negative && !dur.is_zero() {
                        s.push('-');
                    }
                    s.push('P');
                    match dur.kind {
                        DurationKind::YearsMonths => {
                            if dur.years != 0 {
                                s.push_str(&format!("{}Y", dur.years.abs()));
                            }
                            if dur.months != 0 {
                                s.push_str(&format!("{}M", dur.months.abs()));
                            }
                        }
                        DurationKind::DaysTime => {
                            if dur.days != 0 {
                                s.push_str(&format!("{}D", dur.days.abs()));
                            }
                            if dur.hours != 0 || dur.minutes != 0 || dur.seconds != 0 {
                                s.push('T');
                                if dur.hours != 0 {
                                    s.push_str(&format!("{}H", dur.hours.abs()));
                                }
                                if dur.minutes != 0 {
                                    s.push_str(&format!("{}M", dur.minutes.abs()));
                                }
                                if dur.seconds != 0 {
                                    s.push_str(&format!("{}S", dur.seconds.abs()));
                                }
                            }
                        }
                    }
                    if s == "P" {
                        s.push('T');
                        s.push('0');
                        s.push('S');
                    }
                    write!(f, "{}", s)
                }
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
