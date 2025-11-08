// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::{extract, open, MarkdownArgs};
use proc_macro2::{Span, TokenStream};
use syn::parse2;

pub fn include_markdown(item: TokenStream) -> syn::Result<TokenStream> {
    let args: MarkdownArgs = parse2(item).map_err(|_| {
        syn::Error::new(
            Span::call_site(),
            "expected (path, name) literal string arguments",
        )
    })?;
    let file = open(&args.path.value()).map_err(|err| syn::Error::new(args.path.span(), err))?;
    let content =
        extract(file, &args.name.value()).map_err(|err| syn::Error::new(args.name.span(), err))?;

    Ok(content.parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_two_args() {
        let tokens = quote! { "../README.md", "example" };
        include_markdown(tokens.clone()).expect("expected TokenStream");

        let args: MarkdownArgs = parse2(tokens).expect("expected parse2");
        assert_eq!(args.path.value(), "../README.md");
        assert_eq!(args.name.value(), "example");
    }

    #[test]
    fn parse_no_args_err() {
        let tokens = TokenStream::new();
        include_markdown(tokens).expect_err("expected parse error");
    }

    #[test]
    fn parse_one_args_err() {
        let tokens = quote! { "../README.md" };
        include_markdown(tokens).expect_err("expected parse error");
    }

    #[test]
    fn parse_three_args_err() {
        let tokens = quote! { "../README.md", "example", "other" };
        include_markdown(tokens).expect_err("expected parse error");
    }

    #[test]
    fn parse_no_sep_err() {
        let tokens = quote! { "../README.md" "example" };
        include_markdown(tokens).expect_err("expected parse error");
    }
}
