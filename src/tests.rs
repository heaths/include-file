// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::{include_file, open, MarkdownArgs};
use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::quote;
use std::{io, path::PathBuf};
use syn::parse2;

fn collect<R: io::Read>(
    _name: &str,
    _iter: io::Lines<io::BufReader<R>>,
) -> io::Result<(u32, Vec<String>)> {
    Ok((1, vec![r#"println!("example");"#.into()]))
}

#[test]
fn parse_two_args() {
    let tokens = quote! { "README.md", "example" };
    include_file(tokens.clone(), collect).expect("expected TokenStream");

    let args: MarkdownArgs = parse2(tokens).expect("expected parse2");
    assert_eq!(args.path.value(), "README.md");
    assert_eq!(args.name.value(), "example");
}

#[test]
fn parse_no_args_err() {
    let tokens = TokenStream::new();
    include_file(tokens, collect).expect_err("expected parse error");
}

#[test]
fn parse_one_args_err() {
    let tokens = quote! { "README.md" };
    include_file(tokens, collect).expect_err("expected parse error");
}

#[test]
fn parse_three_args_err() {
    let tokens = quote! { "README.md", "example", "other" };
    include_file(tokens, collect).expect_err("expected parse error");
}

#[test]
fn parse_no_sep_err() {
    let tokens = quote! { "README.md" "example" };
    include_file(tokens, collect).expect_err("expected parse error");
}

#[test]
fn parse_semicolon_sep_err() {
    let tokens = quote! { "README.md"; "example" };
    include_file(tokens, collect).expect_err("expected parse error");
}

#[test]
fn parse_scope_param() {
    let tokens = quote! { "README.md", "example", scope };
    let args: MarkdownArgs = parse2(tokens).expect("expected parse2");
    assert_eq!(args.path.value(), "README.md");
    assert_eq!(args.name.value(), "example");
    assert!(args.scope.is_some());
    assert!(args.relative.is_none());
}

#[test]
fn parse_relative_param() {
    let tokens = quote! { "README.md", "example", relative };
    let args: MarkdownArgs = parse2(tokens).expect("expected parse2");
    assert_eq!(args.path.value(), "README.md");
    assert_eq!(args.name.value(), "example");
    assert!(args.scope.is_none());
    assert!(args.relative.is_some());
}

#[test]
fn parse_both_params() {
    let tokens = quote! { "README.md", "example", scope, relative };
    let args: MarkdownArgs = parse2(tokens).expect("expected parse2");
    assert_eq!(args.path.value(), "README.md");
    assert_eq!(args.name.value(), "example");
    assert!(args.scope.is_some());
    assert!(args.relative.is_some());
}

#[test]
fn parse_both_params_reverse_order() {
    let tokens = quote! { "README.md", "example", relative, scope };
    let args: MarkdownArgs = parse2(tokens).expect("expected parse2");
    assert_eq!(args.path.value(), "README.md");
    assert_eq!(args.name.value(), "example");
    assert!(args.scope.is_some());
    assert!(args.relative.is_some());
}

#[test]
fn parse_unsupported_param_err() {
    let tokens = quote! { "README.md", "example", invalid };
    include_file(tokens, collect).expect_err("expected unsupported parameter error");
}

#[test]
fn parse_unsupported_param_with_valid_err() {
    let tokens = quote! { "README.md", "example", scope, invalid };
    include_file(tokens, collect).expect_err("expected unsupported parameter error");
}

#[test]
fn parse_string_as_third_param_err() {
    let tokens = quote! { "README.md", "example", "scope" };
    include_file(tokens, collect).expect_err("expected unsupported parameter error");
}

#[test]
fn parse_semicolon_after_second_arg_err() {
    let tokens = quote! { "README.md", "example"; scope };
    include_file(tokens, collect).expect_err("expected parse error");
}

#[test]
fn parse_pipe_after_second_arg_err() {
    let tokens = quote! { "README.md", "example" | scope };
    include_file(tokens, collect).expect_err("expected unexpected token error");
}

#[test]
fn parse_non_comma_separator_err() {
    let tokens = quote! { "README.md", "example", scope; relative };
    include_file(tokens, collect).expect_err("expected parse error");
}

#[test]
fn parse_token_without_comma_err() {
    let tokens = quote! { "README.md", "example" scope };
    include_file(tokens, collect).expect_err("expected unexpected token error");
}

#[test]
fn include_file_scope() {
    let tokens = quote! { "README.md", "example", scope };
    let mut actual = include_file(tokens, collect)
        .expect("expected include_file")
        .into_iter();
    assert!(matches!(
        actual.next(),
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace,
    ));
}

#[test]
fn include_file_no_scope() {
    let tokens = quote! { "README.md", "example" };
    let mut actual = include_file(tokens, collect)
        .expect("expected include_file")
        .into_iter();
    assert!(!matches!(
        actual.next(),
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace,
    ));
}

#[test]
fn open_file() {
    let (file, _) = open(None, "README.md").expect("expected README.md");
    assert!(matches!(file.metadata(), Ok(meta) if meta.is_file()));
}

#[test]
fn open_relative_file() {
    let (file, _) = open(Some(file!().into()), "../README.md").expect("expected README.md");
    assert!(matches!(file.metadata(), Ok(meta) if meta.is_file()));
}

#[test]
fn open_err() {
    assert!(matches!(open(None, "missing.txt"), Err(err) if err.kind() == io::ErrorKind::NotFound));
}

#[test]
fn display_path_without_relative() {
    // Without `relative`, the user-supplied path is already relative to CARGO_MANIFEST_DIR.
    let (_, display_path) = open(None, "tests/README.adoc").expect("expected tests/README.adoc");
    assert_eq!(display_path, "tests/README.adoc");
}

#[test]
fn display_path_with_relative() {
    // With `relative`, the user path is relative to the calling source file.
    // Simulates include_asciidoc!("README.adoc", "example", relative) called from
    // tests/readme.rs: the note should still report "tests/README.adoc".
    let src = PathBuf::from("tests/readme.rs");
    let (_, display_path) = open(Some(src), "README.adoc").expect("expected tests/README.adoc");
    assert_eq!(display_path, "tests/README.adoc");
}
