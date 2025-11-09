// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use include_file::{include_asciidoc, include_markdown};

#[test]
fn test_asciidoc() -> Result<(), Box<dyn std::error::Error>> {
    include_asciidoc!("tests/README.adoc", "example");
    Ok(())
}

#[test]
fn test_markdown() -> Result<(), Box<dyn std::error::Error>> {
    include_markdown!("README.md", "example");
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
