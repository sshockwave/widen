use widen::type_enum;

#[derive(Debug, PartialEq)]
struct ParseError(String);

type_enum! {
    #[derive(Debug, PartialEq)]
    enum Error {
        ParseError
    }
}

#[test]
fn constructs_and_matches_shorthand_payloads() {
    let payload = String::from("invalid number");
    let address = payload.as_ptr();
    let Error::ParseError(ParseError(payload)) = Error::ParseError(ParseError(payload));
    assert_eq!(payload, "invalid number");
    assert_eq!(payload.as_ptr(), address);
}

type_enum! {
    #[derive(Debug, PartialEq)]
    pub enum Generic<'a, T, const N: usize>
    where
        T: Copy,
    {
        T,
        Values([T; N]),
        Borrowed { values: &'a [T; N] },
        #[cfg(any())]
        Unavailable,
    }
}

#[test]
fn preserves_generics_explicit_fields_and_attributes() {
    let values = [1, 2, 3];
    assert_eq!(Generic::<u32, 3>::T(7), Generic::T(7));
    assert_eq!(Generic::Values(values), Generic::Values([1, 2, 3]));
    let Generic::Borrowed { values: borrowed } = (Generic::Borrowed { values: &values }) else {
        unreachable!()
    };
    assert!(std::ptr::eq(borrowed, &values));
}

#[cfg(feature = "derive")]
#[test]
fn supports_subset_and_thiserror_derives() {
    use widen::Subset;

    #[derive(Debug, thiserror::Error)]
    #[error("parse failed")]
    struct ParseError;

    type_enum! {
        #[derive(Debug, thiserror::Error, Subset)]
        enum Error {
            #[error(transparent)]
            ParseError,
            #[error(transparent)]
            Io(#[from] std::io::Error),
        }
    }

    fn convert(error: Error) -> Box<dyn std::error::Error + Send + Sync> {
        error.widen()
    }

    assert_eq!(
        convert(Error::ParseError(ParseError)).to_string(),
        "parse failed"
    );
    let error = Error::from(std::io::Error::other("read failed"));
    assert_eq!(convert(error).to_string(), "read failed");
}
