//! WASM Merge Operator Host port trait (§4.3, §4.12).

#![forbid(unsafe_code)]

use super::BoxFuture;
use contextra_types::Result;
use std::any::Any;
use std::sync::Arc;

/// Host interface for admitting WASM merge functions into a CRDT-verified operator registry.
pub trait MergeOperatorHost: Send + Sync {
    /// Admits a compiled WASM merge function into the registry after CRDT verification.
    ///
    /// The `func` parameter is passed as an type-erased `Arc<dyn Any + Send + Sync>` to decouple
    /// Ring 0 (`contextra-ports`) from concrete WASM sandbox types in higher rings.
    fn admit_wasm_function<'a>(
        &'a self,
        func: Arc<dyn Any + Send + Sync>,
    ) -> BoxFuture<'a, Result<[u8; 32]>>;
}
