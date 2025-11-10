// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use proc_macro2::TokenStream;
use std::{fs, io};

pub fn include_textile(item: TokenStream) -> syn::Result<TokenStream> {
    super::include_file(item, collect::<fs::File>)
}

fn collect<R: io::Read>(name: &str, iter: io::Lines<io::BufReader<R>>) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut in_block = false;
    let mut is_double_period = false;

    for line in iter {
        let line = line?;

        if !in_block {
            // Look for a code block starting with bc(rust#name). or bc(rust#name)..
            let trimmed = line.trim();

            if trimmed.starts_with("bc(rust#") && has_matching_id(trimmed, name) {
                is_double_period = trimmed.contains("..");

                // Extract content after the first space on the same line
                // Code MUST start on the same line in Textile
                if let Some(space_pos) = trimmed.find(' ') {
                    let content = &trimmed[space_pos + 1..];
                    if !content.is_empty() {
                        lines.push(content.to_string());
                        in_block = true;
                    }
                }
            }
        } else {
            let trimmed = line.trim();

            if is_double_period {
                // Double period: collect until next block command (text followed by .)
                if is_block_tag(trimmed) {
                    // Remove trailing empty lines before the block tag
                    while let Some(last) = lines.last() {
                        if last.trim().is_empty() {
                            lines.pop();
                        } else {
                            break;
                        }
                    }
                    break;
                }
                // Collect the line
                lines.push(line);
            } else {
                // Single period: collect until first blank line
                if trimmed.is_empty() {
                    break;
                }
                // Collect the line
                lines.push(line);
            }
        }
    }

    Ok(lines)
}

fn has_matching_id(line: &str, name: &str) -> bool {
    // Look for bc(rust#name). or bc(rust#name)..
    // Examples: bc(rust#example).
    //           bc(rust#example)..

    let pattern = format!("bc(rust#{})", name);
    if let Some(pos) = line.find(&pattern) {
        let after_pattern = &line[pos + pattern.len()..];
        // Check if followed by . or ..
        return after_pattern.starts_with('.') || after_pattern.starts_with("..");
    }

    false
}

fn is_block_tag(line: &str) -> bool {
    // Check if line starts a new textile block
    // Common block tags: p., bq., bc., h1., h2., etc.
    if line.is_empty() {
        return false;
    }

    // Check for common block patterns
    let patterns = [
        "p.", "bq.", "bc.", "bc(", "h1.", "h2.", "h3.", "h4.", "h5.", "h6.", "###.", "table.",
        "pre.",
    ];

    for pattern in &patterns {
        if line.starts_with(pattern) {
            return true;
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
    fn extract_no_code_blocks() {
        let content = r#"This is a Textile file
with no code blocks at all.
Just plain text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_no_matching_id() {
        let content = r#"Some text here.

bc(rust#other). fn main() {
    println!("Hello");
}

p. More text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_single_period_with_content() {
        let content = r#"Some introduction text.

bc(rust#example). println!("hello, world!");

p. Text after the block."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, r#"println!("hello, world!");"#);
    }

    #[test]
    fn extract_double_period_multiline() {
        let content = r#"Some introduction text.

bc(rust#example).. fn test() {
    assert_eq!(2 + 2, 4);
}

p. Text after the block."#;
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
    fn extract_multiline_until_next_block() {
        let content = r#"Some introduction text.

bc(rust#example).. let x = 42;
let y = x + 1;

bq. This is a quote block that ends the code."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"let x = 42;
let y = x + 1;"#
        );
    }

    #[test]
    fn extract_multiple_blocks_one_match() {
        let content = r#"Here's the first block:

bc(python#other). print("Not this one")

p. And here's the one we want:

bc(rust#example). println!("This is the one!");

p. And another one."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, r#"println!("This is the one!");"#);
    }

    #[test]
    fn extract_content_on_next_line() {
        let content = r#"Text before.

bc(rust#example).. fn with_content() {
    println!("On next line");
}

p. Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn with_content() {
    println!("On next line");
}"#
        );
    }

    #[test]
    fn extract_with_class_before_id() {
        let content = r#"Text before.

bc(rust#example). let value = 123;

p. Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, "let value = 123;");
    }

    #[test]
    fn extract_until_eof() {
        let content = r#"Text before.

bc(rust#example).. struct Point {
    x: i32,
    y: i32,
}"#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"struct Point {
    x: i32,
    y: i32,
}"#
        );
    }

    #[test]
    fn extract_empty_lines_within_block() {
        let content = r#"Text before.

bc(rust#example).. fn first() {}

fn second() {}

p. Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn first() {}

fn second() {}"#
        );
    }

    #[test]
    fn extract_different_language() {
        let content = r#"Text before.

bc(python#example). print("hello")

p. Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }
}
