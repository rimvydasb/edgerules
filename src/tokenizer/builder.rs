use crate::ast::token::EToken;
use crate::ast::token::EToken::Expression;
use crate::ast::token::EToken::Unparsed;
use crate::ast::token::EUnparsedToken::Literal;
use crate::ast::token::ExpressionEnum::{Value, Variable};
use crate::tokenizer::utils::TokenChain;
use crate::typesystem::errors::ParseErrorEnum;
use crate::typesystem::values::ValueEnum;
use crate::utils::to_string;
use log::{error, trace};
use std::collections::vec_deque::VecDeque;
use std::fmt;

type FactoryFunction = fn(
    left: &mut TokenChain,
    token: EToken,
    right: &mut TokenChain,
) -> Result<EToken, ParseErrorEnum>;

//----------------------------------------------------------------------------------------------

#[derive(Clone)]
// 1 - position starting from 0
pub struct ItemBuildTask {
    pub position: usize,
    pub priority: u32,
    pub factory: FactoryFunction,
}

impl fmt::Display for ItemBuildTask {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "({}:{})", self.position, self.priority)
    }
}

//----------------------------------------------------------------------------------------------

fn calc_level(priority: u32, ctx: u32) -> u32 {
    ctx * 100 + priority
}

pub struct ASTBuilder {
    result: TokenChain,
    last_priority_list: VecDeque<ItemBuildTask>,
    current_level: u32,
}

impl ASTBuilder {
    pub fn new() -> ASTBuilder {
        ASTBuilder {
            result: TokenChain::new(),
            last_priority_list: VecDeque::new(),
            current_level: 0,
        }
    }

    pub fn merge(&mut self) {
        self.merge_level(calc_level(0, self.current_level));
    }

    pub fn merge_level(&mut self, current_level: u32) {
        trace!(
            "merge_left_if_can. lastPriorityList={}",
            to_string(&mut self.last_priority_list.clone())
        );

        while let Some(check_build_task) = self.last_priority_list.back() {
            // eliminate bigger priority items from the left side
            if check_build_task.priority >= current_level {
                trace!(
                    "merge this: '{:?}' at '{:?}' lvl={}",
                    self.result[check_build_task.position],
                    check_build_task.position,
                    current_level
                );

                if let Some(build_task) = self.last_priority_list.pop_back() {
                    trace!(
                        "full = {:?} --- split at {}",
                        self.result,
                        build_task.position
                    );

                    let mut right = TokenChain(self.result.split_off(build_task.position));

                    trace!("left = {:?}", self.result);

                    if let Some(token) = right.pop_front() {
                        trace!("mid = {:?}", token);
                        trace!("right = {:?}", right);

                        let build_result =
                            (build_task.factory)(&mut self.result, token, &mut right);

                        match build_result {
                            Ok(token) => self.result.push_back(token),

                            // @Todo: errors are pushed back in chain instead of proper error stacking and throwing. Need to work with it.
                            Err(error) => {
                                error!("Push back: {}", error);
                                self.result.push_back(EToken::ParseError(error))
                            }
                        }

                        self.result.append(&mut right);
                    }
                }
            } else {
                if let Some(token) = self.result.get(check_build_task.position) {
                    trace!(
                        "no merge: position={} item={} lvl={} lastPriorityList={}",
                        check_build_task.position,
                        token,
                        current_level,
                        to_string(&mut self.last_priority_list.clone())
                    );
                } else {
                    trace!(
                        "no merge: position={} left={:?}",
                        check_build_task.position,
                        self.result
                    );
                }

                break;
            }
        }
    }

    pub fn last_variable(&mut self) -> Option<String> {
        if let Some(Expression(Variable(_))) = self.result.back() {
            if let Some(Expression(Variable(link))) = self.result.pop_back() {
                return Some(link.get_name());
            }
        }

        None
    }

    pub fn last_token(&mut self) -> Option<&EToken> {
        self.result.back()
    }

    /// If the last token is an unparsed literal equal to `expected`,
    /// remove it and return true. Otherwise, return false and leave the chain intact.
    pub fn pop_literal_if(&mut self, expected: &str) -> bool {
        if let Some(Unparsed(Literal(maybe))) = self.result.back() {
            if maybe == expected {
                // Safe unwrap; just checked it's Some
                let _ = self.result.pop_back();
                return true;
            }
        }
        false
    }

    pub fn push_node(&mut self, priority: u32, token: EToken, factory: FactoryFunction) {
        let lvl = calc_level(priority, self.current_level);

        self.merge_level(lvl);

        self.last_priority_list.push_back(ItemBuildTask {
            position: self.result.len(),
            priority: lvl,
            factory,
        });

        self.push_element(token);
    }

    pub fn finalize(mut self) -> TokenChain {
        trace!(
            "Finalizing. lastPriorityList={}",
            to_string(&mut self.last_priority_list.clone())
        );

        // @TODO need to make sure that infinity loop will not occur
        while self.last_priority_list.back().is_some() {
            trace!("still merging all");
            self.merge_level(0);
        }

        self.result
    }

    pub fn push_value(&mut self, value: ValueEnum) {
        self.result.push_back(Expression(Value(value)));
    }

    // pub fn push_expression(&mut self, expression: EExpression) {
    //     self.result.push_back(Expression(expression));
    // }

    // @Todo: deprecate
    pub fn push_element(&mut self, token: EToken) {
        self.result.push_back(token);
    }

    pub fn incl_level(&mut self) {
        self.current_level += 1;
    }

    pub fn dec_level(&mut self) {
        self.current_level -= 1;
    }
}

//--------------------------------------------------------------------------------------------------
// Factory
//--------------------------------------------------------------------------------------------------

pub mod factory {
    use crate::ast::annotations::AnnotationEnum;
    use crate::ast::context::context_object_builder::ContextObjectBuilder;
    use crate::ast::context::context_object_type::FormalParameter;
    use crate::ast::foreach::ForFunction;
    use crate::ast::functions::function_types::{
        BinaryFunction, MultiFunction, UnaryFunction, BINARY_BUILT_IN_FUNCTIONS,
        BUILT_IN_ALL_FUNCTIONS, MULTI_BUILT_IN_FUNCTIONS, UNARY_BUILT_IN_FUNCTIONS,
    };
    use crate::ast::ifthenelse::IfThenElseFunction;
    use crate::ast::metaphors::decision_tables::DecisionTable;
    use crate::ast::metaphors::functions::FunctionDefinition;
    use crate::ast::operators::comparators::{ComparatorEnum, ComparatorOperator};
    use crate::ast::operators::logical_operators::{LogicalOperator, LogicalOperatorEnum};
    use crate::ast::operators::math_operators::{MathOperator, MathOperatorEnum, NegationOperator};
    use crate::ast::selections::{ExpressionFilter, FieldSelection};
    use crate::ast::sequence::CollectionExpression;
    // use crate::ast::token::DefinitionEnum::Metaphor as MetaphorDef;
    use crate::ast::token::EToken;
    use crate::ast::token::EToken::*;
    use crate::ast::token::EUnparsedToken::*;
    use crate::ast::token::ExpressionEnum::*;
    use crate::ast::token::*;
    use crate::ast::user_function_call::UserFunctionCall;
    use crate::tokenizer::parser::parse_type;
    use crate::tokenizer::utils::*;
    use crate::typesystem::errors::ParseErrorEnum;
    use crate::typesystem::errors::ParseErrorEnum::{
        FunctionWrongNumberOfArguments, UnknownError, UnknownParseError,
    };
    use crate::typesystem::types::ValueType;
    use log::trace;
    use std::collections::vec_deque::VecDeque;

    fn pop_back_as_expected(deque: &mut VecDeque<EToken>, expected: &str) -> bool {
        if let Some(Unparsed(Literal(maybe))) = deque.pop_back() {
            if maybe.eq(expected) {
                return true;
            }
        }

        false
    }

    pub fn build_assignment(
        left: &mut TokenChain,
        _token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let left_token = left.pop_left().map_err(|err| {
            UnknownError("Left assignment side is not complete".to_string()).before(err)
        })?;

        let right_token = right.pop_right().map_err(|err| {
            UnknownError(format!("'{}' assignment side is not complete", left_token)).before(err)
        })?;

        // Detect if this is a `type Alias : ...` statement by checking the token immediately preceding the name
        let is_type_stmt = matches!(
            left.back(),
            Some(Unparsed(Literal(ref s))) if s == "type"
        );

        match (left_token, right_token) {
            // Type alias: type Alias : <Type>
            (Expression(Variable(link)), Unparsed(TypeReferenceLiteral(tref))) if is_type_stmt => {
                let _ = left.pop_left_as_expected("type");
                Ok(Definition(DefinitionEnum::UserType(UserTypeDefinition {
                    name: link.get_name(),
                    body: UserTypeBody::TypeRef(tref),
                })))
            }
            // Typed placeholder: field : <Type>
            (Expression(Variable(link)), Unparsed(TypeReferenceLiteral(tref))) => Ok(Expression(
                ObjectField(link.get_name(), Box::new(TypePlaceholder(tref))),
            )),
            // Type alias with object body: type Alias : { ... }
            (Expression(Variable(link)), Expression(StaticObject(object))) if is_type_stmt => {
                let _ = left.pop_left_as_expected("type");

                // Enforce: no functions or typed placeholders inside type definitions; only nested type objects
                {
                    let obj_ref = object.borrow();
                    if !obj_ref.metaphors.is_empty() {
                        return Err(UnknownError(
                            "Type definition cannot contain function definitions".to_string(),
                        ));
                    }
                    for (fname, entry) in obj_ref.expressions.iter() {
                        let expr = &entry.borrow().expression;
                        match expr {
                            StaticObject(_) => { /* ok: nested type object */ }
                            TypePlaceholder(_) => { /* ok: typed field in type body */ }
                            _ => {
                                return Err(UnknownError(format!(
                                    "Type definition contains non-type field '{}'",
                                    fname
                                )));
                            }
                        }
                    }
                }

                Ok(Definition(DefinitionEnum::UserType(UserTypeDefinition {
                    name: link.get_name(),
                    body: UserTypeBody::TypeObject(object),
                })))
            }
            (Expression(Variable(link)), Expression(right)) => {
                Ok(Expression(ObjectField(link.get_name(), Box::new(right))))
            }
            (
                Unparsed(FunctionDefinitionLiteral(annotations, function_name, arguments)),
                Expression(StaticObject(object)),
            ) => {
                // let plain = SimpleObject::try_unwrap(object)
                //     .map_err(|_err| UnknownError(format!("'{}' failed to construct", function_name)))?;

                let function =
                    FunctionDefinition::build(annotations, function_name, arguments, object)?;
                Ok(Definition(DefinitionEnum::Metaphor(function.into())))
            }
            (
                Unparsed(FunctionDefinitionLiteral(annotations, function_name, _arguments)),
                Expression(Collection(_rows)),
            ) => {
                if AnnotationEnum::get_decision_table(&annotations).is_some() {
                    let decision_table =
                        DecisionTable::build(annotations, function_name, _arguments, _rows)?;
                    Ok(Definition(DefinitionEnum::Metaphor(decision_table.into())))
                } else {
                    Err(UnknownError(format!(
                        "function '{}' body is a collection. Must be a structure",
                        function_name
                    )))
                }
            }
            (Unparsed(FunctionDefinitionLiteral(_annotations, name, _)), _) => Err(UnknownError(
                format!("function '{}' body is not defined", name),
            )),
            (unexpected, Expression(_right)) => Err(UnknownError(format!(
                "'{}' cannot be a variable name",
                unexpected
            ))),
            (a, b) => Err(UnknownError(format!(
                "'{}' is not proper variable name to assign '{}'",
                a, b
            ))),
        }
    }

    pub fn build_context(
        _left: &mut TokenChain,
        _token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        // @Todo: need to check level before adding and stop adding if smaller
        let mut obj = ContextObjectBuilder::new();

        while let Some(right_token) = right.pop_front() {
            match right_token {
                Definition(definition) => {
                    obj.add_definition(definition)?;
                }
                Expression(ObjectField(field_name, expression)) => {
                    // if let Object(_) = &mut *expression {
                    //     if obj.object_type == ValueType::AnyType {
                    //         let type_name = capitalize(field_name.clone()).add("Type");
                    //         obj.object_type = ValueType::ObjectType(type_name)
                    //     }
                    // }
                    obj.add_expression(field_name.as_str(), *expression)?;
                }

                // @Todo: need to accumulate errors instead of just returning - same applies for an array
                Unparsed(unparsed) => {
                    return Err(UnknownError(format!(
                        "'{}' is not a proper context element",
                        unparsed
                    )));
                }
                ParseError(error) => {
                    return Err(error);
                }
                _ => {
                    return Err(UnknownError(format!(
                        "'{}' is not a proper object field",
                        right_token
                    )));
                }
            }
        }

        Ok(Expression(StaticObject(obj.build())))
    }

    pub fn build_function_call(
        _left: &mut TokenChain,
        function_name: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let name_result = function_name.into_string_or_literal()?;
        let name = name_result.as_str();
        let mut expressions = right.drain_expressions()?;

        if expressions.len() == 1 {
            if let Some(function) = UNARY_BUILT_IN_FUNCTIONS.get(name) {
                let expression = expressions.pop().unwrap();
                return Ok(Expression(
                    UnaryFunction::build(function.clone(), expression).into(),
                ));
            }
        } else if expressions.len() == 2 {
            if let Some(function) = BINARY_BUILT_IN_FUNCTIONS.get(name) {
                let right_expression = expressions.pop().unwrap();
                let left_expression = expressions.pop().unwrap();
                return Ok(Expression(
                    BinaryFunction::build(function.clone(), left_expression, right_expression)
                        .into(),
                ));
            }
        }

        if !expressions.is_empty() {
            if let Some(function) = MULTI_BUILT_IN_FUNCTIONS.get(name) {
                return Ok(Expression(
                    MultiFunction::build(function.clone(), expressions).into(),
                ));
            }
        }

        match BUILT_IN_ALL_FUNCTIONS.get(name) {
            None => {
                if !expressions.is_empty() {
                    Ok(Expression(FunctionCall(Box::new(UserFunctionCall::new(
                        name.to_string(),
                        expressions,
                    )))))
                } else {
                    Err(UnknownError(format!(
                        "{} function does not have any arguments",
                        name
                    )))
                }
            }
            Some(finding) => Err(FunctionWrongNumberOfArguments(
                name.to_string(),
                finding.clone(),
                expressions.len(),
            )),
        }
    }

    pub fn build_cast(
        left: &mut TokenChain,
        _token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let left_expr = left.pop_left_expression().map_err(|err| {
            UnknownError("Left 'as' side is not complete".to_string()).before(err)
        })?;
        let right_token = right.pop_right().map_err(|err| {
            UnknownError("Type after 'as' is not complete".to_string()).before(err)
        })?;

        match right_token {
            Unparsed(TypeReferenceLiteral(tref)) => Ok(Expression(FunctionCall(Box::new(
                crate::ast::expression::CastCall::new(left_expr, tref),
            )))),
            _ => Err(UnknownError("Invalid type after 'as'".to_string())),
        }
    }

    // create tokens chain
    pub fn build_function_definition(
        left: &mut TokenChain,
        token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let mut arguments = Vec::new();
        let mut annotations = Vec::new();

        while let Some(left_token) = left.pop_back() {
            match left_token {
                Unparsed(Annotation(annotation)) => {
                    annotations.push(annotation);
                }
                _ => {
                    left.push_back(left_token);
                    break;
                }
            }
        }

        while let Some(right_token) = right.pop_front() {
            match right_token {
                Unparsed(Comma) => {
                    if arguments.is_empty() {
                        return Err(UnknownError(
                            "Very first function argument is missing".to_string(),
                        ));
                    }
                }
                ParseError(err) => return Err(err),
                Expression(expression) => {
                    let parameter = parse_function_parameter(expression)?;
                    arguments.push(parameter);
                }
                other => {
                    return Err(UnknownError(format!(
                        "Unsupported token `{}` in function parameter list",
                        other
                    )));
                }
            }
        }

        Ok(Unparsed(FunctionDefinitionLiteral(
            annotations,
            token.into_string_or_literal()?,
            arguments,
        )))
    }

    fn parse_function_parameter(
        expression: ExpressionEnum,
    ) -> Result<FormalParameter, ParseErrorEnum> {
        match expression {
            Variable(variable) => {
                if variable.path.len() != 1 {
                    return Err(UnknownError(format!(
                        "Function parameter must be a simple identifier, got `{}`",
                        variable
                    )));
                }

                Ok(FormalParameter::with_type_ref(variable.get_name(), None))
            }
            ObjectField(name, boxed_expression) => {
                let annotation = extract_type_annotation(*boxed_expression)?;
                Ok(FormalParameter::with_type_ref(name, annotation))
            }
            _ => Err(UnknownError(format!(
                "Unsupported expression `{}` in function parameter list",
                expression
            ))),
        }
    }

    fn extract_type_annotation(
        expression: ExpressionEnum,
    ) -> Result<Option<ComplexTypeRef>, ParseErrorEnum> {
        match expression {
            TypePlaceholder(tref) => Ok(Some(tref)),
            Variable(variable) => {
                if variable.path.len() != 1 {
                    return Err(UnknownError(format!(
                        "Type annotation must be a simple identifier, got `{}`",
                        variable
                    )));
                }

                let type_name = variable.get_name();
                Ok(Some(parse_type(&type_name)))
            }
            Value(_) => Err(UnknownError(
                "Default values for function parameters are not supported".to_string(),
            )),
            other => Err(UnknownError(format!(
                "Unsupported type annotation expression `{}`",
                other
            ))),
        }
    }

    pub fn build_sequence(
        _left: &mut TokenChain,
        _token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let mut args: Vec<ExpressionEnum> = Vec::new();

        // Todo: need to check level before adding and stop adding if smaller, but for some reasons no errors can be reproduced.
        while let Some(right_token) = right.pop_front() {
            match right_token {
                Expression(expression) => args.push(expression),
                Unparsed(Comma) => {
                    if args.is_empty() {
                        right.clear(); // forgets all possible other errors
                        return Err(UnknownError(
                            "Very first sequence element is missing".to_string(),
                        ));
                    }
                }
                ParseError(error) => {
                    right.clear(); // forgets all possible other errors
                    return Err(error);
                }
                Unparsed(_) => {
                    right.clear(); // forgets all possible other errors
                    return Err(UnknownError(format!(
                        "'{}' is not a proper sequence element",
                        right_token
                    )));
                }
                Definition(_) => {
                    right.clear(); // forgets all possible other errors
                    return Err(UnknownError(
                        "Function definition is not allowed in sequence".to_string(),
                    ));
                }
            }
        }

        if args.is_empty() {
            return Err(UnknownError(
                "Function definition is not allowed in sequence".to_string(),
            ));
        }

        Ok(Expression(Collection(CollectionExpression::build(args))))
    }

    pub fn build_filter(
        left: &mut TokenChain,
        token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        // application.applicants[0].age
        // left-----------------^   ^------right

        let left_token = left
            .pop_left()
            .map_err(|err| UnknownError("Filter not applicable".to_string()).before(err))?;

        let right_token = right.pop_right().map_err(|err| {
            UnknownError(format!("Filter '{}' not applicable", left_token)).before(err)
        })?;

        match (left_token, right_token) {
            (Expression(left_expression), Expression(right_expression)) => {
                match ExpressionFilter::build(left_expression, right_expression) {
                    Ok(selection) => Ok(Expression(Filter(Box::new(selection)))),
                    Err(error) => Err(error),
                }
            }
            (_left_unknown, _right_unknown) => {
                Err(UnknownError(format!("Filter not completed '{}'", token)))
            }
        }
    }

    pub fn build_range(
        left: &mut TokenChain,
        token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        // number in 1..5
        // left------^  ^------right

        let left_token = left
            .pop_left()
            .map_err(|err| UnknownError("Range not applicable".to_string()).before(err))?;

        let right_token = right.pop_right().map_err(|err| {
            UnknownError(format!("Range '{}' not applicable", left_token)).before(err)
        })?;

        match (left_token, right_token) {
            (Expression(left_expression), Expression(right_expression)) => Ok(Expression(
                RangeExpression(Box::new(left_expression), Box::new(right_expression)),
            )),
            (_left_unknown, _right_unknown) => Err(UnknownParseError(format!(
                "Range not completed '{}'",
                token
            ))),
        }
    }

    pub fn build_field_selection(
        left: &mut TokenChain,
        token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        // application.applicants[0].age
        // left-----------------^   ^------right

        let left_token = left
            .pop_left()
            .map_err(|_| UnknownError("Field not applicable".to_string()))?;
        let right_token = right
            .pop_right()
            .map_err(|_| UnknownError(format!("Field '{}' not applicable", left_token)))?;

        match (left_token, right_token) {
            (Expression(left_expression), Expression(right_expression)) => {
                match FieldSelection::build(left_expression, right_expression) {
                    Ok(selection) => Ok(Expression(Selection(Box::new(selection)))),
                    Err(error) => Err(error),
                }
            }
            (_left_unknown, _right_unknown) => Err(UnknownParseError(format!(
                "Selection not completed '{}'",
                token
            ))),
        }
    }

    pub fn build_if_then_else(
        left: &mut TokenChain,
        _token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        // ... if ... then ... else ...
        // left---------------^    ^-----------------right

        let then_content = left
            .pop_left_expression()
            .map_err(|err| UnknownError("Error in then... part".to_string()).before(err))?;

        let _then = left.pop_left_as_expected("then")?;

        let if_condition = left
            .pop_left_expression()
            .map_err(|err| UnknownError("Error in if... part".to_string()).before(err))?;

        let _if_part = left.pop_left_as_expected("if")?;

        let else_content = right
            .pop_right_expression()
            .map_err(|err| UnknownError("Error in else... part".to_string()).before(err))?;

        let func = IfThenElseFunction::build(if_condition, then_content, else_content)?;

        Ok(Expression(FunctionCall(Box::new(func))))
    }

    pub fn build_for_each_return(
        left: &mut TokenChain,
        _token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        // ... for in_loop_variable in in_expression return return_expression
        // left-------------------------------------^       ^-----------------right

        let new_token: EToken = if let Some(Expression(return_expression)) = right.pop_front() {
            if let Some(Expression(in_expression)) = left.pop_back() {
                if pop_back_as_expected(left, "in") {
                    if let Some(Expression(in_loop_variable)) = left.pop_back() {
                        if pop_back_as_expected(left, "for") {
                            Expression(FunctionCall(Box::new(ForFunction::new(
                                format!("{}", in_loop_variable),
                                in_expression,
                                return_expression,
                            )?)))
                        } else {
                            return Err(UnknownParseError("??? ... in ... return ...".to_string()));
                        }
                    } else {
                        return Err(UnknownParseError("for [???] in ... return ...".to_string()));
                    }
                } else {
                    return Err(UnknownParseError(
                        "for ... [in?] ... return ...".to_string(),
                    ));
                }
            } else {
                return Err(UnknownParseError("for ... in [???] return ...".to_string()));
            }
        } else {
            return Err(UnknownParseError("for ... in ... return [???]".to_string()));
        };

        Ok(new_token)
    }

    pub fn build_any_operator(
        left: &mut TokenChain,
        token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let op = MathOperatorEnum::try_from(token)?;
        let left_token = left.pop_left().map_err(|err| {
            UnknownError(format!("Left '{}' operator side is not complete", op)).before(err)
        })?;
        let right_token = right.pop_right().map_err(|err| {
            UnknownError(format!("{} {} - not completed", left_token, op)).before(err)
        })?;

        match (left_token, right_token) {
            (Expression(_left), Expression(_right)) => Ok(Expression(Operator(Box::new(
                MathOperator::build(op, _left, _right)?,
            )))),
            (Unparsed(_left), Expression(_right)) => {
                // @Todo: that's absolutely not clear
                left.push_back(Unparsed(_left));
                if op == MathOperatorEnum::Subtraction {
                    Ok(Expression(FunctionCall(Box::new(NegationOperator::new(
                        _right,
                    )))))
                } else {
                    Err(UnknownError(format!("Not completed '{}'", op)))
                }
            }
            (_left, _right) => {
                trace!("left={:?} right={:?}", _left, _right);
                Err(UnknownError(format!("Not completed '{}'", op)))
            }
        }
    }

    pub fn build_comparator(
        left: &mut TokenChain,
        token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let comparator = ComparatorEnum::try_from(token.into_string_or_literal()?.as_str())?;

        let left_token = left.pop_left().map_err(|err| {
            UnknownError(format!(
                "Left '{}' comparator side is not complete",
                comparator
            ))
            .before(err)
        })?;

        let right_token = right.pop_right().map_err(|err| {
            UnknownError(format!("{} {} - not completed", left_token, comparator)).before(err)
        })?;

        match (left_token, right_token) {
            (Expression(left_token), Expression(right_token)) => {
                let comparator_operator =
                    ComparatorOperator::build(comparator, left_token, right_token)?;
                Ok(Expression(Operator(Box::new(comparator_operator))))
            }
            (Unparsed(BracketOpen), Expression(right_token)) => {
                left.push_back(Unparsed(BracketOpen));
                let comparator_operator =
                    ComparatorOperator::build(comparator, ContextVariable, right_token)?;
                Ok(Expression(Operator(Box::new(comparator_operator))))
            }
            (_left, _right) => Err(UnknownError(format!("Not completed '{}'", comparator))),
        }
    }

    pub fn build_logical_operator(
        left: &mut TokenChain,
        token: EToken,
        right: &mut TokenChain,
    ) -> Result<EToken, ParseErrorEnum> {
        let operator = LogicalOperatorEnum::try_from(token.into_string_or_literal()?.as_str())?;

        // Support unary prefix: `not <expr>`
        if let LogicalOperatorEnum::Not = operator {
            let right_token = right.pop_right().map_err(|err| {
                UnknownError("'not' right side is not complete".to_string()).before(err)
            })?;

            match right_token {
                Expression(right_expr) => {
                    let function = LogicalOperator::build(
                        operator,
                        right_expr,
                        // placeholder right operand, ignored by Not
                        ExpressionEnum::from(true),
                    )?;
                    return Ok(Expression(Operator(Box::new(function))));
                }
                _ => return Err(UnknownError("Not completed 'not'".to_string())),
            }
        }

        // Binary logical operators: and, or, xor
        let left_token = left.pop_left().map_err(|err| {
            UnknownError(format!("Left '{}' operator side is not complete", operator)).before(err)
        })?;

        let right_token = right.pop_right().map_err(|err| {
            UnknownError(format!("{} {} - not completed", left_token, operator)).before(err)
        })?;

        match (left_token, right_token) {
            (Expression(left_token), Expression(right_token)) => {
                let function = LogicalOperator::build(operator, left_token, right_token)?;
                Ok(Expression(Operator(Box::new(function))))
            }
            (_left, _right) => Err(UnknownError(format!("Not completed '{}'", operator))),
        }
    }
}
