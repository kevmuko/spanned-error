# spanned-error

A proc macro that automatically captures [tracing](https://docs.rs/tracing) spans and source locations at error sites.

## Usage

Annotate functions with `#[spanned_error]` and write normal `Result<T, E>` signatures. The macro rewrites the return type to `Result<T, SpannedError<E>>` and transforms every `?` and `return Err(...)` to capture the current tracing span and `#[track_caller]` location.

```rust
use spanned_error::prelude::*;

#[spanned_error]
async fn approve(&self, id: String) -> Result<Response, MyError> {
    let row = sqlx::query!("SELECT ...").fetch_optional(&pool).await?;
    if row.is_none() {
        return Err(MyError::NotFound);
    }
    Ok(response)
}
```

The compiled signature becomes `Result<Response, SpannedError<MyError>>`, and every error path automatically gets span + location metadata.

## Consuming `SpannedError`

Use `enter()` to re-enter the captured span when converting errors into responses. This ensures logs emitted during error handling are attributed to the original error site:

```rust
impl<E: Into<ErrorResponse> + fmt::Display> From<SpannedError<E>> for ErrorResponse {
    fn from(spanned: SpannedError<E>) -> Self {
        let _guard = spanned.enter();

        let message = spanned.inner.to_string();
        let response: ErrorResponse = spanned.inner.into();
        let status = StatusCode::from(response.code);

        if status.is_server_error() {
            error!(error = %message, code = ?response.code, ?status, "server error response");
        } else if status.is_client_error() {
            warn!(error = %message, code = ?response.code, ?status, "client error response");
        }

        response
    }
}
```

## How it works

### Autoref specialization

The macro uses [autoref-based specialization](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md) (the same pattern as `anyhow`) to detect whether an error is already wrapped in `SpannedError`:

- **Plain error** — gets a fresh `SpannedError` wrap with current span + location
- **Already `SpannedError<E>`** — preserves the original span + location, only converts the inner error via `From`

This means nested `#[spanned_error]` functions preserve the original error site through the call chain.

### What gets transformed

| Pattern | Transformation |
|---|---|
| `expr?` | Match + autoref dispatch wrap |
| `return Err(e)` | Wrap `e` via autoref dispatch |
| Tail `Err(e)` | Wrap `e` via autoref dispatch |

Closures, async blocks, and nested `fn` items are **not** transformed — errors from these bubble out and get wrapped by the outer `?`.

## SpannedError\<E\>

```rust
pub struct SpannedError<E> {
    pub inner: E,
    pub span: tracing::Span,
    pub location: &'static Location<'static>,
}
```

- `enter()` — re-enters the captured span with a child span annotated with the source location
- `Debug` / `Display` / `Error` — delegate to the inner error

## Attribute ordering

`#[spanned_error]` must be the closest attribute to the function (below `#[instrument]`, `#[tracing::instrument]`, etc.):

```rust
#[instrument(skip(self))]
#[spanned_error]
async fn my_fn() -> Result<(), MyError> { ... }
```

## Limitations

- Return type must be a literal `Result<T, E>` (not a type alias)
- `std::result::Result<T, E>` and bare `Result<T, E>` both work (checks last path segment)
