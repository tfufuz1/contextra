//! Brücken-Konvertierungen für Domänentypen (ImportanceScore, QueryRewriteOutput, Clock, Rng).

use crate::error::{LocalClock, LocalImportanceScore, LocalQueryRewriteOutput, LocalRng};

impl From<LocalQueryRewriteOutput> for contextra_types::QueryRewriteOutput {
    fn from(val: LocalQueryRewriteOutput) -> Self {
        contextra_types::QueryRewriteOutput::new(
            val.text_query,
            val.semantic_query,
            val.anchor_entities,
        )
    }
}

impl From<contextra_types::QueryRewriteOutput> for LocalQueryRewriteOutput {
    fn from(val: contextra_types::QueryRewriteOutput) -> Self {
        LocalQueryRewriteOutput::new(val.text_query, val.semantic_query, val.anchor_entities)
    }
}

impl From<LocalImportanceScore> for contextra_types::ImportanceScore {
    fn from(score: LocalImportanceScore) -> Self {
        contextra_types::ImportanceScore::new(score.0)
    }
}

impl From<contextra_types::ImportanceScore> for LocalImportanceScore {
    fn from(score: contextra_types::ImportanceScore) -> Self {
        LocalImportanceScore(score.value())
    }
}

impl LocalClock for contextra_ports::SystemClock {
    fn now(&self) -> std::time::SystemTime {
        let nanos = contextra_ports::Clock::now_unix_nanos(self);
        std::time::UNIX_EPOCH + std::time::Duration::from_nanos(nanos)
    }

    fn monotonic_nanos(&self) -> u64 {
        contextra_ports::Clock::monotonic_nanos(self)
    }
}

impl LocalRng for contextra_ports::SeededRng {
    fn next_u64(&self) -> u64 {
        contextra_ports::Rng::next_u64(self)
    }
}
