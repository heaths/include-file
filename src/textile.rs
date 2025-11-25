// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

// cspell:ignore notextile peekable myclass

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
            // or bc[rust](#name). or bc(#name)[rust].
            let trimmed = line.trim();

            if trimmed.starts_with("bc") && has_matching_id(trimmed, name) {
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
    // Look for bc(rust#name). or bc(rust#name).. or bc[rust](#name). or bc(#name)[rust].
    // Examples: bc(rust#example).
    //           bc(rust#example)..
    //           bc[rust](#example).
    //           bc(#example)[rust]..

    // Pattern 1: bc(rust#name)
    let pattern1 = format!("bc(rust#{})", name);
    if let Some(pos) = line.find(&pattern1) {
        let after_pattern = &line[pos + pattern1.len()..];
        // Check if followed by . or ..
        if after_pattern.starts_with('.') || after_pattern.starts_with("..") {
            return true;
        }
    }

    // Pattern 2: bc[rust](#name)
    let pattern2 = format!("bc[rust](#{})", name);
    if let Some(pos) = line.find(&pattern2) {
        let after_pattern = &line[pos + pattern2.len()..];
        // Check if followed by . or ..
        if after_pattern.starts_with('.') || after_pattern.starts_with("..") {
            return true;
        }
    }

    // Pattern 3: bc(#name)[rust]
    let pattern3 = format!("bc(#{})[rust]", name);
    if let Some(pos) = line.find(&pattern3) {
        let after_pattern = &line[pos + pattern3.len()..];
        // Check if followed by . or ..
        if after_pattern.starts_with('.') || after_pattern.starts_with("..") {
            return true;
        }
    }

    false
}

fn is_block_tag(line: &str) -> bool {
    // Check if line starts a new textile block
    // Block tags can have formatting characters like: p<., h1>., table(class)., etc.
    // Valid formatting chars before period: <, >, =, and content in () or []
    if line.is_empty() {
        return false;
    }

    // Known block types
    let block_types = [
        "p",
        "bq",
        "bc",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "table",
        "pre",
        "notextile",
    ];

    for block_type in &block_types {
        if let Some(after_block) = line.strip_prefix(block_type) {
            // Block tag must be followed by valid formatting chars and then a period
            // Valid chars: <, >, =, and content within () or []
            let mut chars = after_block.chars().peekable();
            let mut found_period = false;

            while let Some(ch) = chars.next() {
                match ch {
                    '.' => {
                        found_period = true;
                        break;
                    }
                    '<' | '>' | '=' => {
                        // Valid alignment/formatting character, continue
                    }
                    '(' => {
                        // Parentheses can be:
                        // 1. Empty or for indentation: p(. or p((.
                        // 2. With content: table(myclass).
                        // Look ahead to check if there's content before ) or .
                        let mut depth = 1;
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch == '.' {
                                // Period found, this paren is just formatting
                                break;
                            } else if next_ch == ')' {
                                chars.next(); // consume )
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            } else if next_ch == '(' {
                                chars.next(); // consume (
                                depth += 1;
                            } else {
                                chars.next(); // consume content character
                            }
                        }
                    }
                    ')' => {
                        // Closing paren for indentation styling
                    }
                    '[' => {
                        // Consume everything until matching ]
                        for inner_ch in chars.by_ref() {
                            if inner_ch == ']' {
                                break;
                            }
                        }
                    }
                    _ => {
                        // Invalid character before period
                        break;
                    }
                }
            }

            if found_period {
                return true;
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

bc[rust](#example). println!("hello, world!");

p. Text after the block."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, r#"println!("hello, world!");"#);
    }
    #[test]
    fn extract_double_period_multiline() {
        let content = r#"Some introduction text.

bc(#example)[rust].. fn test() {
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

bc[rust](#example).. let x = 42;
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

bc(#example)[rust]. println!("This is the one!");

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

bc[rust](#example). let value = 123;

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

    #[test]
    fn extract_ends_at_right_aligned_header() {
        let content = r#"Text before.

bc(rust#example).. let x = 1;
let y = 2;

h1>. Right Aligned Header"#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"let x = 1;
let y = 2;"#
        );
    }

    #[test]
    fn extract_ends_at_justified_paragraph() {
        let content = r#"Text before.

bc(rust#example).. fn test() {
    println!("test");
}

p<>. Justified paragraph text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn test() {
    println!("test");
}"#
        );
    }

    #[test]
    fn extract_ends_at_centered_header() {
        let content = r#"Text before.

bc(rust#example).. let value = 42;

h2=. Centered Header"#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, "let value = 42;");
    }

    #[test]
    fn extract_ends_at_indented_paragraph() {
        let content = r#"Text before.

bc(rust#example).. struct Point {
    x: i32,
}

p(. Indented paragraph."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"struct Point {
    x: i32,
}"#
        );
    }

    #[test]
    fn extract_ends_at_table_with_class() {
        let content = r#"Text before.

bc(rust#example).. let data = vec![1, 2, 3];

table(myclass). Some table content"#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, "let data = vec![1, 2, 3];");
    }

    #[test]
    fn extract_ends_at_notextile_block() {
        let content = r#"Text before.

bc(rust#example).. fn example() {
    println!("code");
}

notextile. Raw content here."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn example() {
    println!("code");
}"#
        );
    }

    #[test]
    fn extract_bc_with_bracket_syntax() {
        let content = r#"Text before.

bc[rust](#example). let value = 100;

p. Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, "let value = 100;");
    }

    #[test]
    fn extract_ends_at_right_padded_paragraph() {
        let content = r#"Text before.

bc(rust#example).. let x = 10;
let y = 20;

p))). Right padded paragraph."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"let x = 10;
let y = 20;"#
        );
    }

    #[test]
    fn extract_ends_at_combined_padding_paragraph() {
        let content = r#"Text before.

bc(rust#example).. fn test() {
    return 42;
}

p()). Left indent and right padding."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn test() {
    return 42;
}"#
        );
    }
}
