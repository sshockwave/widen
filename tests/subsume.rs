#![cfg(feature = "derive")]

use thiserror::Error;
use widen::Widen;

#[derive(Debug, Error, PartialEq)]
#[error("normal: {0}")]
struct Normal(String);

impl From<String> for Normal {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Error)]
enum A {
    #[error("shared")]
    Shared,
    #[error("exclusive to A")]
    Exclusive,
    #[error(transparent)]
    Normal(Normal),
}

enum B {
    Shared,
    Exclusive,
    Normal(String),
}

#[derive(Debug, Error, PartialEq, Widen)]
enum C {
    #[error("{}", A::Shared)]
    #[subsume(A::Shared, B::Shared)]
    SharedVariant,

    #[error("exclusive to A")]
    #[subsume(A::Exclusive)]
    VariantA,

    #[error("exclusive to B")]
    #[subsume(B::Exclusive)]
    VariantB,

    #[error(transparent)]
    #[subsume(A::Normal, from(B::Normal))]
    Normal(#[from] Normal),

    #[error("only in C")]
    Extra,

    #[cfg(any())]
    #[subsume(Unavailable::Variant)]
    Unavailable,
}

#[test]
fn merges_shared_units_and_preserves_exclusive_cases() {
    assert_eq!(C::from(A::Shared), C::SharedVariant);
    assert_eq!(C::from(B::Shared), C::SharedVariant);
    assert_eq!(C::from(A::Exclusive), C::VariantA);
    assert_eq!(C::from(B::Exclusive), C::VariantB);
    assert_eq!(C::from(A::Shared).to_string(), "shared");
    assert_eq!(C::Extra.to_string(), "only in C");
}

#[test]
fn forwards_owned_payloads_and_converts_only_marked_sources() {
    let text = String::from("forwarded");
    let address = text.as_ptr();
    let C::Normal(Normal(text)) = C::from(A::Normal(Normal(text))) else {
        unreachable!()
    };
    assert_eq!(text, "forwarded");
    assert_eq!(text.as_ptr(), address);

    assert_eq!(
        C::from(B::Normal("converted".into())),
        C::Normal(Normal("converted".into())),
    );
    // thiserror's payload From implementation coexists with both enum conversions.
    assert_eq!(
        C::from(Normal("payload".into())),
        C::Normal(Normal("payload".into())),
    );
}

#[test]
fn question_mark_infers_the_destination() {
    fn promote(result: Result<u32, A>) -> Result<u32, C> {
        Ok(result? + 1)
    }
    assert_eq!(promote(Ok(7)), Ok(8));
    assert_eq!(promote(Err(A::Exclusive)), Err(C::VariantA));
}

mod source {
    pub enum Generic<'a, T, const N: usize> {
        Values { values: &'a [T; N] },
        Pair(T, T),
    }
}

#[derive(Debug, PartialEq, Widen)]
enum Generic<'a, T = u32, const N: usize = 2>
where
    T: Copy + 'a,
{
    #[subsume(source::Generic::<'a, T, N>::Values)]
    Borrowed { values: &'a [T; N] },
    #[subsume(source::Generic<'a, T, N>::Pair)]
    Pair(T, T),
}

#[test]
fn supports_matching_named_and_tuple_fields_with_generics() {
    let values = [1, 2];
    let Generic::Borrowed { values: borrowed } =
        Generic::from(source::Generic::Values { values: &values })
    else {
        unreachable!()
    };
    assert!(std::ptr::eq(borrowed, &values));
    assert_eq!(
        Generic::from(source::Generic::Pair(3, 4)),
        Generic::<'_, u32, 2>::Pair(3, 4)
    );
}

enum Byte {
    Value(u8),
}

#[derive(Debug, PartialEq, Widen)]
enum Converted<T>
where
    T: From<u8>,
{
    #[subsume(from(Byte::Value))]
    Value(T),
}

#[test]
fn respects_explicit_generic_conversion_bounds() {
    assert_eq!(Converted::<u32>::from(Byte::Value(7)), Converted::Value(7));
}

#[test]
fn supports_external_enums_and_merges_cases_from_the_same_source() {
    use core::cmp::Ordering;

    #[derive(Debug, PartialEq, Widen)]
    enum Comparison {
        #[subsume(Ordering::Less, Ordering::Greater)]
        Unequal,
        #[subsume(Ordering::Equal)]
        Equal,
    }

    assert_eq!(Comparison::from(Ordering::Less), Comparison::Unequal);
    assert_eq!(Comparison::from(Ordering::Greater), Comparison::Unequal);
    assert_eq!(Comparison::from(Ordering::Equal), Comparison::Equal);
}
