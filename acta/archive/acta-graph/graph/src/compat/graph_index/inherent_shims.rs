// TODO(acta-altlast): Bruecken-Modul der Altquelle (HR-6 verbietet Bruecken). Referenz fuer die Trait-Implementierungen (GraphIndex, Purger); beim Umbau in direkte Implementierungen aufloesen.
//! Inherent shims delegating to GraphIndex methods.

use crate::csr::CsrGraph;

impl CsrGraph {
    /// Delegiert an [`acta_base::GraphIndex::personalized_page_rank_at`].
    pub fn personalized_page_rank_at<'a>(
        &'a self,
        seed_nodes: &'a [acta_base::EntityId],
        config: &'a acta_base::PprConfig,
        seq_no: u64,
    ) -> acta_base::BoxFuture<
        'a,
        acta_base::Result<Vec<(acta_base::EntityId, f32)>>,
    > {
        acta_base::GraphIndex::personalized_page_rank_at(self, seed_nodes, config, seq_no)
    }

    /// Delegiert an [`acta_base::GraphIndex::rollback_to_tx`].
    pub fn rollback_to_tx<'a>(
        &'a self,
        tx_id: acta_base::TxId,
    ) -> acta_base::BoxFuture<'a, acta_base::Result<()>> {
        acta_base::GraphIndex::rollback_to_tx(self, tx_id)
    }
}
