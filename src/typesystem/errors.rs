use crate::ast::functions::function_types::EFunctionType;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use crate::ast::token::EToken;
use crate::ast::Link;
use crate::typesystem::errors::LinkingErrorEnum::{
    CyclicReference, DifferentTypesDetected, FieldNotFound, NotLinkedYet, OperationNotSupported,
    OtherLinkingError, TypesNotCompatible,
};
use crate::typesystem::errors::ParseErrorEnum::{
    FunctionWrongNumberOfArguments, InvalidType, MissingLiteral, UnexpectedLiteral,
    UnexpectedToken, UnknownError, UnknownParseError, UnknownType,
};
use crate::typesystem::errors::RuntimeErrorEnum::{
    EvalError, RuntimeCyclicReference, RuntimeFieldNotFound, TypeNotSupported,
};
use crate::typesystem::types::ValueType;

/// Error Stacking
/// Other libraries:
/// - https://crates.io/crates/anyhow
/// - https://github.com/dtolnay/thiserror
/// - https://crates.io/crates/handle-error
/// - influence taken from: https://github.com/dtolnay/anyhow/blob/8b4fc43429fd9a034649e0f919c646ec6626c4c7/src/context.rs#L58
pub trait ErrorStack<T: Display>: Sized {
    fn update_context(&mut self, content: String);

    #[allow(dead_code)]
    fn get_context(&self) -> &Vec<String>;

    fn get_error_type(&self) -> &T;

    #[allow(dead_code)]
    fn before_happened<O>(self, other: O) -> Self
    where
        O: ErrorStack<T>,
    {
        let mut new = self;
        for context in other.get_context().iter() {
            new.update_context(context.clone());
        }

        new.update_context(format!("{}", other.get_error_type()));

        new
    }

    fn with_context<C, F>(self, context: F) -> Self
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        let mut new = self;
        new.update_context(format!("{}", context()));

        new
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GeneralStackedError<T: Display> {
    pub error: T,
    pub context: Vec<String>,
}

impl<T: Display> ErrorStack<T> for GeneralStackedError<T> {
    fn update_context(&mut self, content: String) {
        self.context.push(content);
    }

    fn get_context(&self) -> &Vec<String> {
        &self.context
    }

    fn get_error_type(&self) -> &T {
        &self.error
    }
}

impl<T: Display> Display for GeneralStackedError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;

        let mut index = 0;
        if !self.context.is_empty() {
            write!(f, "\nContext:\n")?;

            for context in &self.context {
                index += 1;
                writeln!(f, "  {}. {}", index, context)?;
            }
        }

        Ok(())
    }
}

pub type RuntimeError = GeneralStackedError<RuntimeErrorEnum>;

impl RuntimeError {
    pub fn new(error: RuntimeErrorEnum) -> Self {
        RuntimeError {
            error,
            context: vec![],
        }
    }

    pub fn eval_error(message: String) -> Self {
        RuntimeError::new(EvalError(message.to_string()))
    }

    pub fn cyclic_reference(field: &str, object: &str) -> Self {
        RuntimeError::new(RuntimeCyclicReference(
            field.to_string(),
            object.to_string(),
        ))
    }

    pub fn field_not_found(field: &str, object: &str) -> Self {
        RuntimeError::new(RuntimeFieldNotFound(field.to_string(), object.to_string()))
    }

    pub fn type_not_supported(current: ValueType) -> Self {
        RuntimeError::new(TypeNotSupported(current))
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseErrorEnum {
    UnknownType(String),
    UnexpectedToken(Box<EToken>, Option<String>),
    UnexpectedLiteral(String, Option<String>),
    MissingLiteral(String),
    /// function_name, type, got
    FunctionWrongNumberOfArguments(String, EFunctionType, usize),

    // @todo: InvalidType(current, expected), also same as TypeNotSupported
    InvalidType(String),

    // @todo: remove this - all parse errors must be known
    UnknownParseError(String),

    // @Todo: remove this, same crap as UnknownParseError
    UnknownError(String),

    // @Todo: remove this as well, there should be EmptyError(expected) instead
    Empty,
}

impl Display for ParseErrorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Helper closure to prefix [parse] only if not already present
        let prefix_parse = |msg: &str| {
            if msg.starts_with("[parse]") {
                msg.to_string()
            } else {
                format!("[parse] {}", msg)
            }
        };

        match self {
            UnknownType(maybe_type) => write!(f, "{}", prefix_parse(maybe_type)),
            UnknownParseError(message) => write!(f, "{}", prefix_parse(message)),
            UnexpectedToken(token, expected) => {
                if let Some(expected) = expected {
                    write!(f, "{}", prefix_parse(&format!("Unexpected '{}', expected '{}'", token, expected)))
                } else {
                    write!(f, "{}", prefix_parse(&format!("Unexpected '{}'", token)))
                }
            }
            ParseErrorEnum::Empty => f.write_str("[parse] -Empty-"),
            UnknownError(message) => write!(f, "{}", prefix_parse(message)),
            InvalidType(error) => write!(f, "{}", prefix_parse(error)),
            UnexpectedLiteral(literal, expected) => {
                if let Some(expected) = expected {
                    write!(f, "{}", prefix_parse(&format!("Unexpected '{}', expected '{}'", literal, expected)))
                } else {
                    write!(f, "{}", prefix_parse(&format!("Unexpected '{}'", literal)))
                }
            }
            MissingLiteral(literal) => {
                write!(f, "{}", prefix_parse(&format!("Missing '{}'", literal)))
            }
            FunctionWrongNumberOfArguments(function_name, function_type, existing) => {
                if existing == &0 {
                    return write!(f, "{}", prefix_parse(&format!("Function '{}' got no arguments", function_name)));
                }
                match function_type {
                    EFunctionType::Custom(expected) => {
                        write!(
                            f,
                            "{}",
                            prefix_parse(&format!(
                                "Function '{}' expected {} arguments, but got {}",
                                function_name, expected, existing
                            ))
                        )
                    }
                    EFunctionType::Binary => {
                        write!(
                            f,
                            "{}",
                            prefix_parse(&format!(
                                "Binary function '{}' expected 2 arguments, but got {}",
                                function_name, existing
                            ))
                        )
                    }
                    EFunctionType::Multi => {
                        write!(
                            f,
                            "{}",
                            prefix_parse(&format!(
                                "Function '{}' expected 1 or more arguments, but got {}",
                                function_name, existing
                            ))
                        )
                    }
                    EFunctionType::Unary => {
                        write!(
                            f,
                            "{}",
                            prefix_parse(&format!(
                                "Function '{}' expected 1 argument, but got {}",
                                function_name, existing
                            ))
                        )
                    }
                }
            }
        }
    }
}

impl ParseErrorEnum {
    // @todo: complete normal error stacking
    pub fn before(self, before_error: ParseErrorEnum) -> ParseErrorEnum {
        if before_error == ParseErrorEnum::Empty {
            return self;
        }

        UnknownError(format!("{} → {}", before_error, self))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeErrorEnum {
    // message
    EvalError(String),

    // field, object
    RuntimeCyclicReference(String, String),

    // object, field
    RuntimeFieldNotFound(String, String),

    /// @Todo: update this: (existing type, expected type, method name)
    /// @Todo: this is absolutely unclear how it happens in runtime, because linking solves types.
    /// Add [unexpected] prefix to the error message for me to indicate that this is a linking/runtime mismatch
    /// It could be possible that in is not reproducible with tests, but find out if it happens in real world
    TypeNotSupported(ValueType),

    // @Todo: remove this, it's not a runtime error (remove RuntimeError::into_runtime as well)
    Unlinked,
}

impl Display for RuntimeErrorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EvalError(message) => write!(f, "[runtime] {}", message),
            TypeNotSupported(value_type) => {
                write!(f, "[runtime] Type '{}' is not supported", value_type)
            }
            RuntimeCyclicReference(object, field) => write!(
                f,
                "[runtime] Field {}.{} appears in a cyclic reference loop",
                object, field
            ),
            RuntimeFieldNotFound(object, field) => {
                write!(f, "[runtime] Field '{}' not found in {}", field, object)
            }
            RuntimeErrorEnum::Unlinked => f.write_str("[runtime] Unlinked"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkingErrorEnum {
    // subject, unexpected, expected
    TypesNotCompatible(Option<String>, ValueType, Option<Vec<ValueType>>),

    // subject, type 1, type 2
    DifferentTypesDetected(Option<String>, ValueType, ValueType),

    FunctionNotFound(String),

    // object, field
    FieldNotFound(String, String),

    // object, field
    CyclicReference(String, String),

    // operation, left type, right type
    // e.g., "+" operation not supported for types Integer and String
    OperationNotSupported(String, ValueType, ValueType),

    // @todo: remove this, it's not a linking error
    OtherLinkingError(String),

    NotLinkedYet,
}

pub type LinkingError = GeneralStackedError<LinkingErrorEnum>;

impl LinkingError {
    pub fn new(error: LinkingErrorEnum) -> Self {
        LinkingError {
            error,
            context: vec![],
        }
    }

    pub fn not_linked() -> Self {
        LinkingError::new(NotLinkedYet)
    }

    pub fn other_error(message: String) -> Self {
        LinkingError::new(OtherLinkingError(message))
    }

    pub fn field_not_found(object: &str, field: &str) -> Self {
        LinkingError::new(FieldNotFound(object.to_string(), field.to_string()))
    }

    pub fn different_types(subject: Option<String>, type1: ValueType, type2: ValueType) -> Self {
        LinkingError::new(DifferentTypesDetected(subject, type1, type2))
    }

    pub fn operation_not_supported(operation: &str, left: ValueType, right: ValueType) -> Self {
        LinkingError::new(OperationNotSupported(operation.to_string(), left, right))
    }

    pub fn types_not_compatible(
        subject: Option<String>,
        unexpected: ValueType,
        expected: Option<Vec<ValueType>>,
    ) -> Self {
        LinkingError::new(TypesNotCompatible(subject, unexpected, expected))
    }

    pub fn expect_type(
        subject: Option<String>,
        expression_type: ValueType,
        expected: &[ValueType],
    ) -> Link<ValueType> {
        let actual = expression_type;
        if expected.contains(&actual) {
            return Ok(actual);
        }
        LinkingError::types_not_compatible(subject, actual, Some(expected.to_vec())).into()
    }

    pub fn expect_array_type(
        subject: Option<String>,
        expression_type: ValueType,
    ) -> Link<ValueType> {
        match expression_type {
            ValueType::ListType(Some(list_type)) => Ok(*list_type),
            ValueType::ListType(None) => Ok(ValueType::UndefinedType),
            other => LinkingError::types_not_compatible(
                subject,
                other,
                Some(vec![ValueType::ListType(None)]),
            )
            .into(),
        }
    }

    // pub fn expect_object_type(subject: &str, expression_type: ValueType) -> Result<(), LinkingError> {
    //     if expression_type.is_object_type() {
    //         return Ok(());
    //     }
    //     LinkingError::types_not_compatible(Some(subject.to_string()), expression_type, None).into()
    // }

    pub fn expect_single_type(
        subject: &str,
        expression_type: ValueType,
        expected: &ValueType,
    ) -> Link<ValueType> {
        if &expression_type == expected {
            return Ok(expression_type);
        }
        LinkingError::types_not_compatible(
            Some(subject.to_string()),
            expression_type,
            Some(vec![expected.clone()]),
        )
        .into()
    }

    pub fn expect_same_types(subject: &str, left: ValueType, right: ValueType) -> Link<ValueType> {
        if left == right {
            return Ok(left);
        }
        LinkingError::different_types(Some(subject.to_string()), left, right).into()
    }

    pub fn expect_same_item_types(subject: &str, items: &[ValueType]) -> Link<ValueType> {
        if items.is_empty() {
            return Ok(ValueType::UndefinedType);
        }
        let first = items[0].clone();
        for item in items {
            if item != &first {
                return LinkingError::different_types(
                    Some(subject.to_string()),
                    first,
                    item.clone(),
                )
                .into();
            }
        }
        Ok(first)
    }
}

impl<T, O> From<GeneralStackedError<O>> for Result<T, GeneralStackedError<O>>
where
    T: Sized,
    O: Sized + Display,
{
    fn from(val: GeneralStackedError<O>) -> Self {
        Err(val)
    }
}

impl From<LinkingError> for RuntimeError {
    fn from(value: LinkingError) -> Self {
        Self::into_runtime(value)
    }
}

impl From<ParseErrorEnum> for RuntimeError {
    fn from(err: ParseErrorEnum) -> Self {
        RuntimeError::eval_error(err.to_string())
    }
}

impl RuntimeError {
    fn into_runtime(error: LinkingError) -> Self {
        let mut runtime_error = match &error.error {
            FieldNotFound(object, field) => RuntimeError::field_not_found(object, field),
            CyclicReference(object, field) => RuntimeError::cyclic_reference(object, field),
            NotLinkedYet => RuntimeError::new(RuntimeErrorEnum::Unlinked),
            _ => RuntimeError::eval_error(error.error.to_string()),
        };

        runtime_error.context = error.context.clone();

        runtime_error
    }
}

impl Display for LinkingErrorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypesNotCompatible(subject, unexpected, expected) => {
                let clean_subject = subject.clone().unwrap_or_else(|| "Unexpected".to_string());
                if let Some(expected) = expected {
                    // joins expected types into a string separated by " or "
                    let expected_str = expected
                        .iter()
                        .map(|value_type| value_type.to_string())
                        .collect::<Vec<String>>()
                        .join(" or ");
                    write!(
                        f,
                        "[link] {} type '{}', expected '{}'",
                        clean_subject, unexpected, expected_str
                    )
                } else {
                    write!(f, "{} type '{}'", clean_subject, unexpected)
                }
            }
            DifferentTypesDetected(subject, left, right) => match subject {
                Some(subject) => {
                    write!(
                        f,
                        "[link] {} types `{}` and `{}` must match",
                        subject, left, right
                    )
                }
                None => write!(
                    f,
                    "[link] Operation is not supported for different types: {} and {}",
                    left, right
                ),
            },
            LinkingErrorEnum::FunctionNotFound(name) => {
                write!(f, "[link] Function '{}' not found", name)
            }
            FieldNotFound(object, field) => {
                write!(f, "[link] Field '{}' not found in {}", field, object)
            }
            CyclicReference(object, field) => {
                write!(
                    f,
                    "[link] Field {}.{} appears in a cyclic reference loop",
                    object, field
                )
            }
            OtherLinkingError(error) => write!(f, "[link] {}", error),
            NotLinkedYet => f.write_str("[link] Not linked yet"),
            OperationNotSupported(op, left, right) => {
                write!(
                    f,
                    "[link] Operation '{}' not supported for types '{}' and '{}'",
                    op, left, right
                )
            }
        }
    }
}

// @Todo: implement global usage
// @Todo: implement normal error stacking
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct LinkingErrors {
    pub errors: Vec<LinkingError>,
}
