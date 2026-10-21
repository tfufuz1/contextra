// UI test prepared for W3-01 flip when `issue()` visibility becomes pub(crate).
// Checks that calling LayerWitness::issue from an external crate fails to compile.

use contextra_crypto::deletion_witness::{LayerWitness, WitIssueArgs};
use contextra_crypto::deletion_proof::{
    claim_text::{WitAssurance, WitClaimText},
    evidence::WitLayerEvidence,
    probe::WitStoreGeneration,
    DeletionLayer,
};
use contextra_types::TxId;

fn main() {
    let args = WitIssueArgs {
        layer: DeletionLayer::LsmMemtable,
        scope_hash: [0u8; 32],
        scanned_at_tx: TxId(10),
        as_of_tx: TxId(10),
        assurance: WitAssurance::S1,
        claim: WitClaimText::S1,
        footprint_hash: [0u8; 32],
        scope_write_seq: 1,
        scanner_name: "test",
        scanner_version: 1,
        store_generation: WitStoreGeneration(1),
        layer_evidence: WitLayerEvidence::Memtable {
            memtables_scanned: 1,
            entries_found: 0,
            tombstones_found: 0,
        },
        key_identifiers_claimed: false,
    };

    #[allow(deprecated)]
    let _ = LayerWitness::issue(args);
}
