use std::env;
use std::fs;
use std::io::{self, Read};

use edge_rules::runtime::edge_rules::EdgeRulesModel;

pub fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();

    let code = read_code(&args)?;

    match eval_value(&code) {
        Ok(Some(output)) => {
            println!("{}", output);
        }
        _ => {
            let mut service = EdgeRulesModel::new();
            match service.append_source(&code) {
                Ok(()) => match service.to_runtime() {
                    Ok(runtime) => match runtime.eval_all() {
                        Ok(()) => println!("{}", runtime.context.borrow().to_code()),
                        Err(error) => println!("{}", error),
                    },
                    Err(error) => println!("{}", error),
                },
                Err(error) => println!("{}", error),
            }
        }
    }

    Ok(())
}

fn eval_value(code: &str) -> Result<Option<String>, String> {
    let mut service = EdgeRulesModel::new();
    service.append_source(code).map_err(|e| e.to_string())?;
    let runtime = service.to_runtime().map_err(|e| e.to_string())?;

    match runtime.evaluate_field("value") {
        Ok(val) => Ok(Some(format!("{}", val))),
        Err(_) => Ok(None),
    }
}

fn read_code(args: &[String]) -> Result<String, String> {
    if let Some(first) = args.first() {
        if let Some(path) = first.strip_prefix('@') {
            fs::read_to_string(path).map_err(|error| format!("Failed to read file '{}': {}", path, error))
        } else {
            Ok(args.join(" "))
        }
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| format!("Failed to read stdin: {}", error))?;
        Ok(buffer)
    }
}
