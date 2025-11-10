// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![doc = include_str!("../README.md")]

mod asciidoc;
mod markdown;
mod org;
#[cfg(test)]
mod tests;
mod textile;

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

/// Include code from within a source block in an AsciiDoc file.
///
/// Two arguments are required: a file path relative to the current source file,
/// and an id defined within the source block attributes as shown below.
///
/// All AsciiDoc [source blocks](https://docs.asciidoctor.org/asciidoc/latest/verbatim/source-blocks/)
/// with delimited [listing blocks](https://docs.asciidoctor.org/asciidoc/latest/verbatim/listing-blocks/) are supported.
///
/// # Examples
///
/// Consider the following source block in a crate `README.adoc` AsciiDoc file:
///
/// ```asciidoc
/// [,rust,id="example"]
/// ----
/// let m = example()?;
/// assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
/// ----
/// ```
///
/// We can include this code block in our Rust tests:
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
///     include_asciidoc!("../README.adoc", "example");
///     Ok(())
/// }
/// ```
#[proc_macro]
pub fn include_asciidoc(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    asciidoc::include_asciidoc(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Include code from within a code fence in a Markdown file.
///
/// Two arguments are required: a file path relative to the current source file,
/// and a name defined within the code fence as shown below.
///
/// All CommonMark [code fences](https://spec.commonmark.org/current/#fenced-code-blocks) are supported.
///
/// # Examples
///
/// Consider the following code fence in a crate `README.md` Markdown file:
///
/// ````markdown
/// ```rust example
/// let m = example()?;
/// assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
/// ```
/// ````
///
/// In Rust documentation comments, we can use `# line` to hide setup code.
/// That's not possible in Markdown, so we can include only the code we want to demonstrate;
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

/// Include code from within a code block in a Textile file.
///
/// Two arguments are required: a file path relative to the current source file,
/// and an id defined within the code block as shown below.
///
/// All Textile [code blocks](https://textile-lang.com/doc/block-code) are supported.
///
/// # Examples
///
/// Consider the following code block in a crate `README.textile` Textile file:
///
/// ```textile
/// bc(rust#example). let m = example()?;
/// assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
/// ```
///
/// In Rust documentation comments, we can use `# line` to hide setup code.
/// That's not possible in Textile, so we can include only the code we want to demonstrate;
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
///     include_textile!("../README.textile", "example");
///     Ok(())
/// }
/// ```
#[proc_macro]
pub fn include_textile(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    textile::include_textile(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Include code from within a source block in an Org file.
///
/// Two arguments are required: a file path relative to the current source file,
/// and a name defined with `#+NAME:` immediately before the source block as shown below.
///
/// All Org [source code blocks](https://orgmode.org/manual/Structure-of-Code-Blocks.html) are supported.
///
/// # Examples
///
/// Consider the following source block in a crate `README.org` Org file:
///
/// ```org
/// #+NAME: example
/// #+BEGIN_SRC rust
/// let m = example()?;
/// assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
/// #+END_SRC
/// ```
///
/// In Rust documentation comments, we can use `# line` to hide setup code.
/// That's not possible in Org, so we can include only the code we want to demonstrate;
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
///     include_org!("../README.org", "example");
///     Ok(())
/// }
/// ```
#[proc_macro]
pub fn include_org(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    org::include_org(item.into())
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

fn include_file<F>(item: TokenStream, f: F) -> syn::Result<TokenStream>
where
    F: FnOnce(&str, io::Lines<io::BufReader<fs::File>>) -> io::Result<Vec<String>>,
{
    let args: MarkdownArgs = parse2(item).map_err(|_| {
        syn::Error::new(
            Span::call_site(),
            "expected (path, name) literal string arguments",
        )
    })?;
    let file = open(&args.path.value()).map_err(|err| syn::Error::new(args.path.span(), err))?;
    let content = extract(file, &args.name.value(), f)
        .map_err(|err| syn::Error::new(args.name.span(), err))?;

    Ok(content.parse()?)
}

fn open(path: &str) -> io::Result<fs::File> {
    let manifest_dir: PathBuf = option_env!("CARGO_MANIFEST_DIR")
        .ok_or_else(|| io::Error::other("no manifest directory"))?
        .into();
    let path = manifest_dir.join(path);
    fs::File::open(path)
}

fn extract<R, F>(buffer: R, name: &str, f: F) -> io::Result<String>
where
    R: io::Read,
    F: FnOnce(&str, io::Lines<io::BufReader<R>>) -> io::Result<Vec<String>>,
{
    let reader = io::BufReader::new(buffer);
    let lines = f(name, reader.lines())?;
    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("code fence '{}' not found", name),
        ));
    }

    Ok(lines.join("\n"))
}
