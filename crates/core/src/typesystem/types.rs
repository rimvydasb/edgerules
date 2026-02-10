use crate::ast::context::context_object::ContextObject;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub trait ToSchema {
    fn to_schema(&self) -> String;
}

pub trait TypedValue {
    fn get_type(&self) -> ValueType;
}

/// FEEL related documentation:
/// https://docs.camunda.io/docs/components/modeler/feel/language-guide/feel-data-types/
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum ValueType {
    NumberType,
    StringType,
    BooleanType,
    DateType,
    TimeType,
    DateTimeType,
    PeriodType,

    // Range is not a type, it is a filter method
    RangeType,

    // This is the type of the list, for example number[], Only homogenous lists are supported now
    ListType(Option<Box<ValueType>>),

    // Represents Years-months-duration and Days-time-duration
    DurationType,

    /// **Main considerations:**
    /// - Context there is the same as FEEL.
    /// - ContextObject is a Context type itself. No other meta layer should be introduced.
    /// - ExecutionContext type is a ContextObject
    /// - ContextObject instance is ExecutionContext
    /// - @Todo: it is a question if RefCell is necessary - context object must be immutable btw
    ObjectType(Rc<RefCell<ContextObject>>),

    // @Todo: this must not exists in runtime - if it exists, runtime must not start
    UndefinedType,
}

impl ValueType {
    pub fn ptr_eq(&self, other: &ValueType) -> bool {
        match (self, other) {
            (ValueType::ObjectType(a), ValueType::ObjectType(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }

    pub fn get_list_type(&self) -> Option<ValueType> {
        match self {
            ValueType::ListType(Some(list_type)) => Some(*list_type.clone()),
            _ => None,
        }
    }

    pub fn list_of(inner: ValueType) -> Self {
        ValueType::ListType(Some(Box::new(inner)))
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
            ValueType::DateTimeType => f.write_str("datetime"),
            ValueType::PeriodType => f.write_str("period"),
            ValueType::ListType(maybe_type) => match maybe_type {
                Some(boxed_type) => write!(f, "{}[]", boxed_type),
                None => f.write_str("[]"),
            },
            ValueType::DurationType => f.write_str("duration"),
            ValueType::ObjectType(value) => write!(f, "{}", value.borrow().to_schema()),

            // Todo: remove it
            ValueType::RangeType => f.write_str("range"),

            ValueType::UndefinedType => f.write_str("undefined"),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// SpecialValueEnum
//--------------------------------------------------------------------------------------------------

// 1 - Not Applicable -> value is marked as optional and is not necessary. Functions will ignore it if possible.
// 2 - Missing -> value is mandatory, but not present. Functions will not be applied for this value and result will be Missing
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, Eq, PartialEq)]
pub enum SpecialValueEnum {
    Missing(String),
    NotApplicable(String),
    NotFound(String),
}

impl SpecialValueEnum {
    pub const DEFAULT_ORIGIN: &'static str = "N/A";

    fn origin(field_name: Option<&str>) -> String {
        match field_name {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => Self::DEFAULT_ORIGIN.to_string(),
        }
    }

    pub fn missing(origin: impl Into<String>) -> Self {
        SpecialValueEnum::Missing(origin.into())
    }

    pub fn missing_for(field_name: Option<&str>) -> Self {
        SpecialValueEnum::Missing(Self::origin(field_name))
    }

    pub fn not_applicable(origin: impl Into<String>) -> Self {
        SpecialValueEnum::NotApplicable(origin.into())
    }

    pub fn not_applicable_for(field_name: Option<&str>) -> Self {
        SpecialValueEnum::NotApplicable(Self::origin(field_name))
    }

    pub fn not_found(origin: impl Into<String>) -> Self {
        SpecialValueEnum::NotFound(origin.into())
    }

    pub fn not_found_for(field_name: Option<&str>) -> Self {
        SpecialValueEnum::NotFound(Self::origin(field_name))
    }
}

impl Display for SpecialValueEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SpecialValueEnum::Missing(field) => write!(f, "Missing('{}')", field),
            SpecialValueEnum::NotApplicable(field) => write!(f, "NotApplicable('{}')", field),
            SpecialValueEnum::NotFound(field) => write!(f, "NotFound('{}')", field),
        }
    }
}

pub type Float = rust_decimal::Decimal;

pub type Integer = i64;

//--------------------------------------------------------------------------------------------------
// number
//--------------------------------------------------------------------------------------------------

pub mod number {
    use crate::typesystem::types::number::NumberEnum::{Int, Real, SV};
    use crate::typesystem::types::ValueType::NumberType;
    use crate::typesystem::types::{Float, Integer, SpecialValueEnum, TypedValue, ValueType};
    use std::cmp::Ordering;
    use std::fmt;
    use std::fmt::{Display, Formatter};
    use std::ops::{Add, Div, Mul, Rem, Sub};

    #[allow(non_snake_case)]
    #[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
    #[derive(PartialEq, Clone)]
    pub enum NumberEnum {
        Real(Float),
        Int(Integer),

        // @Todo: fraction mathematics is not implemented yet (Fraction(numerator, denominator))
        SV(SpecialValueEnum),
    }

    impl NumberEnum {
        pub const ZERO: i64 = 0;

        pub fn negate(&self) -> NumberEnum {
            match self {
                Real(value) => Real(-*value),
                Int(value) => Int(-*value),
                other => other.clone(),
            }
        }

        pub fn has_remaining(&self) -> bool {
            match self {
                Real(value) => value.fract() != Float::ZERO,
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
                Real(value) => write!(f, "{}", value.normalize()),
                Int(value) => write!(f, "{}", value),
                SV(value) => write!(f, "{}", value),
            }
        }
    }

    impl Add for NumberEnum {
        type Output = NumberEnum;

        fn add(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a + b),
                (Int(a), Int(b)) => NumberEnum::from(a + b),
                (Real(a), Int(b)) => NumberEnum::from(a + Float::from(b)),
                (Int(a), Real(b)) => NumberEnum::from(Float::from(a) + b),
                (SV(SpecialValueEnum::NotApplicable(_)), any) => any,
                (any, SV(SpecialValueEnum::NotApplicable(_))) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
            }
        }
    }

    impl Sub for NumberEnum {
        type Output = NumberEnum;

        fn sub(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a - b),
                (Int(a), Int(b)) => NumberEnum::from(a - b),
                (Real(a), Int(b)) => NumberEnum::from(a - Float::from(b)),
                (Int(a), Real(b)) => NumberEnum::from(Float::from(a) - b),
                (SV(SpecialValueEnum::NotApplicable(_)), any) => any.negate(),
                (any, SV(SpecialValueEnum::NotApplicable(_))) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
            }
        }
    }

    impl Mul for NumberEnum {
        type Output = NumberEnum;

        fn mul(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a * b),
                (Int(a), Int(b)) => NumberEnum::from(a * b),
                (Real(a), Int(b)) => NumberEnum::from(a * Float::from(b)),
                (Int(a), Real(b)) => NumberEnum::from(Float::from(a) * b),
                (SV(SpecialValueEnum::NotApplicable(_)), any) => any,
                (any, SV(SpecialValueEnum::NotApplicable(_))) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
            }
        }
    }

    impl Div for NumberEnum {
        type Output = NumberEnum;

        fn div(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a / b),
                (Int(a), Int(b)) => NumberEnum::from(Float::from(a) / Float::from(b)),
                (Real(a), Int(b)) => NumberEnum::from(a / Float::from(b)),
                (Int(a), Real(b)) => NumberEnum::from(Float::from(a) / b),
                (SV(SpecialValueEnum::NotApplicable(field)), _any) => SV(SpecialValueEnum::NotApplicable(field)),
                (any, SV(SpecialValueEnum::NotApplicable(_))) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
            }
        }
    }

    impl Rem for NumberEnum {
        type Output = NumberEnum;

        fn rem(self, rhs: Self) -> Self::Output {
            match (self, rhs) {
                (Real(a), Real(b)) => NumberEnum::from(a % b),
                (Int(a), Int(b)) => NumberEnum::from(a % b),
                (Real(a), Int(b)) => NumberEnum::from(a % Float::from(b)),
                (Int(a), Real(b)) => NumberEnum::from(Float::from(a) % b),
                (SV(value @ SpecialValueEnum::Missing(_)), _any) => SV(value),
                (any, SV(SpecialValueEnum::Missing(_))) => any,
                (SV(any), _) => SV(any),
                (_, SV(any)) => SV(any),
            }
        }
    }

    impl From<Float> for NumberEnum {
        fn from(value: Float) -> Self {
            if value == Float::ZERO {
                Int(0)
            } else {
                Real(value)
            }
        }
    }

    impl From<f64> for NumberEnum {
        fn from(value: f64) -> Self {
            if value == 0.0 {
                Int(0)
            } else {
                // Best effort conversion
                Real(Float::from_f64_retain(value).unwrap_or(Float::ZERO))
            }
        }
    }

    impl PartialOrd for NumberEnum {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            match (self, other) {
                (Real(a), Real(b)) => a.partial_cmp(b),
                (Int(a), Int(b)) => a.partial_cmp(b),
                (Real(a), Int(b)) => a.partial_cmp(&(Float::from(*b))),
                (Int(a), Real(b)) => Float::from(*a).partial_cmp(b),
                (SV(SpecialValueEnum::Missing(_)), _any) => Some(Ordering::Less),
                (_, SV(SpecialValueEnum::Missing(_))) => Some(Ordering::Greater),
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
    #[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
    #[derive(PartialEq, Clone)]
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
                (StringEnum::SV(SpecialValueEnum::Missing(_)), _any) => Some(Ordering::Less),
                (_, StringEnum::SV(SpecialValueEnum::Missing(_))) => Some(Ordering::Greater),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::typesystem::types::number::NumberEnum;
    use crate::typesystem::types::SpecialValueEnum;

    #[test]
    fn test_numbers() {
        let missing = SpecialValueEnum::missing_for(None);
        let not_applicable = SpecialValueEnum::not_applicable_for(None);
        let not_found = SpecialValueEnum::not_found_for(None);

        // Add
        assert_eq!(NumberEnum::from(10) + NumberEnum::SV(missing.clone()), NumberEnum::SV(missing.clone()));
        assert_eq!(NumberEnum::SV(missing.clone()) + NumberEnum::from(10), NumberEnum::SV(missing.clone()));
        assert_eq!(NumberEnum::from(10) + NumberEnum::from(10), NumberEnum::from(20));
        assert_eq!(NumberEnum::SV(not_found.clone()) + NumberEnum::from(10), NumberEnum::SV(not_found.clone()));

        // Rem
        assert_eq!(NumberEnum::from(10) % NumberEnum::SV(missing.clone()), NumberEnum::from(10));
        assert_eq!(NumberEnum::SV(missing.clone()) % NumberEnum::from(10), NumberEnum::SV(missing.clone()));
        assert_eq!(NumberEnum::from(10) % NumberEnum::from(10), NumberEnum::from(0));
        assert_eq!(NumberEnum::SV(not_found.clone()) % NumberEnum::from(10), NumberEnum::SV(not_found.clone()));

        assert!(NumberEnum::from(10) <= NumberEnum::from(10));

        // Missing

        assert!(NumberEnum::from(10) > NumberEnum::SV(missing.clone()));
        assert!(matches!(
            NumberEnum::SV(missing.clone()).partial_cmp(&NumberEnum::from(10)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(NumberEnum::SV(missing.clone()) != NumberEnum::from(10));

        // NotApplicable
        assert!(matches!(
            NumberEnum::from(10).partial_cmp(&NumberEnum::SV(not_applicable.clone())),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(matches!(
            NumberEnum::SV(not_applicable.clone()).partial_cmp(&NumberEnum::from(10)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(NumberEnum::SV(not_applicable.clone()) != NumberEnum::from(10));

        // NotApplicable
        assert!(matches!(
            NumberEnum::from(10).partial_cmp(&NumberEnum::SV(not_found.clone())),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(matches!(
            NumberEnum::SV(not_found.clone()).partial_cmp(&NumberEnum::from(10)),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal) | None
        ));
        assert!(NumberEnum::SV(not_found.clone()) != NumberEnum::from(10));

        assert!(NumberEnum::SV(missing.clone()) != NumberEnum::SV(not_found.clone()));
        assert!(NumberEnum::SV(missing) != NumberEnum::SV(not_applicable));

        assert!(
            NumberEnum::SV(SpecialValueEnum::missing_for(None)) == NumberEnum::SV(SpecialValueEnum::missing_for(None))
        );
        assert!(
            NumberEnum::SV(SpecialValueEnum::not_found_for(None))
                == NumberEnum::SV(SpecialValueEnum::not_found_for(None))
        );
        assert!(
            NumberEnum::SV(SpecialValueEnum::not_applicable_for(None))
                == NumberEnum::SV(SpecialValueEnum::not_applicable_for(None))
        );
    }

    #[test]
    fn test_string() {}
}
