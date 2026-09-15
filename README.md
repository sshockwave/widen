# widen

Convert between sum types by their payloads, and propagate errors through your
own error wrapper on stable Rust.

## Convert ordinary enums

`Subset<T>` means that `T` accepts every payload type in the source enum.
`#[derive(Subset)]` generates the conversion by matching each variant and calling
`From` on its payload. Tuple and named variants with one field are supported,
including generic enums.

```rust
# #[cfg(feature = "derive")]
# {
use widen::Subset;

struct ParseError;
struct ConfigError;

#[derive(Subset)]
enum ReadError {
    Parse(ParseError),
}

enum AppError {
    Parse(ParseError),
    Config(ConfigError),
}

impl From<ParseError> for AppError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

fn convert(error: ReadError) -> AppError {
    error.widen()
}
# }
```

The derive generates only `Subset`; provide the payload `From` implementations
yourself or use a derive such as `thiserror::Error` with `#[from]`.
Conversions follow those implementations, so preserving payloads is a convention,
not a property enforced by the trait.

## Shorthand declarations

`type_enum!` removes the repeated type name in variants such as
`ParseError(ParseError)`:

```rust
# #[derive(Debug)]
# struct ParseError;
# #[derive(Debug)]
# struct ConfigError;
widen::type_enum! {
    #[derive(Debug)]
    enum Error {
        ParseError,
        ConfigError,
        Io(std::io::Error),
    }
}
```

Bare identifiers expand to variants with the same name and payload type.
Explicit tuple and named variants, generics, visibility, and attributes are
preserved. Use explicit variants when field attributes are needed, such as
`Io(#[from] std::io::Error)` with `thiserror`.

This is syntax shorthand only: add `#[derive(Subset)]` and `From` implementations
as usual. It is available even with derive support disabled.

## Propagate through your own error wrapper

`result.map_err(Subset::widen)?` generally needs a target annotation because `?`
allows another conversion. The `Subset` implementation for `Result` instead marks
the complete error with `Widen<E, Target>`. Your wrapper supplies the conversion
to a known target.

```rust
use widen::{Subset, Widen};

// In the crate that defines your error wrapper:
struct Traced<E> {
    error: E,
    context: Vec<String>,
}

impl<E> From<E> for Traced<E> {
    fn from(error: E) -> Self {
        Self { error, context: Vec::new() }
    }
}

impl<E: Subset<F>, F> From<Widen<Traced<E>, F>> for Traced<F> {
    fn from(error: Widen<Traced<E>, F>) -> Self {
        let Traced { error, context } = error.into_inner();
        Self { error: error.widen(), context }
    }
}

// Callers need no target annotation:
fn higher<E: Subset<F>, F>(result: Result<u32, Traced<E>>)
    -> Result<u32, Traced<F>>
{
    let value = result.widen()?;
    Ok(value + 1)
}
```

Keep any backtrace or other wrapper metadata in this conversion, just as the
example keeps `context`. The wrapper's crate owns the `From` implementation;
individual destination enums need no wrapper-specific implementation.

The `Target` marker occupies no space. It also prevents overlap with the existing
`From<E> for Traced<E>`: overlap would require the impossible recursive type
equation `F = Widen<Traced<E>, F>`. This uses standard `Result` and `From`, with no
custom `Try` implementation.

The runtime library is `no_std`. Derive support is enabled by default and can be
disabled with `default-features = false`.
