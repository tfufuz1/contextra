//! Adapters between engine primitives and the erasure ports.

use contextra_ports::scanner::WitnessLayer;
use contextra_ports::{BoxFuture, Cancel, ErasureKernel, ProgressSink};
use contextra_types::{ErasureError, ErasureId, LayerProgress};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// `ProgressSink` that persists layer progress through the kernel (write-ahead).
pub struct KernelProgress {
    /// Kernel receiving the progress records.
    pub kernel: Arc<dyn ErasureKernel>,
    /// Job the progress belongs to.
    pub id: ErasureId,
}

impl ProgressSink for KernelProgress {
    fn record<'a>(
        &'a self,
        layer: &'a WitnessLayer,
        p: &'a LayerProgress,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move { self.kernel.record_layer(self.id, layer, p).await })
    }
}

/// `Cancel` port backed by a `tokio_util` token.
#[derive(Clone)]
pub struct TokenCancel(pub CancellationToken);

impl Cancel for TokenCancel {
    fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }

    fn cancelled(&self) -> BoxFuture<'_, ()> {
        Box::pin(self.0.cancelled())
    }
}
