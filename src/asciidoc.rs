// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use proc_macro2::TokenStream;
use std::{fs, io, path::PathBuf};

pub fn include_asciidoc(item: TokenStream, root: Option<PathBuf>) -> syn::Result<TokenStream> {
    super::include_file(item, root, collect::<fs::File>)
}

fn collect<R: io::Read>(name: &str, iter: io::Lines<io::BufReader<R>>) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut in_block = false;
    let mut delimiter_checked = false;
    let mut use_delimiters = false;

    for line in iter {
        let line = line?;

        if !in_block {
            // Look for a source block attribute line like [source,rust] or [,rust]
            let trimmed = line.trim();

            // Check if this is a source block declaration
            if trimmed.starts_with("[source,rust") || trimmed.starts_with("[,rust") {
                // Check if it contains the matching id attribute
                if has_matching_id(trimmed, name) {
                    in_block = true;
                    // Next line will determine if we use delimiters
                }
            }
        } else if !delimiter_checked {
            // First line after the attribute line - check if it's a delimiter
            delimiter_checked = true;
            if line.trim() == "----" {
                use_delimiters = true;
                continue; // Don't collect the opening delimiter
            } else {
                // Not using delimiters, check if this line should be collected
                if line.trim().is_empty() || line.trim() == "----" {
                    // Empty line or ---- (from outer block) means end of non-delimited block
                    break;
                }
                lines.push(line);
            }
        } else if use_delimiters {
            // We're using delimiters, collect until closing ----
            if line.trim() == "----" {
                // Found closing delimiter
                break;
            }
            lines.push(line);
        } else {
            // Not using delimiters, collect until blank line or ----
            if line.trim().is_empty() || line.trim() == "----" {
                // Found blank line or ---- (from outer block), stop collecting
                break;
            }
            lines.push(line);
        }
    }

    Ok(lines)
}

fn has_matching_id(line: &str, name: &str) -> bool {
    // Look for id="name" in the attribute line
    // Examples: [source,rust,id="example"]
    //           [,rust,id="example"]

    if let Some(id_pos) = line.find("id=") {
        let after_id = &line[id_pos + 3..];

        // Check if id value is quoted
        if let Some(after_quote) = after_id.strip_prefix('"') {
            // Find closing quote
            if let Some(end_quote) = after_quote.find('"') {
                let id_value = &after_quote[..end_quote];
                return id_value == name;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::collect;
    use crate::extract;
    use std::io;

    #[test]
    fn extract_no_source_blocks() {
        let content = r#"This is an AsciiDoc file
with no source blocks at all.
Just plain text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_no_matching_id() {
        let content = r#"Some text here.

[source,rust]
----
fn main() {
    println!("Hello");
}
----

More text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_source_rust_with_delimiters() {
        let content = r#"Some introduction text.

[source,rust,id="example"]
----
fn main() {
    println!("Hello, world!");
}
----

Text after the block."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn main() {
    println!("Hello, world!");
}"#
        );
    }

    #[test]
    fn extract_shorthand_rust_with_delimiters() {
        let content = r#"Some introduction text.

[,rust,id="example"]
----
fn test() {
    assert_eq!(2 + 2, 4);
}
----

Text after the block."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn test() {
    assert_eq!(2 + 2, 4);
}"#
        );
    }

    #[test]
    fn extract_source_rust_without_delimiters() {
        let content = r#"Some introduction text.

[source,rust,id="example"]
let x = 42;
let y = x + 1;

This text should not be included.
More text here."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"let x = 42;
let y = x + 1;"#
        );
    }

    #[test]
    fn extract_shorthand_rust_without_delimiters() {
        let content = r#"Some introduction text.

[,rust,id="example"]
fn inline() {
    println!("No delimiters");
}

This text after the blank line should not be included.
Neither should this."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn inline() {
    println!("No delimiters");
}"#
        );
    }

    #[test]
    fn extract_multiple_blocks_one_match() {
        let content = r#"Here's the first block:

[source,python,id="other"]
----
print("Not this one")
----

And here's the one we want:

[,rust,id="example"]
----
fn main() {
    println!("This is the one!");
}
----

And another one:

[source,java]
----
System.out.println("Also not this one");
----"#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn main() {
    println!("This is the one!");
}"#
        );
    }

    #[test]
    fn extract_nested_delimiters() {
        let content = r#"Outer content:

[source,rust,id="example"]
----
// Comment with ---- in it
fn nested() {
    let s = "----";
    println!("{}", s);
}
----

After the block."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"// Comment with ---- in it
fn nested() {
    let s = "----";
    println!("{}", s);
}"#
        );
    }

    #[test]
    fn extract_id_in_middle() {
        let content = r#"Text before.

[,rust,id="example",role="highlight"]
----
fn with_attributes() {}
----

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, "fn with_attributes() {}");
    }

    #[test]
    fn extract_empty_lines_within_delimiters() {
        let content = r#"Text before.

[,rust,id="example"]
----
fn first() {}

fn second() {}
----

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn first() {}

fn second() {}"#
        );
    }

    #[test]
    fn extract_within_outer_code_block() {
        let content = r####"Text before.

[,asciidoc]
----
[,rust,id="example"]
let m = example()?;
assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
----

Text after."####;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r###"let m = example()?;
assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);"###
        );
    }
}
