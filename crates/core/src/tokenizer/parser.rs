use std::collections::vec_deque::VecDeque;

use crate::ast::operators::comparators::ComparatorEnum;
use crate::ast::operators::logical_operators::LogicalOperatorEnum;
use crate::ast::operators::math_operators::MathOperatorEnum;
use crate::ast::token::EPriorities::*;
use crate::ast::token::EToken::*;
use crate::ast::token::EUnparsedToken::*;
use crate::ast::token::ExpressionEnum::*;
use crate::ast::token::*;
use crate::ast::variable::VariableLink;
use crate::error_token;
use crate::tokenizer::builder::factory::*;
use crate::tokenizer::builder::ASTBuilder;
use crate::tokenizer::utils::{CharStream, Either};
use crate::tokenizer::C_ASSIGN;
use crate::typesystem::errors::ParseErrorEnum;
use crate::typesystem::types::TypedValue;
use std::borrow::Cow;

const RANGE_LITERAL: &str = "..";
const ASSIGN_LITERAL: &str = ":";
const OBJECT_LITERAL: &str = "OBJECT";
const DOT_LITERAL: &str = ".";

use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum;
use crate::typesystem::values::ValueEnum::NumberValue;
use crate::typesystem::types::string::StringEnum;

/// @Tbc brackets counting and error returning
pub fn tokenize(input: &str) -> VecDeque<EToken> {
    let mut ast_builder = ASTBuilder::new();
    let mut source = CharStream::new(input);

    // informs parser that following token is separate from previous one
    let token_break = false;

    //let mut open: u32 = 0;
    //let mut seq_open: u32 = 0;
    //let mut ctx_open: u32 = 0;

    let mut left_side = true;
    let mut after_colon = false;

    // @Todo: does it worth having function_def_gate_open and type_def_gate_open to simplify further parsing?
    // function_def_gate_open and type_def_gate_open can be integers that are alawys increased or decreased depending on ctx
    // Also, maybe those def gate opens could help validate special situations, such as only under type gate we can open < > as
    // type holders <string>, but not in any other case.

    //let length: usize = input.chars().count();
    //let mut position: usize = 0;

    while let Some(symbol) = source.peek() {
        trace!("- got {}", symbol);

        match symbol {
            '0'..='9' => {
                let number = source.get_number();
                ast_builder.push_value(NumberValue(number));
                after_colon = false;

                if source.dot_was_skipped {
                    if let Some('.') = source.peek() {
                        // two dots detected
                        source.next_char();

                        ast_builder.push_node(
                            RangePriority as u32,
                            Unparsed(Literal(RANGE_LITERAL.into())),
                            build_range,
                        );
                    }
                }
            }
            '/' => {
                source.next_char();

                if source.next_if_eq(&'/').is_some() {
                    while let Some(comment_text) = source.next_char() {
                        if comment_text == '\n' {
                            break;
                        }
                    }
                } else {
                    // no comments
                    ast_builder.push_node(
                        DivideMultiply as u32,
                        MathOperatorEnum::Division.into(),
                        build_any_operator,
                    );
                }
            }
            &C_ASSIGN => {
                source.next_char();

                left_side = false;
                after_colon = true;

                ast_builder.push_node(
                    Assign as u32,
                    Unparsed(Literal(ASSIGN_LITERAL.into())),
                    build_assignment,
                );
            }
            '+' | '-' | '*' | '×' | '÷' | '^' | '%' => {
                let extracted = source.next_char().unwrap();

                // Detect unary context for '-'
                let mut priority = match extracted {
                    '+' => Plus,
                    '-' => Minus,
                    '*' | '×' | '÷' | '%' => DivideMultiply,
                    '^' => PowerPriority,
                    _ => ErrorPriority,
                };

                if extracted == '-' {
                    // If start of stream or previous token was not an expression, it's unary
                    let is_unary = if let Some(token) = ast_builder.last_token() {
                        !matches!(
                            token,
                            Expression(_)
                                | Unparsed(BracketOpen)
                                | Unparsed(Literal(Cow::Borrowed(")"))) // Check for closing paren if it were stored? No, ) calls merge.
                        )
                    } else {
                        true
                    };

                    // Double check logic:
                    // 1 - 2 -> last is 1 (Expression). is_unary = false. Binary.
                    // 1 * -2 -> last is * (MathOperatorToken). is_unary = true. Unary.
                    // (-2) -> ( starts level. last is None (if start) or operator before (. is_unary = true.

                    if is_unary {
                        priority = UnaryPriority;
                    }
                }

                ast_builder.push_node(
                    priority as u32,
                    MathOperatorEnum::build_from_char(extracted),
                    build_any_operator,
                );
            }
            '{' => {
                source.next_char();

                //ctx_open += 1;
                ast_builder.incl_level();

                left_side = true;
                after_colon = false;

                ast_builder.push_node(
                    ContextPriority as u32,
                    Unparsed(Literal(OBJECT_LITERAL.into())),
                    build_context,
                );

                //ctx_open += 1;
                ast_builder.incl_level();
            }
            ';' | '\n' => {
                source.next_char();

                left_side = true;

                //----------------------> ctx instead of 0!!!!
                ast_builder.merge();
            }
            '}' => {
                source.next_char();

                //ctx_open -= 1;
                ast_builder.dec_level();

                ast_builder.merge();

                //ctx_open -= 1;
                ast_builder.dec_level();
            }
            '(' => {
                source.next_char();

                // prioritizing function/call merge
                ast_builder.incl_level();
                after_colon = false;

                if ast_builder.last_variable().is_some() {
                    if let Some(function_var) = ast_builder.pop_last_variable() {
                        if left_side {
                            let has_func_prefix = ast_builder.pop_literal_if("func")
                                || ast_builder.pop_unparsed_if(UserFunctionGateOpen);

                            if has_func_prefix {
                                ast_builder.push_node(
                                    FunctionCallPriority as u32,
                                    Unparsed(FunctionNameToken(function_var)),
                                    build_function_definition,
                                );
                            } else {
                                ast_builder.push_node(
                                    FunctionCallPriority as u32,
                                    Unparsed(FunctionNameToken(function_var)),
                                    build_function_call,
                                );
                            }
                        } else {
                            ast_builder.push_node(
                                FunctionCallPriority as u32,
                                Unparsed(FunctionNameToken(function_var)),
                                build_function_call,
                            );
                        }
                    }
                }

                // prioritizing arguments merge
                ast_builder.incl_level();
            }
            ')' => {
                source.next_char();

                ast_builder.dec_level();

                ast_builder.merge();

                ast_builder.dec_level();
            }
            ',' => {
                source.next_char();

                ast_builder.push_element(Unparsed(Comma));
            }
            ' ' | '\t' | '\r' => {
                source.next_char();
            }
            'a'..='z' | 'A'..='Z' => {
                let variable = source.get_literal_token();

                match variable {
                    Either::Left(literal) => {
                        match literal.as_str() {
                            "if" => {
                                // just jumping upper with no turning back
                                //ast_builder.incl_level();
                                ast_builder.push_element(Unparsed(Literal(literal.into())));
                                ast_builder.incl_level();
                            }

                            "then" => {
                                ast_builder.merge();
                                ast_builder.dec_level();
                                ast_builder.push_element(Unparsed(Literal(literal.into())));
                                ast_builder.incl_level();
                            }

                            "else" => {
                                ast_builder.merge();
                                ast_builder.dec_level();
                                ast_builder.push_node(
                                    ReservedWords as u32,
                                    Unparsed(Literal(literal.into())),
                                    build_if_then_else,
                                )
                            }

                            "for" => ast_builder.push_element(Unparsed(Literal(literal.into()))),

                            "in" => {
                                ast_builder.push_element(Unparsed(Literal(literal.into())));
                                ast_builder.incl_level();
                            }

                            "return" => {
                                // Treat as keyword only when not starting an assignment field
                                let is_field = match source.peek() {
                                    Some(&C_ASSIGN) => true,
                                    Some(c) if c.is_whitespace() => {
                                        matches!(source.peek_skip_whitespace(), Some(C_ASSIGN))
                                    }
                                    _ => false,
                                };

                                if is_field {
                                    ast_builder.push_element(VariableLink::new_unlinked(literal).into());
                                    after_colon = false;
                                    continue;
                                }

                                ast_builder.merge();
                                ast_builder.dec_level();
                                ast_builder.push_node(
                                    ReservedWords as u32,
                                    Unparsed(Literal(literal.into())),
                                    build_for_each_return,
                                )
                            }
                            //result.push_back(Unparsed(ReturnLiteral)),
                            "true" => {
                                ast_builder.push_element(Expression(ExpressionEnum::from(true)));
                                after_colon = false;
                            }

                            "false" => {
                                ast_builder.push_element(Expression(ExpressionEnum::from(false)));
                                after_colon = false;
                            }

                            "not" => ast_builder.push_node(
                                LogicalOperatorEnum::Not as u32,
                                Unparsed(Literal(literal.into())),
                                build_logical_operator,
                            ),

                            "and" => ast_builder.push_node(
                                LogicalOperatorEnum::And as u32,
                                Unparsed(Literal(literal.into())),
                                build_logical_operator,
                            ),

                            "or" => ast_builder.push_node(
                                LogicalOperatorEnum::Or as u32,
                                Unparsed(Literal(literal.into())),
                                build_logical_operator,
                            ),

                            "xor" => ast_builder.push_node(
                                LogicalOperatorEnum::Xor as u32,
                                Unparsed(Literal(literal.into())),
                                build_logical_operator,
                            ),

                            "func" => {
                                ast_builder.push_element(Unparsed(UserFunctionGateOpen));
                            }
                            "type" => {
                                ast_builder.push_element(Unparsed(UserTypeDefinitionGateOpen));
                            }
                            "as" => {
                                // Insert cast operator and immediately parse trailing type
                                ast_builder.push_node(
                                    CastPriority as u32,
                                    Unparsed(Literal(literal.into())),
                                    build_cast,
                                );
                                let tref = parse_complex_type_no_angle(&mut source);
                                ast_builder.push_element(Unparsed(TypeReferenceLiteral(tref)));
                                after_colon = false;
                            }
                            _ => {
                                if after_colon {
                                    if let Some(tref) = parse_type_with_trailing_lists(
                                        literal.as_str(),
                                        &mut source,
                                    ) {
                                        ast_builder
                                            .push_element(Unparsed(TypeReferenceLiteral(tref)));
                                        after_colon = false;
                                        continue;
                                    }
                                }

                                ast_builder
                                    .push_element(VariableLink::new_unlinked(literal).into());
                                after_colon = false;
                            }
                        }
                    }
                    Either::Right(expression) => {
                        ast_builder
                            .push_element(VariableLink::new_unlinked_path(expression).into());
                        after_colon = false;
                    }
                }
            }
            '.' => {
                source.next_char();
                after_colon = false;

                // two dots detected
                if let Some('.') = source.peek() {
                    source.next_char();

                    // three dots detected
                    if let Some('.') = source.peek() {
                        source.next_char();

                        ast_builder.push_element(Expression(ContextVariable));
                    } else {
                        ast_builder.push_node(
                            RangePriority as u32,
                            Unparsed(Literal(RANGE_LITERAL.into())),
                            build_range,
                        );
                    }
                } else {
                    // merge_left_if_can must already be called with ]
                    ast_builder.push_node(
                        FieldSelectionPriority as u32,
                        Unparsed(Literal(DOT_LITERAL.into())),
                        build_field_selection,
                    );
                }
            }
            //----------------------------------------------------------------------------------
            '[' => {
                // @Tbc: implement range

                // can be 1. Array Start, 2. Filter, 3. Range Start

                ast_builder.incl_level();

                // derive isArray
                let is_select: bool = !token_break
                    && if let Some(token) = ast_builder.last_token() {
                        matches!(
                            token,
                            Expression(Variable(_))
                                | Expression(FunctionCall(_))
                                | Expression(Collection(_))
                        )
                    } else {
                        // if first item general
                        false
                    };

                if is_select {
                    ast_builder.push_node(FilterArray as u32, Unparsed(BracketOpen), build_filter);
                } else {
                    ast_builder.push_node(
                        FilterArray as u32,
                        Unparsed(BracketOpen),
                        build_sequence,
                    );
                };

                source.next_char();

                ast_builder.incl_level();
                after_colon = false;
            }
            //----------------------------------------------------------------------------------
            ']' => {
                source.next_char();

                ast_builder.dec_level();

                ast_builder.merge();

                ast_builder.dec_level();
                after_colon = false;
            }
            '=' | '<' | '>' => {
                if *symbol == '<' && after_colon {
                    source.next_char();
                    // @Todo: investigate that, not sure if it make sense: type parsing should be done in builder.rs, investigate that
                    // I'm expecting having something like build_function_definition, so it is build_type_definition_part // e.g. <string,"unknown">
                    match parse_complex_type_in_angle(&mut source) {
                        Ok(tref) => ast_builder.push_element(Unparsed(TypeReferenceLiteral(tref))),
                        Err(err) => ast_builder.push_element(EToken::ParseError(err)),
                    }
                    after_colon = false;
                } else if let Some(comparator) = ComparatorEnum::parse(&mut source) {
                    ast_builder.push_node(
                        ComparatorPriority as u32,
                        Unparsed(ComparatorToken(comparator)),
                        build_comparator,
                    );
                } else {
                    ast_builder.push_element(error_token!(
                        "Unrecognized comparator after '{}'",
                        source.next_char().unwrap()
                    ));
                }
            }
            '"' | '\'' => {
                let string_starter = source.next_char().unwrap();

                let literal = source.get_all_till(string_starter);

                ast_builder.push_element(Expression(ExpressionEnum::from(literal)));
                after_colon = false;
            }
            _ => {
                ast_builder.push_element(error_token!(
                    "unexpected character '{}'",
                    source.next_char().unwrap()
                ));
            }
        }
    }

    //if seqOpen > 0 { panic!("Sequence not closed"); }

    ast_builder.finalize().0
}

pub fn parse_complex_type_in_angle(source: &mut CharStream) -> Result<ComplexTypeRef, ParseErrorEnum> {
    let mut name = String::new();
    while let Some(symbol) = source.peek().cloned() {
        if symbol == '>' || symbol == ',' {
            break;
        } else {
            name.push(symbol);
            source.next_char();
        }
    }
    let mut type_ref = parse_type(name.trim());

    source.skip_whitespace();
    if let Some(',') = source.peek() {
        source.next_char();
        source.skip_whitespace();
        parse_default_value(&mut type_ref, source)?;
    }

    if let Some('>') = source.peek() {
        source.next_char();
    } else {
        return Err(ParseErrorEnum::WrongFormat(
            "Missing closing '>' in type reference".to_string(),
        ));
    }

    Ok(type_ref)
}

fn parse_default_value(
    type_ref: &mut ComplexTypeRef,
    source: &mut CharStream,
) -> Result<(), ParseErrorEnum> {
    let val = match source.peek() {
        Some('[') => {
            source.next_char();
            source.skip_whitespace();
            let mut elements = Vec::new();
            while let Some(&symbol) = source.peek() {
                if symbol == ']' {
                    source.next_char();
                    break;
                }
                let element = if symbol == '"' || symbol == '\'' {
                    let quote = source.next_char().unwrap();
                    ValueEnum::StringValue(StringEnum::from(source.get_all_till(quote)))
                } else if symbol.is_numeric() || symbol == '-' {
                    ValueEnum::NumberValue(source.get_number())
                } else {
                    let literal = source.get_alphanumeric();
                    if literal == "true" {
                        ValueEnum::BooleanValue(true)
                    } else if literal == "false" {
                        ValueEnum::BooleanValue(false)
                    } else {
                        return Err(ParseErrorEnum::WrongFormat(format!(
                            "Unsupported element in list default: {}",
                            literal
                        )));
                    }
                };
                elements.push(element);
                source.skip_whitespace();
                if let Some(',') = source.peek() {
                    source.next_char();
                    source.skip_whitespace();
                }
            }

            let item_type = if let Some(first) = elements.first() {
                first.get_type()
            } else {
                ValueType::UndefinedType
            };

            Some(ValueEnum::Array(
                crate::typesystem::values::ArrayValue::PrimitivesArray {
                    values: elements,
                    item_type,
                },
            ))
        }
        Some('"') | Some('\'') => {
            let quote = source.next_char().unwrap();
            Some(ValueEnum::StringValue(StringEnum::from(source.get_all_till(quote))))
        }
        Some('t') | Some('f') => {
            let literal = source.get_alphanumeric();
            if literal == "true" {
                Some(ValueEnum::BooleanValue(true))
            } else if literal == "false" {
                Some(ValueEnum::BooleanValue(false))
            } else {
                return Err(ParseErrorEnum::WrongFormat(format!(
                    "Invalid boolean default: {}",
                    literal
                )));
            }
        }
        Some(symbol) if symbol.is_numeric() || *symbol == '-' => {
            Some(ValueEnum::NumberValue(source.get_number()))
        }
        _ => None,
    };

    if let Some(mut value) = val {
        match type_ref {
            ComplexTypeRef::BuiltinType(val_type, ref mut default_opt) => {
                let actual_ty = value.get_type();
                if actual_ty != *val_type && !matches!(val_type, ValueType::UndefinedType) {
                    return Err(ParseErrorEnum::WrongFormat(format!(
                        "Default value type mismatch: expected {}, got {}",
                        val_type, actual_ty
                    )));
                }
                *default_opt = Some(value);
            }
            ComplexTypeRef::List(ref inner, ref mut default_opt) => {
                let actual_ty = value.get_type();
                if !matches!(actual_ty, ValueType::ListType(_)) {
                    return Err(ParseErrorEnum::WrongFormat(format!(
                        "Default value type mismatch: expected list, got {}",
                        actual_ty
                    )));
                }

                // Refine item type for empty list default
                if let ValueEnum::Array(crate::typesystem::values::ArrayValue::PrimitivesArray {
                    ref mut item_type,
                    ..
                }) = value
                {
                    if let ComplexTypeRef::BuiltinType(val_type, _) = &**inner {
                        *item_type = val_type.clone();
                    }
                }

                *default_opt = Some(value);
            }
            ComplexTypeRef::Alias(_, ref mut default_opt) => {
                *default_opt = Some(value);
            }
        }
    }
    Ok(())
}

fn parse_complex_type_no_angle(source: &mut CharStream) -> ComplexTypeRef {
    source.skip_whitespace();
    let ident = source.get_alphanumeric_or(&['[', ']']);
    parse_type(ident.as_str())
}

fn parse_type_with_trailing_lists(base: &str, source: &mut CharStream) -> Option<ComplexTypeRef> {
    let mut layers = 0usize;
    loop {
        let mut iter = source.iter.clone();
        match (iter.next(), iter.peek().copied()) {
            (Some('['), Some(']')) => {
                source.next_char();
                source.next_char();
                layers += 1;
            }
            _ => break,
        }
    }

    if layers == 0 {
        return None;
    }

    let mut tref = parse_type(base);
    for _ in 0..layers {
        tref = ComplexTypeRef::List(Box::new(tref), None);
    }

    Some(tref)
}

pub fn parse_type(name: &str) -> ComplexTypeRef {
    let mut string = name;
    let mut layers = 0usize;

    while string.len() >= 2 && string.ends_with("[]") {
        string = &string[..string.len() - 2];
        layers += 1;
    }

    let mut return_type = match string {
        "number" => ComplexTypeRef::BuiltinType(ValueType::NumberType, None),
        "string" => ComplexTypeRef::BuiltinType(ValueType::StringType, None),
        "boolean" => ComplexTypeRef::BuiltinType(ValueType::BooleanType, None),
        "date" => ComplexTypeRef::BuiltinType(ValueType::DateType, None),
        "time" => ComplexTypeRef::BuiltinType(ValueType::TimeType, None),
        "datetime" => ComplexTypeRef::BuiltinType(ValueType::DateTimeType, None),
        "duration" => ComplexTypeRef::BuiltinType(ValueType::DurationType, None),
        _ => ComplexTypeRef::Alias(string.to_owned(), None),
    };

    for _ in 0..layers {
        return_type = ComplexTypeRef::List(Box::new(return_type), None);
    }

    return_type
}
