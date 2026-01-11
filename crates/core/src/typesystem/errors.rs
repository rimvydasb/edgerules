use crate::ast::context::duplicate_name_error::DuplicateNameError;
use crate::ast::functions::function_types::EFunctionType;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::ast::token::EToken;
use crate::ast::Link;
use crate::typesystem::errors::LinkingErrorEnum::{
    CyclicReference, DifferentTypesDetected, FieldNotFound, NotLinkedYet, OperationNotSupported,
    OtherLinkingError, TypesNotCompatible,
};
use crate::typesystem::errors::ParseErrorEnum::{
    FunctionWrongNumberOfArguments, MissingLiteral, OtherError, Stacked, UnexpectedEnd,
    UnexpectedLiteral, UnexpectedToken, WrongFormat,
};
use crate::typesystem::errors::RuntimeErrorEnum::{
    DivisionByZero, EvalError, RuntimeCyclicReference, RuntimeFieldNotFound, TypeNotSupported,
    UnexpectedError, ValueParsingError,
};
use crate::typesystem::types::ValueType;

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq, Clone)]
pub enum ErrorStage {
    Linking,
    Runtime,
}

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

    fn with_context<C, F>(mut self, context: F) -> Self
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.update_context(format!("{}", context()));
        self
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq, Clone)]
struct ErrorData<T: Display> {
    pub error: T,
    pub context: Vec<String>,
    pub location: Vec<String>,
    pub expression: Option<String>,
    pub stage: Option<ErrorStage>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq, Clone)]
pub struct GeneralStackedError<T: Display> {
    inner: Box<ErrorData<T>>,
}

impl<T: Display> GeneralStackedError<T> {
    pub fn kind(&self) -> &T {
        &self.inner.error
    }

    pub fn context(&self) -> &[String] {
        &self.inner.context
    }

    pub fn location(&self) -> &[String] {
        &self.inner.location
    }

    pub fn expression(&self) -> Option<&String> {
        self.inner.expression.as_ref()
    }

    pub fn stage(&self) -> Option<&ErrorStage> {
        self.inner.stage.as_ref()
    }

    pub fn location_mut(&mut self) -> &mut Vec<String> {
        &mut self.inner.location
    }

    pub fn set_expression(&mut self, expression: String) {
        self.inner.expression = Some(expression);
    }

    pub fn set_stage(&mut self, stage: ErrorStage) {
        self.inner.stage = Some(stage);
    }

    pub fn has_expression(&self) -> bool {
        self.inner.expression.is_some()
    }

    pub fn has_stage(&self) -> bool {
        self.inner.stage.is_some()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Display + fmt::Debug> std::error::Error for GeneralStackedError<T> {}

impl<T: Display> ErrorStack<T> for GeneralStackedError<T> {
    fn update_context(&mut self, content: String) {
        self.inner.context.push(content);
    }

    fn get_context(&self) -> &Vec<String> {
        &self.inner.context
    }

    fn get_error_type(&self) -> &T {
        &self.inner.error
    }
}

impl<T: Display> Display for GeneralStackedError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.error)?;
        let mut index = 0;
        if !self.inner.context.is_empty() {
            write!(f, "\nContext:\n")?;

            for context in &self.inner.context {
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
        GeneralStackedError {
            inner: Box::new(ErrorData {
                error,
                context: vec![],
                location: vec![],
                expression: None,
                stage: Some(ErrorStage::Runtime),
            }),
        }
    }

    pub fn unexpected(message: impl Into<String>) -> Self {
        RuntimeError::new(UnexpectedError(message.into()))
    }

    pub fn eval_error(message: impl Into<String>) -> Self {
        RuntimeError::new(EvalError(message.into()))
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

    pub fn parsing(from: ValueType, to: ValueType) -> Self {
        RuntimeError::new(ValueParsingError(from, to, 0))
    }

    pub fn parsing_code(from: ValueType, to: ValueType, code: u8) -> Self {
        RuntimeError::new(ValueParsingError(from, to, code))
    }

    pub fn parsing_from_string(to: ValueType, code: u8) -> Self {
        RuntimeError::new(ValueParsingError(ValueType::StringType, to, code))
    }

    pub fn division_by_zero() -> Self {
        RuntimeError::new(DivisionByZero)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq)]
pub enum ParseErrorEnum {
    UnexpectedToken(Box<EToken>, Option<String>),
    UnexpectedLiteral(String, Option<String>),
    MissingLiteral(String),
    /// function_name, type, got
    FunctionWrongNumberOfArguments(String, EFunctionType, usize),

    /// Expected format description
    WrongFormat(String),

    /// Other parsing errors that are not strictly format-related
    OtherError(String),

    /// Ordered stack of errors to preserve context
    Stacked(Vec<ParseErrorEnum>),

    UnexpectedEnd,
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
            WrongFormat(message) => write!(f, "{}", prefix_parse(message)),
            UnexpectedToken(token, expected) => {
                if let Some(expected) = expected {
                    write!(
                        f,
                        "{}",
                        prefix_parse(&format!("Unexpected '{}', expected '{}'", token, expected))
                    )
                } else {
                    write!(f, "{}", prefix_parse(&format!("Unexpected '{}'", token)))
                }
            }
            UnexpectedEnd => f.write_str("[parse] Unexpected end"),
            OtherError(message) => write!(f, "{}", prefix_parse(message)),
            Stacked(errors) => {
                let formatted = errors
                    .iter()
                    .map(|err| err.to_string())
                    .collect::<Vec<String>>()
                    .join(" → ");
                f.write_str(&formatted)
            }
            UnexpectedLiteral(literal, expected) => {
                if let Some(expected) = expected {
                    write!(
                        f,
                        "{}",
                        prefix_parse(&format!(
                            "Unexpected '{}', expected '{}'",
                            literal, expected
                        ))
                    )
                } else {
                    write!(f, "{}", prefix_parse(&format!("Unexpected '{}'", literal)))
                }
            }
            MissingLiteral(literal) => {
                write!(f, "{}", prefix_parse(&format!("Missing '{}'", literal)))
            }
            FunctionWrongNumberOfArguments(function_name, function_type, existing) => {
                if existing == &0 {
                    return write!(
                        f,
                        "{}",
                        prefix_parse(&format!("Function '{}' got no arguments", function_name))
                    );
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

impl From<DuplicateNameError> for ParseErrorEnum {
    fn from(error: DuplicateNameError) -> Self {
        ParseErrorEnum::OtherError(error.to_string())
    }
}

impl ParseErrorEnum {
    pub fn before(self, before_error: ParseErrorEnum) -> ParseErrorEnum {
        if before_error == UnexpectedEnd {
            return self;
        }

        let mut previous_errors = match before_error {
            Stacked(errors) => errors,
            other => vec![other],
        };

        match self {
            Stacked(mut errors) => {
                previous_errors.append(&mut errors);
                Stacked(previous_errors)
            }
            other => {
                previous_errors.push(other);
                Stacked(previous_errors)
            }
        }
    }
}

/// ValueParsingError error codes:
/// 0 - Generic parsing error
/// 101 - Date adjustment overflowed year range
/// 102 - Invalid month produced during calendarDiff
/// 103 - Invalid date produced during calendarDiff
/// 104 - Period components must be non-negative before applying the sign
/// 105 - Period months overflow the supported range
/// 106 - Period components overflow the supported range
/// 107 - Period months and days must carry the same sign
/// 110 - Duration days overflow the supported range
/// 111 - Duration hours overflow the supported range
/// 112 - Duration minutes overflow the supported range
/// 113 - Duration seconds overflow the supported range
/// 114 - Duration overflow while calculating seconds
/// 115 - Duration components must be non-negative before applying the sign
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq, Clone)]
pub enum RuntimeErrorEnum {
    // @Todo: ideally all eval errors must be eliminated and replaced with specific errors
    // message
    EvalError(String),

    // value parsing error occurs when parsing typed values from strings, e.g. `eval_duration`, to duration or other type
    // @Todo: this error should occur only when string is passed to a typed value parser, TBC, TBA
    // @Todo: need to develop linking aware constant string parsing, e.g. @P2D and report errors during linking, TBC, TBA
    // fromType, toType, errorCode
    ValueParsingError(ValueType, ValueType, u8),

    // e.g., division by zero
    DivisionByZero,

    // field, object
    RuntimeCyclicReference(String, String),

    // object, field
    RuntimeFieldNotFound(String, String),

    /// @Todo: update this: (existing type, expected type, method name)
    /// @Todo: this is absolutely unclear how it happens in runtime, because linking solves types.
    /// Add [unexpected] prefix to the error message for me to indicate that this is a linking/runtime mismatch
    /// It could be possible that in is not reproducible with tests, but find out if it happens in real world
    TypeNotSupported(ValueType),

    /// This error never appears in normal runtime, but used as a guard for maybe unlinked references.
    /// This error also means development mistake and will not be covered by tests.
    UnexpectedError(String),
}

impl Display for RuntimeErrorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EvalError(message) => write!(f, "[runtime] {}", message),
            ValueParsingError(from, to, code) => {
                if *code > 0 {
                    write!(
                        f,
                        "[runtime] Failed to parse '{}' from '{}'. (Error code: {})",
                        to, from, code
                    )
                } else {
                    write!(f, "[runtime] Failed to parse '{}' from '{}'", to, from)
                }
            }
            DivisionByZero => write!(f, "[runtime] Division by zero"),
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
            UnexpectedError(message) => {
                write!(f, "[runtime] Unexpected error: {}", message)
            }
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum LinkingErrorEnum {
    // subject, unexpected, expected
    TypesNotCompatible(Option<String>, ValueType, Option<Vec<ValueType>>),

    // subject, type 1, type 2
    DifferentTypesDetected(Option<String>, ValueType, ValueType),

    FunctionNotFound {
        name: String,
        known_metaphors: Vec<String>,
    },

    // object, field
    FieldNotFound(String, String),

    // object, field
    CyclicReference(String, String),

    // operation, left type, right type
    // e.g., "+" operation not supported for types Integer and String
    OperationNotSupported(String, ValueType, ValueType),

    // @Todo: this one must be split to multiple other enums
    OtherLinkingError(String),

    NotLinkedYet,
}

pub type LinkingError = GeneralStackedError<LinkingErrorEnum>;

impl LinkingError {
    pub fn new(error: LinkingErrorEnum) -> Self {
        GeneralStackedError {
            inner: Box::new(ErrorData {
                error,
                context: vec![],
                location: vec![],
                expression: None,
                stage: Some(ErrorStage::Linking),
            }),
        }
    }

    pub fn not_linked() -> Self {
        LinkingError::new(NotLinkedYet)
    }

    pub fn other_error(message: impl Into<String>) -> Self {
        LinkingError::new(OtherLinkingError(message.into()))
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

impl From<DuplicateNameError> for RuntimeError {
    fn from(err: DuplicateNameError) -> Self {
        RuntimeError::eval_error(err.to_string())
    }
}

impl RuntimeError {
    fn into_runtime(error: LinkingError) -> Self {
        let mut runtime_error = match &error.inner.error {
            FieldNotFound(object, field) => RuntimeError::field_not_found(object, field),
            CyclicReference(object, field) => RuntimeError::cyclic_reference(object, field),
            NotLinkedYet => RuntimeError::unexpected(format!("{}", error)),
            _ => RuntimeError::eval_error(error.inner.error.to_string()),
        };

        runtime_error.inner.context = error.inner.context.clone();
        runtime_error.inner.location = error.inner.location.clone();
        runtime_error.inner.expression = error.inner.expression.clone();
        runtime_error.inner.stage = Some(ErrorStage::Runtime);

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
            LinkingErrorEnum::FunctionNotFound {
                name,
                known_metaphors,
            } => {
                write!(f, "[link] Function '{}(...)' not found", name)?;
                if known_metaphors.is_empty() {
                    write!(f, ". No metaphors in scope.")
                } else {
                    let formatted_candidates = known_metaphors
                        .iter()
                        .map(|metaphor_name| format!("{}(...)", metaphor_name))
                        .collect::<Vec<String>>()
                        .join(", ");
                    write!(f, ". Known metaphors in scope: {}.", formatted_candidates)
                }
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
