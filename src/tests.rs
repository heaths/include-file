// Copyright 2025 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use super::{extract, open};
use std::io;

#[test]
fn open_file() {
    let file = open("../README.md").expect("expected README.md");
    assert!(matches!(file.metadata(), Ok(meta) if meta.is_file()));
}

#[test]
fn open_err() {
    assert!(matches!(open("missing.txt"), Err(err) if err.kind() == io::ErrorKind::NotFound));
}

#[test]
fn extract_no_code_fences() {
    let content = r#"This is a markdown file
with no code fences at all.
Just plain text."#;
    let result = extract(content.as_bytes(), "example");
    assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
}

#[test]
fn extract_no_matching_name() {
    let content = r#"Some text here.

```rust
fn main() {
    println!("Hello");
}
```

More text."#;
    let result = extract(content.as_bytes(), "example");
    assert!(matches!(result, Err(err) if err.kind() == io::ErrorKind::NotFound));
}

#[test]
fn extract_multiple_fences_one_match() {
    let content = r#"Here's the first fence:

```javascript
console.log("Not this one");
```

And here's the one we want:

~~~rust example
fn main() {
    println!("Hello, world!");
}
~~~

And another one:

```python
print("Also not this one")
```"#;
    let result = extract(content.as_bytes(), "example").expect("expected content");
    assert_eq!(
        result,
        r#"fn main() {
    println!("Hello, world!");
}"#
    );
}

#[test]
fn extract_nested_code_fence() {
    let content = r#"Outer content:

````markdown example
# Example

Here's a nested code fence:

```rust
fn nested() {
    println!("Inner code");
}
```

More content.
````

After the fence."#;
    let result = extract(content.as_bytes(), "example").expect("expected content");
    assert_eq!(
        result,
        r#"# Example

Here's a nested code fence:

```rust
fn nested() {
    println!("Inner code");
}
```

More content."#
    );
}

#[test]
fn extract_with_indentation() {
    let content = r#"Normal text.

  ~~~rust example
  fn indented() {
      println!("Indented code");
  }
  ~~~

More text."#;
    let result = extract(content.as_bytes(), "example").expect("expected content");
    assert_eq!(
        result,
        r#"fn indented() {
    println!("Indented code");
}"#
    );
}

#[test]
fn extract_backticks_with_name() {
    let content = r#"Text before.

```rust example
let x = 42;
let y = x + 1;
```

Text after."#;
    let result = extract(content.as_bytes(), "example").expect("expected content");
    assert_eq!(
        result,
        r#"let x = 42;
let y = x + 1;"#
    );
}

#[test]
fn extract_tildes_with_name() {
    let content = r#"Text before.

~~~python example
def hello():
    print("Hello")
~~~

Text after."#;
    let result = extract(content.as_bytes(), "example").expect("expected content");
    assert_eq!(
        result,
        r#"def hello():
    print("Hello")"#
    );
}

#[test]
fn extract_more_closing_chars() {
    let content = r#"Text before.

```rust example
fn test() {
    println!("test");
}
`````

Text after."#;
    let result = extract(content.as_bytes(), "example").expect("expected content");
    assert_eq!(
        result,
        r#"fn test() {
    println!("test");
}"#
    );
}
