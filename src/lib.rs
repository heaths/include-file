// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![doc = include_str!("../README.md")]

mod asciidoc;
mod markdown;
mod org;
#[cfg(test)]
mod tests;
mod textile;

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use std::{
    env, fs,
    io::{self, BufRead},
    path::PathBuf,
};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    LitStr, Meta, Token,
};

/// Include code from within a source block in an AsciiDoc file.
///
/// All AsciiDoc [source blocks](https://docs.asciidoctor.org/asciidoc/latest/verbatim/source-blocks/)
/// with delimited [listing blocks](https://docs.asciidoctor.org/asciidoc/latest/verbatim/listing-blocks/) are supported.
///
/// # Arguments
///
/// * `path` (*Required*) Path relative to the crate root directory.
/// * `name` (*Required*) Name of the code fence to include.
/// * `scope` Include the snippet in braces `{ .. }`.
/// * `relative` (*Requires rustc 1.88 or newer*) Path is relative to the source file calling the macro.
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
///     include_asciidoc!("README.adoc", "example");
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
/// All CommonMark [code fences](https://spec.commonmark.org/current/#fenced-code-blocks) are supported.
///
/// # Arguments
///
/// * `path` (*Required*) Path relative to the crate root directory.
/// * `name` (*Required*) Name of the code fence to include.
/// * `scope` Include the snippet in braces `{ .. }`.
/// * `relative` (*Requires rustc 1.88 or newer*) Path is relative to the source file calling the macro.
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
///     include_markdown!("README.md", "example");
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
/// All Textile [code blocks](https://textile-lang.com/doc/block-code) are supported.
///
/// # Arguments
///
/// * `path` (*Required*) Path relative to the crate root directory.
/// * `name` (*Required*) Name of the code fence to include.
/// * `scope` Include the snippet in braces `{ .. }`.
/// * `relative` (*Requires rustc 1.88 or newer*) Path is relative to the source file calling the macro.
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
///     include_textile!("README.textile", "example");
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
/// All Org [source code blocks](https://orgmode.org/manual/Structure-of-Code-Blocks.html) are supported.
///
/// # Arguments
///
/// * `path` (*Required*) Path relative to the crate root directory.
/// * `name` (*Required*) Name of the code fence to include.
/// * `scope` Include the snippet in braces `{ .. }`.
/// * `relative` (*Requires rustc 1.88 or newer*) Path is relative to the source file calling the macro.
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
///     include_org!("README.org", "example");
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
    scope: Option<Span>,
    relative: Option<Span>,
}

impl Parse for MarkdownArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const REQ_PARAMS: &str = r#"missing required string parameters ("path", "name")"#;

        let path = input
            .parse()
            .map_err(|err| syn::Error::new(err.span(), REQ_PARAMS))?;
        input.parse::<Token![,]>()?;
        let name = input
            .parse()
            .map_err(|err| syn::Error::new(err.span(), REQ_PARAMS))?;

        let mut scope = None;
        let mut relative = None;

        if input.parse::<Token![,]>().is_ok() {
            let params = input.parse_terminated(Meta::parse, Token![,])?;
            for param in params {
                if param.path().is_ident("scope") {
                    scope = Some(param.span());
                } else if param.path().is_ident("relative") {
                    relative = Some(param.span());
                } else {
                    return Err(syn::Error::new(param.span(), "unsupported parameter"));
                }
            }
        } else if !input.is_empty() {
            return Err(syn::Error::new(input.span(), "unexpected token"));
        }

        Ok(Self {
            path,
            name,
            scope,
            relative,
        })
    }
}

fn include_file<F>(item: TokenStream, f: F) -> syn::Result<TokenStream>
where
    F: FnOnce(&str, io::Lines<io::BufReader<fs::File>>) -> io::Result<Vec<String>>,
{
    let args: MarkdownArgs = parse2(item)?;
    let root = match args.relative {
        #[cfg(span_locations)]
        Some(span) => span.local_file(),
        #[cfg(not(span_locations))]
        Some(span) => return Err(syn::Error::new(span, "requires rustc 1.88 or newer")),
        None => None,
    };
    let file =
        open(root, &args.path.value()).map_err(|err| syn::Error::new(args.path.span(), err))?;
    let content = extract(file, &args.name.value(), f)
        .map_err(|err| syn::Error::new(args.name.span(), err))?;

    let mut content = content.parse()?;
    if args.scope.is_some() {
        content = TokenTree::Group(Group::new(Delimiter::Brace, content)).into();
    }

    Ok(content)
}

fn open(root: Option<PathBuf>, path: &str) -> io::Result<fs::File> {
    let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| io::Error::other("no manifest directory"))?
        .into();
    let root = match root {
        Some(path) => path
            .parent()
            .map(|dir| manifest_dir.join(dir))
            .ok_or_else(|| io::Error::other("no source parent directory"))?,
        None => manifest_dir,
    };
    let path = root.join(path);
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
