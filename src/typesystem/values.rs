use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::typesystem::errors::RuntimeError;
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::{Float, Integer, SpecialValueEnum, TypedValue, ValueType};
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::rc::Rc;
use time::{Date, PrimitiveDateTime, Time};

use crate::ast::utils::results_to_code;

use crate::runtime::execution_context::ExecutionContext;
use crate::typesystem::types::string::StringEnum::String;
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
    NumberValue(NumberEnum),
    BooleanValue(bool),
    StringValue(StringEnum),
    Array(Vec<Result<ValueEnum, RuntimeError>>, ValueType),
    // @Todo: inclusive or exclusive range
    // @Todo: infinity or static
    // @Todo: range is not a value, it one of filter methods
    RangeValue(Range<Integer>),
    /// If reference is provided, it is possible to update it if additional calculation is done. All context is still immutable, but for performance reasons, calculations will not be recalculated.
    Reference(Rc<RefCell<ExecutionContext>>),
    DateValue(ValueOrSv<Date, SpecialValueEnum>),
    TimeValue(ValueOrSv<Time, SpecialValueEnum>),
    DateTimeValue(ValueOrSv<PrimitiveDateTime, SpecialValueEnum>),
    DurationValue(ValueOrSv<DurationValue, SpecialValueEnum>),
    TypeValue(ValueType),
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

// impl ValueEnum {
//     pub fn unwrap_array(value: ValueEnum) -> ValueEnum {
//         if let Array(mut array, existing_type) = value {
//             if array.len() == 1 {
//                 if let Ok(Array(_, _)) = array.get(0).unwrap() {
//                     array.pop().unwrap().unwrap()
//                 } else {
//                     Array(array, existing_type)
//                 }
//             } else {
//                 Array(array, existing_type)
//             }
//         } else {
//             value
//         }
//     }
// }

impl TypedValue for ValueEnum {
    fn get_type(&self) -> ValueType {
        match self {
            ValueEnum::NumberValue(_) => ValueType::NumberType,
            ValueEnum::BooleanValue(_) => ValueType::BooleanType,
            ValueEnum::StringValue(_) => ValueType::StringType,
            ValueEnum::Array(_, value_type) => value_type.clone(),
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
            Array(args, _) => {
                write!(f, "[{}]", results_to_code(args))
            }
            NumberValue(number) => write!(f, "{}", number),
            StringValue(str) => write!(f, "{}", str),
            Reference(reference) => write!(f, "{}", reference.borrow()),
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
                    if s == "P" { s.push('T'); s.push('0'); s.push('S'); }
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

// Todo: eliminate assignet_to_field
// impl From<ContextObject> for ValueEnum {
//     fn from(object: ContextObject) -> Self {
//         Reference(Rc::new(RefCell::new(ExecutionContext::create_for(Rc::new( RefCell::new(object)), "#root".to_string()))))
//     }
// }

impl From<Float> for ValueEnum {
    fn from(value: Float) -> Self {
        NumberValue(NumberEnum::from(value))
    }
}

impl From<Rc<RefCell<ExecutionContext>>> for ValueEnum {
    fn from(value: Rc<RefCell<ExecutionContext>>) -> Self {
        Reference(value)
    }
}

impl From<Integer> for ValueEnum {
    fn from(value: Integer) -> Self {
        NumberValue(NumberEnum::from(value))
    }
}

impl From<&str> for ValueEnum {
    fn from(value: &str) -> Self {
        StringValue(String(value.to_string()))
    }
}

impl<T> From<Vec<T>> for ValueEnum
where
    T: Into<ValueEnum>,
{
    fn from(values: Vec<T>) -> Self {
        if values.is_empty() {
            Array(Vec::new(), ValueType::UndefinedType)
        } else {
            let values_enum = values
                .into_iter()
                .map(|value| Ok(value.into()))
                .collect::<Vec<Result<ValueEnum, RuntimeError>>>();

            let first_value = values_enum.first().unwrap();
            let init_type = first_value.as_ref().unwrap().get_type().clone();

            Array(values_enum, init_type)
        }
    }
}

// impl From<Vec<Result<ValueEnum, RuntimeError>>> for ValueEnum {
//     fn from(values: Vec<Result<ValueEnum, RuntimeError>>) -> Self {
//
//         match values.get(0) {
//             None => Array(values, ValueType::AnyType),
//             Some(value) => {
//                 match value {
//                     Ok(value) => Array(values, value.get_type()),
//                     Err(_) => Array(values, ValueType::AnyType)
//                 }
//             }
//         }
//     }
// }
