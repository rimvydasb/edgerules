use crate::ast::token::EToken::{Expression, ParseError, Unparsed};
use crate::ast::token::EUnparsedToken::LiteralToken;
use crate::ast::token::{EToken, ExpressionEnum};
use crate::test_support::EToken::Definition;
use crate::typesystem::errors::ParseErrorEnum;
use crate::typesystem::errors::ParseErrorEnum::{
    MissingLiteral, UnexpectedEnd, UnexpectedLiteral, UnexpectedToken, WrongFormat,
};
use crate::typesystem::types::number::NumberEnum;
use crate::typesystem::types::{Float, Integer};
use std::collections::vec_deque::VecDeque;
use std::fmt::Display;
use std::iter::Peekable;
use std::ops;
use std::str::Chars;
//----------------------------------------------------------------------------------------------

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone, PartialEq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

//----------------------------------------------------------------------------------------------
// TokenChain
//----------------------------------------------------------------------------------------------

#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct TokenChain(pub VecDeque<EToken>);

impl Default for TokenChain {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenChain {
    pub fn new() -> Self {
        TokenChain(VecDeque::new())
    }

    pub fn pop_left(&mut self) -> Result<EToken, ParseErrorEnum> {
        self.pop_helper(|s| s.pop_back())
    }

    pub fn pop_right(&mut self) -> Result<EToken, ParseErrorEnum> {
        self.pop_helper(|s| s.pop_front())
    }

    fn pop_helper<F>(&mut self, pop_fn: F) -> Result<EToken, ParseErrorEnum>
    where
        F: FnOnce(&mut Self) -> Option<EToken>,
    {
        if let Some(token) = pop_fn(self) {
            if let ParseError(error) = token {
                Err(error)
            } else {
                Ok(token)
            }
        } else {
            Err(UnexpectedEnd)
        }
    }

    pub fn pop_left_expression(&mut self) -> Result<ExpressionEnum, ParseErrorEnum> {
        self.pop_expression_helper(|s| s.pop_back())
    }

    pub fn pop_right_expression(&mut self) -> Result<ExpressionEnum, ParseErrorEnum> {
        self.pop_expression_helper(|s| s.pop_front())
    }

    fn pop_expression_helper<F>(&mut self, pop_fn: F) -> Result<ExpressionEnum, ParseErrorEnum>
    where
        F: FnOnce(&mut Self) -> Option<EToken>,
    {
        match pop_fn(self) {
            None => Err(UnexpectedEnd),
            Some(Expression(expression)) => Ok(expression),
            Some(ParseError(error)) => Err(error),
            Some(Unparsed(token)) => Err(UnexpectedToken(Box::new(Unparsed(token)), None)),
            Some(Definition(_definition)) => Err(WrongFormat("Expected expression, got definition".to_string())),
        }
    }

    pub fn pop_left_as_expected(&mut self, expected: &str) -> Result<String, ParseErrorEnum> {
        self.pop_as_expected_helper(|s| s.pop_back(), expected)
    }

    #[allow(dead_code)]
    pub fn pop_right_as_expected(&mut self, expected: &str) -> Result<String, ParseErrorEnum> {
        self.pop_as_expected_helper(|s| s.pop_front(), expected)
    }

    fn pop_as_expected_helper<F>(&mut self, pop_fn: F, expected: &str) -> Result<String, ParseErrorEnum>
    where
        F: FnOnce(&mut Self) -> Option<EToken>,
    {
        if let Some(Unparsed(LiteralToken(maybe))) = pop_fn(self) {
            return if maybe == expected {
                Ok(maybe.into_owned())
            } else {
                Err(UnexpectedLiteral(expected.to_string(), Some(expected.to_string())))
            };
        }

        Err(MissingLiteral(expected.to_string()))
    }

    pub fn drain_expressions(&mut self) -> Result<Vec<ExpressionEnum>, ParseErrorEnum> {
        let mut arguments = Vec::new();

        while let Some(token) = self.pop_front() {
            match token {
                ParseError(error) => return Err(error),
                Unparsed(_) => {}
                Expression(expression) => arguments.push(expression),
                unknown => return Err(UnexpectedToken(Box::new(unknown), None)),
            }
        }

        Ok(arguments)
    }
}

impl Display for TokenChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tokens: Vec<String> = self.0.iter().map(|t| format!("{}", t)).collect();
        write!(f, "TokenChain[{}]", tokens.join(", "))
    }
}

impl ops::Deref for TokenChain {
    type Target = VecDeque<EToken>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for TokenChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

//----------------------------------------------------------------------------------------------
// TokenChain END.
//----------------------------------------------------------------------------------------------

//----------------------------------------------------------------------------------------------
// CharStream
//----------------------------------------------------------------------------------------------

pub struct CharStream<'a> {
    pub iter: Peekable<Chars<'a>>,
    pub dot_was_skipped: bool,
}

static NUMBER_PARSE_ERROR: &str = "Absolutely unexpected error while parsing number";

impl<'a> CharStream<'a> {
    // Constructor
    pub fn new(input: &'a str) -> Self {
        CharStream { iter: input.chars().peekable(), dot_was_skipped: false }
    }

    pub fn get_alphanumeric(&mut self) -> String {
        let mut alphanumeric = String::new();

        while let Some(c) = self.iter.peek() {
            if c.is_alphanumeric() {
                alphanumeric.push(*c);
                self.iter.next();
            } else {
                break;
            }
        }

        alphanumeric
    }

    pub fn get_alphanumeric_or(&mut self, or: &[char]) -> String {
        let mut alphanumeric = String::new();

        while let Some(c) = self.iter.peek() {
            if c.is_alphanumeric() || or.contains(c) {
                alphanumeric.push(*c);
                self.iter.next();
            } else {
                break;
            }
        }

        alphanumeric
    }

    pub fn skip_whitespace(&mut self) -> &mut Self {
        while self.iter.next_if_eq(&' ').is_some() {}

        self
    }

    pub fn get_number(&mut self) -> NumberEnum {
        let mut number_string = String::new();

        let mut dot_detected = false;

        while let Some(&symbol) = self.iter.peek() {
            if symbol == '.' {
                self.iter.next();

                // second following dot is not allowed
                if let Some(&'.') = self.iter.peek() {
                    self.dot_was_skipped = true;
                    break;
                }

                // another dot is also not allowed like 1.2.3
                if dot_detected {
                    self.dot_was_skipped = true;
                    break;
                }

                dot_detected = true;
                number_string.push(symbol);
            } else if symbol.is_numeric() {
                number_string.push(symbol);
                self.iter.next();
            } else {
                break;
            }
        }

        if dot_detected {
            NumberEnum::Real(number_string.parse::<Float>().expect(NUMBER_PARSE_ERROR))
        } else if number_string == "0" {
            NumberEnum::Int(0)
        } else {
            match number_string.parse::<Integer>() {
                Ok(int_val) => NumberEnum::Int(int_val),
                Err(_) => NumberEnum::Real(number_string.parse::<Float>().expect(NUMBER_PARSE_ERROR)),
            }
        }
    }

    /// or use let path: Vec<&str> = literal.as_str().split(".").collect();
    pub fn get_literal_token(&mut self) -> Either<String, Vec<String>> {
        let mut word = String::new();
        let mut sentence: Vec<String> = Vec::new();

        while let Some(&symbol) = self.iter.peek() {
            match symbol {
                'a'..='z' | 'A'..='Z' | 'α'..='ω' | '0'..='9' => {
                    word.push(symbol);
                    self.iter.next();
                }
                '.' => {
                    sentence.push(word);
                    word = String::new();
                    self.iter.next();
                }
                _ => {
                    break;
                }
            }
        }

        if sentence.is_empty() {
            Either::Left(word)
        } else {
            sentence.push(word);
            Either::Right(sentence)
        }
    }

    /// gets all symbols till the given symbol including the given symbol
    pub fn get_all_till(&mut self, symbol: char) -> String {
        let mut result = String::new();

        for c in self.iter.by_ref() {
            if c == symbol {
                break;
            }
            result.push(c);
        }

        result
    }

    pub fn parse_arguments(&mut self) -> Option<Vec<String>> {
        self.iter.next_if_eq(&'(')?;

        let mut result: Vec<String> = Vec::new();

        while let Some(symbol) = self.iter.peek() {
            match symbol {
                '"' => {
                    self.iter.next();
                    let mut arg = String::new();
                    while let Some(c) = self.iter.peek() {
                        if *c == '"' {
                            self.iter.next();
                            break;
                        }
                        arg.push(*c);
                        self.iter.next();
                    }
                    result.push(arg);
                }
                ')' => {
                    return Some(result);
                }
                _ => {
                    self.iter.next();
                }
            }
        }

        Some(result)
    }

    // Override iter.next() method
    pub fn next_char(&mut self) -> Option<char> {
        self.iter.next()
    }

    // Override next_if_eq() method
    pub fn next_if_eq(&mut self, symbol: &char) -> Option<char> {
        self.iter.next_if_eq(symbol)
    }

    // Override peek() method
    pub fn peek(&mut self) -> Option<&char> {
        self.iter.peek()
    }

    pub fn peek_skip_whitespace(&self) -> Option<char> {
        let mut iter = self.iter.clone();
        iter.find(|&c| c != ' ' && c != '\t' && c != '\r')
    }
}

//----------------------------------------------------------------------------------------------
// CharStream END.
//----------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::tokenizer::utils::Either::{Left, Right};
    use crate::utils::test::init_logger;

    #[test]
    fn test_common() {
        init_logger();

        assert_eq!(CharStream::new("(\"first-hit\")").parse_arguments().unwrap(), vec!["first-hit"]);

        assert_eq!(CharStream::new(" (\"first-hit\")").parse_arguments(), None);

        assert_eq!(
            CharStream::new("(\"first-hit\",\"another\")").parse_arguments().unwrap(),
            vec!["first-hit", "another"]
        );

        assert_eq!(
            CharStream::new("(\"first-hit\",\"another\"").parse_arguments().unwrap(),
            vec!["first-hit", "another"]
        );

        assert_eq!(
            CharStream::new("(\"first-hit\",\"another)").parse_arguments().unwrap(),
            vec!["first-hit", "another)"]
        );

        // testing CharStream get_alphanumeric method
        assert_eq!(CharStream::new("abc").get_alphanumeric(), "abc");
        assert_eq!(CharStream::new("abc123").get_alphanumeric(), "abc123");
        assert_eq!(CharStream::new("abc123_").get_alphanumeric(), "abc123");
        assert_eq!(CharStream::new("abc123_ ").get_alphanumeric(), "abc123");
        assert_eq!(CharStream::new(" abc123_ ").get_alphanumeric(), "");
        assert_eq!(CharStream::new("abc&123_ ").get_alphanumeric(), "abc");

        // testing CharStream skip_whitespace method
        assert_eq!(CharStream::new(" abc123_ ").skip_whitespace().next_char().unwrap(), 'a');
        assert_eq!(CharStream::new("xbc123_ ").skip_whitespace().next_char().unwrap(), 'x');
        assert_eq!(CharStream::new("       zbc123_ ").skip_whitespace().next_char().unwrap(), 'z');

        // testing CharStream get_number method
        assert_eq!(CharStream::new("123").get_number(), NumberEnum::from(123));
        assert_eq!(CharStream::new("123.5").get_number(), NumberEnum::from(123.5));
        assert_eq!(CharStream::new("0000.5").get_number(), NumberEnum::from(0.5));
        assert_eq!(CharStream::new("1000.5").get_number(), NumberEnum::from(1000.5));
        assert_eq!(CharStream::new("0.5").get_number(), NumberEnum::from(0.5));
        assert_eq!(CharStream::new("0.5x").get_number(), NumberEnum::from(0.5));
        assert_eq!(CharStream::new("0.5.1").get_number(), NumberEnum::from(0.5));
        assert_eq!(CharStream::new("10..12").get_number(), NumberEnum::from(10));
        assert_eq!(CharStream::new("1..3").get_number(), NumberEnum::from(1));
        {
            let mut stream = CharStream::new("13..1");
            assert_eq!(stream.get_number(), NumberEnum::from(13));
            assert_eq!(stream.peek().unwrap(), &'.');
            assert_eq!(stream.next_char().unwrap(), '.');
            assert_eq!(stream.next_char().unwrap(), '1');
            assert!(stream.dot_was_skipped);
        }
        // testing CharStream get_literal_token method
        assert_eq!(CharStream::new("abc").get_literal_token(), Left("abc".to_string()));
        assert_eq!(CharStream::new("b c").get_literal_token(), Left("b".to_string()));
        assert_eq!(CharStream::new("b ").get_literal_token(), Left("b".to_string()));
        assert_eq!(CharStream::new("  ").get_literal_token(), Left("".to_string()));

        assert_eq!(CharStream::new("aaa.bbb").get_literal_token(), Right(vec!["aaa".to_string(), "bbb".to_string()]));
        assert_eq!(CharStream::new("aaa. ").get_literal_token(), Right(vec!["aaa".to_string(), "".to_string()]));
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        assert_eq!(CharStream::new(" .5x").get_number(), NumberEnum::from(0.5));
    }
}
