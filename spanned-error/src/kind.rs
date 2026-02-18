use crate::SpannedError;

pub struct Wrapped;
pub struct Unwrapped;

// Higher priority: matches &SpannedError<E> directly
pub trait WrappedKind {
    fn spanned_kind(&self) -> Wrapped;
}
impl<E> WrappedKind for SpannedError<E> {
    fn spanned_kind(&self) -> Wrapped {
        Wrapped
    }
}

// Lower priority: matches via autoref (&T -> &&T)
pub trait UnwrappedKind {
    fn spanned_kind(&self) -> Unwrapped;
}
impl<T> UnwrappedKind for &T {
    fn spanned_kind(&self) -> Unwrapped {
        Unwrapped
    }
}

impl Wrapped {
    pub fn into_spanned<E, F: From<E>>(self, err: SpannedError<E>) -> SpannedError<F> {
        SpannedError {
            inner: F::from(err.inner),
            span: err.span,
            location: err.location,
        }
    }
}

impl Unwrapped {
    #[track_caller]
    pub fn into_spanned<E, F: From<E>>(self, err: E) -> SpannedError<F> {
        SpannedError {
            inner: F::from(err),
            span: tracing::Span::current(),
            location: std::panic::Location::caller(),
        }
    }
}
