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

// Verify that two includes in the same function generate unique guard names.
#[test]
fn test_multiple_includes() -> Result<(), Box<dyn std::error::Error>> {
    include_markdown!("README.md", "example", scope);
    include_markdown!("README.md", "example", scope);
    Ok(())
}

// Verify that when multiple guards coexist in the same scope (no `scope` keyword)
// and a later snippet panics, only that snippet's guard prints the "note:" line.
// Without the explicit drop() emitted after each body, all live guards would
// unwind together and each would print a separate note, making it hard to
// identify which snippet actually panicked.
#[cfg(feature = "asciidoc")]
#[test]
#[should_panic(expected = "intentional assert failure")]
fn test_only_one_note_on_panic() {
    // First snippet succeeds; its guard is explicitly dropped right after.
    include_asciidoc!("tests/README.adoc", "simple-assert");
    // Second snippet panics; only this guard should print the "note:" line.
    include_asciidoc!("tests/README.adoc", "assert-fail");
}

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
