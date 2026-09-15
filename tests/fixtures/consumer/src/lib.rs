#[cfg(test)]
mod tests {
    use sets::Subset;
    use widen_test_traced::Traced;

    #[derive(Debug, PartialEq)]
    struct ParseError(&'static str);

    #[derive(Debug, PartialEq)]
    struct IoError(u32);

    #[derive(Subset)]
    enum ReadError {
        Parse(ParseError),
        Io(IoError),
    }

    // The destination needs no Subset derive or wrapper-specific conversion.
    #[derive(Debug, PartialEq)]
    enum AppError {
        Parse(ParseError),
        Io(IoError),
    }

    impl From<ParseError> for AppError {
        fn from(error: ParseError) -> Self {
            Self::Parse(error)
        }
    }

    impl From<IoError> for AppError {
        fn from(error: IoError) -> Self {
            Self::Io(error)
        }
    }

    fn lower(result: Result<u32, ReadError>) -> Result<u32, Traced<ReadError>> {
        let value = result?;
        Ok(value)
    }

    fn higher(result: Result<u32, Traced<ReadError>>) -> Result<u32, Traced<AppError>> {
        let value = result.widen()?;
        Ok(value + 1)
    }

    fn generic<E: Subset<F>, F>(result: Result<u32, Traced<E>>) -> Result<u32, Traced<F>> {
        let value = result.widen()?;
        Ok(value + 1)
    }

    #[test]
    fn preserves_plain_from_and_infers_target() {
        assert_eq!(higher(lower(Ok(7))), Ok(8));
        assert_eq!(
            higher(lower(Err(ReadError::Io(IoError(42))))),
            Err(Traced::from(AppError::Io(IoError(42)))),
        );
    }

    #[test]
    fn preserves_metadata_and_both_payloads() {
        for (source, expected) in [
            (ReadError::Io(IoError(42)), AppError::Io(IoError(42))),
            (
                ReadError::Parse(ParseError("invalid number")),
                AppError::Parse(ParseError("invalid number")),
            ),
        ] {
            let result = higher(Err(Traced {
                error: source,
                context: vec!["read configuration", "startup"],
            }));
            assert_eq!(
                result,
                Err(Traced {
                    error: expected,
                    context: vec!["read configuration", "startup"],
                }),
            );
        }
    }

    #[test]
    fn infers_target_in_generic_function() {
        assert_eq!(
            generic(lower(Err(ReadError::Io(IoError(42))))),
            Err(Traced::from(AppError::Io(IoError(42)))),
        );
    }
}
