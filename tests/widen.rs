#![cfg(feature = "derive")]

use thiserror::Error;
use widen::Widen;

#[derive(Debug, Error, PartialEq)]
#[error("invalid number: {0}")]
struct ParseError(String);

#[derive(Debug, Error, PartialEq)]
#[error("I/O error: {0}")]
struct IoError(u32);

#[derive(Debug, Error, PartialEq)]
#[error("missing configuration")]
struct ConfigError;

#[derive(Debug, Error, Widen)]
enum ReadError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Io(#[from] IoError),
}

#[derive(Debug, Error, PartialEq)]
enum AppError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error(transparent)]
    Config(#[from] ConfigError),
}

fn convert(error: ReadError) -> AppError {
    error.widen()
}

fn tail_return(result: Result<(), ReadError>) -> Result<(), AppError> {
    result.map_err(Widen::widen)
}

#[test]
fn converts_every_payload_with_thiserror_from_impls() {
    assert_eq!(
        convert(ReadError::Parse(ParseError("abc".into()))),
        AppError::Parse(ParseError("abc".into())),
    );
    assert_eq!(
        convert(ReadError::Io(IoError(42))),
        AppError::Io(IoError(42)),
    );
    assert_eq!(tail_return(Ok(())), Ok(()));
    assert_eq!(
        tail_return(Err(ReadError::Io(IoError(42)))),
        Err(AppError::Io(IoError(42))),
    );
}

#[test]
fn can_widen_to_the_same_enum() {
    let error: ReadError = ReadError::Io(IoError(42)).widen();
    assert!(matches!(error, ReadError::Io(IoError(42))));
}

#[derive(Widen)]
enum Borrowed<'a, T, const N: usize>
where
    T: Copy + 'a,
{
    Values { values: &'a [T; N] },
}

#[test]
fn supports_named_fields_lifetimes_and_const_generics() {
    fn recover<T: Copy, const N: usize>(values: &[T; N]) -> &[T; N] {
        Borrowed::Values { values }.widen()
    }
    let values = [1, 2, 3];
    assert!(std::ptr::eq(recover(&values), &values));
}

#[derive(Widen)]
enum Collision<__WidenTarget = u32> {
    Value(__WidenTarget),
}

#[test]
fn preserves_default_generics_and_avoids_generated_name_collisions() {
    fn recover<T>(value: T) -> T {
        Collision::Value(value).widen()
    }
    assert_eq!(recover(String::from("payload")), "payload");
}

#[derive(Widen)]
enum Empty {}

#[test]
fn empty_enum_can_widen_to_any_target() {
    fn convert(empty: Empty) -> u32 {
        empty.widen()
    }
    let _: fn(Empty) -> u32 = convert;
}
