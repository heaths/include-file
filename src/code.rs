// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use proc_macro2::{Span, TokenStream};
use std::{
    fmt, fs,
    io::{self, BufRead},
    path::PathBuf,
};
use syn::{
    parse::{Parse, ParseStream},
    parse2, LitStr, Token,
};

pub fn include_code(item: TokenStream) -> syn::Result<TokenStream> {
    let args: CodeArgs = parse2(item)?;
    let file = open(&args.path.value()).map_err(|err| syn::Error::new(Span::call_site(), err))?;
    let content =
        extract(file, &args.name.value()).map_err(|err| syn::Error::new(Span::call_site(), err))?;

    Ok(content.parse()?)
}

struct CodeArgs {
    path: LitStr,
    _sep: Token![,],
    name: LitStr,
}

impl fmt::Debug for CodeArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodeArgs")
            .field("path", &self.path.value())
            .field("_sep", &",")
            .field("name", &self.name.value())
            .finish()
    }
}

impl Parse for CodeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            path: input.parse()?,
            _sep: input.parse()?,
            name: input.parse()?,
        })
    }
}

fn open(path: &str) -> io::Result<fs::File> {
    let file_path = PathBuf::from(file!());
    let path = file_path
        .parent()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "could not get parent of current source file",
            )
        })?
        .join(path);
    fs::File::open(path)
}

fn extract<R: io::Read>(buffer: R, name: &str) -> io::Result<String> {
    let reader = io::BufReader::new(buffer);
    let mut lines = Vec::new();
    let mut in_fence = false;
    let mut fence_char = '\0';
    let mut fence_count = 0;
    let mut fence_indent = 0;

    for line in reader.lines() {
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
                    if count == fence_count {
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

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("code fence with name '{}' not found", name),
        ));
    }

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn open_file() {
        let file = open("../README.md").expect("expected README.md");
        assert!(matches!(file.metadata(), Ok(meta) if meta.is_file()));
    }

    #[test]
    fn open_err() {
        assert!(matches!(open("missing.txt"), Err(err) if err.kind() == io::ErrorKind::NotFound));
    }

    #[test]
    fn parse_two_args() {
        let tokens = quote! { "../README.md", "example" };
        include_code(tokens.clone()).expect("expected TokenStream");

        let args: CodeArgs = parse2(tokens).expect("expected parse2");
        assert_eq!(args.path.value(), "../README.md");
        assert_eq!(args.name.value(), "example");
    }

    #[test]
    fn parse_no_args_err() {
        let tokens = TokenStream::new();
        include_code(tokens).expect_err("expected parse error");
    }

    #[test]
    fn parse_one_args_err() {
        let tokens = quote! { "../README.md" };
        include_code(tokens).expect_err("expected parse error");
    }

    #[test]
    fn parse_three_args_err() {
        let tokens = quote! { "../README.md", "example", "other" };
        include_code(tokens).expect_err("expected parse error");
    }

    #[test]
    fn parse_no_sep_err() {
        let tokens = quote! { "../README.md" "example" };
        include_code(tokens).expect_err("expected parse error");
    }

    #[test]
    fn extract_no_code_fences() {
        let content = r#"This is a markdown file
with no code fences at all.
Just plain text."#;
        let result = extract(content.as_bytes(), "example");
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
        let result = extract(content.as_bytes(), "example");
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
        let result = extract(content.as_bytes(), "example").expect("expected content");
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
        let result = extract(content.as_bytes(), "example").expect("expected content");
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
        let result = extract(content.as_bytes(), "example").expect("expected content");
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
        let result = extract(content.as_bytes(), "example").expect("expected content");
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
        let result = extract(content.as_bytes(), "example").expect("expected content");
        assert_eq!(
            result,
            r#"def hello():
    print("Hello")"#
        );
    }
}
