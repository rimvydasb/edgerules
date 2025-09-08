use crate::ast::context::context_object::ContextObject;
use crate::ast::context::context_object_builder::ContextObjectBuilder;
use crate::ast::context::context_object_type::EObjectContent;
use crate::ast::expression::StaticLink;
use crate::ast::token::EToken::{Definition, Expression};
use crate::ast::token::ExpressionEnum::ObjectField;
use crate::ast::token::{DefinitionEnum, EToken, ExpressionEnum};
use crate::ast::utils::array_to_code_sep;
use crate::link::linker;
use crate::link::node_data::{ContentHolder, Node, NodeData};
use crate::runtime::execution_context::ExecutionContext;
use crate::tokenizer::parser::tokenize;
use crate::typesystem::errors::ParseErrorEnum::{Empty, UnexpectedToken, UnknownParseError};
use crate::typesystem::errors::{LinkingError, ParseErrorEnum, RuntimeError};
use crate::typesystem::values::ValueEnum;
use crate::utils::to_display;
use log::trace;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
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
// Engine
//--------------------------------------------------------------------------------------------------

struct EdgeRulesEngine {}

impl EdgeRulesEngine {
    /// Code parsing to a single expression.
    /// Returns either a single expression or a single definition.
    /// If there are multiple expressions or definitions, then it returns a list of errors and not mapped tokens.
    pub fn parse_code(code: &str) -> Result<ParsedItem, ParseErrors> {
        let mut result = tokenize(&String::from(code));

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
                _ => {
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
                    // it could be that some tokes will be valid, but not mapped to AST root
                    failed_tokens.push(other);
                }
            }
        }

        if errors.is_empty() {
            for token in failed_tokens {
                // since no errors, I can assume that all tokens are unexpected
                errors.push(UnexpectedToken(Box::new(token), None));
            }

            if errors.is_empty() {
                // only if no code is provided
                errors.push(Empty);
            }
        }

        Err(ParseErrors(errors))
    }

    // pub fn evaluate_context(ctx: Rc<RefCell<ExecutionContext>>) {
    //     let names = (*ctx).borrow().get_field_names().clone();
    //
    //     for name in names.iter() {
    //         if let Ok(Reference(new_context)) = ExecutionContext::eval_field(Rc::clone(&ctx), name.as_str()) {
    //             Self::evaluate_context(new_context)
    //         }
    //     }
    // }

    // Evaluates a single expression
    // Server and Runtime is destroyed after evaluation so it is super inefficient, better use for testing only
    // pub fn evaluate_code(code: &str, field: &str) -> Result<EvalResult, EvalError> {
    //     let engine = EdgeRules::new();
    //     engine.load_source(code)?;
    //     let runtime = engine.to_runtime();
    //     let result = runtime.evaluate_field(field)?;
    //
    //     Ok(EvalResult(Rc::clone(&runtime.context), result))
    // }
}

//--------------------------------------------------------------------------------------------------
// Service
//--------------------------------------------------------------------------------------------------

/// Service is stateless
pub struct EdgeRules {
    pub ast_root: ContextObjectBuilder,
}

impl Default for EdgeRules {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeRules {
    pub fn new() -> EdgeRules {
        EdgeRules {
            ast_root: ContextObjectBuilder::new(),
        }
    }

    pub fn load_source(&mut self, code: &str) -> Result<(), ParseErrors> {
        let parsed = EdgeRulesEngine::parse_code(code)?;

        match parsed {
            // @Todo: object field must be normal expression wrapped in object
            ParsedItem::Expression(ObjectField(field, field_expression)) => {
                self.ast_root
                    .add_expression(field.as_str(), *field_expression);
            }
            ParsedItem::Expression(ExpressionEnum::StaticObject(context_object)) => {
                self.ast_root.append(context_object);
            }
            ParsedItem::Definition(definition) => {
                self.ast_root.add_definition(definition);
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

    pub fn to_runtime(self) -> Result<EdgeRulesRuntime, LinkingError> {
        let static_context = self.ast_root.build();

        linker::link_parts(Rc::clone(&static_context))?;

        Ok(EdgeRulesRuntime::new(static_context))
    }

    pub fn evaluate_all(mut self, code: &str) -> String {
        match self.load_source(code) {
            Ok(_service) => match self.to_runtime() {
                Ok(runtime) => match runtime.eval_all() {
                    Ok(()) => runtime.context.borrow().to_code(),
                    Err(error) => error.to_string(),
                },
                Err(error) => error.to_string(),
            },
            Err(error) => to_display(error.errors(), "\n"),
        }
    }

    /// Evaluates a field/path using the currently loaded source.
    /// Usage: create EdgeRules, load_source(...), then evaluate_field("a.b.c").
    pub fn evaluate_field(&mut self, field: &str) -> String {
        // Build a non-consuming runtime snapshot to preserve current builder state
        let current_builder = std::mem::replace(&mut self.ast_root, ContextObjectBuilder::new());
        let static_context = current_builder.build();

        let result = match linker::link_parts(Rc::clone(&static_context)) {
            Ok(()) => {
                let runtime = EdgeRulesRuntime::new(Rc::clone(&static_context));
                match runtime.evaluate_field(field) {
                    Ok(v) => v.to_string(),
                    Err(e) => e.to_string(),
                }
            }
            Err(e) => e.to_string(),
        };

        // Restore the builder state
        self.ast_root.append(static_context);

        result
    }

    //
    // Evaluates a single expression that can use the loaded code context.
    // The loaded code is not modified, the expression is evaluated in a temporary context
    // that contains the loaded code snapshot.
    //
    // This is useful for REPL or interactive evaluation of expressions that depend on the
    // loaded code context.
    //
    // @Todo: optimize to avoid building and linking the static context on every call.
    //
    pub fn evaluate_expression(&mut self, code: &str) -> Result<ValueEnum, EvalError> {
        // 1) Detach current builder and build a static snapshot of the loaded code
        let current_builder = std::mem::replace(&mut self.ast_root, ContextObjectBuilder::new());
        let static_context = current_builder.build();
        linker::link_parts(Rc::clone(&static_context))?;

        // 2) Create a temporary service, append current snapshot, then append dummy via load_source
        let mut temp_service = EdgeRules {
            ast_root: ContextObjectBuilder::new(),
        };
        temp_service.ast_root.append(Rc::clone(&static_context));
        // Note: '_' is not a valid identifier in the current grammar, use a safe dummy name
        let dummy_name = "tmp000001";
        temp_service.load_source(format!("{} : {}", dummy_name, code).as_str())?;

        // 3) Build runtime and evaluate the dummy variable
        let runtime = temp_service.to_runtime()?;
        let result = runtime.evaluate_field(dummy_name)?;

        // 4) Restore original builder (without the dummy), destroying temp state
        self.ast_root.append(static_context);

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
        let mut variable = ExpressionEnum::variable(name);
        variable.link(Rc::clone(&self.static_tree))?;
        variable.eval(Rc::clone(&self.context))
    }

    /**
     * Evaluates all expressions in the context tree, starting from the root context
     */
    pub fn eval_all(&self) -> Result<(), RuntimeError> {
        Self::eval_all_context(Rc::clone(&self.context))
    }

    fn eval_all_context(ctx: Rc<RefCell<ExecutionContext>>) -> Result<(), RuntimeError> {
        if ctx.borrow().promise_eval_all {
            return Ok(());
        }

        ctx.borrow_mut().promise_eval_all = true;

        let field_names = ctx.borrow().object.borrow().get_field_names();

        trace!(
            "eval_all_context: {}(..) for {:?}",
            ctx.borrow().node().node_type,
            field_names
        );

        for name in field_names {
            let name_str = name.as_str();

            match ctx.borrow().get(name_str)? {
                EObjectContent::ExpressionRef(expression) => {
                    ctx.borrow().node().lock_field(name_str)?;
                    let value = expression.borrow().expression.eval(Rc::clone(&ctx));
                    ctx.borrow().stack_insert(name_str.to_string(), value);
                    ctx.borrow().node().unlock_field(name_str);
                }
                EObjectContent::ObjectRef(reference) => {
                    NodeData::attach_child(&ctx, &reference);
                    Self::eval_all_context(Rc::clone(&reference))?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Utilities
//--------------------------------------------------------------------------------------------------

pub fn expr(code: &str) -> Result<ExpressionEnum, EvalError> {
    match EdgeRulesEngine::parse_code(code)? {
        ParsedItem::Expression(expression) => Ok(expression),
        other => Err(other.into_error()),
    }
}

//--------------------------------------------------------------------------------------------------
// Test
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
pub mod test {
    use crate::ast::token::{EToken, EUnparsedToken};
    use crate::runtime::edge_rules::{EdgeRules, EvalError};

    use crate::typesystem::errors::LinkingErrorEnum;
    use crate::typesystem::errors::ParseErrorEnum::{UnexpectedToken, UnknownError};

    use crate::typesystem::types::number::NumberEnum::Int;

    use crate::typesystem::values::ValueEnum;
    use crate::utils::test::{init_logger, test_code, test_code_lines};

    #[test]
    fn now() -> Result<(), EvalError> {
        init_logger();

        Ok(())
    }

    #[test]
    fn test_service() -> Result<(), EvalError> {
        init_logger();

        {
            let mut service = EdgeRules::new();
            service.load_source("value: 2 + 2")?;
            service.load_source("value: 2 + 3")?;
            match service.to_runtime() {
                Ok(runtime) => {
                    let result = runtime.evaluate_field("value")?;
                    assert_eq!(result, ValueEnum::NumberValue(Int(5)));
                }
                Err(error) => {
                    panic!("Failed to link: {:?}", error);
                }
            }
        }

        test_code("value").expect_parse_error(UnexpectedToken(
            Box::new(EToken::Unparsed(EUnparsedToken::Comma)),
            None,
        ));
        test_code("value: 2 + 2").expect_num("value", Int(4));
        test_code("value: 2 + ").expect_parse_error(UnknownError("any".to_string()));
        test_code("{ value: 2 + 2 }").expect_num("value", Int(4));
        test_code("{ v1 : 100; value: v1 + v1 }").expect_num("value", Int(200));

        Ok(())
    }

    #[test]
    fn test_service_evaluate_field_with_existing_state() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRules::new();
        service.load_source("{ value: 3 }")?;
        let result = service.evaluate_field("value");
        assert_eq!(result, "3");

        service.load_source("value: 2 + 2")?;
        let out1 = service.evaluate_field("value");
        assert_eq!(out1, "4");

        service.load_source("value: 2 + 3")?;
        let out2 = service.evaluate_field("value");
        assert_eq!(out2, "5");

        Ok(())
    }

    #[test]
    fn test_service_evaluate_field_with_path_depth() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRules::new();
        service.load_source(
            "{ calendar : { config : { start : 7 }; sub : { inner : { value : 42 } } } }",
        )?;

        let out1 = service.evaluate_field("calendar.config.start");
        assert_eq!(out1, "7");

        let out2 = service.evaluate_field("calendar.sub.inner.value");
        assert_eq!(out2, "42");

        service.load_source("{ calendar: { config: { start: 7; end: start + 5 } } }")?;

        // Evaluate a field/path from the loaded model
        let start = service.evaluate_field("calendar.config.start");
        assert_eq!(start, "7");

        let end = service.evaluate_field("calendar.config.end");
        assert_eq!(end, "12");

        Ok(())
    }

    #[test]
    fn test_evaluate_expression_with_loaded_context() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRules::new();
        service.load_source("{ value: 3 }")?;

        let result = service.evaluate_expression("2 + value")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(5)));

        Ok(())
    }

    #[test]
    fn test_evaluate_pure_expression_without_context() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRules::new();
        let result = service.evaluate_expression("2 + 3")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(5)));

        let result = service.evaluate_expression("sum(1,2,3)")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(6)));

        Ok(())
    }

    #[test]
    fn test_evaluate_expression_unknown_variable_fails() {
        init_logger();

        let mut service = EdgeRules::new();
        let err = service.evaluate_expression("x + 1").unwrap_err();
        match err {
            EvalError::FailedExecution(_e) => {}
            other => panic!("Expected runtime error, got: {:?}", other),
        }
    }

    #[test]
    fn test_evaluate_expression_with_function_indirect() -> Result<(), EvalError> {
        init_logger();

        let mut service = EdgeRules::new();
        service.load_source("{ f(a) : { result : a + 1 }; tmp : f(2).result }")?;
        let result = service.evaluate_expression("tmp")?;
        assert_eq!(result, ValueEnum::NumberValue(Int(3)));

        Ok(())
    }

    #[test]
    fn test_linking() -> Result<(), EvalError> {
        init_logger();

        test_code("{ a : 1; b : a  }")
            .expect_type("Type<a: number, b: number>")
            .expect_num("a", Int(1));

        test_code("{ a : z; b : a; z : 8 * 2  }")
            .expect_type("Type<a: number, b: number, z: number>")
            .expect_num("a", Int(16));

        test_code("{ a : {x : 1}; b : a.x }")
            .expect_type("Type<a: Type<x: number>, b: number>")
            .expect_num("b", Int(1));

        test_code("{ c : b; a : {x : 1}; b : a.x }")
            .expect_type("Type<a: Type<x: number>, b: number, c: number>")
            .expect_num("c", Int(1));

        // roundtrip test
        test_code("{ c : b; a : {x : 1; aa: b}; b : a.x }")
            .expect_type("Type<a: Type<x: number, aa: number>, b: number, c: number>")
            .expect_num("c", Int(1));

        // messy handover test
        test_code("{ c : b; a : {x : {y : 1}}; b : a.x; d : c.y }")
            .expect_type("Type<a: Type<x: Type<y: number>>, b: Type<y: number>, c: Type<y: number>, d: number>")
            .expect_num("d", Int(1));

        // deep roundtrip test
        test_code("{ c : b; a : {x : {x : 1; aa: b}}; b : a.x.x }")
            .expect_type("Type<a: Type<x: Type<x: number, aa: number>>, b: number, c: number>")
            .expect_num("c", Int(1));

        test_code("{ f(arg1) :  { a : arg1 } }").expect_type("Type<>");

        test_code("{ f(arg1) :  { a : arg1 }; b : 1 }")
            .expect_type("Type<b: number>")
            .expect_num("b", Int(1));

        test_code("{ f(arg1) :  { a : arg1 }; b : f(1) }").expect_type("Type<b: Type<a: number>>");

        test_code("{ f(arg1) :  { a : arg1 }; b : f(1).a }")
            .expect_type("Type<b: number>")
            .expect_num("b", Int(1));

        // possibility to call a function from a sub-context
        test_code_lines(&[
            "func1(a) : { result : a }",
            "subContext : {",
            "subResult : func1(35).result",
            "}",
            "value : subContext.subResult",
        ])
        .expect_num("value", Int(35));

        // argument as a parameter works well
        test_code_lines(&[
            "myInput : 35",
            "func1(a) : { result : a }",
            "subContext : {",
            "subResult : func1(myInput).result",
            "}",
            "value : subContext.subResult",
        ])
        .expect_num("value", Int(35));

        Ok(())
    }

    #[test]
    fn calendar_self_reference_in_array_elements() -> Result<(), EvalError> {
        init_logger();

        let tb = test_code_lines(&[
            "calendar : {",
            "    shift : 2",
            "    days : [",
            "        { start : calendar.shift + 1 },",
            "        { start : calendar.shift + 31 }",
            "    ]",
            "    firstDay : days[0].start",
            "    secondDay : days[1].start",
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
            "calendar : {",
            "    shift : 2",
            "    start1(calendar) : { result : calendar.shift + 1 }",
            "    firstDay : start1(calendar).result",
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
            "calendar : {",
            "    shift : 2",
            "    start2(x, cal) : { result : cal.shift + x }",
            "    firstDay : start2(1, calendar).result",
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
            "calendar : {",
            "    config : { shift : 2 }",
            "    start1(calendar) : { result : calendar.shift + 1 }",
            "    firstDay : start1(config).result",
            "}",
        ]);
        tb.expect_num("calendar.firstDay", Int(3));

        Ok(())
    }
}
