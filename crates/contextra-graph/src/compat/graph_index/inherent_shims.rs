//! Inherent shims delegating to GraphIndex methods.

use crate::csr::CsrGraph;

impl CsrGraph {
    /// Delegiert an [`contextra_ports::GraphIndex::personalized_page_rank_at`].
    pub fn personalized_page_rank_at<'a>(
        &'a self,
        seed_nodes: &'a [contextra_types::EntityId],
        config: &'a contextra_types::PprConfig,
        seq_no: u64,
    ) -> contextra_ports::BoxFuture<
        'a,
        contextra_types::Result<Vec<(contextra_types::EntityId, f32)>>,
    > {
        contextra_ports::GraphIndex::personalized_page_rank_at(self, seed_nodes, config, seq_no)
    }

    /// Delegiert an [`contextra_ports::GraphIndex::rollback_to_tx`].
    pub fn rollback_to_tx<'a>(
        &'a self,
        tx_id: contextra_types::TxId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        contextra_ports::GraphIndex::rollback_to_tx(self, tx_id)
    }
}
