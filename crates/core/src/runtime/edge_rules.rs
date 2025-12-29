use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::context::duplicate_name_error::DuplicateNameError;
use crate::ast::expression::StaticLink;
use crate::ast::token::EToken;
use crate::ast::token::EToken::{Definition, Expression};
use crate::ast::token::ExpressionEnum::ObjectField;
use crate::ast::user_function_call::UserFunctionCall;
use crate::ast::utils::array_to_code_sep;
use crate::link::node_data::{ContentHolder, Node};
use crate::runtime::execution_context::ExecutionContext;
use crate::tokenizer::parser::tokenize;
use crate::typesystem::errors::ParseErrorEnum::{
    OtherError, UnexpectedEnd, UnexpectedToken, WrongFormat,
};
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::types::{TypedValue, ValueType};
use crate::typesystem::values::ValueEnum;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub use crate::ast::context::context_object::{ExpressionEntry, MethodEntry};
pub use crate::ast::context::context_object_builder::ContextObjectBuilder;
pub use crate::ast::metaphors::functions::FunctionDefinition;
pub use crate::ast::token::{DefinitionEnum, ExpressionEnum, UserTypeBody};
pub use crate::link::linker::link_parts;
//--------------------------------------------------------------------------------------------------
// Errors
//--------------------------------------------------------------------------------------------------

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq)]
pub enum EvalError {
    // Parse errors returned from tokenizer
    FailedParsing(ParseErrors),

    // Failed to evaluate expression, and runtime error
    FailedExecution(RuntimeError),
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq, Clone)]
pub struct EvalResult(Rc<RefCell<ExecutionContext>>, ValueEnum);

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum ParsedItem {
    Expression(ExpressionEnum),
    Definition(DefinitionEnum),
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq)]
pub struct InvocationSpec {
    pub method_path: String,
    pub arguments: Vec<ExpressionEnum>,
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum ContextQueryErrorEnum {
    DuplicateNameError(DuplicateNameError),

    // returns context path that was not found
    ContextNotFoundError(String),

    // returns given query to the entry that was not found
    EntryNotFoundError(String),

    // returns wrong path or empty if path is empty
    WrongFieldPathError(Option<String>),
}

impl From<DuplicateNameError> for ContextQueryErrorEnum {
    fn from(err: DuplicateNameError) -> Self {
        ContextQueryErrorEnum::DuplicateNameError(err)
    }
}

impl From<ContextQueryErrorEnum> for ParseErrorEnum {
    fn from(err: ContextQueryErrorEnum) -> Self {
        OtherError(err.to_string())
    }
}

impl Display for ContextQueryErrorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextQueryErrorEnum::DuplicateNameError(err) => err.fmt(f),
            ContextQueryErrorEnum::ContextNotFoundError(path) => {
                write!(f, "Context '{}' not found", path)
            }
            ContextQueryErrorEnum::EntryNotFoundError(path) => {
                write!(f, "Entry '{}' not found", path)
            }
            ContextQueryErrorEnum::WrongFieldPathError(path) => match path {
                Some(path) => write!(f, "Invalid path '{}'", path),
                None => write!(f, "Field path is empty"),
            },
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
struct FieldPath<'a> {
    segments: Vec<&'a str>,
}

impl<'a> FieldPath<'a> {
    fn parse(input: &'a str) -> Result<Self, ContextQueryErrorEnum> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ContextQueryErrorEnum::WrongFieldPathError(None));
        }

        let segments: Vec<&str> = trimmed.split('.').map(|segment| segment.trim()).collect();
        if segments.iter().any(|segment| segment.is_empty()) {
            return Err(ContextQueryErrorEnum::WrongFieldPathError(Some(
                trimmed.to_string(),
            )));
        }

        Ok(FieldPath { segments })
    }

    fn is_root(&self) -> bool {
        self.segments.len() == 1
    }

    fn leaf(&self) -> &'a str {
        self.segments
            .last()
            .copied()
            .expect("FieldPath always contains at least one segment")
    }

    fn parent_segments(&self) -> &[&'a str] {
        debug_assert!(
            !self.is_root(),
            "parent_segments should not be called for root paths"
        );
        &self.segments[..self.segments.len() - 1]
    }

    fn parent_path(&self) -> String {
        self.parent_segments().join(".")
    }
}

impl ParsedItem {
    pub fn into_error(self) -> EvalError {
        match self {
            ParsedItem::Expression(expression) => EvalError::FailedParsing(
                ParseErrors::unexpected_token(Expression(expression), None),
            ),
            ParsedItem::Definition(definition) => EvalError::FailedParsing(
                ParseErrors::unexpected_token(Definition(definition), None),
            ),
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(PartialEq)]
pub struct ParseErrors(Vec<ParseErrorEnum>);

impl ParseErrors {
    pub fn unexpected_token(token: EToken, expected: Option<String>) -> Self {
        ParseErrors(vec![UnexpectedToken(Box::new(token), expected)])
    }

    pub fn errors(&self) -> &Vec<ParseErrorEnum> {
        &self.0
    }
}

impl Display for ParseErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", array_to_code_sep(self.0.iter(), "; "))
    }
}

impl From<ParseErrors> for EvalError {
    fn from(res: ParseErrors) -> Self {
        EvalError::FailedParsing(res)
    }
}

impl From<ParseErrorEnum> for EvalError {
    fn from(err: ParseErrorEnum) -> Self {
        EvalError::FailedParsing(ParseErrors(vec![err]))
    }
}

impl From<DuplicateNameError> for EvalError {
    fn from(err: DuplicateNameError) -> Self {
        EvalError::FailedParsing(ParseErrors(vec![ParseErrorEnum::from(err)]))
    }
}

impl From<ContextQueryErrorEnum> for EvalError {
    fn from(err: ContextQueryErrorEnum) -> Self {
        EvalError::FailedExecution(RuntimeError::eval_error(err.to_string()))
    }
}

impl From<RuntimeError> for EvalError {
    fn from(res: RuntimeError) -> Self {
        EvalError::FailedExecution(res)
    }
}

impl From<LinkingError> for EvalError {
    fn from(res: LinkingError) -> Self {
        EvalError::FailedExecution(RuntimeError::eval_error(res.to_string()))
    }
}

impl From<Rc<RefCell<LinkingError>>> for EvalError {
    fn from(res: Rc<RefCell<LinkingError>>) -> Self {
        EvalError::FailedExecution(RuntimeError::eval_error(res.borrow().to_string()))
    }
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::FailedParsing(errors) => write!(f, "{}", errors),
            EvalError::FailedExecution(err) => write!(f, "{}", err),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Service
//--------------------------------------------------------------------------------------------------

/// Service is stateless
pub struct EdgeRulesModel {
    pub ast_root: ContextObjectBuilder,
}

impl Default for EdgeRulesModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Reusable model holder that can be later converted to runtime to be executed.
/// Model is reused across multiple executions.
impl EdgeRulesModel {
    pub fn new() -> Self {
        Self {
            ast_root: ContextObjectBuilder::new(),
        }
    }

    fn parse_item(code: &str) -> Result<ParsedItem, ParseErrors> {
        let mut result = tokenize(code);

        if result.len() == 1 {
            match result.pop_front() {
                Some(Expression(expression)) => {
                    return Ok(ParsedItem::Expression(expression));
                }
                Some(Definition(definition)) => {
                    return Ok(ParsedItem::Definition(definition));
                }
                Some(other) => {
                    trace!("Single unexpected token: {:?}", other);
                    result.push_front(other);
                }
                None => {
                    trace!("No tokens found");
                }
            }
        }

        let mut errors: Vec<ParseErrorEnum> = Vec::new();
        let mut failed_tokens: Vec<EToken> = Vec::new();

        while let Some(token) = result.pop_front() {
            match token {
                EToken::ParseError(error) => {
                    errors.push(error);
                }
                EToken::Unparsed(unparsed) => {
                    errors.push(WrongFormat(unparsed.to_string()));
                }
                other => {
                    failed_tokens.push(other);
                }
            }
        }

        if errors.is_empty() {
            for token in failed_tokens {
                errors.push(UnexpectedToken(Box::new(token), None));
            }

            if errors.is_empty() {
                errors.push(UnexpectedEnd);
            }
        }

        Err(ParseErrors(errors))
    }

    pub fn parse_expression(code: &str) -> Result<ExpressionEnum, ParseErrors> {
        match Self::parse_item(code) {
            Ok(ParsedItem::Expression(expression)) => Ok(expression),
            Ok(ParsedItem::Definition(definition)) => Err(ParseErrors::unexpected_token(
                Definition(definition),
                Some("expression".to_string()),
            )),
            Err(errors) => Self::parse_expression_via_field(code, errors),
        }
    }

    fn parse_expression_via_field(
        code: &str,
        original_errors: ParseErrors,
    ) -> Result<ExpressionEnum, ParseErrors> {
        const DUMMY_NAME: &str = "tmp000001";
        match Self::parse_item(&format!("{DUMMY_NAME}: {code}")) {
            Ok(ParsedItem::Expression(ObjectField(_, field_expression))) => Ok(*field_expression),
            Ok(ParsedItem::Expression(unexpected)) => Err(ParseErrors::unexpected_token(
                Expression(unexpected),
                Some("expression".to_string()),
            )),
            Ok(ParsedItem::Definition(definition)) => Err(ParseErrors::unexpected_token(
                Definition(definition),
                Some("expression".to_string()),
            )),
            Err(_fallback_error) => Err(original_errors),
        }
    }

    pub fn set_invocation(
        &mut self,
        field_path: &str,
        spec: InvocationSpec,
    ) -> Result<(), ContextQueryErrorEnum> {
        // @Todo: why FieldPath::parse(field_path)?; is not used instead of validate_invocation_method?
        let method_path = Self::validate_invocation_method(&spec.method_path)?;
        let expression = ExpressionEnum::from(UserFunctionCall::new(method_path, spec.arguments));
        self.set_expression(field_path, expression)
    }

    fn validate_invocation_method(method_path: &str) -> Result<String, ContextQueryErrorEnum> {
        let trimmed = method_path.trim();
        if trimmed.is_empty() {
            return Err(ContextQueryErrorEnum::WrongFieldPathError(Some(
                method_path.to_string(),
            )));
        }
        Ok(trimmed.to_string())
    }

    pub fn set_expression(
        &mut self,
        field_path: &str,
        expression: ExpressionEnum,
    ) -> Result<(), ContextQueryErrorEnum> {
        let path = FieldPath::parse(field_path)?;
        let field_name = path.leaf();

        if path.is_root() {
            return self
                .ast_root
                .set_expression(field_name, expression)
                .map_err(ContextQueryErrorEnum::from);
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;
        {
            parent.borrow_mut().remove_field(field_name);
        }

        ContextObject::add_expression_field(&parent, field_name, expression)
            .map_err(ContextQueryErrorEnum::from)
    }

    pub fn remove_expression(&mut self, field_path: &str) -> Result<(), ContextQueryErrorEnum> {
        let path = FieldPath::parse(field_path)?;
        let field_name = path.leaf();

        if path.is_root() {
            self.ast_root.remove_field(field_name);
            return Ok(());
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;
        parent.borrow_mut().remove_field(field_name);
        Ok(())
    }

    pub fn get_expression(
        &self,
        field_path: &str,
    ) -> Result<Rc<RefCell<ExpressionEntry>>, ContextQueryErrorEnum> {
        let path = FieldPath::parse(field_path)?;
        let field_name = path.leaf();

        if path.is_root() {
            return self
                .ast_root
                .get_expression(field_name)
                .ok_or_else(|| ContextQueryErrorEnum::EntryNotFoundError(field_path.to_string()));
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;
        let expression = {
            let borrowed = parent.borrow();
            borrowed.expressions.get(field_name).cloned()
        };
        expression.ok_or_else(|| ContextQueryErrorEnum::EntryNotFoundError(field_path.to_string()))
    }

    pub fn get_expression_type(
        &mut self,
        field_path: &str,
    ) -> Result<ValueType, ContextQueryErrorEnum> {
        let runtime = self
            .to_runtime_snapshot()
            .map_err(|err| ContextQueryErrorEnum::ContextNotFoundError(err.to_string()))?;
        runtime.get_type(field_path)
    }

    pub fn set_user_type(
        &mut self,
        type_path: &str,
        type_definition: UserTypeBody,
    ) -> Result<(), ContextQueryErrorEnum> {
        let path = FieldPath::parse(type_path)?;
        let type_name = path.leaf();

        if path.is_root() {
            self.ast_root.remove_user_type_definition(type_name);
            self.ast_root
                .set_user_type_definition(type_name.to_string(), type_definition);
            return Ok(());
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;
        ContextObject::remove_user_type_definition(&parent, type_name);
        ContextObject::set_user_type_definition(&parent, type_name, type_definition);
        Ok(())
    }

    pub fn remove_user_type(&mut self, type_path: &str) -> Result<(), ContextQueryErrorEnum> {
        let path = FieldPath::parse(type_path)?;
        let type_name = path.leaf();

        if path.is_root() {
            self.ast_root.remove_user_type_definition(type_name);
            return Ok(());
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;
        ContextObject::remove_user_type_definition(&parent, type_name);
        Ok(())
    }

    pub fn get_user_type(&self, type_path: &str) -> Result<UserTypeBody, ContextQueryErrorEnum> {
        let path = FieldPath::parse(type_path)?;
        let type_name = path.leaf();

        if path.is_root() {
            return self
                .ast_root
                .get_user_type(type_name)
                .ok_or_else(|| ContextQueryErrorEnum::EntryNotFoundError(type_path.to_string()));
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;

        let user_type = {
            let borrowed = parent.borrow();
            borrowed.get_user_type(type_name)
        };
        user_type.ok_or_else(|| ContextQueryErrorEnum::EntryNotFoundError(type_path.to_string()))
    }

    pub fn set_user_function(
        &mut self,
        definition: FunctionDefinition,
        context_path: Option<Vec<&str>>,
    ) -> Result<(), ContextQueryErrorEnum> {
        if let Some(path) = context_path {
            if path.is_empty() {
                return self.insert_root_user_function(definition);
            }

            let parent = self.resolve_context_or_error(path.as_slice())?;

            {
                parent.borrow_mut().remove_field(definition.name.as_str());
            }

            return ContextObject::add_user_function(&parent, definition)
                .map_err(ContextQueryErrorEnum::from);
        }

        self.insert_root_user_function(definition)
    }

    pub fn remove_user_function(
        &mut self,
        function_path: &str,
    ) -> Result<(), ContextQueryErrorEnum> {
        let path = FieldPath::parse(function_path)?;
        let function_name = path.leaf();

        if path.is_root() {
            self.ast_root.remove_field(function_name);
            return Ok(());
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;
        parent.borrow_mut().remove_field(function_name);
        Ok(())
    }

    pub fn get_user_function(
        &self,
        function_path: &str,
    ) -> Result<Rc<RefCell<MethodEntry>>, ContextQueryErrorEnum> {
        let path = FieldPath::parse(function_path)?;
        let function_name = path.leaf();

        if path.is_root() {
            return self
                .ast_root
                .get_user_function(function_name)
                .ok_or_else(|| {
                    ContextQueryErrorEnum::EntryNotFoundError(function_path.to_string())
                });
        }

        let parent = self.resolve_context_or_error(path.parent_segments())?;
        let function = {
            let borrowed = parent.borrow();
            borrowed.get_function(function_name)
        };
        function.ok_or_else(|| ContextQueryErrorEnum::EntryNotFoundError(function_path.to_string()))
    }

    pub fn merge_context_object(
        &mut self,
        object: Rc<RefCell<ContextObject>>,
    ) -> Result<(), DuplicateNameError> {
        self.ast_root
            .merge_context_object(object)
    }

    fn resolve_context_or_error(
        &self,
        path_segments: &[&str],
    ) -> Result<Rc<RefCell<ContextObject>>, ContextQueryErrorEnum> {
        debug_assert!(!path_segments.is_empty());
        if let Some(ctx) = self.ast_root.resolve_context(path_segments) {
            return Ok(ctx);
        }
        if let Some(ctx) = self.resolve_function_context(path_segments) {
            return Ok(ctx);
        }
        Err(ContextQueryErrorEnum::ContextNotFoundError(
            path_segments.join("."),
        ))
    }

    fn resolve_function_context(
        &self,
        path_segments: &[&str],
    ) -> Option<Rc<RefCell<ContextObject>>> {
        let function_name = path_segments.first()?;
        let function = self.ast_root.get_user_function(function_name)?;
        let mut current = Rc::clone(&function.borrow().function_definition.body);

        // @Todo: this is code duplication with ContextObject::resolve_context
        for segment in path_segments.iter().skip(1) {
            let next = {
                let borrowed = current.borrow();
                borrowed.node().get_child(segment)
            };
            match next {
                Some(child) => current = child,
                None => return None,
            }
        }
        Some(current)
    }

    fn insert_root_user_function(
        &mut self,
        definition: FunctionDefinition,
    ) -> Result<(), ContextQueryErrorEnum> {
        self.ast_root.remove_field(definition.name.as_str());
        self.ast_root
            .add_definition(DefinitionEnum::UserFunction(definition))
            .map(|_| ())
            .map_err(ContextQueryErrorEnum::from)
    }

    pub fn append_source(&mut self, code: &str) -> Result<(), ParseErrors> {
        let parsed = Self::parse_item(code)?;

        match parsed {
            ParsedItem::Expression(ObjectField(field, field_expression)) => {
                self.set_expression(field.as_str(), *field_expression)
                    .map_err(Self::context_update_error)?;
            }
            ParsedItem::Expression(ExpressionEnum::StaticObject(context_object)) => {
                self.merge_context_object(context_object)
                    .map_err(Self::context_update_error)?;
            }
            ParsedItem::Definition(definition) => match definition {
                DefinitionEnum::UserFunction(definition) => self
                    .set_user_function(definition, None)
                    .map_err(Self::context_update_error)?,
                DefinitionEnum::UserType(user_type) => self
                    .set_user_type(user_type.name.as_str(), user_type.body)
                    .map_err(Self::context_update_error)?,
            },
            ParsedItem::Expression(unexpected) => {
                return Err(ParseErrors::unexpected_token(
                    Expression(unexpected),
                    Some("value assignment expression or object".to_string()),
                ));
            }
        }

        Ok(())
    }

    pub fn load_source(&mut self, code: &str) -> Result<(), ParseErrors> {
        self.append_source(code)
    }

    fn context_update_error(err: impl Into<ContextQueryErrorEnum>) -> ParseErrors {
        ParseErrors(vec![ParseErrorEnum::from(err.into())])
    }

    /// Converts the model into a runtime instance.
    /// No further code modifications allowed after this call
    pub fn to_runtime(self) -> Result<EdgeRulesRuntime, LinkingError> {
        let static_context = self.ast_root.build();
        Ok(EdgeRulesRuntime::new(link_parts(static_context)?))
    }

    /// Gets a runtime snapshot of the current model state.
    /// Model can be further modified after this call
    pub fn to_runtime_snapshot(&mut self) -> Result<EdgeRulesRuntime, LinkingError> {
        let current_builder = std::mem::take(&mut self.ast_root);
        let static_context = current_builder.build();
        let linked_context = link_parts(static_context)?;
        let result = EdgeRulesRuntime::new(Rc::clone(&linked_context));
        // @Todo: need to find a cheaper way to clone the AST tree
        // @Todo: need to find a way to preserve already set links to speed up the next linking
        self.ast_root
            .append(linked_context)
            .map_err(|err| LinkingError::other_error(err.to_string()))?;
        Ok(result)
    }
}

//--------------------------------------------------------------------------------------------------
// Runtime
//--------------------------------------------------------------------------------------------------

pub struct EdgeRulesRuntime {
    pub context: Rc<RefCell<ExecutionContext>>,
    pub static_tree: Rc<RefCell<ContextObject>>,
}

/**
 * Runtime is stateful, it holds the execution context
 */
impl EdgeRulesRuntime {
    pub fn new(static_tree: Rc<RefCell<ContextObject>>) -> EdgeRulesRuntime {
        let context = ExecutionContext::create_root_context(static_tree.clone());
        EdgeRulesRuntime {
            context,
            static_tree,
        }
    }

    /**
     * Evaluates a single field in the root context
     */
    pub fn evaluate_field(&self, name: &str) -> Result<ValueEnum, RuntimeError> {
        let expression = EdgeRulesModel::parse_expression(name).map_err(|errors| {
            RuntimeError::eval_error(format!("Failed to parse `{}`: {}", name, errors))
        })?;

        self.evaluate_expression(expression)
    }

    /**
     * Calls a method with given arguments that is already defined in the context
     */
    pub fn call_method(
        &self,
        name: &str,
        args: Vec<ExpressionEnum>,
    ) -> Result<ValueEnum, RuntimeError> {
        let call = UserFunctionCall::new(name.to_string(), args);
        self.evaluate_expression(ExpressionEnum::from(call))
    }

    pub fn evaluate_expression(
        &self,
        mut expression: ExpressionEnum,
    ) -> Result<ValueEnum, RuntimeError> {
        expression.link(Rc::clone(&self.static_tree))?;
        expression.eval(Rc::clone(&self.context))
    }

    pub fn evaluate_expression_str(&self, code: &str) -> Result<ValueEnum, EvalError> {
        let expression = EdgeRulesModel::parse_expression(code)?;
        Ok(self.evaluate_expression(expression)?)
    }

    /**
     * Evaluates all expressions in the context tree, starting from the root context
     */
    pub fn eval_all(&self) -> Result<(), RuntimeError> {
        ExecutionContext::eval_all_fields(&self.context)
    }

    pub fn get_type(&self, field_path: &str) -> Result<ValueType, ContextQueryErrorEnum> {
        if field_path == "*" {
            // For wildcard requests, we link all top-level functions to ensure they appear in the schema.
            // This is necessary because link_parts does not link function bodies by default.
            let field_names = self.static_tree.borrow().get_field_names();
            for name in field_names {
                if let Ok(EObjectContent::UserFunctionRef(metaphor)) = self.static_tree.borrow().get(name) {
                    if metaphor.borrow().field_type.is_err() {
                        let body = Rc::clone(&metaphor.borrow().function_definition.body);
                        let _ = link_parts(Rc::clone(&body));
                        let vt = ValueType::ObjectType(body);
                        metaphor.borrow_mut().field_type = Ok(vt);
                    }
                }
            }
            return Ok(ValueType::ObjectType(Rc::clone(&self.static_tree)));
        }

        let path = FieldPath::parse(field_path)?;
        let field_name = path.leaf();

        let parent = if path.is_root() {
            Rc::clone(&self.static_tree)
        } else {
            self.static_tree
                .borrow()
                .resolve_context(path.parent_segments())
                .ok_or_else(|| {
                    ContextQueryErrorEnum::ContextNotFoundError(path.parent_path().to_string())
                })?
        };

        let borrowed = parent.borrow();
        match borrowed.get(field_name) {
            Ok(content) => match content {
                EObjectContent::ExpressionRef(entry) => entry
                    .borrow()
                    .field_type
                    .clone()
                    .map_err(|_| ContextQueryErrorEnum::EntryNotFoundError(field_path.to_string())),
                EObjectContent::UserFunctionRef(entry) => {
                    let mut entry_mut = entry.borrow_mut();
                    if let Ok(vt) = &entry_mut.field_type {
                        return Ok(vt.clone());
                    }
                    let body = Rc::clone(&entry_mut.function_definition.body);
                    let _ = link_parts(Rc::clone(&body));
                    let vt = ValueType::ObjectType(body);
                    entry_mut.field_type = Ok(vt.clone());
                    Ok(vt)
                }
                EObjectContent::ObjectRef(obj) => Ok(ValueType::ObjectType(Rc::clone(&obj))),
                EObjectContent::Definition(vt) => Ok(vt),
                EObjectContent::ConstantValue(v) => Ok(v.get_type()),
            },
            Err(_) => {
                if let Some(body) = borrowed.get_user_type(field_name) {
                    let vt = match body {
                        UserTypeBody::TypeRef(tref) => borrowed.resolve_type_ref(&tref).map_err(|e: LinkingError| {
                            ContextQueryErrorEnum::ContextNotFoundError(e.to_string())
                        })?,
                        UserTypeBody::TypeObject(obj) => ValueType::ObjectType(obj),
                    };
                    return Ok(vt);
                }
                Err(ContextQueryErrorEnum::EntryNotFoundError(
                    field_path.to_string(),
                ))
            }
        }
    }
}
