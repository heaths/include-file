# Copilot Instructions for include-file

This repository provides procedural macros for including code snippets from various documentation formats
(Markdown, AsciiDoc, Org, Textile, et. al.) into Rust source code.

## Development Guidelines

### Expert Rust Developer Context

You are working on a Rust procedural macro library that:

- Parses documentation files (Markdown, AsciiDoc, Org, Textile)
- Extracts named code fences/blocks from these files
- Includes the extracted code as Rust source code via proc macros
- Supports both crate-relative and source-relative file paths

### Testing Workflow

1. **Run tests for specific modules**: When adding or modifying code, initially test only the affected modules:

   ```bash
   # Test a specific module
   cargo test markdown::tests
   cargo test asciidoc::tests
   
   # Test integration tests
   cargo test --test readme
   ```

2. **Run full test suite**: Before completing any request, run the full test suite:

   ```bash
   cargo test
   ```

3. **Test organization**:
   - Unit tests are in each module's `tests` submodule (e.g., `src/markdown.rs` has `#[cfg(test)] mod tests`)
   - Integration tests are in `tests/readme.rs`
   - Each format has a corresponding README in `tests/` (e.g., `tests/README.adoc`, `tests/README.org`)

### Formatting and Linting Requirements

**Always run these checks before completing any request:**

1. **Format Rust code:**

   ```bash
   cargo fmt
   ```

2. **Check spelling** (uses `.cspell.json` configuration):

   ```bash
   npx cspell .
   ```

3. **Lint Markdown files:**

   ```bash
   npx markdownlint-cli README.md .github/**/*.md
   ```

These ensure code style consistency and documentation quality across the project.

### Code Organization Patterns

When adding support for a new documentation format:

1. **Create a new module** in `src/`:
   - File: `src/newformat.rs`
   - Module structure:

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

2. **Add the module** to `src/lib.rs`:
   - Declare the module: `mod newformat;`
   - Add the public macro with comprehensive documentation
   - Follow the pattern of existing macros (e.g., `include_markdown!`, `include_asciidoc!`)

3. **Create a README** in `tests/`:
   - File: `tests/README.newformat`
   - Follow the style and structure of existing READMEs:
     - Title: "Macros for including file content"
     - Explain what the macro does and why it's useful
     - Show examples with actual code fences in the new format
     - Include a full test example showing setup and usage
     - Keep the tone and style consistent with `tests/README.adoc`, `tests/README.org`, `tests/README.textile`

4. **Add integration tests** in `tests/readme.rs`:
   - Import the new macro
   - Add a test function following the existing pattern
   - Ensure the test uses the new README file

### Key Implementation Details

- **Path resolution**: The `open()` function handles both crate-relative (default) and source-relative paths
- **Error handling**: Use `syn::Error` for compile-time errors with helpful span information
- **Parsing strategy**: Each format module implements a `collect()` function that:
  - Takes a name and line iterator
  - Searches for named code blocks/fences
  - Returns collected lines as a `Vec<String>`
  - Returns empty vec if not found (converted to error by `extract()`)

### Common Patterns

- Use `#[cfg(test)]` for test modules
- Add copyright header to all source files: `// Copyright 2025 Heath Stewart.`
- Follow MIT license requirements
- Use descriptive test names: `test_<format>_<scenario>`
- Keep tests focused and independent

### Dependencies

- `proc-macro2`: Token stream manipulation with span locations
- `syn`: Parsing macro arguments
- `quote`: Only in dev dependencies for tests

### Rust Version

This project requires Rust 1.85.0 or newer (see `rust-toolchain.toml`).

## Example Workflow

When asked to add a new format:

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
   - `npx cspell .`
   - `npx markdownlint-cli README.md .github/**/*.md`
10. Verify all tests pass and checks complete successfully

## Questions to Ask

When uncertain about:

- Format-specific syntax → Consult the format's official documentation
- Parser edge cases → Add test cases to capture the behavior
- Integration patterns → Reference existing format implementations
