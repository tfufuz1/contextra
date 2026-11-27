Module: store (Checkpoint-Registry), guard (RAII-Rollback), manifest (Blake3), meta, orphan, hardlink_cloner.

Harte Regeln:
- Rollbacks sind deterministisch. Bei Panic und Drop greift der `CheckpointGuard`. Ein Guard darf nie stillschweigend nichts zurücksetzen.
- Die Crate verwendet keine `unwrap`- oder `expect`-Aufrufe im Produktionscode.
- Hardlink-Klone prüfen Inodes. Verhalten über Dateisystemgrenzen hinweg ist dokumentiert.
- Txid-Monotonie und Tenant-Isolation sind getestet (`txid_monotonicity_and_recovery.rs`, `crash_recovery_and_tenant_isolation.rs`). Diese Tests dürfen nicht abgeschwächt werden.
- Manifest-Änderungen brauchen Fault-Injection (`manifest_fault_injection.rs`).

Verifikation: `cargo test -p contextra-checkpoint --locked`, `cargo bench -p contextra-checkpoint`, sowie der Systematic-Crash-Test.

Optimierungsfokus: Kopierkosten minimieren durch Hardlinks, Blake3-Hashing parallelisieren wo sicher, Cache-Pinning nur solange nötig.
