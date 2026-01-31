extern crate proc_macro;
use std::fmt::Display;

pub fn array_to_code_sep(parts: impl Iterator<Item = impl Display>, sep: &str) -> String {
    parts.map(|s| format!("{}", s)).collect::<Vec<String>>().join(sep)
}

pub fn trim(s: &str, start: char, end: char) -> &str {
    let mut start_idx = 0;
    let mut end_idx = s.len() - 1;

    if s.chars().nth(start_idx) == Some(start) && s.chars().nth(end_idx) == Some(end) {
        start_idx += 1;
        end_idx -= 1;
    }

    &s[start_idx..=end_idx]
}
