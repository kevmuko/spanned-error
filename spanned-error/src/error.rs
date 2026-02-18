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
