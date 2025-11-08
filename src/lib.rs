// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![doc = include_str!("../README.md")]

mod markdown;
#[cfg(test)]
mod tests;

use std::{
    fmt, fs,
    io::{self, BufRead},
    path::PathBuf,
};
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

/// Include code from within a code fence in a markdown file.
///
/// Two arguments are required: a file path relative to the current source file,
/// and a name defined within the code fence as shown below.
///
/// All CommonMark [code fences](https://spec.commonmark.org/current/#fenced-code-blocks) are supported.
///
/// # Examples
///
/// Consider the following code fence in a crate `README.md` markdown file:
///
/// ````markdown
/// ```rust example
/// let m = example()?;
/// assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
/// ```
/// ````
///
/// In Rust documentation comments, we can use `# line` to hide setup code.
/// That's not possible in markdown, so we can include only the code we want to demonstrate;
/// however, we can still compile and even run it in Rust tests:
///
/// ```no_run
/// struct Model {
///     name: String,
/// }
///
/// fn example() -> Result<Model, Box<dyn std::error::Error>> {
///     Ok(Model { name: "example".into() })
/// }
///
/// #[test]
/// fn test_example() -> Result<(), Box<dyn std::error::Error>> {
///     include_markdown!("../README.md", "example");
///     Ok(())
/// }
/// ```
#[proc_macro]
pub fn include_markdown(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    markdown::include_markdown(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

struct MarkdownArgs {
    path: LitStr,
    name: LitStr,
}

impl fmt::Debug for MarkdownArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MarkdownArgs")
            .field("path", &self.path.value())
            .field("name", &self.name.value())
            .finish()
    }
}

impl Parse for MarkdownArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse()?;
        input.parse::<Token![,]>()?;
        let name = input.parse()?;
        Ok(Self { path, name })
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

    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("code fence '{}' not found", name),
        ));
    }

    Ok(lines.join("\n"))
}
