use crate::ast::context::function_context::RETURN_EXPRESSION;
use std::collections::vec_deque::VecDeque;
use std::fmt::Display;
use std::ops::Add;

pub fn to_display<T: Display>(vec: &[T], sep: &str) -> String {
    vec.iter()
        .map(|s| format!("{}", s))
        .collect::<Vec<String>>()
        .join(sep)
}

pub fn to_string<T: Display>(deque: &mut VecDeque<T>) -> String {
    deque.iter_mut().fold(String::new(), |acc, item| {
        acc.add(item.to_string().as_str())
    })
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
    "",
    "   ",
    "      ",
    "         ",
    "            ",
    "               ",
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
        self.ident += 1;
        self
    }

    pub fn back(&mut self) -> &mut Self {
        self.ident -= 1;
        self
    }
}

impl Default for Lines {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Lines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
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

impl Default for Line {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! error_token {
    ($($arg:tt)*) => {{
        EToken::ParseError($crate::typesystem::errors::ParseErrorEnum::UnknownParseError(format!($($arg)*)))
    }}
}

#[cfg(test)]
#[allow(non_snake_case)]
pub mod test {
    use log::info;
    use std::io::Write;
    use std::sync::Once;

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
}
