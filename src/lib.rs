// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

extern crate proc_macro;
mod code;

use proc_macro::TokenStream;

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
/// println!("{m:#?}");
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
///     include_code!("../README.md", "example");
///     Ok(())
/// }
/// ```
#[proc_macro]
pub fn include_code(item: TokenStream) -> TokenStream {
    code::include_code(item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
