# Instructions for include-file

This repository provides procedural macros for including code snippets from various documentation formats
(Markdown, AsciiDoc, Org, Textile, et al.) into Rust source code.

## Project Overview

- Rust procedural macro library
- Parses documentation files (Markdown, AsciiDoc, Org, Textile)
- Extracts named code fences/blocks from these files
- Includes the extracted code as Rust source code via proc macros
- Supports both crate-relative and source-relative file paths

## Testing

- **Run targeted tests first** when adding or modifying code:

  ```bash
  cargo test markdown::tests
  cargo test asciidoc::tests
  cargo test --test readme
  ```

- **Run full test suite** before completing any task:

  ```bash
  cargo test
  ```

- **Test organization**:
  - Unit tests: each module's `tests` submodule (e.g., `src/markdown.rs` has `#[cfg(test)] mod tests`)
  - Integration tests: `tests/readme.rs`
  - Format-specific READMEs in `tests/` (e.g., `tests/README.adoc`, `tests/README.org`)

## Formatting and Linting

Run all checks before completing any task:

1. **Format Rust code:**

   ```bash
   cargo fmt
   ```

2. **Check spelling** — use the **check-spelling** skill after modifying any source files, documentation,
   or adding new identifiers that may not be in the dictionary:
   - **check-spelling**: [Check and fix spelling in project source files using cSpell](.github/skills/check-spelling/SKILL.md)

3. **Lint Markdown files** — use the **lint-markdown** skill after creating or modifying any `.md` files:
   - **lint-markdown**: [Check and fix formatting and other issues in markdown files using markdownlint-cli2](.github/skills/lint-markdown/SKILL.md)

When both a repository skill and a plugin skill could handle the same task, always prefer the repository skill.

**Do not consider a task complete without running both skills at least once.**

## Code Organization

When adding support for a new documentation format:

- **Create a new module** `src/newformat.rs`:

  ```rust
  use proc_macro2::TokenStream;
  use std::{fs, io};

  pub fn include_newformat(item: TokenStream) -> syn::Result<TokenStream> {
      super::include_file(item, collect::<fs::File>)
  }

  fn collect<R: io::Read>(name: &str, iter: io::Lines<io::BufReader<R>>) -> io::Result<Vec<String>> {
      // Format-specific parsing logic
  }

  #[cfg(test)]
  mod tests {
      // Unit tests for the format parser
  }
  ```

- **Add the module** to `src/lib.rs`:
  - Declare: `mod newformat;`
  - Add public macro with comprehensive documentation
  - Follow the pattern of existing macros (e.g., `include_markdown!`, `include_asciidoc!`)

- **Create a README** at `tests/README.<format>`:
  - Follow structure of existing READMEs (`tests/README.adoc`, `tests/README.org`, `tests/README.textile`)
  - Title: "Macros for including file content"
  - Include examples with actual code fences in the new format

- **Add integration tests** in `tests/readme.rs`:
  - Import the new macro
  - Follow existing test patterns

## Key Implementation Details

- **Path resolution**: `open()` handles both crate-relative (default) and source-relative paths
- **Error handling**: use `syn::Error` for compile-time errors with helpful span information
- **Parsing strategy**: each format module implements a `collect()` function that:
  - Takes a name and line iterator
  - Searches for named code blocks/fences
  - Returns collected lines as `Vec<String>`
  - Returns empty vec if not found (converted to error by `extract()`)

## Conventions

- Use `#[cfg(test)]` for test modules
- Copyright header on all source files: `// Copyright 2025 Heath Stewart.`
- Follow MIT license requirements
- Use descriptive test names: `test_<format>_<scenario>`
- Keep tests focused and independent

## Dependencies

- `proc-macro2`: token stream manipulation with span locations
- `syn`: parsing macro arguments
- `quote`: token stream quoting (regular dependency)

## Rust Version

- Requires Rust 1.85.0 or newer (see `rust-toolchain.toml`)

## Pull Request Conventions

- Draft PRs do not need a "[WIP]" prefix
- Remove "[WIP]" from PR title once the request is resolved
- Use simple, descriptive titles following git commit message conventions
- Keep PR descriptions brief; summarize overall purpose rather than listing every file modified
- Update PR description after making changes to reflect a concise summary

## Example Workflow: Adding a New Format

1. Study existing format implementations (`src/markdown.rs`, `src/asciidoc.rs`, etc.)
2. Create the new module with parser logic
3. Add comprehensive unit tests in the module
4. Update `src/lib.rs` with the new public macro
5. Create `tests/README.<format>` following existing patterns
6. Add integration test in `tests/readme.rs`
7. Run targeted tests: `cargo test newformat::tests`
8. Run full suite: `cargo test`
9. Format and lint:
   - `cargo fmt`
   - Run **check-spelling** skill
   - Run **lint-markdown** skill
10. Verify all tests pass and checks complete successfully

## When Uncertain

- **Format-specific syntax**: consult the format's official documentation
- **Parser edge cases**: add test cases to capture the behavior
- **Integration patterns**: reference existing format implementations
