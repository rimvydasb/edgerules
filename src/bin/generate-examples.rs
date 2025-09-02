use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use edge_rules::code_to_trace;

// Generates tests/EXAMPLES-output.md by executing `edgerules` code blocks
// in tests/EXAMPLES.md and writing results into the following placeholder
// blocks that start with `// output`.
fn main() -> std::io::Result<()> {
    let input_path = Path::new("tests/EXAMPLES.md");
    let output_path = Path::new("tests/EXAMPLES-output.md");

    let input = fs::File::open(input_path)?;
    let reader = BufReader::new(input);

    let mut out = fs::File::create(output_path)?;

    let mut in_code_block = false;
    let mut code_lang_is_edgerules = false;
    let mut current_block: Vec<String> = Vec::new();
    let mut pending_eval_output: Option<String> = None;

    // Helper to flush a completed code block to output, possibly replacing
    // placeholder blocks with evaluated output.
    let flush_block = |out: &mut fs::File,
                       code_lang_is_edgerules: bool,
                       block_lines: &mut Vec<String>,
                       pending_eval_output: &mut Option<String>|
     -> std::io::Result<()> {
        if code_lang_is_edgerules {
            // Determine if this is an output placeholder block (first non-empty trimmed line is `// output`).
            let mut first_non_empty: Option<&str> = None;
            for l in block_lines.iter() {
                let t = l.trim();
                if !t.is_empty() {
                    first_non_empty = Some(t);
                    break;
                }
            }

            let is_placeholder = matches!(first_non_empty, Some("// output"));

            if is_placeholder {
                // Replace entire block content with the evaluated output if available.
                writeln!(out, "```edgerules")?;
                if let Some(result) = pending_eval_output.take() {
                    for line in result.lines() {
                        writeln!(out, "{}", line)?;
                    }
                } else {
                    // No pending output; keep placeholder comment for visibility.
                    writeln!(out, "// output (no evaluation available)")?;
                }
                writeln!(out, "```")?;
                return Ok(());
            }

            // Not a placeholder: this is an example input block. Evaluate and store for the next placeholder.
            let code = block_lines.join("\n");
            let result = code_to_trace(&code);
            *pending_eval_output = Some(result);
        }

        // Write the original block (for non-placeholder or non-edgerules blocks)
        if code_lang_is_edgerules {
            writeln!(out, "```edgerules")?;
        } else {
            writeln!(out, "```")?; // generic fence if we ever enter non-edgerules; will be adjusted by start line
        }
        for l in block_lines.iter() {
            writeln!(out, "{}", l)?;
        }
        writeln!(out, "```")?;
        Ok(())
    };

    // We stream lines and reconstruct output, handling code fences carefully.
    for line in reader.lines() {
        let line = line?;

        // Detect code fence starts/ends.
        if !in_code_block {
            // Code fence start?
            if line.starts_with("```") {
                in_code_block = true;
                code_lang_is_edgerules = line.trim() == "```edgerules";
                current_block.clear();
                // Write the line back to output only for non-edgerules blocks after flush; here we defer writing.
                if !code_lang_is_edgerules {
                    // For non-edgerules blocks, we will just mirror them verbatim.
                    writeln!(out, "{}", line)?;
                }
            } else {
                // Outside code block: copy through
                writeln!(out, "{}", line)?;
            }
        } else {
            // Inside a code block
            if line.trim() == "```" {
                // Closing block: flush
                if code_lang_is_edgerules {
                    flush_block(&mut out, true, &mut current_block, &mut pending_eval_output)?;
                } else {
                    // Non-edgerules block: mirror
                    for l in current_block.iter() {
                        writeln!(out, "{}", l)?;
                    }
                    writeln!(out, "```")?;
                }
                in_code_block = false;
                code_lang_is_edgerules = false;
                current_block.clear();
            } else {
                current_block.push(line);
            }
        }
    }

    // If file ended within a code block, close it gracefully (unlikely in well-formed docs)
    if in_code_block {
        if code_lang_is_edgerules {
            flush_block(&mut out, true, &mut current_block, &mut pending_eval_output)?;
        } else {
            for l in current_block.iter() {
                writeln!(out, "{}", l)?;
            }
            writeln!(out, "```")?;
        }
    }

    Ok(())
}
