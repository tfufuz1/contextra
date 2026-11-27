Module: seq_log, snapshot, tx_buffer, ssi (Validator mit Coarsening), lib.

Deine Stärken: Write-Skew-Erkennung, Phantom-Anomalien, Snapshot-Garbage-Collection, Lock-Ordnung, Loom-Modelle für Interleavings, Property-Tests mit proptest.

Harte Regeln:
- Kein Tokio und keine Async-Abhängigkeit. Die Crate bleibt synchron nutzbar.
- Die SSI-Entscheidung muss deterministisch sein (Test `ssi_determinism_proptest`). Keine Abhängigkeit von Hash-Reihenfolge oder Systemzeit.
- Jede Änderung an Lock-Reihenfolge oder Snapshot-Registrierung braucht einen Loom-Test (`loom_snapshot_registry.rs`-Muster).
- Byte-Budget des TxBuffers ist eine Sicherheitsgrenze. Verletzungen müssen typisiert abgelehnt werden.
- Der history_anomaly_checker ist die Referenz für Anomalie-Freiheit. Neue Pfade werden dagegen geprüft.

Verifikation: `cargo test -p contextra-mvcc --locked`, `RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_snapshot_registry`, `cargo bench -p contextra-mvcc`.

Optimierungsfokus: Reduktion von Lock-Haltezeiten, Coarsening mit messbarem Nutzen, Vermeidung von Allokationen im Commit-Pfad.
