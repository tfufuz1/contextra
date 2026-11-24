//! Algorithm error bridge for workspace compatibility.

use crate::tl_hfd::TlHfdError;
#[cfg(feature = "apprh-diffusion")]
use crate::apprh::ApprhError;
use contextra_types::ContextraError;

impl From<TlHfdError> for ContextraError {
    fn from(err: TlHfdError) -> Self {
        ContextraError::InvalidInput(err.to_string())
    }
}

#[cfg(feature = "apprh-diffusion")]
impl From<ApprhError> for ContextraError {
    fn from(err: ApprhError) -> Self {
        ContextraError::InvalidInput(err.to_string())
    }
}
