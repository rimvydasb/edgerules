use std::collections::vec_deque::VecDeque;

use log::trace;
use crate::ast::annotations::AnnotationEnum;
use crate::ast::operators::comparators::ComparatorEnum;
use crate::ast::token::*;
use crate::ast::token::EToken::*;
use crate::ast::token::EPriorities::*;
use crate::tokenizer::utils::{CharStream, Either};
use crate::ast::operators::logical_operators::LogicalOperatorEnum;
use crate::ast::operators::math_operators::MathOperatorEnum;
use crate::ast::token::ExpressionEnum::*;
use crate::ast::token::EUnparsedToken::*;
use crate::ast::variable::VariableLink;
use crate::error_token;
use crate::tokenizer::C_ASSIGN;
use crate::tokenizer::builder::ASTBuilder;
use crate::tokenizer::builder::factory::*;


use crate::typesystem::values::ValueEnum::NumberValue;

/// @TODO brackets counting and error returning
pub fn tokenize(input: &String) -> VecDeque<EToken> {
    let mut ast_builder = ASTBuilder::new();
    let mut source = CharStream::new(input);

    // informs parser that following token is separate from previous one
    let token_break = false;

    //let mut open: u32 = 0;
    //let mut seq_open: u32 = 0;
    //let mut ctx_open: u32 = 0;

    let mut left_side = true;
    //let length: usize = input.chars().count();
    //let mut position: usize = 0;

    while let Some(symbol) = source.peek() {
        trace!("- got {} for {}", symbol, input);

        match symbol {
            '0'..='9' => {
                let number = source.get_number();
                ast_builder.push_value(NumberValue(number));

                if source.dot_was_skipped {
                    if let Some('.') = source.peek() {
                        // two dots detected
                        source.next();

                        ast_builder.push_node(RangePriority as u32,
                                              Unparsed(Literal("..".to_string())),
                                              build_range);
                    }
                }
            }
            '/' => {
                source.next();

                if source.next_if_eq(&'/').is_some() {
                    while let Some(comment_text) = source.next() {
                        if comment_text == '\n' { break; }
                    };
                } else {
                    // no comments
                    ast_builder.push_node(DivideMultiply as u32,
                                          MathOperatorEnum::Division.into(),
                                          build_any_operator);
                }
            }
            &C_ASSIGN => {
                source.next();

                left_side = false;

                ast_builder.push_node(Assign as u32,
                                      Unparsed(Literal(C_ASSIGN.to_string())),
                                      build_assignment);
            }
            '+' | '-' | '*' | '×' | '÷' | '^' => {
                let extracted = source.next().unwrap();

                // @Todo: must be in a builder
                let priority = match extracted {
                    '+' => Plus,
                    '-' => Minus,
                    '*' | '×' | '÷' => DivideMultiply,
                    '^' => PowerPriority,
                    _ => ErrorPriority
                };

                ast_builder.push_node(priority as u32,
                                      MathOperatorEnum::build(extracted.to_string().as_str()),
                                      build_any_operator);
            }
            '{' => {
                source.next();

                //ctx_open += 1;
                ast_builder.incl_level();

                left_side = true;

                ast_builder.push_node(ContextPriority as u32,
                                      Unparsed(Literal("OBJECT".to_string())),
                                      build_context);

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

                // prioritizing function merge
                ast_builder.incl_level();

                if let Some(function_name) = ast_builder.last_variable() {
                    if left_side {
                        ast_builder.push_node(FunctionCallPriority as u32,
                                              Unparsed(Literal(function_name)),
                                              build_function_definition);
                    } else {
                        ast_builder.push_node(FunctionCallPriority as u32,
                                              Unparsed(Literal(function_name)),
                                              build_function_call);
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
                                ast_builder.push_element(Unparsed(Literal(literal)));
                                ast_builder.incl_level();
                            }

                            "then" => {
                                ast_builder.merge();
                                ast_builder.dec_level();
                                ast_builder.push_element(Unparsed(Literal(literal)));
                                ast_builder.incl_level();
                            }

                            "else" => {
                                ast_builder.merge();
                                ast_builder.dec_level();
                                ast_builder.push_node(ReservedWords as u32,
                                                      Unparsed(Literal(literal)),
                                                      build_if_then_else)
                            }

                            "for" => ast_builder.push_element(Unparsed(Literal(literal))),

                            "in" => {
                                ast_builder.push_element(Unparsed(Literal(literal)));
                                ast_builder.incl_level();
                            }

                            "return" => {
                                ast_builder.merge();
                                ast_builder.dec_level();
                                ast_builder.push_node(ReservedWords as u32,
                                                      Unparsed(Literal(literal)),
                                                      build_for_each_return)
                            }
                            //result.push_back(Unparsed(ReturnLiteral)),

                            "and" => ast_builder.push_node(LogicalOperatorEnum::And as u32,
                                                           Unparsed(Literal(literal)),
                                                           build_logical_operator),

                            "or" => ast_builder.push_node(LogicalOperatorEnum::Or as u32,
                                                          Unparsed(Literal(literal)),
                                                          build_logical_operator),

                            "xor" => ast_builder.push_node(LogicalOperatorEnum::Xor as u32,
                                                           Unparsed(Literal(literal)),
                                                           build_logical_operator),

                            _ => ast_builder.push_element(VariableLink::new_unlinked(literal).into()),
                        }
                    }
                    Either::Right(expression) => {
                        ast_builder.push_element(VariableLink::new_unlinked_path(expression).into());
                    }
                }
            }
            '.' => {
                source.next();

                // two dots detected
                if let Some('.') = source.peek() {
                    source.next();

                    // three dots detected
                    if let Some('.') = source.peek() {
                        source.next();

                        ast_builder.push_element(Expression(ContextVariable));
                    } else {
                        ast_builder.push_node(RangePriority as u32,
                                              Unparsed(Literal("..".to_string())),
                                              build_range);
                    }
                } else {

                    // merge_left_if_can must already be called with ]
                    ast_builder.push_node(FieldSelectionPriority as u32,
                                          Unparsed(Literal(".".to_string())),
                                          build_field_selection);
                }
            }
            //----------------------------------------------------------------------------------
            '[' => {

                // @Todo: implement range

                // can be 1. Array Start, 2. Filter, 3. Range Start

                ast_builder.incl_level();

                // derive isArray
                let is_select: bool = !token_break && {
                    if let Some(token) = ast_builder.last_token() {
                        match token {
                            // allows filter after, will be replaced by actual filter token
                            Expression(Variable(_)) | Expression(FunctionCall(_)) | Expression(Collection(_)) => true,

                            // treated as a new array
                            _ => false
                        }
                    } else {
                        // if first item general
                        false
                    }
                };

                if is_select {
                    ast_builder.push_node(FilterArray as u32,
                                          Unparsed(BracketOpen),
                                          build_filter);
                } else {
                    ast_builder.push_node(FilterArray as u32,
                                          Unparsed(BracketOpen),
                                          build_sequence);
                };

                source.next();

                ast_builder.incl_level();
            }
            //----------------------------------------------------------------------------------
            ']' => {
                source.next();

                ast_builder.dec_level();

                ast_builder.merge();

                ast_builder.dec_level();
            }
            '=' | '<' | '>' => {
                // @Todo: simplify operator acquisition
                if let Some(comparator) = ComparatorEnum::parse(&mut source) {
                    ast_builder.push_node(ComparatorPriority as u32,
                                          Unparsed(Literal(comparator.as_str().to_string())),
                                          build_comparator);
                } else {
                    ast_builder.push_element(error_token!("Unrecognized comparator after '{}'", source.next().unwrap()));
                }
            }
            '"' | '\'' => {
                let string_starter = source.next().unwrap();

                let literal = source.get_all_till(string_starter);

                ast_builder.push_element(Expression(ExpressionEnum::from(literal)));
            }
            '@' => {
                source.next();

                let annotation = AnnotationEnum::parse(&mut source);

                ast_builder.push_element(annotation);
            }
            _ => {
                ast_builder.push_element(error_token!("unexpected character '{}'", source.next().unwrap()));
            }
        }
    }

    //if seqOpen > 0 { panic!("Sequence not closed"); }

    ast_builder.finalize().0
}
