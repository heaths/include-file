# Macros for including file content

[![releases](https://img.shields.io/github/v/release/heaths/include-file.svg?logo=github)](https://github.com/heaths/include-file/releases/latest)
[![docs](https://img.shields.io/docsrs/include-file?logo=rust)](https://docs.rs/include-file)
[![ci](https://github.com/heaths/include-file/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/heaths/include-file/actions/workflows/ci.yml)

Macros like `include_markdown!("README.md", "example")` allow you to include incomplete code from markdown code fences.
Though Rust doc tests let you hide setup code from being rendered, you cannot do the same when rendering markdown.
You can demonstrate just the code you want in markdown while maintaining the benefit of compiling it in tests.

## Examples

The `include_markdown!()` macro resolves a file path relative to the directory containing the crate `Cargo.toml` manifest file.

Consider a crate `README.md` with the following content:

````markdown
The `example()` function returns a model that implements `Debug` so you can easily print it:

```rust example
let m = example()?;
assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
```
````

We didn't define the `example()` function nor the type of `m`. In Rust doc tests you could do so with lines prefaced with `#` e.g.:

```rust
/// ```
/// # #[derive(Debug)] struct Model { name: String }
/// # fn example() -> Result<Model, Box<dyn std::error::Error>> { Ok(Model { name: "example".into() }) }
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let m = example()?;
/// println!("{m:#?}");
/// # Ok(()) }
/// ```
fn f() {}
```

All those lines would render in a markdown file. Instead, we could use `include_markdown!("README.md", "example")` to include the example content from `README.md` above in a test to make sure it compiles and even runs.

```rust
#[derive(Debug)]
struct Model {
    name: String,
}

fn example() -> Result<Model, Box<dyn std::error::Error>> {
    Ok(Model {
        name: "example".into(),
    })
}

#[test]
fn test_example() -> Result<(), Box<dyn std::error::Error>> {
    include_markdown!("README.md", "example");
    Ok(())
}
```

## Macros

Macro              | Feature    | Description
------------------ | ---------- | ---
`include_asciidoc` | `asciidoc` | Includes Rust snippets from AsciiDoc files, commonly with `.asciidoc`, `.adoc`, or `.asc` extensions.
`include_markdown` |            | Includes Rust snippets from Markdown files, commonly with `.markdown`, `.mdown`, `.mkdn`, or `.md` extensions.
`include_org`      | `org`      | Includes Rust snippets from Org files, commonly with `.org` extension.
`include_textile`  | `textile`  | Includes Rust snippets from Textile files, commonly with `.textile` extension.

All of these macros also support the following parameters:

Parameter  | Description
---------- | ---
`path`     | (*Required*) Path relative to the crate root directory.
`name`     | (*Required*) Name of the code fence to include.
`scope`    | Include the snippet in braces `{ .. }`.
`relative` | (*Requires rustc 1.88 or newer*) Path is relative to the source file calling the macro. May show an error in rust-analyzer until [rust-lang/rust-analyzer#15950](https://github.com/rust-lang/rust-analyzer/issues/15950) is fixed.

## License

Licensed under the [MIT](LICENSE.txt) license.
