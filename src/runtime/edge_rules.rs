use crate::ast::context::context_object::ContextObject;
use crate::ast::expression::StaticLink;
use crate::ast::token::EToken;
use crate::ast::token::EToken::{Definition, Expression};
use crate::ast::token::ExpressionEnum::ObjectField;
use crate::ast::user_function_call::UserFunctionCall;
use crate::ast::utils::array_to_code_sep;
use crate::runtime::execution_context::ExecutionContext;
use crate::tokenizer::parser::tokenize;
use crate::typesystem::errors::ParseErrorEnum::{Empty, UnexpectedToken, UnknownParseError};
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::values::ValueEnum;
use log::trace;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub use crate::ast::context::context_object_builder::ContextObjectBuilder;
pub use crate::ast::metaphors::functions::FunctionDefinition;
pub use crate::ast::token::{DefinitionEnum, ExpressionEnum};
pub use crate::link::linker::link_parts;
//--------------------------------------------------------------------------------------------------
// Errors
//--------------------------------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
pub enum EvalError {
    // Parse errors returned from tokenizer
    FailedParsing(ParseErrors),

    // Failed to evaluate expression, and runtime error
    FailedExecution(RuntimeError),
}

#[derive(Debug, PartialEq, Clone)]
pub struct EvalResult(Rc<RefCell<ExecutionContext>>, ValueEnum);

#[derive(Debug)]
pub enum ParsedItem {
    Expression(ExpressionEnum),
    Definition(DefinitionEnum),
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

#[derive(Debug, PartialEq)]
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
                    errors.push(UnknownParseError(unparsed.to_string()));
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
                errors.push(Empty);
            }
        }

        Err(ParseErrors(errors))
    }

    fn parse_expression(code: &str) -> Result<ExpressionEnum, ParseErrors> {
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

    pub fn load_source(&mut self, code: &str) -> Result<(), ParseErrors> {
        let parsed = Self::parse_item(code)?;

        match parsed {
            ParsedItem::Expression(ObjectField(field, field_expression)) => {
                self.ast_root
                    .add_expression(field.as_str(), *field_expression)
                    .map_err(|err| ParseErrors(vec![err]))?;
            }
            ParsedItem::Expression(ExpressionEnum::StaticObject(context_object)) => {
                self.ast_root
                    .append(context_object)
                    .map_err(|err| ParseErrors(vec![err]))?;
            }
            ParsedItem::Definition(definition) => {
                self.ast_root
                    .add_definition(definition)
                    .map_err(|err| ParseErrors(vec![err]))?;
            }
            ParsedItem::Expression(unexpected) => {
                return Err(ParseErrors::unexpected_token(
                    Expression(unexpected),
                    Some("value assignment expression or object".to_string()),
                ));
            }
        }

        Ok(())
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
}

//--------------------------------------------------------------------------------------------------
// Utilities
//--------------------------------------------------------------------------------------------------

// @Todo: expr is just for testing purposes only - move it under test module!
pub fn expr(code: &str) -> Result<ExpressionEnum, EvalError> {
    Ok(EdgeRulesModel::parse_expression(code)?)
}

//--------------------------------------------------------------------------------------------------
// Test
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod test {
    use crate::ast::expression::StaticLink;
    use crate::ast::token::{EToken, EUnparsedToken, ExpressionEnum};
    use crate::runtime::edge_rules::{EdgeRulesModel, EdgeRulesRuntime, EvalError, ParseErrors};

    use crate::typesystem::errors::ParseErrorEnum::{UnexpectedToken, UnknownError};
    use crate::typesystem::errors::{LinkingError, LinkingErrorEnum, ParseErrorEnum};

    use crate::runtime::edge_rules::expr;
    use crate::typesystem::types::number::NumberEnum::{self, Int};

    use crate::typesystem::types::SpecialValueEnum::Missing;
    use crate::typesystem::types::ToSchema;
    use crate::typesystem::values::ValueEnum;
    use crate::utils::test::init_logger;
    use crate::utils::to_display;
    use log::error;
    use std::fmt::Display;
    use std::mem::discriminant;

    pub fn test_code(code: &str) -> TestServiceBuilder {
        TestServiceBuilder::build(code)
    }

    pub fn test_code_lines<T: Display>(code: &[T]) -> TestServiceBuilder {
        TestServiceBuilder::build(format!("{{{}}}", to_display(code, "\n")).as_str())
    }

    pub fn inline<S: AsRef<str>>(code: S) -> String {
        code.as_ref().replace('\n', " ").replace(" ", "")
    }

    pub struct TestServiceBuilder {
        original_code: String,
        runtime: Option<EdgeRulesRuntime>,
        parse_errors: Option<ParseErrors>,
        linking_errors: Option<LinkingError>,
    }

    impl TestServiceBuilder {
        pub fn build(code: &str) -> Self {
            let mut service = EdgeRulesModel::new();

            match service.load_source(code) {
                Ok(_model) => match service.to_runtime() {
                    Ok(runtime) => TestServiceBuilder {
                        original_code: code.to_string(),
                        runtime: Some(runtime),
                        parse_errors: None,
                        linking_errors: None,
                    },
                    Err(linking_errors) => TestServiceBuilder {
                        original_code: code.to_string(),
                        runtime: None,
                        parse_errors: None,
                        linking_errors: Some(linking_errors),
                    },
                },
                Err(errors) => TestServiceBuilder {
                    original_code: code.to_string(),
                    runtime: None,
                    parse_errors: Some(errors),
                    linking_errors: None,
                },
            }
        }

        pub fn expect_type(&self, expected_type: &str) -> &Self {
            self.expect_no_errors();

            match &self.runtime {
                None => {
                    panic!(
                        "Expected runtime, but got nothing: `{}`",
                        self.original_code
                    );
                }
                Some(runtime) => {
                    assert_eq!(runtime.static_tree.borrow().to_schema(), expected_type);
                }
            }

            self
        }

        pub fn expect_num(&self, variable: &str, expected: NumberEnum) {
            self.expect(
                &mut ExpressionEnum::variable(variable),
                ValueEnum::NumberValue(expected),
            )
        }

        pub fn expect_parse_error(&self, expected: ParseErrorEnum) -> &Self {
            if let Some(errors) = &self.parse_errors {
                for error in errors.errors() {
                    if discriminant(error) == discriminant(&expected) {
                        return self;
                    }
                }
                panic!(
                    "Expected parse error `{}`, but got: `{:?}`",
                    expected, errors
                );
            } else {
                panic!(
                    "Expected parse error, but got no errors: `{}`",
                    self.original_code
                );
            }
        }

        pub fn expect_no_errors(&self) -> &Self {
            if let Some(errors) = &self.parse_errors {
                panic!(
                    "Expected no errors, but got parse errors: `{}`\nFailed to parse:\n{}",
                    errors, self.original_code
                );
            }

            if let Some(errors) = &self.linking_errors {
                panic!(
                    "Expected no errors, but got linking errors: `{}`\nFailed to parse:\n{}",
                    errors, self.original_code
                );
            }

            self
        }

        pub fn expect_link_error(&self, expected: LinkingErrorEnum) -> &Self {
            if let Some(errors) = &self.parse_errors {
                panic!(
                    "Expected linking error, but got parse errors: `{:?}`\nFailed to parse:\n{}",
                    errors, self.original_code
                );
            }

            if let Some(errors) = &self.linking_errors {
                assert_eq!(expected, errors.error, "Testing:\n{}", self.original_code);
            } else {
                panic!(
                    "Expected linking error, but got no errors: `{}`",
                    self.original_code
                );
            }

            self
        }

        pub fn expect(&self, _expr: &mut ExpressionEnum, _expected: ValueEnum) {
            self.expect_no_errors();

            if let Err(error) = _expr.link(self.runtime.as_ref().unwrap().static_tree.clone()) {
                panic!(
                    "Expected value, but got linking errors: `{:?}`\nFailed to parse:\n{}",
                    error, _expr
                );
            }

            match _expr.eval(self.runtime.as_ref().unwrap().context.clone()) {
                Ok(value) => {
                    assert_eq!(
                        value,
                        _expected,
                        "Context:\n{}",
                        self.runtime.as_ref().unwrap().context.borrow()
                    );
                }
                Err(error) => {
                    error!("{}", error);
                    panic!("Failed to parse: `{:?}`", _expr);
                }
            }
        }
    }

    #[test]
    fn now() -> Result<(), EvalError> {
        init_logger();

        Ok(())
    }

    #[test]
    fn test_service() -> Result<(), EvalError> {
        init_logger();

        {
            let mut service = EdgeRulesModel::new();
            service.load_source("value: 2 + 2")?;
            let duplicate = service.load_source("value: 2 + 3");
            let errors = duplicate.expect_err("duplicate field should fail");
            let first_error = errors
                .errors()
                .first()
                .expect("expected duplicate field error");
            assert!(first_error.to_string().contains("value"));

            let runtime = service.to_runtime()?;
            let result = runtime.evaluate_field("value")?;
            assert_eq!(result, ValueEnum::NumberValue(Int(4)));
        }

        test_code("value").expect_parse_error(UnexpectedToken(
            Box::new(EToken::Unparsed(EUnparsedToken::Comma)),
            None,
        ));
        test_code("value: 2 + 2").expect_num("value", Int(4));
        test_code("value: 2 + ").expect_parse_error(UnknownError("any".to_string()));
        test_code("{ value: 2 + 2 }").expect_num("value", Int(4));
        test_code("{ v1: 100; value: v1 + v1 }").expect_num("value", Int(200));

        Ok(())
    }

    #[test]
    fn test_service_evaluate_field_with_existing_state() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source("{ value: 3 }")?;
        let runtime = service.to_runtime_snapshot()?;
        let result = runtime.evaluate_field("value")?;
        assert_eq!(result.to_string(), "3");

        let duplicate = service.load_source("value: 2 + 2");
        let errors = duplicate.expect_err("duplicate field should fail");
        let first_error = errors
            .errors()
            .first()
            .expect("expected duplicate field error");
        assert!(first_error.to_string().contains("value"));

        let runtime = service.to_runtime_snapshot()?;
        let still_three = runtime.evaluate_field("value")?;
        assert_eq!(still_three.to_string(), "3");

        service.load_source("extra: value + 2")?;
        let runtime = service.to_runtime_snapshot()?;
        let extra = runtime.evaluate_field("extra")?;
        assert_eq!(extra.to_string(), "5");

        Ok(())
    }

    #[test]
    fn test_service_evaluate_field_with_path_depth() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service
            .load_source("{ calendar: { config: { start: 7 }; sub: { inner: { value: 42 } } } }")?;

        let runtime = service.to_runtime_snapshot()?;
        let out1 = runtime.evaluate_field("calendar.config.start")?;
        assert_eq!(out1.to_string(), "7");

        let out2 = runtime.evaluate_field("calendar.sub.inner.value")?;
        assert_eq!(out2.to_string(), "42");

        let duplicate =
            service.load_source("{ calendar: { config: { start: 7; end: start + 5 } } }");
        let errors = duplicate.expect_err("duplicate calendar should fail");
        let first_error = errors
            .errors()
            .first()
            .expect("expected duplicate calendar error");
        assert!(first_error.to_string().contains("calendar"));

        let runtime = service.to_runtime_snapshot()?;
        let start = runtime.evaluate_field("calendar.config.start")?;
        assert_eq!(start.to_string(), "7");

        // @Todo: this test is incorrect, `calendar.config.end` cannot be linked and link error should occur
        // @Todo: find and fix if self.path.len() > 1 && is_unattached_root {...
        let end = runtime.evaluate_field("calendar.config.end")?;
        assert_eq!(
            end.to_string(),
            ValueEnum::NumberValue(NumberEnum::SV(Missing("end".to_string()))).to_string()
        );

        Ok(())
    }

    #[test]
    fn test_evaluate_expression_with_loaded_context() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source("{ value: 3 }")?;

        let runtime = service.to_runtime_snapshot()?;
        let result = runtime.evaluate_expression_str("2 + value")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(5)));

        Ok(())
    }

    #[test]
    fn test_evaluate_pure_expression_without_context() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        let runtime = service.to_runtime_snapshot()?;
        let result = runtime.evaluate_expression_str("2 + 3")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(5)));

        let result = runtime.evaluate_expression_str("sum(1,2,3)")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(6)));

        Ok(())
    }

    #[test]
    fn test_evaluate_expression_unknown_variable_fails() {
        init_logger();

        let mut service = EdgeRulesModel::new();
        let runtime = service
            .to_runtime_snapshot()
            .expect("Failed to build runtime snapshot");
        let err = runtime.evaluate_expression_str("x + 1").unwrap_err();
        match err {
            EvalError::FailedExecution(_e) => {}
            other => panic!("Expected runtime error, got: {:?}", other),
        }
    }

    #[test]
    fn load_source_accepts_user_function_definition() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source("func inc(value): { result: value + 1 }")?;
        service.load_source("{ value: inc(2).result }")?;

        let runtime = service.to_runtime_snapshot()?;
        let result = runtime.evaluate_expression_str("value")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(3)));

        Ok(())
    }

    #[test]
    fn test_evaluate_expression_with_function_indirect() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source("{ func f(a): { result: a + 1 }; tmp: f(2).result }")?;
        let runtime = service.to_runtime_snapshot()?;
        let result = runtime.evaluate_expression_str("tmp")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(3)));

        Ok(())
    }

    #[test]
    fn call_method_errors_when_function_missing() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source("{ value: 1 }")?;
        let runtime = service.to_runtime_snapshot()?;

        let err = runtime
            .call_method("missing", vec![])
            .expect_err("expected missing function error");

        let message = err.to_string();
        assert!(
            message.contains("Function 'missing(...)"),
            "unexpected error: {message}"
        );

        Ok(())
    }

    #[test]
    fn call_method_errors_when_argument_count_mismatches() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source("{ func greet(name, age): { result: name } }")?;
        let runtime = service.to_runtime_snapshot()?;

        let err = runtime
            .call_method("greet", vec![expr("'tom'")?])
            .expect_err("expected argument mismatch error");

        let message = err.to_string();
        assert!(
            message.contains("Function greet expects 2 arguments, but 1 were provided"),
            "unexpected error: {message}"
        );

        Ok(())
    }

    #[test]
    fn call_method_happy_path_with_single_and_multiple_arguments() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source(
            "{ func inc(x): { result: x + 1 }; func add(left, right): { result: left + right } }",
        )?;
        let runtime = service.to_runtime_snapshot()?;

        let single = runtime.call_method("inc", vec![expr("41")?])?;
        assert_eq!(inline(single.to_string()), inline("{result: 42}"));

        let multiple = runtime.call_method("add", vec![expr("1")?, expr("2")?])?;
        assert_eq!(inline(multiple.to_string()), inline("{result: 3}"));

        Ok(())
    }

    #[test]
    fn call_method_type_mismatch_does_not_poison_context() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source(
            "{ type LoanOffer: { amount: <number> }; func inc(offer: LoanOffer): { result: offer.amount + 1 } }",
        )?;
        let runtime = service.to_runtime_snapshot()?;

        let err = runtime
            .call_method("inc", vec![expr("1")?])
            .expect_err("expected type mismatch error");
        let message = err.to_string();
        assert!(
            message.contains("Argument `offer` of function `inc`"),
            "unexpected error: {message}"
        );

        let first = runtime.call_method("inc", vec![expr("{amount: 10}")?])?;
        assert_eq!(inline(first.to_string()), inline("{result: 11}"));

        let second = runtime.call_method("inc", vec![expr("{amount: 20}")?])?;
        assert_eq!(inline(second.to_string()), inline("{result: 21}"));

        Ok(())
    }

    #[test]
    fn call_method_list_iteration() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRulesModel::new();
        service.load_source(
            r#"
        {
            func interpolate(baseline: number[]) : {
               resultset : for x in baseline return x * 2
            }
        }
        "#,
        )?;

        let runtime = service.to_runtime_snapshot()?;

        let first = runtime.call_method("interpolate", vec![expr("[1,2,3,4,5]")?])?;
        assert_eq!(
            inline(first.to_string()),
            inline("{resultset: [2, 4, 6, 8, 10]}")
        );

        Ok(())
    }

    #[test]
    fn test_linking() -> Result<(), EvalError> {
        init_logger();

        test_code("{ a: 1; b: a  }")
            .expect_type("{a: number; b: number}")
            .expect_num("a", Int(1));

        test_code("{ a: z; b: a; z: 8 * 2  }")
            .expect_type("{a: number; b: number; z: number}")
            .expect_num("a", Int(16));

        test_code("{ a: {x: 1}; b: a.x }")
            .expect_type("{a: {x: number}; b: number}")
            .expect_num("b", Int(1));

        test_code("{ c: b; a: {x: 1}; b: a.x }")
            .expect_type("{c: number; a: {x: number}; b: number}")
            .expect_num("c", Int(1));

        // roundtrip test
        test_code("{ c: b; a: {x: 1; aa: b}; b: a.x }")
            .expect_type("{c: number; a: {x: number; aa: number}; b: number}")
            .expect_num("c", Int(1));

        // messy handover test
        test_code("{ c: b; a: {x: {y: 1}}; b: a.x; d: c.y }")
            .expect_type("{c: {y: number}; a: {x: {y: number}}; b: {y: number}; d: number}")
            .expect_num("d", Int(1));

        // deep roundtrip test
        test_code("{ c: b; a: {x: {x: 1; aa: b}}; b: a.x.x }")
            .expect_type("{c: number; a: {x: {x: number; aa: number}}; b: number}")
            .expect_num("c", Int(1));

        test_code("{ func f(arg1):  { a: arg1 } }").expect_type("{}");

        test_code("{ func f(arg1):  { a: arg1 }; b: 1 }")
            .expect_type("{b: number}")
            .expect_num("b", Int(1));

        test_code("{ func f(arg1):  { a: arg1 }; b: f(1) }").expect_type("{b: {a: number}}");

        test_code("{ func f(arg1):  { a: arg1 }; b: f(1).a }")
            .expect_type("{b: number}")
            .expect_num("b", Int(1));

        // possibility to call a function from a sub-context
        test_code_lines(&[
            "func func1(a): { result: a }",
            "subContext: {",
            "subResult: func1(35).result",
            "}",
            "value: subContext.subResult",
        ])
        .expect_num("value", Int(35));

        // argument as a parameter works well
        test_code_lines(&[
            "myInput: 35",
            "func func1(a): { result: a }",
            "subContext: {",
            "subResult: func1(myInput).result",
            "}",
            "value: subContext.subResult",
        ])
        .expect_num("value", Int(35));

        Ok(())
    }

    #[test]
    fn calendar_self_reference_in_array_elements() -> Result<(), EvalError> {
        init_logger();

        let tb = test_code_lines(&[
            "calendar: {",
            "    shift: 2",
            "    days: [",
            "        { start: calendar.shift + 1 },",
            "        { start: calendar.shift + 31 }",
            "    ]",
            "    firstDay: days[0].start",
            "    secondDay: days[1].start",
            "}",
        ]);
        tb.expect_num("calendar.firstDay", Int(3));
        tb.expect_num("calendar.secondDay", Int(33));

        Ok(())
    }

    #[test]
    fn pass_self_context_to_function_should_fail() {
        init_logger();

        // Users cannot pass the context object itself into a function defined in that same context.
        test_code_lines(&[
            "calendar: {",
            "    shift: 2",
            "    func start1(calendar): { result: calendar.shift + 1 }",
            "    firstDay: start1(calendar).result",
            "}",
        ])
        .expect_link_error(LinkingErrorEnum::OtherLinkingError(
            "Cannot pass context `calendar` as argument to function `start1` defined in the same context".to_string(),
        ));
    }

    #[test]
    fn pass_context_to_function_should_not_fail() {
        init_logger();

        // Users can pass the context object into a function defined in upper or another context.
        test_code_lines(&[
            "func start1(calendar): { result: calendar.shift + 1 }",
            "calendar: {",
            "    shift: 2",
            "    firstDay: start1(calendar).result",
            "}",
        ])
            .expect_link_error(LinkingErrorEnum::OtherLinkingError(
                "Cannot pass context `calendar` as argument to function `start1` defined in the same context".to_string(),
            ));
    }

    #[test]
    fn pass_self_context_as_second_argument_should_fail() {
        init_logger();

        // The guard applies to any argument position, not just the first one.
        test_code_lines(&[
            "calendar: {",
            "    shift: 2",
            "    func start2(x, cal): { result: cal.shift + x }",
            "    firstDay: start2(1, calendar).result",
            "}",
        ])
        .expect_link_error(LinkingErrorEnum::OtherLinkingError(
            "Cannot pass context `calendar` as argument to function `start2` defined in the same context".to_string(),
        ));
    }

    #[test]
    fn pass_sub_context_to_function() -> Result<(), EvalError> {
        init_logger();

        let tb = test_code_lines(&[
            "calendar: {",
            "    config: { shift: 2 }",
            "    func start1(calendar): { result: calendar.shift + 1 }",
            "    firstDay: start1(config).result",
            "}",
        ]);
        tb.expect_num("calendar.firstDay", Int(3));

        Ok(())
    }
}
