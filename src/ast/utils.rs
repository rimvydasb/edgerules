use std::fmt::Display;
use crate::typesystem::errors::RuntimeError;

extern crate proc_macro;

pub fn array_to_code_sep(parts: impl Iterator<Item=impl Display>, sep: &str) -> String {

    parts.map(|s| format!("{}", s))
        .collect::<Vec<String>>()
        .join(sep)
}

pub fn results_to_code(parts: &Vec<Result<impl Display, RuntimeError>>) -> String {
    parts
        .iter()
        .map(|s| {
            match s {
                Ok(p) => format!("{}", p),
                Err(p) => format!("{}", p)
            }
        }).collect::<Vec<String>>()
        .join(", ")
}
