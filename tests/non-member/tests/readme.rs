// Copyright 2026 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use include_file::include_markdown;

#[test]
fn test_example() -> Result<(), Box<dyn std::error::Error>> {
    include_markdown!("README.md", "example");
    Ok(())
}

// Verify that the guard reports a path relative to the crate manifest directory,
// not the parent workspace.
#[test]
fn test_panic_reports_crate_relative_path() {
    let exe = std::env::current_exe().unwrap();
    let output = std::process::Command::new(&exe)
        .arg("--exact")
        .arg("panic_in_readme")
        .arg("--nocapture")
        .env("RUST_BACKTRACE", "0")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("README.md"),
        "expected 'README.md' in stderr:\n{stderr}"
    );
    let not_expected = if cfg!(windows) {
        "non-member\\README.md"
    } else {
        "non-member/README.md"
    };
    assert!(
        !stderr.contains(not_expected),
        "path should be crate-relative, not workspace-relative:\n{stderr}"
    );
}

#[test]
#[should_panic(expected = "intentional assert failure")]
fn panic_in_readme() {
    include_markdown!("README.md", "assert-fail");
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
