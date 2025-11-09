// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::{include_file, open, MarkdownArgs};
use proc_macro2::TokenStream;
use quote::quote;
use std::io;
use syn::parse2;

fn collect<R: io::Read>(
    _name: &str,
    _iter: io::Lines<io::BufReader<R>>,
) -> io::Result<Vec<String>> {
    Ok(vec![r#"println!("example");"#.into()])
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
fn open_file() {
    let file = open("README.md").expect("expected README.md");
    assert!(matches!(file.metadata(), Ok(meta) if meta.is_file()));
}

#[test]
fn open_err() {
    assert!(matches!(open("missing.txt"), Err(err) if err.kind() == io::ErrorKind::NotFound));
}
