// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use proc_macro2::TokenStream;
use std::{fs, io};

pub fn include_org(item: TokenStream) -> syn::Result<TokenStream> {
    super::include_file(item, collect::<fs::File>)
}

fn collect<R: io::Read>(name: &str, iter: io::Lines<io::BufReader<R>>) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut in_block = false;
    let mut found_name = false;

    for line in iter {
        let line = line?;

        if !in_block {
            let trimmed = line.trim();

            // Look for #+NAME: immediately before #+BEGIN_SRC (case-insensitive)
            if trimmed.len() >= 7
                && trimmed[..7].eq_ignore_ascii_case("#+NAME:")
                && has_matching_name(trimmed, name)
            {
                found_name = true;
            } else if found_name
                && trimmed.len() >= 11
                && trimmed[..11].eq_ignore_ascii_case("#+BEGIN_SRC")
                && is_rust_block(trimmed)
            {
                in_block = true;
                found_name = false;
            } else if found_name {
                // Reset if we see any line that's not BEGIN_SRC after finding a name
                // This ensures NAME must be immediately before BEGIN_SRC
                found_name = false;
            }
        } else {
            let trimmed = line.trim();

            // Check for end of block (case-insensitive)
            if trimmed.len() >= 9 && trimmed[..9].eq_ignore_ascii_case("#+END_SRC") {
                break;
            }

            // Collect the line
            lines.push(line);
        }
    }

    Ok(lines)
}

fn has_matching_name(line: &str, name: &str) -> bool {
    // Look for #+NAME: followed by whitespace and the name (case-insensitive)
    // Example: #+NAME: example or #+name: example
    let trimmed = line.trim();
    if trimmed.len() >= 7 && trimmed[..7].eq_ignore_ascii_case("#+NAME:") {
        let rest = trimmed[7..].trim_start();
        // Check if the rest matches the name exactly (no extra characters after)
        return rest == name;
    }
    false
}

fn is_rust_block(line: &str) -> bool {
    // Check if the line is #+BEGIN_SRC rust (case-insensitive, with possible whitespace)
    // Example: #+BEGIN_SRC rust or #+begin_src rust
    let trimmed = line.trim();
    if trimmed.len() >= 11 && trimmed[..11].eq_ignore_ascii_case("#+BEGIN_SRC") {
        let rest = trimmed[11..].trim_start();
        // Check if it starts with "rust" (followed by whitespace or end of line)
        return rest == "rust" || rest.starts_with("rust ");
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
        let content = r#"This is an Org file
with no code blocks at all.
Just plain text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_no_matching_name() {
        let content = r#"Some text here.

#+NAME: other
#+BEGIN_SRC rust
fn main() {
    println!("Hello");
}
#+END_SRC

More text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_basic_block() {
        let content = r#"Some introduction text.

#+NAME: example
#+BEGIN_SRC rust
println!("hello, world!");
#+END_SRC

Text after the block."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, r#"println!("hello, world!");"#);
    }

    #[test]
    fn extract_multiline_block() {
        let content = r#"Some introduction text.

#+NAME: example
#+BEGIN_SRC rust
fn test() {
    assert_eq!(2 + 2, 4);
}
#+END_SRC

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
    fn extract_multiple_blocks_one_match() {
        let content = r#"Here's the first block:

#+NAME: other
#+BEGIN_SRC python
print("Not this one")
#+END_SRC

And here's the one we want:

#+NAME: example
#+BEGIN_SRC rust
println!("This is the one!");
#+END_SRC

And another one."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, r#"println!("This is the one!");"#);
    }

    #[test]
    fn extract_with_indentation() {
        let content = r#"Text before.

#+NAME: example
#+BEGIN_SRC rust
    let indented = "value";
    println!("{}", indented);
#+END_SRC

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"    let indented = "value";
    println!("{}", indented);"#
        );
    }

    #[test]
    fn extract_empty_lines_within_block() {
        let content = r#"Text before.

#+NAME: example
#+BEGIN_SRC rust
fn first() {}

fn second() {}
#+END_SRC

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
    fn extract_until_eof() {
        let content = r#"Text before.

#+NAME: example
#+BEGIN_SRC rust
struct Point {
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
    fn extract_different_language() {
        let content = r#"Text before.

#+NAME: example
#+BEGIN_SRC python
print("hello")
#+END_SRC

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_name_without_begin_src() {
        let content = r#"Text before.

#+NAME: example
Some other text here.

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_name_not_immediately_before() {
        let content = r#"Text before.

#+NAME: example

#+BEGIN_SRC rust
println!("hello");
#+END_SRC

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_lowercase_directives() {
        let content = r#"Text before.

#+name: example
#+begin_src rust
println!("lowercase directives");
#+end_src

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, r#"println!("lowercase directives");"#);
    }

    #[test]
    fn extract_mixed_case_directives() {
        let content = r#"Text before.

#+Name: example
#+Begin_Src rust
println!("mixed case");
#+End_Src

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(result, r#"println!("mixed case");"#);
    }

    #[test]
    fn extract_lowercase_multiline() {
        let content = r#"Some text.

#+name: example
#+begin_src rust
fn test() {
    assert_eq!(1 + 1, 2);
}
#+end_src

More text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn test() {
    assert_eq!(1 + 1, 2);
}"#
        );
    }
}
