extern crate core;
extern crate log;
use crate::runtime::edge_rules::EdgeRules;
use crate::utils::to_display;
mod ast;
mod link;
pub mod runtime;
mod tokenizer;
mod typesystem;
pub mod utils;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub fn code_to_trace(code: &str) -> String {
    let mut engine = EdgeRules::new();
    match engine.load_source(code) {
        Ok(_service) => match engine.to_runtime() {
            Ok(runtime) => match runtime.eval_all() {
                Ok(()) => runtime.context.borrow().to_code(),
                Err(error) => error.to_string(),
            },
            Err(error) => error.to_string(),
        },
        Err(error) => to_display(error.errors(), "\n"),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use crate::code_to_trace;
    use crate::utils::test::{init_logger, init_test};
    use log::trace;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    fn process_file(input_file_name: &str) -> std::io::Result<()> {
        let output_file_name = format!("{}.out", input_file_name);
        let input = fs::read_to_string(input_file_name)?;
        let mut output_file = fs::File::create(output_file_name)?;
        output_file.write_all(code_to_trace(&input).as_bytes())?;

        Ok(())
    }

    #[test]
    fn to_code_test() -> std::io::Result<()> {
        init_test("to code");
        process_file("tests/valid/filters.txt")?;
        Ok(())
    }

    #[test]
    fn file_test() {
        // {
        //     let data = fs::read_to_string("tests/nested_1.txt").expect("Unable to read file");
        //     let trace = code_to_trace(data.as_str());
        //
        //     debug!("{}", &trace);
        //     assert_eq!(true, trace.contains("assignment side is not complete"))
        // }

        init_logger();

        {
            let _data = fs::read_to_string("tests/errors/error1.txt").expect("Unable to read file");
            //let trace = code_to_trace(data.as_str());

            //debug!("{}", &trace);
            //assert_eq!(true, trace.contains("assignment side is not complete"))
        }

        {
            let _data = fs::read_to_string("tests/record_1.txt").expect("Unable to read file");

            trace!("--------------------------------------------------------------");
            //trace!("{}", code_to_trace(data.as_str()));
            trace!("--------------------------------------------------------------");
            //assert_eq!(true, code_to_trace(data.as_str()).contains("operator side is not complete"))
        }
    }
}
