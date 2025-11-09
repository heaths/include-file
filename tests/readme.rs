// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use include_file::{include_asciidoc, include_markdown};

#[test]
fn test_asciidoc() -> Result<(), Box<dyn std::error::Error>> {
    include_asciidoc!("tests/README.adoc", "example");
    Ok(())
}

#[test]
#[cfg_attr(not(span_locations), ignore)]
fn test_relative_asciidoc() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(span_locations, not(rust_analyzer)))]
    {
        // rust-analyzer does not implement Span::local_file(): https://github.com/rust-lang/rust-analyzer/issues/15950
        include_file::include_relative_asciidoc!("README.adoc", "example");
        Ok(())
    }
    #[cfg(any(not(span_locations), rust_analyzer))]
    panic!("not supported")
}

#[test]
fn test_markdown() -> Result<(), Box<dyn std::error::Error>> {
    include_markdown!("README.md", "example");
    Ok(())
}

#[test]
#[cfg_attr(not(span_locations), ignore)]
fn test_relative_markdown() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(span_locations, not(rust_analyzer)))]
    {
        // rust-analyzer does not implement Span::local_file(): https://github.com/rust-lang/rust-analyzer/issues/15950
        include_file::include_relative_markdown!("../README.md", "example");
        Ok(())
    }
    #[cfg(any(not(span_locations), rust_analyzer))]
    panic!("not supported")
}

#[derive(Debug)]
struct Model {
    #[allow(dead_code)]
    name: String,
}

fn example() -> Result<Model, Box<dyn std::error::Error>> {
    Ok(Model {
        name: "example".into(),
    })
}
