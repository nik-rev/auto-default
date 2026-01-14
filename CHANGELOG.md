# v0.2.0

It is now an error to use `#[auto_default(skip)]` on a field that has a default value:

```rust
#[auto_default]
struct User {
    #[auto_default(skip)]
    age: u32 = 4,
}
```

The `#[auto_default(skip)]` attribute will do nothing:

# v0.1.0

Initial release
