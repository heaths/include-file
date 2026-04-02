// Copyright 2026 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::{path::Path, process::Command};

/// Runs `cargo test` for the non-member crate under `tests/non-member/`.
///
/// This crate is intentionally excluded from the workspace so that it
/// compiles and runs as a standalone crate, verifying that the guard
/// struct reports paths relative to its own manifest directory.
#[test]
fn non_member_crate() {
    let cargo = env!("CARGO");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let non_member_dir = manifest_dir.join("tests/non-member");

    let output = Command::new(cargo)
        .arg("test")
        .current_dir(&non_member_dir)
        .output()
        .expect("failed to run cargo test");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "cargo test failed in {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            non_member_dir.display()
        );
    }
}
