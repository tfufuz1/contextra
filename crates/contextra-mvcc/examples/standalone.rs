//! Minimales, autarkes Standalone-Beispiel für `contextra-mvcc` (≤ 30 Zeilen).
use contextra_mvcc::{IndexOp, ReadSet, SequenceLogSsiValidator, SnapshotRegistry, SsiValidator, TxBuffer};
use contextra_mvcc::types::{DocId, TxId};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (registry, ssi, buffer) = (Arc::new(SnapshotRegistry::new()), SequenceLogSsiValidator::new(), TxBuffer::<String>::new());
    let (tx1, guard1) = (TxId::new(1), registry.register(10));
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"acc:A", 10);
    buffer.begin(tx1);
    buffer.stage(tx1, IndexOp::Insert { doc_id: DocId::from(1u64), data: "v1".into() })?;

    // Paralleler Schreiber commitet acc:A bei Sequenz 15
    ssi.record_commit_key(b"acc:A", 15);

    // Tx1 muss durch SSI-Anti-Dependency-Prüfung konfliktieren
    assert!(ssi.validate(tx1, &rs1).is_err(), "Write-Skew muss erkannt werden");
    buffer.discard(tx1);
    drop(guard1);
    println!("Contextra-MVCC standalone example executed successfully!");
    Ok(())
}
