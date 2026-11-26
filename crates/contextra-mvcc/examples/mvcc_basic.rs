//! Eigenständiges Beispiel zur isolierten Verwendung von `contextra-mvcc`.
//!
//! Zeigt den vollständigen Transaktionslebenszyklus:
//! 1. Snapshot-Registrierung (SnapshotRegistry)
//! 2. Transaktionspufferung (TxBuffer)
//! 3. ReadSet-Erfassung und SSI-Validierung (SequenceLogSsiValidator)
//! 4. Commit und Sequenz-Fortschritt

use std::sync::Arc;

use contextra_mvcc::{
    IndexOp, ReadSet, SequenceLogSsiValidator, SnapshotRegistry, SsiValidator, TxBuffer,
};
use contextra_types::types::{DocId, TxId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Contextra MVCC Standalone-Beispiel ===");

    // 1. Initialisiere die Registry und den SSI-Validator
    let registry = Arc::new(SnapshotRegistry::new());
    let ssi_validator = SequenceLogSsiValidator::new();
    let tx_buffer = TxBuffer::<String>::new();

    // 2. Transaktion 1 startet bei Snapshot-Sequenz 10
    let tx1 = TxId::new(101);
    let guard1 = registry.register(10);
    println!(
        "Tx1 registriert mit Snapshot seq={}, min_active_seq={:?}",
        guard1.seq_no(),
        registry.min_read_snapshot()
    );

    // 3. Tx1 liest 'account:A' und 'account:B'
    let mut read_set_1 = ReadSet::new();
    read_set_1.record_read(b"account:A", 10);
    read_set_1.record_read(b"account:B", 10);

    // Tx1 modifiziert 'account:A' im Staging-Puffer
    tx_buffer.begin(tx1);
    tx_buffer.stage(
        tx1,
        IndexOp::Insert {
            doc_id: DocId::from(1u64),
            data: "balance:90".to_string(),
        },
    )?;

    // 4. Parallele Transaktion 2 committet Änderung an 'account:B' bei Sequenz 15
    let _tx2 = TxId::new(102);
    ssi_validator.record_commit_key(b"account:B", 15);
    println!("Tx2 hat Schlüssel 'account:B' bei Sequenz 15 committet.");

    // 5. Tx1 versucht zu committen -> SSI-Validierung muss Write-Skew / Konflikt erkennen!
    match ssi_validator.validate(tx1, &read_set_1) {
        Ok(()) => {
            println!("Tx1 Validierung erfolgreich (unerwartet).");
        }
        Err(e) => {
            println!("Tx1 Konflikt erfolgreich erkannt (Serializable Snapshot Isolation): {e}");
            tx_buffer.discard(tx1);
            println!("Tx1 verworfen.");
        }
    }

    // 6. Beende Snapshot von Tx1
    drop(guard1);
    println!(
        "Guard freigegeben, min_read_snapshot={:?}",
        registry.min_read_snapshot()
    );

    println!("=== Beispiel erfolgreich beendet ===");
    Ok(())
}
