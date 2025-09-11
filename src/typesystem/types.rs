use crate::ast::context::context_object::ContextObject;
use crate::typesystem::errors::ParseErrorEnum;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub trait TypedValue {
    fn get_type(&self) -> ValueType;

    #[allow(dead_code)]
    fn instance_of(&self, another: &dyn TypedValue) -> bool {
        self.get_type() == another.get_type()
    }

    #[allow(dead_code)]
    fn instance_of_type(&self, another: ValueType) -> bool {
        self.get_type() == another
    }
}

/// FEEL related documentation:
/// https://docs.camunda.io/docs/components/modeler/feel/language-guide/feel-data-types/
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum ValueType {
    NumberType,
    StringType,
    BooleanType,
    DateType,
    TimeType,
    DateTimeType,

    // Range is not a type, it is a filter method
    RangeType,

    // Only homogenous lists are supported now
    ListType(Box<ValueType>),

    // Represents Years-months-duration and Days-time-duration
    DurationType,

    /// **Main considerations:**
    /// - Context there is the same as FEEL.
    /// - ContextObject is a Context type itself. No other meta layer should be introduced.
    /// - ExecutionContext type is a ContextObject
    /// - ContextObject instance is ExecutionContext
    /// - @Todo: it is a question if RefCell is necessary - context object must be immutable btw
    ObjectType(Rc<RefCell<ContextObject>>),

    UndefinedType,
    // Todo: remove it and update
    //AnyType,
}

impl ValueType {
    pub fn get_list_type(&self) -> Option<ValueType> {
        match self {
            ValueType::ListType(list_type) => Some(*list_type.clone()),
            _ => None,
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::NumberType => f.write_str("number"),
            ValueType::StringType => f.write_str("string"),
            ValueType::BooleanType => f.write_str("boolean"),
            ValueType::DateType => f.write_str("date"),
            ValueType::TimeType => f.write_str("time"),
            ValueType::DateTimeType => f.write_str("date and time"),
            ValueType::ListType(value) => write!(f, "list of {}", value),
            ValueType::DurationType => f.write_str("duration"),
            ValueType::ObjectType(value) => write!(f, "{}", value.borrow().to_type_string()),

            // Todo: remove it
            ValueType::RangeType => f.write_str("range"),

            // Todo: remove it
            //ValueType::AnyType => f.write_str("any"),
            ValueType::UndefinedType => f.write_str("undefined"),
        }
    }
}

// todo: support objects
impl TryFrom<&str> for ValueType {
    type Error = ParseErrorEnum;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.starts_with("list of ") {
            let value = value.replace("list of ", "");
            return Ok(ValueType::ListType(Box::new(ValueType::try_from(
                value.as_str(),
            )?)));
        }

        match value {
            "number" => Ok(ValueType::NumberType),
            "string" => Ok(ValueType::StringType),
            "boolean" => Ok(ValueType::BooleanType),
            "date" => Ok(ValueType::DateType),
            "time" => Ok(ValueType::TimeType),
            "date and time" => Ok(ValueType::DateTimeType),
            "duration" => Ok(ValueType::DurationType),
            _ => Err(ParseErrorEnum::UnknownType(value.to_string())),
        }
    }
}

// impl TryFrom<ValueType> for ValueEnum {
//     type Error = ();
//
//     fn try_from(value: ValueType) -> Result<Self, Self::Error> {
//         match value {
//             ValueType::NumberType => Ok(ValueEnum::NumberValue(NumberEnum::SV(SpecialValueEnum::Missing))),
//             ValueType::StringType => Ok(ValueEnum::StringValue(StringEnum::SV(SpecialValueEnum::Missing))),
//             ValueType::BooleanType => Ok(ValueEnum::BooleanValue(false)),
//             ValueType::DateType => Ok(DateValue(Sv(SpecialValueEnum::Missing))),
//             ValueType::TimeType => Ok(TimeValue(Sv(SpecialValueEnum::Missing))),
//             ValueType::DateTimeType => Ok(ValueEnum::DateTimeValue(Sv(SpecialValueEnum::Missing))),
//             ValueType::ListType(value) => Ok(ValueEnum::Array(vec![], *value)),
//             ValueType::DurationType => Ok(ValueEnum::DurationValue(Sv(SpecialValueEnum::Missing))),
//             ValueType::ObjectType(_) => Err(()),
//             ValueType::AnyType => Err(()),
//             ValueType::RangeType => Err(())
//         }
//     }
// }

//--------------------------------------------------------------------------------------------------
// SpecialValueEnum
//--------------------------------------------------------------------------------------------------

// 1 - Not Applicable -> value is marked as optional and is not necessary. Functions will ignore it if possible.
// 2 - Missing -> value is mandatory, but not present. Functions will not be applied for this value and result will be Missing
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SpecialValueEnum {
    Missing,
    NotApplicable,
    NotFound,
}

impl Display for SpecialValueEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SpecialValueEnum::Missing => f.write_str("Missing"),
            SpecialValueEnum::NotApplicable => f.write_str("NotApplicable"),
            SpecialValueEnum::NotFound => f.write_str("NotFound"),
        }
    }
}

pub type Float = f64;

pub type Integer = i64;

//--------------------------------------------------------------------------------------------------
// number
//--------------------------------------------------------------------------------------------------

pub mod number {
    use crate::typesystem::types::number::NumberEnum::{Fraction, Int, Real, SV};
    use crate::typesystem::types::ValueType::NumberType;
    use crate::typesystem::types::{Float, Integer, SpecialValueEnum, TypedValue, ValueType};
    use std::cmp::Ordering;
    use std::fmt;
    use std::fmt::{Debug, Display, Formatter};
    use std::ops::{Add, Div, Mul, Rem, Sub};

    #[allow(non_snake_case)]
    #[derive(Debug, PartialEq, Clone)]
    pub enum NumberEnum {
        Real(Float),
        Int(Integer),
        Fraction(Integer, Integer),
        SV(SpecialValueEnum),
    }

    impl NumberEnum {
        pub const ZERO: i64 = 0;

        pub fn negate(&self) -> NumberEnum {
            match self {
                Real(value) => Real(-*value),
                Int(value) => Int(-*value),
                Fraction(numerator, denominator) => Fraction(-*numerator, *denominator),
                other => other.clone(),
            }
        }

        pub fn has_remaining(&self) -> bool {
            match self {
                Real(value) => value.fract() != 0.0,
                Fraction(_, denominator) => denominator != &NumberEnum::ZERO,
                _ => false,
            }
        }
    }

    impl TypedValue for NumberEnum {
        fn get_type(&self) -> ValueType {
            NumberType
        }
    }

    impl Display for NumberEnum {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Real(value) => write!(f, "{}", value),
                Int(value) => write!(f, "{}", value),
                SV(value) => write!(f, "number.{}", value),
                Fraction(numerator, denominator) => write!(f, "{}/{}", numerator, denominator),
            }
        }
    }

    impl Add for NumberEnum {
        type Output = NumberEnum;

        fn add(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a + b),
                (Int(a), Int(b)) => NumberEnum::from(a + b),
                (Real(a), Int(b)) => NumberEnum::from(a + (b as Float)),
                (Int(a), Real(b)) => NumberEnum::from((a as Float) + b),
                (SV(SpecialValueEnum::Missing), any) => any,
                (any, SV(SpecialValueEnum::Missing)) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
                _ => SV(SpecialValueEnum::NotFound),
            }
        }
    }

    impl Sub for NumberEnum {
        type Output = NumberEnum;

        fn sub(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a - b),
                (Int(a), Int(b)) => NumberEnum::from(a - b),
                (Real(a), Int(b)) => NumberEnum::from(a - (b as Float)),
                (Int(a), Real(b)) => NumberEnum::from((a as Float) - b),
                (SV(SpecialValueEnum::Missing), any) => any.negate(),
                (any, SV(SpecialValueEnum::Missing)) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
                _ => SV(SpecialValueEnum::NotFound),
            }
        }
    }

    impl Mul for NumberEnum {
        type Output = NumberEnum;

        fn mul(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a * b),
                (Int(a), Int(b)) => NumberEnum::from(a * b),
                (Real(a), Int(b)) => NumberEnum::from(a * (b as Float)),
                (Int(a), Real(b)) => NumberEnum::from((a as Float) * b),
                (SV(SpecialValueEnum::Missing), any) => any,
                (any, SV(SpecialValueEnum::Missing)) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
                _ => SV(SpecialValueEnum::NotFound),
            }
        }
    }

    impl Div for NumberEnum {
        type Output = NumberEnum;

        fn div(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a / b),
                (Int(a), Int(b)) => NumberEnum::from(a as Float / b as Float),
                (Real(a), Int(b)) => NumberEnum::from(a / (b as Float)),
                (Int(a), Real(b)) => NumberEnum::from((a as Float) / b),
                (SV(SpecialValueEnum::Missing), _any) => SV(SpecialValueEnum::Missing),
                (any, SV(SpecialValueEnum::Missing)) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
                _ => SV(SpecialValueEnum::NotFound),
            }
        }
    }

    impl Rem for NumberEnum {
        type Output = NumberEnum;

        fn rem(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a % b),
                (Int(a), Int(b)) => NumberEnum::from(a % b),
                (Real(a), Int(b)) => NumberEnum::from(a % (b as Float)),
                (Int(a), Real(b)) => NumberEnum::from((a as Float) % b),
                (SV(SpecialValueEnum::Missing), _any) => SV(SpecialValueEnum::Missing),
                (any, SV(SpecialValueEnum::Missing)) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
                _ => SV(SpecialValueEnum::NotFound),
            }
        }
    }

    impl From<Float> for NumberEnum {
        fn from(value: Float) -> Self {
            if value == 0.0 {
                Int(0)
            } else {
                Real(value)
            }
        }
    }

    impl PartialOrd for NumberEnum {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            match (self, other) {
                (Real(a), Real(b)) => a.partial_cmp(b),
                (Int(a), Int(b)) => a.partial_cmp(b),
                (Real(a), Int(b)) => a.partial_cmp(&(*b as Float)),
                (Int(a), Real(b)) => (*a as Float).partial_cmp(b),
                (SV(SpecialValueEnum::Missing), _any) => Some(Ordering::Less),
                (_, SV(SpecialValueEnum::Missing)) => Some(Ordering::Greater),
                _ => None,
            }
        }
    }

    impl From<Integer> for NumberEnum {
        fn from(value: Integer) -> Self {
            Int(value)
        }
    }
}

//--------------------------------------------------------------------------------------------------
// string
//--------------------------------------------------------------------------------------------------

pub mod string {
    use crate::typesystem::types::ValueType::StringType;
    use crate::typesystem::types::{SpecialValueEnum, TypedValue, ValueType};
    use std::cmp::Ordering;
    use std::fmt::Display;

    #[allow(non_snake_case)]
    #[derive(Debug, PartialEq, Clone)]
    pub enum StringEnum {
        String(String),
        Char(char),
        SV(SpecialValueEnum),
    }

    impl TypedValue for StringEnum {
        fn get_type(&self) -> ValueType {
            StringType
        }
    }

    impl Display for StringEnum {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                StringEnum::String(s) => write!(f, "'{}'", s),
                StringEnum::SV(s) => write!(f, "{}", s),
                StringEnum::Char(s) => write!(f, "{}", s),
            }
        }
    }

    impl From<String> for StringEnum {
        fn from(value: String) -> Self {
            StringEnum::String(value)
        }
    }

    impl From<&str> for StringEnum {
        fn from(value: &str) -> Self {
            StringEnum::String(value.to_string())
        }
    }

    impl From<SpecialValueEnum> for StringEnum {
        fn from(value: SpecialValueEnum) -> Self {
            StringEnum::SV(value)
        }
    }

    impl PartialOrd for StringEnum {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            match (self, other) {
                (StringEnum::String(a), StringEnum::String(b)) => a.partial_cmp(b),
                (StringEnum::SV(SpecialValueEnum::Missing), _any) => Some(Ordering::Less),
                (_, StringEnum::SV(SpecialValueEnum::Missing)) => Some(Ordering::Greater),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::typesystem::types::number::NumberEnum;

    use crate::typesystem::types::SpecialValueEnum::{Missing, NotApplicable, NotFound};

    #[test]
    fn test_numbers() {
        // Add
        assert_eq!(
            NumberEnum::from(10) + NumberEnum::SV(Missing),
            NumberEnum::from(10)
        );
        assert_eq!(
            NumberEnum::SV(Missing) + NumberEnum::from(10),
            NumberEnum::from(10)
        );
        assert_eq!(
            NumberEnum::from(10) + NumberEnum::from(10),
            NumberEnum::from(20)
        );
        assert_eq!(
            NumberEnum::SV(NotFound) + NumberEnum::from(10),
            NumberEnum::SV(NotFound)
        );

        // Rem
        assert_eq!(
            NumberEnum::from(10) % NumberEnum::SV(Missing),
            NumberEnum::from(10)
        );
        assert_eq!(
            NumberEnum::SV(Missing) % NumberEnum::from(10),
            NumberEnum::SV(Missing)
        );
        assert_eq!(
            NumberEnum::from(10) % NumberEnum::from(10),
            NumberEnum::from(0)
        );
        assert_eq!(
            NumberEnum::SV(NotFound) % NumberEnum::from(10),
            NumberEnum::SV(NotFound)
        );

        assert!(NumberEnum::from(10) <= NumberEnum::from(10));

        // Missing

        assert!(NumberEnum::from(10) > NumberEnum::SV(Missing));
        assert!(matches!(
            NumberEnum::SV(Missing).partial_cmp(&NumberEnum::from(10)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(NumberEnum::SV(Missing) != NumberEnum::from(10));

        // NotApplicable
        assert!(matches!(
            NumberEnum::from(10).partial_cmp(&NumberEnum::SV(NotApplicable)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(matches!(
            NumberEnum::SV(NotApplicable).partial_cmp(&NumberEnum::from(10)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(NumberEnum::SV(NotApplicable) != NumberEnum::from(10));

        // NotApplicable
        assert!(matches!(
            NumberEnum::from(10).partial_cmp(&NumberEnum::SV(NotFound)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(matches!(
            NumberEnum::SV(NotFound).partial_cmp(&NumberEnum::from(10)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(NumberEnum::SV(NotFound) != NumberEnum::from(10));

        assert!(NumberEnum::SV(Missing) != NumberEnum::SV(NotFound));
        assert!(NumberEnum::SV(Missing) != NumberEnum::SV(NotApplicable));

        assert!(NumberEnum::SV(Missing) == NumberEnum::SV(Missing));
        assert!(NumberEnum::SV(NotFound) == NumberEnum::SV(NotFound));
        assert!(NumberEnum::SV(NotApplicable) == NumberEnum::SV(NotApplicable));
    }

    #[test]
    fn test_string() {}
}
