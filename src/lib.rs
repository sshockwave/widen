#![no_std]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

extern crate self as widen;

mod macros;

use core::{fmt, marker::PhantomData};

/// Derive [`trait@Subset`] for an enum whose variants each contain one payload.
///
/// The generated implementation requires the destination to implement `From`
/// for every payload type. It does not generate those `From` implementations.
#[cfg(feature = "derive")]
pub use widen_derive::Subset;

/// Widen a value into a destination determined by its context.
///
/// Derived implementations convert an enum by its payload types. The `Result`
/// implementation marks its error for conversion through [`Widen`].
///
/// Inclusion is a convention: arbitrary `From` implementations may transform
/// payloads, and variants with the same payload type may become indistinguishable.
pub trait Subset<T>: Sized {
    /// Convert the value or prepare its error for conversion.
    fn widen(self) -> T;
}

/// An error awaiting conversion into a destination chosen by its receiver.
///
/// `E` is the complete source error, including any wrapper or context.
/// `Target` is a type marker with no stored value. An integration can use it
/// for the destination payload type, such as `F` in `Traced<F>`.
///
/// Including `Target` in the source type allows an implementation such as
/// `From<Widen<Traced<E>, F>> for Traced<F>` to coexist with
/// `From<E> for Traced<E>`. Overlap would require the impossible type equation
/// `F = Widen<Traced<E>, F>`.
#[must_use]
pub struct Widen<E, Target> {
    error: E,
    target: PhantomData<fn() -> Target>,
}

impl<E, Target> Widen<E, Target> {
    /// Mark an error for conversion. The receiving context can infer `Target`.
    pub const fn new(error: E) -> Self {
        Self {
            error,
            target: PhantomData,
        }
    }

    /// Recover the complete source error so an integration can convert it.
    pub fn into_inner(self) -> E {
        self.error
    }
}

impl<E: fmt::Debug, Target> fmt::Debug for Widen<E, Target> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Widen").field(&self.error).finish()
    }
}

impl<T, E, Target> Subset<Result<T, Widen<E, Target>>> for Result<T, E> {
    /// Wrap the error in [`Widen`], preserving successful values.
    ///
    /// With a compatible destination wrapper, `operation().widen()?` infers
    /// `Target` from the enclosing function's return type. This method marks
    /// the error for conversion; the receiving `From` implementation converts it.
    fn widen(self) -> Result<T, Widen<E, Target>> {
        self.map_err(Widen::new)
    }
}
