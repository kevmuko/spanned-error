mod error;
mod kind;

pub use error::{ResultExt, SpannedError, SpannedResultExt};
pub use spanned_error_macros::spanned_error;

pub mod prelude {
    pub use super::{ResultExt, SpannedError, SpannedResultExt, spanned_error};
}

#[doc(hidden)]
pub mod __private {
    pub mod kind {
        pub use crate::kind::*;
    }
}
