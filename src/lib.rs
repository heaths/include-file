// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "asciidoc")]
mod asciidoc;
mod markdown;
#[cfg(feature = "org")]
mod org;
#[cfg(test)]
mod tests;
#[cfg(feature = "textile")]
mod textile;

use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::quote;
use std::{
    env, fs,
    io::{self, BufRead},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    spanned::Spanned,
    LitStr, Meta, Token,
};

static INCLUDE_COUNTER: AtomicU64 = AtomicU64::new(0);

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
#[cfg(feature = "asciidoc")]
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
#[cfg(feature = "textile")]
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
#[cfg(feature = "org")]
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
    F: FnOnce(&str, io::Lines<io::BufReader<fs::File>>) -> io::Result<(u32, Vec<String>)>,
{
    let args: MarkdownArgs = parse2(item)?;
    let root = match args.relative {
        #[cfg(span_locations)]
        Some(span) => span.local_file(),
        #[cfg(not(span_locations))]
        Some(span) => return Err(syn::Error::new(span, "requires rustc 1.88 or newer")),
        None => None,
    };
    let (file, display_path) =
        open(root, &args.path.value()).map_err(|err| syn::Error::new(args.path.span(), err))?;
    let (start_line, content) = extract(file, &args.name.value(), f)
        .map_err(|err| syn::Error::new(args.name.span(), err))?;

    let n = INCLUDE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let guard_type = Ident::new(&format!("__IncludeFileGuard{n}"), Span::call_site());
    let guard_var = Ident::new(&format!("__include_file_guard{n}"), Span::call_site());

    // Compute the file expression for the guard based on whether `relative` was passed.
    // Use Location::caller().file() to resolve paths consistently with panic messages.
    let file_expr: TokenStream = if args.relative.is_some() {
        // Path is relative to the source file.
        // Resolve against caller's directory and normalize.
        let path_str = args.path.value();
        quote! {
            {
                let __caller = ::std::panic::Location::caller().file();
                let __caller_dir = ::std::path::Path::new(__caller)
                    .parent()
                    .unwrap_or(::std::path::Path::new(""));
                let __resolved = __caller_dir.join(#path_str);
                let mut __parts: ::std::vec::Vec<::std::path::Component<'_>> =
                    ::std::vec::Vec::new();
                for __c in __resolved.components() {
                    match __c {
                        ::std::path::Component::ParentDir => { __parts.pop(); }
                        ::std::path::Component::CurDir => {}
                        _ => __parts.push(__c),
                    }
                }
                let __normalized: ::std::path::PathBuf = __parts.iter().collect();
                __normalized.to_string_lossy().into_owned()
            }
        }
    } else {
        // Path is relative to CARGO_MANIFEST_DIR.
        // Find the crate-relative portion of the caller path by testing
        // progressively shorter suffixes against CARGO_MANIFEST_DIR. The
        // prefix that remains is whatever the compiler prepended (e.g., a
        // workspace-relative directory), which we prepend to display_path
        // so the reported path matches what panic messages use.
        let path_str = &display_path;
        quote! {
            {
                let __caller = ::std::panic::Location::caller().file();
                let __manifest = ::std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
                let __path: &str = #path_str;
                let __caller_path = ::std::path::Path::new(__caller);
                let __components: ::std::vec::Vec<::std::path::Component<'_>> =
                    __caller_path.components().collect();
                let mut __prefix_len = 0usize;
                for __skip in 0..__components.len() {
                    let __suffix: ::std::path::PathBuf =
                        __components[__skip..].iter().collect();
                    if __manifest.join(&__suffix).is_file() {
                        __prefix_len = __skip;
                        break;
                    }
                }
                if __prefix_len == 0 {
                    ::std::string::String::from(__path)
                } else {
                    let __prefix: ::std::path::PathBuf =
                        __components[..__prefix_len].iter().collect();
                    __prefix.join(__path).to_string_lossy().into_owned()
                }
            }
        }
    };

    let guard = quote! {
        struct #guard_type {
            file: ::std::string::String,
            line: u32,
        }
        impl ::std::ops::Drop for #guard_type {
            fn drop(&mut self) {
                if ::std::thread::panicking() {
                    ::std::eprintln!(
                        "note: panicked in code included from {}:{}",
                        self.file,
                        self.line
                    );
                }
            }
        }
        let #guard_var = #guard_type {
            file: #file_expr,
            line: #start_line,
        };
    };

    let body: TokenStream = content.parse()?;
    let mut output = guard;
    output.extend(body);
    // Explicitly drop the guard right after the included body so that, when
    // multiple macros are used in the same scope, prior guards are already gone
    // before the next snippet starts.  Without this, a panic in snippet N would
    // unwind all N guards and print N "note:" lines instead of one.
    output.extend(quote! { ::std::mem::drop(#guard_var); });

    if args.scope.is_some() {
        output = TokenTree::Group(Group::new(Delimiter::Brace, output)).into();
    }

    Ok(output)
}

fn open(root: Option<PathBuf>, path: &str) -> io::Result<(fs::File, String)> {
    let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| io::Error::other("no manifest directory"))?
        .into();
    let root_dir = match root {
        Some(ref src) => src
            .parent()
            .map(|dir| manifest_dir.join(dir))
            .ok_or_else(|| io::Error::other("no source parent directory"))?,
        None => manifest_dir.clone(),
    };
    let full_path = root_dir.join(path);
    let file = fs::File::open(&full_path)?;
    let display_path = {
        // Canonicalize to resolve any `..` components; fall back to the
        // unresolved paths if canonicalization fails (e.g., a race with
        // deletion), which is acceptable since we already opened the file.
        let canonical_full = fs::canonicalize(&full_path).unwrap_or_else(|_| full_path.clone());
        let canonical_manifest =
            fs::canonicalize(&manifest_dir).unwrap_or_else(|_| manifest_dir.clone());
        let rel = canonical_full
            .strip_prefix(&canonical_manifest)
            .unwrap_or(std::path::Path::new(path));
        rel.to_string_lossy().into_owned()
    };
    Ok((file, display_path))
}

fn extract<R, F>(buffer: R, name: &str, f: F) -> io::Result<(u32, String)>
where
    R: io::Read,
    F: FnOnce(&str, io::Lines<io::BufReader<R>>) -> io::Result<(u32, Vec<String>)>,
{
    let reader = io::BufReader::new(buffer);
    let (start_line, lines) = f(name, reader.lines())?;
    if lines.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("code fence '{}' not found", name),
        ));
    }

    Ok((start_line, lines.join("\n")))
}
