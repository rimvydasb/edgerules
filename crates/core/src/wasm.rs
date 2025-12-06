#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

use wasm_bindgen::prelude::*;

// Inline JS glue to leverage host RegExp and base64 on Web/Node without pulling heavier Rust
// crates into the WASM build.
#[wasm_bindgen(inline_js = r#"
export function __er_regex_replace(s, pattern, flags, repl) {
  try {
    const re = new RegExp(pattern, flags || 'g');
    return String(s).replace(re, repl);
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_regex_split(s, pattern, flags) {
  try {
    const re = new RegExp(pattern, flags || 'g');
    const SEP = "\u001F"; // Unit Separator as rarely-used delimiter
    const parts = String(s).split(re).map(p => p.split(SEP).join(SEP + SEP));
    return parts.join(SEP);
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_to_base64(s) {
  try {
    if (typeof btoa === 'function') {
      return btoa(String(s));
    }
    // Node.js
    return Buffer.from(String(s), 'utf-8').toString('base64');
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}

export function __er_from_base64(s) {
  try {
    if (typeof atob === 'function') {
      return atob(String(s));
    }
    // Node.js
    return Buffer.from(String(s), 'base64').toString('utf-8');
  } catch (e) {
    return "__er_err__:" + String(e);
  }
}
"#)]
extern "C" {
    fn __er_regex_replace(s: &str, pattern: &str, flags: &str, repl: &str) -> String;
    fn __er_regex_split(s: &str, pattern: &str, flags: &str) -> String;
    fn __er_to_base64(s: &str) -> String;
    fn __er_from_base64(s: &str) -> String;
}

pub(crate) fn regex_replace_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
    repl: &str,
) -> Result<String, String> {
    let f = flags.unwrap_or("g");
    let out = __er_regex_replace(s, pattern, f, repl);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

pub(crate) fn regex_split_js(
    s: &str,
    pattern: &str,
    flags: Option<&str>,
) -> Result<Vec<String>, String> {
    let f = flags.unwrap_or("g");
    let out = __er_regex_split(s, pattern, f);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        let sep = '\u{001F}';
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut chars = out.chars().peekable();
        while let Some(c) = chars.next() {
            if c == sep {
                if let Some(next) = chars.peek() {
                    if *next == sep {
                        current.push(sep);
                        chars.next();
                        continue;
                    }
                }
                parts.push(current);
                current = String::new();
            } else {
                current.push(c);
            }
        }
        parts.push(current);
        Ok(parts)
    }
}

pub(crate) fn to_base64_js(s: &str) -> Result<String, String> {
    let out = __er_to_base64(s);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}

pub(crate) fn from_base64_js(s: &str) -> Result<String, String> {
    let out = __er_from_base64(s);
    if let Some(msg) = out.strip_prefix("__er_err__:") {
        Err(msg.to_string())
    } else {
        Ok(out)
    }
}
