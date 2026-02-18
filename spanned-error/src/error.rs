use std::{fmt, panic::Location};

use tracing::{Span, span::EnteredSpan};

pub struct SpannedError<E> {
    pub inner: E,
    pub span: Span,
    pub location: &'static Location<'static>,
}

impl<E> SpannedError<E> {
    pub fn enter(&self) -> EnteredSpan {
        tracing::info_span!(parent: &self.span, "source", location = %self.location).entered()
    }
}

impl<E: fmt::Debug> fmt::Debug for SpannedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<E: fmt::Display> fmt::Display for SpannedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<E: fmt::Debug + fmt::Display> std::error::Error for SpannedError<E> {}

pub trait ResultExt<T, E> {
    fn into_spanned(self) -> Result<T, SpannedError<E>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn into_spanned(self) -> Result<T, SpannedError<E>> {
        self.map_err(|inner| SpannedError {
            inner,
            span: tracing::Span::current(),
            location: std::panic::Location::caller(),
        })
    }
}
