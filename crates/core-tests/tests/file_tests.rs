use edge_rules::runtime::edge_rules::EdgeRulesModel;
use std::fs;
use std::io::Write;
mod test_utils;
use test_utils::test::init_test;

fn process_file(input_file_name: &str) -> std::io::Result<()> {
    let mut edgerules = EdgeRulesModel::new();
    let output_file_name = format!("{}.out", input_file_name);
    let input = fs::read_to_string(input_file_name)?;
    let mut output_file = fs::File::create(output_file_name)?;
    let result = match edgerules.append_source(&input) {
        Ok(()) => match edgerules.to_runtime() {
            Ok(runtime) => match runtime.eval_all() {
                Ok(()) => runtime.context.borrow().to_code(),
                Err(err) => err.to_string(),
            },
            Err(err) => err.to_string(),
        },
        Err(err) => err.to_string(),
    };
    output_file.write_all(result.as_bytes())?;

    Ok(())
}

#[test]
fn to_code_test() -> std::io::Result<()> {
    init_test("to code");
    process_file("tests/valid/filters.txt")?;
    Ok(())
}
