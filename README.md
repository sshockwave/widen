# widen

![MSRV](https://img.shields.io/badge/MSRV-1.71-blue)

Convert an enum into any type that accepts all its payload types.

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

Pair it with [thiserror] to build error unions from ordinary enums.
See [common patterns on docs.rs] for `thiserror` integration, backtrace capture,
and propagation with `?`. The [derive documentation] covers explicit variant mappings.

The runtime library is `no_std`.

Licensed under either MIT or Apache-2.0, at your option.

[thiserror]: https://docs.rs/thiserror
[common patterns on docs.rs]: https://docs.rs/widen/latest/widen/#common-patterns
[derive documentation]: https://docs.rs/widen/latest/widen/derive.Widen.html#explicit-variant-mappings
