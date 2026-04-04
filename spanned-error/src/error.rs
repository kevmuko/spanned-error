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

    pub fn in_scope<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = self.enter();
        f()
    }

    pub fn map<E2>(self, f: impl FnOnce(E) -> E2) -> SpannedError<E2> {
        SpannedError {
            inner: f(self.inner),
            span: self.span,
            location: self.location,
        }
    }
}

impl<E: std::error::Error> fmt::Debug for SpannedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // {:#?} — multiline for CLI/human reading
            write!(f, "{}", self.inner)?;
            if let Some(cause) = self.inner.source() {
                write!(f, "\n\nCaused by:")?;
                let mut current: &dyn std::error::Error = cause;
                let mut n = 0;
                loop {
                    write!(f, "\n    {n}: {current}")?;
                    n += 1;
                    match current.source() {
                        Some(next) => current = next,
                        None => break,
                    }
                }
            }
        } else {
            // {:?} — single-line for tracing/structured logs
            write!(f, "{}", self.inner)?;
            let mut current: &dyn std::error::Error = &self.inner;
            while let Some(source) = current.source() {
                write!(f, ": {source}")?;
                current = source;
            }
        }

        Ok(())
    }
}

impl<E: fmt::Display> fmt::Display for SpannedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<E: std::error::Error> std::error::Error for SpannedError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

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

pub trait SpannedResultExt<T, E> {
    fn map_spanned<E2>(self, f: impl FnOnce(E) -> E2) -> Result<T, SpannedError<E2>>;
}

impl<T, E> SpannedResultExt<T, E> for Result<T, SpannedError<E>> {
    fn map_spanned<E2>(self, f: impl FnOnce(E) -> E2) -> Result<T, SpannedError<E2>> {
        self.map_err(|e| e.map(f))
    }
}
