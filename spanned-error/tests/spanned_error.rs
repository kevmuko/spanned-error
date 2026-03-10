use std::{fmt, io};

use spanned_error::{ResultExt, SpannedError, spanned_error};

#[derive(Debug)]
enum MyError {
    Io(io::Error),
    Custom(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::Io(e) => write!(f, "io: {e}"),
            MyError::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl From<io::Error> for MyError {
    fn from(e: io::Error) -> Self {
        MyError::Io(e)
    }
}

impl From<SpannedError<io::Error>> for MyError {
    fn from(e: SpannedError<io::Error>) -> Self {
        MyError::Io(e.inner)
    }
}

fn ok_value() -> Result<i32, io::Error> {
    Ok(42)
}

fn plain_err() -> Result<i32, io::Error> {
    Err(io::Error::new(io::ErrorKind::NotFound, "not found"))
}

#[test]
fn sync_try_plain_error() {
    #[spanned_error]
    fn inner() -> Result<i32, MyError> {
        let val = plain_err()?;
        Ok(val)
    }

    let result = inner();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.inner, MyError::Io(_)));
}

#[test]
fn sync_try_ok_path() {
    #[spanned_error]
    fn inner() -> Result<i32, MyError> {
        let val = ok_value()?;
        Ok(val)
    }

    let result = inner();
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn sync_return_err() {
    #[spanned_error]
    fn inner() -> Result<i32, MyError> {
        return Err(MyError::Custom("bad".into()));
    }

    let result = inner();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.inner, MyError::Custom(ref s) if s == "bad"));
}

#[test]
fn sync_tail_err() {
    #[spanned_error]
    fn inner() -> Result<i32, MyError> {
        Err(MyError::Custom("tail".into()))
    }

    let result = inner();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.inner, MyError::Custom(ref s) if s == "tail"));
}

#[test]
fn async_try_plain_error() {
    async fn async_plain_err() -> Result<i32, io::Error> {
        Err(io::Error::new(io::ErrorKind::NotFound, "async not found"))
    }

    #[spanned_error]
    async fn inner() -> Result<i32, MyError> {
        let val = async_plain_err().await?;
        Ok(val)
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let result = rt.block_on(inner());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err().inner, MyError::Io(_)));
}

#[test]
fn nested_spanned_preserves_inner_span() {
    // Inner function returns SpannedError<io::Error>
    #[spanned_error]
    fn inner_fn() -> Result<i32, io::Error> {
        plain_err()?;
        Ok(0)
    }

    // Outer function takes the SpannedError<io::Error> from inner and should
    // preserve its span
    #[spanned_error]
    fn outer_fn() -> Result<i32, MyError> {
        let val = inner_fn()?;
        Ok(val)
    }

    let result = outer_fn();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.inner, MyError::Io(_)));
}

#[test]
fn closures_not_transformed() {
    // The closure's ? should NOT be transformed by the macro.
    // The closure returns Result<i32, io::Error> and the outer ? wraps it.
    #[spanned_error]
    fn inner() -> Result<Vec<i32>, MyError> {
        let items = vec![1, 2, 3];
        let mapped: Result<Vec<i32>, io::Error> = items
            .into_iter()
            .map(|_| -> Result<i32, io::Error> {
                let v = plain_err()?; // closure ?, not transformed
                Ok(v)
            })
            .collect();
        let result = mapped?; // outer ?, transformed
        Ok(result)
    }

    let result = inner();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err().inner, MyError::Io(_)));
}

#[test]
fn result_ext_into_spanned_ok() {
    let result = ok_value().into_spanned();
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn result_ext_into_spanned_err() {
    let result = plain_err().into_spanned();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().inner.kind(), io::ErrorKind::NotFound);
}
