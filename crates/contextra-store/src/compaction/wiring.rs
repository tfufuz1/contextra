// FILE-CONTEXT
// ZWECK: Entrypoint für die spätere Verdrahtung in contextra/src/builder.rs (Regel 7 / Einstiegspunkt-Vertrag).

use crate::compaction::{CompactionConfig, CompactionEngine};
use contextra_core::{ResourceTracker, Result};
use contextra_mvcc::snapshot::SnapshotFloor;
use std::sync::Arc;

/// Composition Root Wiring-Struktur für contextra-store Compaction.
pub struct CompactionEngineWiring {
    pub engine: Arc<CompactionEngine>,
}

impl CompactionEngineWiring {
    /// Erzeugt eine neue `CompactionEngineWiring`-Instanz mit vorgegebenen Abhängigkeiten.
    pub fn build(
        config: CompactionConfig,
        floor: Arc<dyn SnapshotFloor>,
        block_cache: Arc<crate::sstable::BlockCache>,
        key_manager: Option<Arc<crate::wal::KeyManager>>,
        budget: Arc<ResourceTracker>,
        manifest: Option<Arc<crate::manifest::Manifest>>,
    ) -> Result<Self> {
        let engine = Arc::new(CompactionEngine::new(
            config,
            floor,
            block_cache,
            key_manager,
            budget,
            manifest,
        ));
        Ok(Self { engine })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compaction::CompactionConfig;
    use crate::sstable::create_block_cache;

    #[test]
    fn test_compaction_engine_wiring_build() {
        let registry = Arc::new(contextra_core::SnapshotRegistry::new());
        let bc = create_block_cache(1);
        let budget = Arc::new(ResourceTracker::new(contextra_core::ResourceBudget {
            memory_limit: 1024 * 1024,
        }));
        let config = CompactionConfig::default();

        let wiring = CompactionEngineWiring::build(
            config,
            registry as Arc<dyn SnapshotFloor>,
            bc,
            None,
            budget,
            None,
        )
        .expect("build wiring");

        assert_eq!(wiring.engine.min_snapshot_seq_bound(), u64::MAX);
    }
}
