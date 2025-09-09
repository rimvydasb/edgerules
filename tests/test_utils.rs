#[cfg(test)]
#[allow(non_snake_case)]
pub mod test {
    use crate::ast::token::ExpressionEnum;
    use crate::ast::utils::*;
    use crate::tokenizer::parser::tokenize;
    use crate::typesystem::values::ValueEnum;
    use crate::utils::to_display;
    use log::info;
    use regex::Regex;
    use std::fs;
    use std::io::Write;
    use std::sync::Once;

    use std::fmt::Display;
    use std::mem::discriminant;

    use crate::ast::expression::StaticLink;
    use log::error;

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

    pub fn init_test(name: &str) {
        init_logger();
        info!(">>> starting test {}", name);
    }

    pub fn init_logger() {
        INIT.call_once(|| {
            env_logger::builder()
                .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
                .init()
        })
    }

    #[allow(dead_code)]
    fn get_code_from_md(filename: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Read the file content
        let content = fs::read_to_string(filename)?;

        // Define the regular expression to match code fragments
        let re = Regex::new(r"```edgerules\n([\s\S]*?)\n```")?;

        // Extract the code fragments and collect them into a Vec<String>
        let fragments: Vec<String> = re
            .captures_iter(&content)
            .map(|caps| caps[1].to_string())
            .collect();

        Ok(fragments)
    }

    pub fn is_equals(code: &str, expected: &str) {
        let result = &tokenize(&code.to_string());
        let resultLine = array_to_code_sep(result.iter(), ", ");

        if result.len() > 1 {
            panic!(
                "Expected only one token, but got {}.\n text:\n {:?}\n tokens:\n {:?}",
                result.len(),
                resultLine,
                result
            );
        }

        info!("{:?}", resultLine);
        assert_eq!(expected.replace(" ", ""), resultLine.replace(" ", ""));
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
                    assert_eq!(runtime.static_tree.borrow().to_type_string(), expected_type);
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
                    "Expected no errors, but got parse errors : `{}`\nFailed to parse:\n{}",
                    errors, self.original_code
                );
            }

            if let Some(errors) = &self.linking_errors {
                panic!(
                    "Expected no errors, but got linking errors : `{}`\nFailed to parse:\n{}",
                    errors, self.original_code
                );
            }

            self
        }

        pub fn expect_link_error(&self, expected: LinkingErrorEnum) -> &Self {
            if let Some(errors) = &self.parse_errors {
                panic!(
                    "Expected linking error, but got parse errors : `{:?}`\nFailed to parse:\n{}",
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
                    "Expected value, but got linking errors : `{:?}`\nFailed to parse:\n{}",
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
}