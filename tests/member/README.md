# Test Member Crate

```rust example
let m = example()?;
assert_eq!(format!("{m:?}"), r#"Model { name: "example" }"#);
```

```rust assert-fail
assert!(false, "intentional assert failure");
```
