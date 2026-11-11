//! Brücken für AttentionExporter und RequestId.

use crate::attention_exporter::CandleAttentionExporter;
use contextra_ports::{AttentionExporter, RequestId};

impl AttentionExporter for CandleAttentionExporter {
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        self.export_weights(request_id.0)
    }
}

#[cfg(feature = "workspace")]
impl CandleAttentionExporter {
    /// Fassade nach H3: Akzeptiert `RequestId` für Abwärtskompatibilität.
    pub fn record_attention_weights(&self, request_id: impl Into<u64_request_id::IntoU64>, weights: Vec<f32>) {
        self.record_attention_weights_local(request_id.into().0, weights);
    }

    /// Fassade nach H3: Akzeptiert `RequestId` für Abwärtskompatibilität.
    pub fn record_layer_head_scores(&self, request_id: impl Into<u64_request_id::IntoU64>, scores: &[Vec<f32>]) {
        self.record_layer_head_scores_local(request_id.into().0, scores);
    }
}

#[doc(hidden)]
pub mod u64_request_id {
    use super::*;

    pub struct IntoU64(pub u64);

    impl From<u64> for IntoU64 {
        fn from(val: u64) -> Self {
            IntoU64(val)
        }
    }

    impl From<RequestId> for IntoU64 {
        fn from(req: RequestId) -> Self {
            IntoU64(req.0)
        }
    }
}
