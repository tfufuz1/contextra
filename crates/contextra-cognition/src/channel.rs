//! Abstract communication and intake channels for `contextra-cognition`.
//!
//! Decouples cognitive memory consolidation, context compaction, and RAG synthesis
//! from specific agent frameworks, queues, or HTTP transports. External inputs
//! are accepted as abstract streams, channels, or queues.

use crate::memory_consolidation::{PreCondensedInput, TurnSegment};
use crate::ports_local::BoxFuture;
use crate::types_local::{Result, TenantId};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Abstract asynchronous input channel for feeding cognitive inputs.
pub trait CognitiveInputChannel: Send + Sync {
    /// Ingests a pre-condensed input batch into the cognition engine.
    fn send_pre_condensed<'a>(
        &'a self,
        target_tenant_id: &'a TenantId,
        inputs: &'a [PreCondensedInput],
    ) -> BoxFuture<'a, Result<Vec<TurnSegment>>>;
}

/// Abstract asynchronous output sink for dispatching consolidated or synthesized chunks.
pub trait CognitiveOutputSink<T>: Send + Sync {
    /// Emits a consolidated item to the destination sink.
    fn emit<'a>(&'a self, item: T) -> BoxFuture<'a, Result<()>>;
}

/// In-Memory asynchronous MPSC implementation of `CognitiveInputChannel`.
pub struct InMemoryCognitiveChannel {
    tx: mpsc::Sender<(TenantId, Vec<PreCondensedInput>)>,
}

impl InMemoryCognitiveChannel {
    /// Creates a new `InMemoryCognitiveChannel` with given buffer capacity.
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<(TenantId, Vec<PreCondensedInput>)>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self { tx }, rx)
    }
}

impl CognitiveInputChannel for InMemoryCognitiveChannel {
    fn send_pre_condensed<'a>(
        &'a self,
        target_tenant_id: &'a TenantId,
        inputs: &'a [PreCondensedInput],
    ) -> BoxFuture<'a, Result<Vec<TurnSegment>>> {
        let target_tenant = target_tenant_id.clone();
        let input_vec = inputs.to_vec();
        Box::pin(async move {
            let segments = crate::memory_consolidation::intake_pre_condensed_inputs(
                &target_tenant,
                &input_vec,
                None,
                None,
            )?;
            self.tx
                .send((target_tenant, input_vec))
                .await
                .map_err(|e| {
                    contextra_types::ContextraError::Internal(format!("channel closed: {e}"))
                })?;
            Ok(segments)
        })
    }
}

/// Standalone synchronous/direct intake adapter implementing `CognitiveInputChannel`.
pub struct DirectCognitiveChannel;

impl CognitiveInputChannel for DirectCognitiveChannel {
    fn send_pre_condensed<'a>(
        &'a self,
        target_tenant_id: &'a TenantId,
        inputs: &'a [PreCondensedInput],
    ) -> BoxFuture<'a, Result<Vec<TurnSegment>>> {
        Box::pin(async move {
            crate::memory_consolidation::intake_pre_condensed_inputs(
                target_tenant_id,
                inputs,
                None,
                None,
            )
        })
    }
}

/// In-memory queue sink collecting emitted items in an `Arc<parking_lot::Mutex<Vec<T>>>`.
pub struct InMemoryOutputSink<T> {
    items: Arc<parking_lot::Mutex<Vec<T>>>,
}

impl<T: Send + Sync + Clone + 'static> InMemoryOutputSink<T> {
    /// Creates a new empty `InMemoryOutputSink`.
    pub fn new() -> Self {
        Self {
            items: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    /// Returns a copy of all collected items.
    pub fn collected(&self) -> Vec<T> {
        self.items.lock().clone()
    }

    /// Returns the number of collected items.
    pub fn len(&self) -> usize {
        self.items.lock().len()
    }

    /// Checks if the sink is empty.
    pub fn is_empty(&self) -> bool {
        self.items.lock().is_empty()
    }
}

impl<T: Send + Sync + Clone + 'static> Default for InMemoryOutputSink<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + Sync + Clone + 'static> CognitiveOutputSink<T> for InMemoryOutputSink<T> {
    fn emit<'a>(&'a self, item: T) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.items.lock().push(item);
            Ok(())
        })
    }
}
