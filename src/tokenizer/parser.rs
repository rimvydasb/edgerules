use std::collections::vec_deque::VecDeque;

use crate::ast::annotations::AnnotationEnum;
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
use log::trace;

const RANGE_LITERAL: &str = "..";
const ASSIGN_LITERAL: &str = ":";
const OBJECT_LITERAL: &str = "OBJECT";
const DOT_LITERAL: &str = ".";

use crate::typesystem::types::ValueType;
use crate::typesystem::values::ValueEnum::NumberValue;

/// @TODO brackets counting and error returning
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
                        source.next();

                        ast_builder.push_node(
                            RangePriority as u32,
                            Unparsed(Literal(RANGE_LITERAL.into())),
                            build_range,
                        );
                    }
                }
            }
            '/' => {
                source.next();

                if source.next_if_eq(&'/').is_some() {
                    while let Some(comment_text) = source.next() {
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
                source.next();

                left_side = false;
                after_colon = true;

                ast_builder.push_node(
                    Assign as u32,
                    Unparsed(Literal(ASSIGN_LITERAL.into())),
                    build_assignment,
                );
            }
            '+' | '-' | '*' | '×' | '÷' | '^' => {
                let extracted = source.next().unwrap();

                // @Todo: must be in a builder
                let priority = match extracted {
                    '+' => Plus,
                    '-' => Minus,
                    '*' | '×' | '÷' => DivideMultiply,
                    '^' => PowerPriority,
                    _ => ErrorPriority,
                };

                ast_builder.push_node(
                    priority as u32,
                    MathOperatorEnum::build_from_char(extracted),
                    build_any_operator,
                );
            }
            '{' => {
                source.next();

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
                source.next();

                left_side = true;

                //----------------------> ctx instead of 0!!!!
                ast_builder.merge();
            }
            '}' => {
                source.next();

                //ctx_open -= 1;
                ast_builder.dec_level();

                ast_builder.merge();

                //ctx_open -= 1;
                ast_builder.dec_level();
            }
            '(' => {
                source.next();

                // prioritizing function/call merge
                ast_builder.incl_level();
                after_colon = false;

                if ast_builder.last_variable().is_some() {
                    if let Some(function_var) = ast_builder.pop_last_variable() {
                        if left_side {
                            // Enforce new syntax: function definitions must be prefixed with 'func'
                            let has_func_prefix = ast_builder.pop_literal_if("func");

                            if has_func_prefix {
                                ast_builder.push_node(
                                    FunctionCallPriority as u32,
                                    Unparsed(FunctionNameToken(function_var)),
                                    build_function_definition,
                                );
                            } else {
                                ast_builder.push_element(error_token!(
                                    "Function definition must start with 'func'"
                                ));
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
                source.next();

                ast_builder.dec_level();

                ast_builder.merge();

                ast_builder.dec_level();
            }
            ',' => {
                source.next();

                ast_builder.push_element(Unparsed(Comma));
            }
            ' ' | '\t' | '\r' => {
                source.next();
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
                                ast_builder.push_element(Unparsed(Literal(literal.into())));
                            }
                            "type" => {
                                ast_builder.push_element(Unparsed(Literal(literal.into())));
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
                source.next();
                after_colon = false;

                // two dots detected
                if let Some('.') = source.peek() {
                    source.next();

                    // three dots detected
                    if let Some('.') = source.peek() {
                        source.next();

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
                // @Todo: implement range

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

                source.next();

                ast_builder.incl_level();
                after_colon = false;
            }
            //----------------------------------------------------------------------------------
            ']' => {
                source.next();

                ast_builder.dec_level();

                ast_builder.merge();

                ast_builder.dec_level();
                after_colon = false;
            }
            '=' | '<' | '>' => {
                // @Todo: simplify operator acquisition
                if *symbol == '<' && after_colon {
                    source.next();
                    let tref = parse_complex_type_in_angle(&mut source);
                    ast_builder.push_element(Unparsed(TypeReferenceLiteral(tref)));
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
                        source.next().unwrap()
                    ));
                }
            }
            '"' | '\'' => {
                let string_starter = source.next().unwrap();

                let literal = source.get_all_till(string_starter);

                ast_builder.push_element(Expression(ExpressionEnum::from(literal)));
                after_colon = false;
            }
            '@' => {
                source.next();

                let annotation = AnnotationEnum::parse(&mut source);

                ast_builder.push_element(annotation);
            }
            _ => {
                ast_builder.push_element(error_token!(
                    "unexpected character '{}'",
                    source.next().unwrap()
                ));
            }
        }
    }

    //if seqOpen > 0 { panic!("Sequence not closed"); }

    ast_builder.finalize().0
}

fn parse_complex_type_in_angle(source: &mut CharStream) -> ComplexTypeRef {
    let mut name = String::new();
    while let Some(c) = source.peek().cloned() {
        if c == '>' {
            source.next();
            break;
        } else {
            name.push(c);
            source.next();
        }
    }
    parse_type(name.trim())
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
                source.next();
                source.next();
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
        tref = ComplexTypeRef::List(Box::new(tref));
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
        "number" => ComplexTypeRef::Primitive(ValueType::NumberType),
        "string" => ComplexTypeRef::Primitive(ValueType::StringType),
        "boolean" => ComplexTypeRef::Primitive(ValueType::BooleanType),
        "date" => ComplexTypeRef::Primitive(ValueType::DateType),
        "time" => ComplexTypeRef::Primitive(ValueType::TimeType),
        "datetime" => ComplexTypeRef::Primitive(ValueType::DateTimeType),
        "duration" => ComplexTypeRef::Primitive(ValueType::DurationType),
        _ => ComplexTypeRef::Alias(string.to_owned()),
    };

    for _ in 0..layers {
        return_type = ComplexTypeRef::List(Box::new(return_type));
    }

    return_type
}
