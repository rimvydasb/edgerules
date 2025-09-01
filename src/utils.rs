use std::collections::vec_deque::VecDeque;
use std::fmt::{Display};
use std::ops::{Add};
use crate::ast::context::function_context::RETURN_EXPRESSION;

pub fn to_path(deque: VecDeque<&str>) -> String {
    deque.into_iter().map(|s| String::from(s)).collect::<Vec<String>>().join(".")
}

pub fn to_vec<T>(deque: &mut VecDeque<T>) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    while let Some(token) = deque.pop_front() {
        result.push(token);
    }

    result
}

pub fn to_display<T: Display>(vec: &[T], sep: &str) -> String {
    vec.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join(sep)
}

pub fn to_string<T: Display>(deque: &mut VecDeque<T>) -> String {
    deque.iter_mut().fold(String::new(), |acc, item| acc.add(item.to_string().as_str()))
}

pub fn bracket_unwrap(input: String) -> String {
    if input.starts_with('(') && input.ends_with(')') {
        input[1..input.len() - 1].to_string()
    } else {
        input
    }
}

pub fn context_unwrap(input: String) -> String {
    if input.starts_with('{') && input.ends_with('}') {
        let stripped = input[1..input.len() - 1].to_string();
        // @Todo: this is hack that must be solved differently
        match stripped.strip_prefix(format!("{} : ", RETURN_EXPRESSION).as_str()) {
            None => stripped,
            Some(end) => end.to_string(),
        }
    } else {
        input
    }
}

#[allow(dead_code)]
pub fn capitalize(s: String) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

static TABS: [&str; 6] = [
    "", "   ", "      ", "         ", "            ", "               ",
];

pub struct Lines {
    ident: usize,
    lines: Vec<String>,
}

pub struct Line {
    text: String,
}

impl Lines {
    pub fn new() -> Self {
        Lines {
            lines: Vec::new(),
            ident: 0,
        }
    }

    // pub fn add_string(&mut self, line_text: String) -> &mut Self {
    //     self.add_str(line_text.as_str());
    //
    //     self
    // }

    pub fn add(&mut self, line: Line) -> &mut Self {
        self.add_str(line.text.as_str());

        self
    }

    fn get_tab(&self) -> String {
        if let Some(tab) = TABS.get(self.ident) {
            tab.to_string()
        } else {
            let mut new = String::new();
            while new.len() < self.ident {
                if self.ident - new.len() > 5 {
                    new = new.add(TABS.get(5).unwrap());
                } else {
                    new = new.add(TABS.get(1).unwrap());
                }
            }
            new
        }
    }

    pub fn add_str(&mut self, text_str: &str) -> &mut Self {
        let mut text = String::new();
        text.push_str(self.get_tab().as_str());
        text.push_str(text_str);
        self.lines.push(text);

        self
    }

    pub fn tab(&mut self) -> &mut Self {
        self.ident = self.ident + 1;
        self
    }

    pub fn back(&mut self) -> &mut Self {
        self.ident = self.ident - 1;
        self
    }

    pub fn to_string(&self) -> String {
        let mut text = String::new();

        for line in &self.lines {
            text.push_str(line.as_str());
            text.push_str("\n");
        }

        text
    }
}

impl Line {
    pub fn new() -> Line {
        Line {
            text: String::new(),
        }
    }

    pub fn add(&mut self, text: &str) -> &mut Self {
        self.text.push_str(text);
        self
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
pub mod test {
    use std::fs;
    use std::io::Write;
    use log::{info};
    use crate::ast::utils::*;
    use std::sync::Once;
    use crate::ast::token::ExpressionEnum;
    use crate::tokenizer::parser::tokenize;
    use crate::typesystem::errors::RuntimeError;
    use crate::typesystem::values::ValueEnum;
    use crate::utils::to_display;
    use regex::Regex;
    
    use std::fmt::Display;
    use std::mem::discriminant;
    
    use log::error;
    use crate::ast::expression::StaticLink;
    
    use crate::runtime::edge_rules::{EdgeRules, EdgeRulesRuntime, ParseErrors};
    use crate::typesystem::errors::{LinkingError, LinkingErrorEnum, ParseErrorEnum};
    
    use crate::typesystem::types::number::NumberEnum;
    
    

    static INIT: Once = Once::new();

    pub fn any_string() -> String {
        "any".to_string()
    }

    pub fn empty_string() -> String {
        "".to_string()
    }

    pub fn init_test(name : &str) {
        init_logger();
        info!(">>> starting test {}", name);
    }

    pub fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder()
                .format(|buf, record| {
                    writeln!(buf, "{}: {}", record.level(), record.args())
                }).init()
        })
    }

    #[allow(dead_code)]
    fn get_code_from_md(filename: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Read the file content
        let content = fs::read_to_string(filename)?;

        // Define the regular expression to match code fragments
        let re = Regex::new(r"```edgerules\n([\s\S]*?)\n```")?;

        // Extract the code fragments and collect them into a Vec<String>
        let fragments: Vec<String> = re.captures_iter(&content)
            .map(|caps| caps[1].to_string())
            .collect();

        Ok(fragments)
    }

    pub fn is_equals(code: &str, expected: &str) {
        let result = &tokenize(&code.to_string());
        let resultLine = array_to_code_sep(result.iter(), ", ");

        if result.len() > 1 {
            panic!("Expected only one token, but got {}.\n text:\n {:?}\n tokens:\n {:?}", result.len(), resultLine, result);
        }

        info!("{:?}", resultLine);
        assert_eq!(expected.replace(" ", ""), resultLine.replace(" ", ""));
    }

    pub fn is_evaluating_to_error(code: &str, target: &mut ExpressionEnum, expected: RuntimeError) {
        test_code(code).expect_runtime_error(target, expected);
    }

    pub fn is_lines_evaluating_to(code: Vec<&str>, variable: &mut ExpressionEnum, expected: ValueEnum) {
        is_evaluating_to(format!("{{{}}}", to_display(&code, "\n")).as_str(), variable, expected);
    }

    pub fn is_this_one_evaluating_to(code: &str, expected: ValueEnum) {
        is_evaluating_to(code, &mut ExpressionEnum::variable("value"), expected);
    }

    pub fn is_variable_evaluating_to(code: &str, variable: &str, expected: ValueEnum) {
        is_evaluating_to(code, &mut ExpressionEnum::variable(variable), expected);
    }

    pub fn is_evaluating_to(code: &str, target: &mut ExpressionEnum, expected: ValueEnum) {
        test_code(code).expect(target, expected);
    }

    pub fn test_code(code: &str) -> TestServiceBuilder {
        TestServiceBuilder::build(code)
    }

    pub fn test_code_lines<T: Display>(code: &[T]) -> TestServiceBuilder {
        TestServiceBuilder::build(format!("{{{}}}", to_display(code, "\n")).as_str())
    }

    pub struct TestServiceBuilder {
        original_code: String,
        runtime: Option<EdgeRulesRuntime>,
        parse_errors: Option<ParseErrors>,
        linking_errors: Option<LinkingError>,
    }

    impl TestServiceBuilder {
        pub fn build(code: &str) -> Self {
            let mut service = EdgeRules::new();

            match service.load_source(code) {
                Ok(_model) => {
                    match service.to_runtime() {
                        Ok(runtime) => {
                            TestServiceBuilder {
                                original_code: code.to_string(),
                                runtime: Some(runtime),
                                parse_errors: None,
                                linking_errors: None,
                            }
                        }
                        Err(linking_errors) => {
                            TestServiceBuilder {
                                original_code: code.to_string(),
                                runtime: None,
                                parse_errors: None,
                                linking_errors: Some(linking_errors),
                            }
                        }
                    }
                }
                Err(errors) => {
                    TestServiceBuilder {
                        original_code: code.to_string(),
                        runtime: None,
                        parse_errors: Some(errors),
                        linking_errors: None,
                    }
                }
            }
        }

        pub fn expect_type(&self, expected_type: &str) -> &Self {
            self.expect_no_errors();

            match &self.runtime {
                None => {
                    panic!("Expected runtime, but got nothing: `{}`", self.original_code);
                }
                Some(runtime) => {
                    assert_eq!(runtime.static_tree.borrow().to_type_string(), expected_type);
                }
            }

            self
        }

        pub fn expect_num(&self, variable: &str, expected: NumberEnum) {
            self.expect(&mut ExpressionEnum::variable(variable), ValueEnum::NumberValue(expected))
        }

        pub fn expect_parse_error(&self, expected: ParseErrorEnum) -> &Self {
            if let Some(errors) = &self.parse_errors {
                for error in errors.errors() {
                    if discriminant(error) == discriminant(&expected) {
                        return self;
                    }
                }
                panic!("Expected parse error `{}`, but got: `{:?}`", expected, errors);
            } else {
                panic!("Expected parse error, but got no errors: `{}`", self.original_code);
            }
        }

        pub fn expect_no_errors(&self) -> &Self {
            if let Some(errors) = &self.parse_errors {
                panic!("Expected no errors, but got parse errors : `{}`\nFailed to parse:\n{}", errors, self.original_code);
            }

            if let Some(errors) = &self.linking_errors {
                panic!("Expected no errors, but got linking errors : `{}`\nFailed to parse:\n{}", errors, self.original_code);
            }

            self
        }

        pub fn expect_link_error(&self, expected: LinkingErrorEnum) -> &Self {
            if let Some(errors) = &self.parse_errors {
                panic!("Expected linking error, but got parse errors : `{:?}`\nFailed to parse:\n{}", errors, self.original_code);
            }

            if let Some(errors) = &self.linking_errors {
                assert_eq!(expected, errors.error, "Testing:\n{}", self.original_code);
            } else {
                panic!("Expected linking error, but got no errors: `{}`", self.original_code);
            }

            self
        }

        pub fn expect_runtime_error(&self, _expr: &mut ExpressionEnum, _expected: RuntimeError) -> &Self {
            if let Some(errors) = &self.parse_errors {
                panic!("Expected runtime error, but got parse errors : `{:?}`\nFailed to parse:\n{}", errors, self.original_code);
            }

            if let Some(errors) = &self.linking_errors {
                panic!("Expected runtime error, but got linking errors : `{:?}`\nFailed to parse:\n{}", errors, self.original_code);
            }

            if let Err(error) = _expr.link(self.runtime.as_ref().unwrap().static_tree.clone()) {
                panic!("Expected runtime error, but got linking errors : `{:?}`\nFailed to parse:\n{}", error, _expr);
            }

            match _expr.eval(self.runtime.as_ref().unwrap().context.clone()) {
                Ok(value) => {
                    panic!("Expected runtime error, but got value : `{:?}`\nEvaluation is fine:\n{}", value, _expr);
                }
                Err(error) => {
                    assert_eq!(error, _expected, "Testing:\n{}", self.original_code);
                }
            }

            return self;
        }

        pub fn expect(&self, _expr: &mut ExpressionEnum, _expected: ValueEnum) {
            self.expect_no_errors();

            if let Err(error) = _expr.link(self.runtime.as_ref().unwrap().static_tree.clone()) {
                panic!("Expected value, but got linking errors : `{:?}`\nFailed to parse:\n{}", error, _expr);
            }

            match _expr.eval(self.runtime.as_ref().unwrap().context.clone()) {
                Ok(value) => {
                    assert_eq!(value, _expected, "Context:\n{}", self.runtime.as_ref().unwrap().context.borrow());
                }
                Err(error) => {
                    error!("{}", error);
                    panic!("Failed to parse: `{:?}`", _expr);
                }
            }
        }

        pub fn expect_code(&self, expected: &str) -> &Self {
            self.expect_no_errors();

            match &self.runtime {
                None => {
                    panic!("Expected code, but got no runtime: `{}`", self.original_code);
                }
                Some(runtime) => {
                    assert_eq!(expected, runtime.context.borrow().to_code());
                }
            }
            self
        }

        pub fn expect_code_contains(&self, expected: &str) -> &Self {
            self.expect_no_errors();

            match &self.runtime {
                None => {
                    panic!("Expected code, but got no runtime: `{}`", self.original_code);
                }
                Some(runtime) => {
                    assert!(runtime.context.borrow().to_code().contains(expected), "Expected code to contain `{}` but got:\n{}", expected, runtime.context.borrow().to_code());
                }
            }
            self
        }
    }
}

#[macro_export]
macro_rules! error_token {
    ($($arg:tt)*) => {{
        EToken::ParseError($crate::typesystem::errors::ParseErrorEnum::UnknownParseError(format!($($arg)*)))
    }}
}
