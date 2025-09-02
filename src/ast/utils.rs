use crate::typesystem::errors::RuntimeError;
use std::fmt::Display;

extern crate proc_macro;

pub fn array_to_code_sep(parts: impl Iterator<Item = impl Display>, sep: &str) -> String {
    parts
        .map(|s| format!("{}", s))
        .collect::<Vec<String>>()
        .join(sep)
}

pub fn results_to_code(parts: &[Result<impl Display, RuntimeError>]) -> String {
    parts
        .iter()
        .map(|s| match s {
            Ok(p) => format!("{}", p),
            Err(p) => format!("{}", p),
        })
        .collect::<Vec<String>>()
        .join(", ")
}
