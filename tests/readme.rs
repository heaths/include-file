// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

#[cfg(feature = "asciidoc")]
use include_file::include_asciidoc;
use include_file::include_markdown;
#[cfg(feature = "org")]
use include_file::include_org;
#[cfg(feature = "textile")]
use include_file::include_textile;

#[cfg(feature = "asciidoc")]
#[test]
fn test_asciidoc() -> Result<(), Box<dyn std::error::Error>> {
    include_asciidoc!("tests/README.adoc", "example");
    Ok(())
}

#[test]
fn test_markdown() -> Result<(), Box<dyn std::error::Error>> {
    include_markdown!("README.md", "example", scope);
    Ok(())
}

// rust-analyzer does not implement Span::local_file(): https://github.com/rust-lang/rust-analyzer/issues/15950
#[cfg_attr(not(span_locations), ignore = "not supported")]
#[test]
fn test_relative_markdown() -> Result<(), Box<dyn std::error::Error>> {
    // Hide the error from the proc-macro in rust-analyzer.
    #[cfg(all(span_locations, not(rust_analyzer)))]
    {
        include_markdown!("../README.md", "example", relative);
    }

    if cfg!(rust_analyzer) {
        panic!("not supported")
    }

    Ok(())
}

#[cfg(feature = "org")]
#[test]
fn test_org() -> Result<(), Box<dyn std::error::Error>> {
    include_org!("tests/README.org", "example");
    Ok(())
}

#[cfg(feature = "textile")]
#[test]
fn test_textile() -> Result<(), Box<dyn std::error::Error>> {
    include_textile!("tests/README.textile", "example");
    Ok(())
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
