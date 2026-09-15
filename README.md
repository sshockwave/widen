# widen

Convert between Rust sum types using the idea of subset inclusion, and propagate
errors through your own error wrapper on stable Rust.

## Convert ordinary enums

For enums, `Widen<T>` means that `T` accepts every payload type in the source enum.
Without `#[subsume(...)]` attributes, `#[derive(Widen)]` generates the conversion
by matching each variant and calling `From` on its payload. Tuple and named
variants with one field are supported, including generic enums.

```rust
# #[cfg(feature = "derive")]
# {
use widen::Widen;

struct ParseError;
struct ConfigError;

#[derive(Widen)]
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

Here, `ReadError`'s payload types form a **subset** of `AppError`'s. The destination
can **subsume** the source: `AppError` can represent every payload that `ReadError`
carries. Widening embeds the payload in the corresponding destination variant.

In this mode, the derive generates only `Widen`; provide the payload `From`
implementations yourself or use a derive such as `thiserror::Error` with `#[from]`.
Subset inclusion is the model, while `From` implementations control the actual
conversion. Preserving payloads is a convention, not a property enforced by the
trait.

## Map variants explicitly

Use `#[subsume(...)]` on destination variants to flatten source enums, including
unit variants. The derive generates `From<A>`, `From<B>`, and so on, using an
exhaustive match for each source. Source enums need no derive or helper macro.

```rust
# #[cfg(feature = "derive")]
# {
use widen::Widen;

enum A {
    Shared,
    Exclusive,
    Normal(u32),
}

enum B {
    Shared,
    Exclusive,
    Normal(u16),
}

#[derive(Debug, PartialEq, Widen)]
enum C {
    #[subsume(A::Shared, B::Shared)]
    SharedVariant,
    #[subsume(A::Exclusive)]
    VariantA,
    #[subsume(B::Exclusive)]
    VariantB,
    #[subsume(A::Normal, from(B::Normal))]
    Normal(u32),
}

assert_eq!(C::from(A::Shared), C::SharedVariant);
assert_eq!(C::from(B::Exclusive), C::VariantB);
assert_eq!(C::from(B::Normal(7)), C::Normal(7));
# }
```

Plain mappings forward fields unchanged. `from(Source::Variant)` opts into
`.into()` for each field of that source variant. Unit, tuple, and named variants
are supported; source and destination must have the same field shape, including
field names for named variants. Generic source paths and destination enums work
with explicit bounds where payload conversions require them.

Every variant of each listed source must be covered exactly once. Omitting one
is a Rust exhaustiveness error; extra destination variants are allowed. Source
enums marked `#[non_exhaustive]` in another crate cannot be exhaustively flattened.

This mode generates incoming `From` implementations instead of an outgoing
`Widen<T>` implementation. Use `.into()` or ordinary `?` propagation for these
conversions. Existing payload-based `Widen` implementations do not automatically
dispatch through them.

Ordinary derives and `thiserror` attributes can be combined with `subsume`.
For example, `#[error("{}", A::Shared)]` delegates a unit variant's display, and
`#[from]` fields can generate additional payload conversions.

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

This is syntax shorthand only: add `#[derive(Widen)]` and `From` implementations
as usual. It is available even with derive support disabled.

## Propagate through your own error wrapper

`result.map_err(Widen::widen)?` generally needs a target annotation because `?`
allows another conversion. The `Widen` implementation for `Result` instead marks
the complete error with `Widening<E, Target>`. Your wrapper supplies the conversion
to a known target.

```rust
use widen::{Widen, Widening};

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

impl<E: Widen<F>, F> From<Widening<Traced<E>, F>> for Traced<F> {
    fn from(error: Widening<Traced<E>, F>) -> Self {
        let Traced { error, context } = error.into_inner();
        Self { error: error.widen(), context }
    }
}

// Callers need no target annotation:
fn higher<E: Widen<F>, F>(result: Result<u32, Traced<E>>)
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
equation `F = Widening<Traced<E>, F>`. This uses standard `Result` and `From`, with no
custom `Try` implementation.

The runtime library is `no_std`. Derive support is enabled by default and can be
disabled with `default-features = false`.
