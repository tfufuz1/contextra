//! Bounded, opaque storage inspector for testing and diagnostics (SP-FACADE §3).

use crate::catalog::state::CatalogState;
use crate::erase::gate::GateIndex;
use crate::erase::protocol::EraseCoordinator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSnapshotSummary {
    pub spaces: usize,
    pub clusters: usize,
    pub live_files: usize,
    pub graveyard_files: usize,
    pub gated_clusters: usize,
    pub open_erase_tickets: usize,
    pub catalog_epoch: u64,
}

pub fn inspect_state(
    catalog: &CatalogState,
    gate_index: &GateIndex,
    coordinator: &EraseCoordinator,
) -> StorageSnapshotSummary {
    StorageSnapshotSummary {
        spaces: catalog.spaces().count(),
        clusters: catalog.clusters().count(),
        live_files: catalog.files().count(),
        graveyard_files: catalog.graveyard().count(),
        gated_clusters: gate_index.len(),
        open_erase_tickets: coordinator.list_open_tickets().len(),
        catalog_epoch: catalog.epoch(),
    }
}
