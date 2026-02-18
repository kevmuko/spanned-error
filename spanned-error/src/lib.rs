mod error;
mod kind;

pub use error::{ResultExt, SpannedError};
pub use spanned_error_macros::spanned_error;

pub mod prelude {
    pub use super::{ResultExt, SpannedError, spanned_error};
}

#[doc(hidden)]
pub mod __private {
    pub mod kind {
        pub use crate::kind::*;
    }
}
