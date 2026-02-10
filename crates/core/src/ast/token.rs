use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::FormalParameter;
use crate::ast::expression::{EvaluatableExpression, StaticLink};
use crate::ast::metaphors::functions::{FunctionDefinition, InlineFunctionDefinition};
use crate::ast::operators::comparators::ComparatorEnum;
use crate::ast::operators::logical_operators::LogicalOperatorEnum;
use crate::ast::operators::math_operators::{MathOperatorEnum, Operator};
use crate::ast::selections::{ExpressionFilter, FieldSelection};
use crate::ast::sequence::CollectionExpression;
use crate::ast::token::DefinitionEnum::UserFunction;
use crate::ast::token::EToken::*;
use crate::ast::token::EUnparsedToken::*;
use crate::ast::token::ExpressionEnum::*;
use crate::ast::token::ValueEnum::*;
use crate::ast::utils::{array_to_code_sep, trim};
use crate::ast::variable::VariableLink;
use crate::ast::Link;
use crate::link::linker;
use crate::tokenizer::C_ASSIGN;
use crate::typesystem::errors::ParseErrorEnum::UnexpectedToken;
use crate::typesystem::errors::{ErrorStack, LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::string::StringEnum;
use crate::typesystem::types::ValueType::{ObjectType, RangeType};
use crate::typesystem::types::{Float, Integer, ToSchema, TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
//--------------------------------------------------------------------------------------------------

/// 1 - do as a last priority
/// 999 - do first
pub enum EPriorities {
    ContextPriority = 1,
    Assign = 2,
    RangePriority = 3,
    ReservedWords = 5,

    // a = b or a = c
    // Precedence: Not binds tighter than And/Or/Xor, but looser than Comparators
    // so expressions like `not it > 10` parse as `not (it > 10)`.
    GateNot = 14,
    GateAnd = 13,
    GatesXor = 11,
    GatesOr = 9,
    ComparatorPriority = 15,
    CastPriority = 16,
    // Todo: is it really OK?
    FilterArray = 17,
    FieldSelectionPriority = 27,
    Plus = 21,
    Minus = 22,
    DivideMultiply = 23,
    PowerPriority = 25,
    UnaryPriority = 26,
    FunctionCallPriority = 28,

    //CommaPriority = 98,
    ErrorPriority = 99,
}

// @Todo implementation of Display
/// constrains can be applied in:
/// Constraint(EComparator, Box<EExpression>),
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub enum EUnparsedToken {
    CommaToken,
    BracketOpenToken,
    AssignToken,
    RangeToken,
    ObjectToken,
    DotToken,
    LiteralToken(Cow<'static, str>),
    FunctionNameToken(VariableLink),
    FunctionDefinitionLiteralToken(String, Vec<FormalParameter>),
    TypeReferenceLiteralToken(ComplexTypeRef),
    MathOperatorToken(MathOperatorEnum),
    LogicalOperatorToken(LogicalOperatorEnum),
    ComparatorToken(ComparatorEnum),
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum ComplexTypeRef {
    BuiltinType(ValueType, Option<ValueEnum>),
    Alias(String, Option<ValueEnum>),
    List(Box<ComplexTypeRef>, Option<ValueEnum>),
}

impl ComplexTypeRef {
    pub fn undefined() -> Self {
        ComplexTypeRef::BuiltinType(ValueType::UndefinedType, None)
    }

    pub fn is_undefined(&self) -> bool {
        matches!(self, ComplexTypeRef::BuiltinType(ValueType::UndefinedType, _))
    }

    pub fn from_value_type(value_type: ValueType) -> Self {
        ComplexTypeRef::BuiltinType(value_type, None)
    }
}

// @Todo: investigate if this code is even necessary:
impl Display for ComplexTypeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ComplexTypeRef::BuiltinType(vt, Some(default)) => write!(f, "{}, {}", vt, default),
            ComplexTypeRef::BuiltinType(vt, None) => write!(f, "{}", vt),
            ComplexTypeRef::Alias(name, Some(default)) => write!(f, "{}, {}", name, default),
            ComplexTypeRef::Alias(name, None) => write!(f, "{}", name),
            ComplexTypeRef::List(inner, Some(default)) => write!(f, "{}[], {}", inner, default),
            ComplexTypeRef::List(inner, None) => write!(f, "{}[]", inner),
        }
    }
}

pub fn into_valid<T>(values: Vec<Result<T, RuntimeError>>) -> Result<Vec<T>, RuntimeError> {
    let mut clean = Vec::with_capacity(values.len());
    for value in values {
        clean.push(value?);
    }
    Ok(clean)
}

impl TryInto<ExpressionEnum> for EToken {
    type Error = ParseErrorEnum;

    fn try_into(self) -> Result<ExpressionEnum, Self::Error> {
        match self {
            Expression(expression) => Ok(expression),
            _ => Err(UnexpectedToken(Box::new(self), None)),
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum EToken {
    ParseError(ParseErrorEnum),
    Unparsed(EUnparsedToken),
    Expression(ExpressionEnum),
    Definition(DefinitionEnum),
}

impl PartialEq for EToken {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ParseError(a), ParseError(b)) => a == b,
            (Unparsed(a), Unparsed(b)) => a == b,
            (Expression(a), Expression(b)) => a == b,
            (Definition(_), Definition(_)) => false,
            _ => false,
        }
    }
}

impl EToken {
    pub fn into_string_or_literal(self) -> Result<String, ParseErrorEnum> {
        match self {
            Unparsed(LiteralToken(text)) => Ok(text.into_owned()),
            Expression(Value(StringValue(StringEnum::String(value)))) => Ok(value),
            ParseError(error) => Err(error),
            _ => Err(UnexpectedToken(Box::new(self), None)),
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum DefinitionEnum {
    UserFunction(FunctionDefinition),
    InlineUserFunction(InlineFunctionDefinition),
    UserType(UserTypeDefinition),
}

impl Display for DefinitionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserFunction(m) => write!(f, "{}", m),
            DefinitionEnum::InlineUserFunction(m) => write!(f, "{}", m),
            DefinitionEnum::UserType(t) => write!(f, "type {}: {}", t.name, t.body),
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum ExpressionEnum {
    Value(ValueEnum),

    // most common case should be Literal
    Variable(VariableLink),

    /// Context variable does not have a name, but could be aliased with "it" and "..."
    /// Currently used in
    /// 1. filters, for example applicantsList[... > 18] or ages[> 18] or ages[it > 18]
    /// 2. decision table cells, for example: > 18, <= 100, 15 <= ... <= 100, (... + 5) > eligibleAge
    ContextVariable,

    // MathOperator, LogicalOperator, Comparator
    Operator(Box<dyn Operator>),

    RangeExpression(Box<ExpressionEnum>, Box<ExpressionEnum>),

    // invocation of sum, min, max (or user function) on the right side, etc...
    FunctionCall(Box<dyn EvaluatableExpression>),

    Filter(Box<ExpressionFilter>),

    Selection(Box<FieldSelection>),

    Collection(CollectionExpression),

    /// usually object is passed as a value by a reference, so it may be in few places at the same time. However, RefCell may not be necessary.
    /// @Todo: try to remove RefCell
    StaticObject(Rc<RefCell<ContextObject>>),
    /// name and left side
    /// @Todo: move to unparsed
    ObjectField(String, Box<ExpressionEnum>),
    /// Typed placeholder with known type, value provided externally at eval time
    TypePlaceholder(ComplexTypeRef),
}

impl StaticLink for ExpressionEnum {
    fn link(&mut self, ctx: Rc<RefCell<ContextObject>>) -> Link<ValueType> {
        let trace_context = Rc::clone(&ctx);

        let linking_result = match self {
            Variable(variable) => variable.link(ctx),
            FunctionCall(function) => function.link(ctx),
            Operator(operator) => operator.link(ctx),
            Filter(filter) => filter.link(ctx),
            Selection(selection) => selection.link(ctx),
            Collection(collection) => collection.link(ctx),
            ObjectField(_name, field) => field.link(ctx),
            Value(value) => Ok(value.get_type()),
            ContextVariable => match &ctx.borrow().context_type {
                None => LinkingError::not_linked().into(),
                Some(context_type) => Ok(context_type.clone()),
            },
            RangeExpression(_from, _to) => {
                let from = _from.link(ctx.clone());
                let to = _to.link(ctx);
                LinkingError::expect_same_types("Range types", from?, to?)?;
                Ok(RangeType)
            }
            StaticObject(object) => {
                linker::link_parts(Rc::clone(object))?;
                Ok(ObjectType(Rc::clone(object)))
            }
            TypePlaceholder(tref) => ctx.borrow().resolve_type_ref(tref),
        };

        if let Err(error) = linking_result {
            let field_name = trace_context.borrow().node.node_type.to_string();
            return error.with_context(|| format!("Error in:`{}`\nTrace:`{}`", field_name, self)).into();
        }

        linking_result
    }
}

impl From<ExpressionEnum> for Rc<RefCell<ExpressionEnum>> {
    fn from(val: ExpressionEnum) -> Self {
        Rc::new(RefCell::new(val))
    }
}

impl From<ValueEnum> for ExpressionEnum {
    fn from(value: ValueEnum) -> Self {
        Value(value)
    }
}

impl From<&str> for ExpressionEnum {
    fn from(value: &str) -> Self {
        Value(StringValue(StringEnum::String(value.to_string())))
    }
}

impl From<String> for ExpressionEnum {
    fn from(value: String) -> Self {
        Value(StringValue(StringEnum::String(value)))
    }
}

impl From<ContextObject> for ExpressionEnum {
    fn from(value: ContextObject) -> Self {
        StaticObject(Rc::new(RefCell::new(value)))
    }
}

impl From<Integer> for ExpressionEnum {
    fn from(value: Integer) -> Self {
        NumberValue(NumberEnum::from(value)).into()
    }
}

impl From<Float> for ExpressionEnum {
    fn from(value: Float) -> Self {
        NumberValue(NumberEnum::from(value)).into()
    }
}

impl From<f64> for ExpressionEnum {
    fn from(value: f64) -> Self {
        NumberValue(NumberEnum::from(value)).into()
    }
}

impl From<bool> for ExpressionEnum {
    fn from(value: bool) -> Self {
        BooleanValue(value).into()
    }
}

//--------------------------------------------------------------------------------------------------
// To Code
//--------------------------------------------------------------------------------------------------

impl Display for EUnparsedToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FunctionDefinitionLiteralToken(text, args) => {
                write!(f, "{}({})", text, array_to_code_sep(args.iter(), ", "))
            }
            TypeReferenceLiteralToken(r) => write!(f, "<{}>", r),
            LiteralToken(value) => write!(f, "{}", value),
            AssignToken => write!(f, ":"),
            RangeToken => write!(f, ".."),
            ObjectToken => write!(f, "OBJECT"),
            DotToken => write!(f, "."),
            FunctionNameToken(value) => write!(f, "{}", value),
            CommaToken => write!(f, ","),
            BracketOpenToken => write!(f, "["),
            MathOperatorToken(value) => write!(f, "{}", value),
            LogicalOperatorToken(value) => write!(f, "{}", value),
            ComparatorToken(value) => write!(f, "{}", value),
        }
    }
}

impl Display for ExpressionEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Variable(value) => Display::fmt(&value, f),
            Operator(op) => Display::fmt(op, f),
            FunctionCall(function) => Display::fmt(function, f),
            Selection(selection) => Display::fmt(selection, f),
            ObjectField(field_name, right_side) => {
                write!(f, "{} {} {}", field_name, C_ASSIGN, trim(format!("{}", right_side).as_str(), '(', ')'))
            }
            Collection(values) => Display::fmt(values, f),
            Value(value) => Display::fmt(value, f),
            RangeExpression(left, right) => write!(f, "{}..{}", left, right),
            ContextVariable => Display::fmt("...", f),
            Filter(value) => Display::fmt(value, f),
            StaticObject(obj) => write!(f, "{}", obj.borrow()),
            TypePlaceholder(t) => write!(f, "<{}>", t),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum UserTypeBody {
    TypeRef(ComplexTypeRef),
    TypeObject(Rc<RefCell<ContextObject>>),
}

impl Display for UserTypeBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserTypeBody::TypeRef(r) => write!(f, "<{}>", r),
            UserTypeBody::TypeObject(obj) => write!(f, "{}", obj.borrow()),
        }
    }
}

impl ToSchema for UserTypeBody {
    fn to_schema(&self) -> String {
        match self {
            UserTypeBody::TypeRef(reference) => reference.to_string(),
            UserTypeBody::TypeObject(obj) => obj.borrow().to_schema(),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub struct UserTypeDefinition {
    pub name: String,
    pub body: UserTypeBody,
}

impl Display for EToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError(value) => Display::fmt(value, f),
            Unparsed(value) => Display::fmt(value, f),
            Expression(value) => Display::fmt(value, f),
            Definition(value) => Display::fmt(value, f),
        }
    }
}
