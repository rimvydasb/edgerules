use std::env;
use std::fs;
use std::io::{self, Read};

use edge_rules::code_to_trace;
use edge_rules::runtime::edge_rules::EdgeRules;

fn main() {
    // Usage:
    // 1) Pass code as a single argument: `edgerules-wasi "{ value : 1 + 2 }"`
    // 2) Pass @file to read code from a file: `edgerules-wasi @path/to/file.txt`
    // 3) No args: read entire stdin as code

    let args: Vec<String> = env::args().skip(1).collect();

    let code = if let Some(first) = args.first() {
        if let Some(path) = first.strip_prefix('@') {
            fs::read_to_string(path).expect("Failed to read file")
        } else {
            args.join(" ")
        }
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).expect("Failed to read stdin");
        buf
    };

    // Try to evaluate `value` if present; otherwise print a full trace.
    // Fall back to full trace if evaluation fails.
    match eval_value(&code) {
        Ok(Some(output)) => {
            println!("{}", output);
        }
        _ => {
            println!("{}", code_to_trace(&code));
        }
    }
}

fn eval_value(code: &str) -> Result<Option<String>, String> {
    let mut service = EdgeRules::new();
    service.load_source(code).map_err(|e| e.to_string())?;
    let runtime = service.to_runtime().map_err(|e| e.to_string())?;

    match runtime.evaluate_field("value") {
        Ok(val) => Ok(Some(format!("{}", val))),
        Err(_) => Ok(None),
    }
}

