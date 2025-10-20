use std::fmt::Display;

extern crate proc_macro;

pub fn array_to_code_sep(parts: impl Iterator<Item = impl Display>, sep: &str) -> String {
    parts
        .map(|s| format!("{}", s))
        .collect::<Vec<String>>()
        .join(sep)
}
