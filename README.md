# widen

![MSRV](https://img.shields.io/badge/MSRV-1.71-blue)
[![no_std](https://img.shields.io/badge/no_std-compatible-green)](https://crates.io/categories/no-std)
[![Crates.io License](https://img.shields.io/crates/l/widen)](https://crates.io/crates/widen)

Convert an enum to any of its supersets.

```toml
[dependencies]
widen = "0.1"
```

```rust
use widen::Widen;

#[derive(Widen)]
enum Number {
    Byte(u8),
    Word(u16),
}

let value: u32 = Number::Byte(42).widen();
assert_eq!(value, 42);
```

The derive implements `Widen<T>` for any `T: From<u8> + From<u16>`.

See the [API documentation] for common patterns like [`thiserror`] integration,
propagation with `?`, and backtrace capture.
The [derive documentation] covers explicit variant mappings.

[`thiserror`]: https://docs.rs/thiserror
[API documentation]: https://docs.rs/widen/latest/widen/#common-patterns
[derive documentation]: https://docs.rs/widen/latest/widen/derive.Widen.html#explicit-variant-mappings
