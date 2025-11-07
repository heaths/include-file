# Macros for including file content

[![releases](https://img.shields.io/github/v/release/heaths/include-file.svg?logo=github)](https://github.com/heaths/include-file/releases/latest)
[![docs](https://img.shields.io/docsrs/include-file?logo=rust)](https://docs.rs/include-file)
[![ci](https://github.com/heaths/include-file/actions/workflows/ci.yml/badge.svg?event=push)](https://github.com/heaths/include-file/actions/workflows/ci.yml)

Macros like `include_code!("../README.md", "example")` allow you to include incomplete code from markdown code fences.
Though Rust doc tests let you hide setup code from being rendered, you cannot do the same when rendering markdown.
You can demonstrate just the code you want in markdown while maintaining the benefit of compiling it in tests.

## Examples

The `include_code!()` macro, like Rust's built-in `include_str!()`, resolves a file path relative to the current source file.

Consider a crate `README.md` with the following content:

````markdown
The `example()` function returns a model that implements `Debug` so you can easily print it:

```rust example
let m = example()?;
assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
```
````

We didn't define the `example()` function nor the type of `x`. In Rust doc tests you could do so with lines prefaced with `#` e.g.:

```rust
/// ```
/// # #[derive(Debug)] struct Model { name: String }
/// # fn example() -> Result<Model, Box<dyn std::error::Error>> { Ok(Model { name: "example".into() }) }
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let m = example()?;
/// println!("{m:#?}");
/// # Ok(()) }
/// ```
```

All those lines would render in a markdown file. Instead, we could use `include_code!("../README.md", "example")` to include the example content from `README.md` above in a test to make sure it compiles and even runs.

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
    include_code!("../README.md", "example");
    Ok(())
}
```

## License

Licensed under the [MIT](LICENSE.txt) license.
