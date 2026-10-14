// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Candle LLM & KV Prefix Store Wiring for contextra facade (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; Connects CandleLlmClient with prefix store when active.

#![forbid(unsafe_code)]

#[cfg(feature = "candle")]
use std::sync::Arc;

#[cfg(feature = "candle")]
use contextra_core::error::ContextraError;
#[cfg(all(feature = "candle", feature = "kv-stage-b"))]
use contextra_infer_candle::inference::CandleLlmClient;
#[cfg(feature = "candle")]
use contextra_ports::kv::KvPrefixStore;

/// Bundles Candle LLM inference with optional prefix store.
#[cfg(feature = "candle")]
pub struct CandleWiringBundle {
    #[cfg(all(feature = "candle", feature = "kv-stage-b"))]
    pub client: Option<CandleLlmClient>,
}

#[cfg(feature = "candle")]
impl CandleWiringBundle {
    /// Builds `CandleWiringBundle` attaching `prefix_store` if present.
    pub fn build(#[allow(unused_variables)] prefix_store: Option<Arc<dyn KvPrefixStore>>) -> Result<Self, ContextraError> {
        #[cfg(all(feature = "candle", feature = "kv-stage-b"))]
        let client = prefix_store.and_then(|store| {
            CandleLlmClient::from_dir(
                std::path::Path::new("."),
                contextra_infer_candle::model_registry::CandleQuantization::Q4KM,
            )
            .map(|c| c.with_prefix_store(store))
            .ok()
        });

        Ok(Self {
            #[cfg(all(feature = "candle", feature = "kv-stage-b"))]
            client,
        })
    }
}
