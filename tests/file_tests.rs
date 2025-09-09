use std::fs;
use std::io::Write;
use edge_rules::runtime::edge_rules::EdgeRules;
mod test_utils;
use test_utils::test::init_test;

fn process_file(input_file_name: &str) -> std::io::Result<()> {
    let edgerules = EdgeRules::new();
    let output_file_name = format!("{}.out", input_file_name);
    let input = fs::read_to_string(input_file_name)?;
    let mut output_file = fs::File::create(output_file_name)?;
    output_file.write_all(edgerules.evaluate_all(&input).as_bytes())?;

    Ok(())
}

#[test]
fn to_code_test() -> std::io::Result<()> {
    init_test("to code");
    process_file("tests/valid/filters.txt")?;
    Ok(())
}
