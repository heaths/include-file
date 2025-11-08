// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use proc_macro2::TokenStream;
use std::{fs, io};

pub fn include_markdown(item: TokenStream) -> syn::Result<TokenStream> {
    super::include_file(item, collect::<fs::File>)
}

fn collect<R: io::Read>(name: &str, iter: io::Lines<io::BufReader<R>>) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut in_fence = false;
    let mut fence_char = '\0';
    let mut fence_count = 0;
    let mut fence_indent = 0;

    for line in iter {
        let line = line?;

        if !in_fence {
            // Look for the start of a code fence
            let trimmed_start = line.trim_start();
            let indent = line.len() - trimmed_start.len();

            // Check if line starts with ``` or ~~~
            let first_char = trimmed_start.chars().next();
            if first_char == Some('`') || first_char == Some('~') {
                let fence_ch = first_char.unwrap();
                let count = trimmed_start.chars().take_while(|&c| c == fence_ch).count();

                if count >= 3 {
                    // Check if the rest of the line contains the name
                    let after_fence = &trimmed_start[count..];
                    if after_fence.contains(name) {
                        in_fence = true;
                        fence_char = fence_ch;
                        fence_count = count;
                        fence_indent = indent;
                    }
                }
            }
        } else {
            // We're inside a fence, check if this line ends the fence
            let trimmed_start = line.trim_start();
            let indent = line.len() - trimmed_start.len();

            // Check if this line is the closing fence
            if indent == fence_indent {
                let first_char = trimmed_start.chars().next();
                if first_char == Some(fence_char) {
                    let count = trimmed_start
                        .chars()
                        .take_while(|&c| c == fence_char)
                        .count();
                    if count >= fence_count {
                        // Found the closing fence
                        break;
                    }
                }
            }

            // Collect the line content, stripping the expected indentation
            if line.len() >= fence_indent {
                let content = &line[fence_indent..];
                lines.push(content.to_string());
            } else {
                // Line has less indentation than expected, include as-is
                lines.push(line);
            }
        }
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::collect;
    use crate::extract;
    use std::io;

    #[test]
    fn extract_no_code_fences() {
        let content = r#"This is a markdown file
with no code fences at all.
Just plain text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_no_matching_name() {
        let content = r#"Some text here.

```rust
fn main() {
    println!("Hello");
}
```

More text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect);
        assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn extract_multiple_fences_one_match() {
        let content = r#"Here's the first fence:

```javascript
console.log("Not this one");
```

And here's the one we want:

~~~rust example
fn main() {
    println!("Hello, world!");
}
~~~

And another one:

```python
print("Also not this one")
```"#;
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
    fn extract_nested_code_fence() {
        let content = r#"Outer content:

````markdown example
# Example

Here's a nested code fence:

```rust
fn nested() {
    println!("Inner code");
}
```

More content.
````

After the fence."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"# Example

Here's a nested code fence:

```rust
fn nested() {
    println!("Inner code");
}
```

More content."#
        );
    }

    #[test]
    fn extract_with_indentation() {
        let content = r#"Normal text.

  ~~~rust example
  fn indented() {
      println!("Indented code");
  }
  ~~~

More text."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn indented() {
    println!("Indented code");
}"#
        );
    }

    #[test]
    fn extract_backticks_with_name() {
        let content = r#"Text before.

```rust example
let x = 42;
let y = x + 1;
```

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"let x = 42;
let y = x + 1;"#
        );
    }

    #[test]
    fn extract_tildes_with_name() {
        let content = r#"Text before.

~~~python example
def hello():
    print("Hello")
~~~

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"def hello():
    print("Hello")"#
        );
    }

    #[test]
    fn extract_more_closing_chars() {
        let content = r#"Text before.

```rust example
fn test() {
    println!("test");
}
`````

Text after."#;
        let cursor = io::Cursor::new(content);
        let result = extract(cursor, "example", collect).expect("expected content");
        assert_eq!(
            result,
            r#"fn test() {
    println!("test");
}"#
        );
    }
}
