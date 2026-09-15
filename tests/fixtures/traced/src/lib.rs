use widen::{Subset, Widen};

/// Minimal stand-in for an application's error wrapper.
#[derive(Debug, PartialEq)]
pub struct Traced<E> {
    pub error: E,
    pub context: Vec<&'static str>,
}

// Both From impls must coexist in a crate separate from widen and its callers.
impl<E> From<E> for Traced<E> {
    fn from(error: E) -> Self {
        Self {
            error,
            context: Vec::new(),
        }
    }
}

impl<E: Subset<F>, F> From<Widen<Traced<E>, F>> for Traced<F> {
    fn from(error: Widen<Traced<E>, F>) -> Self {
        let Traced { error, context } = error.into_inner();
        Self {
            error: error.widen(),
            context,
        }
    }
}
