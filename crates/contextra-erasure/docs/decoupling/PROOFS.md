# PROOFS.md — Beweisbericht contextra-erasure (Auftrag ERASURE-B)

Stand: 2026-11-11
Rolle: Beweis-Auftrag für `contextra-erasure`: Crash-/Saga-Beweise, Gate-Fail-closed, Ledger-Konsistenz, Loom/Property, Fuzzing.

## 1. Bestandsaufnahme & Matrizen

### 1.1 Saga-Phase × Abbruchtest Matrix

| Saga-Phase | Möglicher Abbruchpunkt | Abbruch-Simulation | Erwartetes Verhalten bei Wiederaufnahme | Test-Abdeckung |
|---|---|---|---|---|
| `Requested` | Vor Gate-Aktivierung | Crash vor Journal-Flush | Replay erkennt unvollständigen Request; keine Residuen, Gate inaktiv | `proof_crash_resume_matrix::test_crash_at_requested_phase` |
| `Gated` | Nach Gate-Aktivierung, vor Key-Destroy | Crash nach `ErasedSet`/`ReadGate` Flush | Gate bleibt aktiv (INV-BOOT-1); Replay setzt bei Schlüsselvernichtung fort | `proof_crash_resume_matrix::test_crash_at_gated_phase` |
| `KeyDestroyed` | Nach Key-Nullung im Vault, vor Schicht-Purge | Crash nach `destroy_key` | Schlüssel bleibt permanent genullt (INV-NO-RESURRECTION-1); Replay führt Schicht-Purge aus | `proof_crash_resume_matrix::test_crash_at_key_destroyed_phase` |
| `Purging` (DocStore) | Mitten im physischen Segment-Rewrite | IO-Unterbrechung / Stromausfall | RewriteJournal verwirft unvollständiges Segment; Replay wiederholt Purge | `proof_crash_resume_matrix::test_crash_at_purging_docstore` |
| `Purging` (Vector) | Mitten im Index-Knoten-Nullen | Abbruch während Vector-Purge | Vektor-Tombstones bleiben wirksam; Replay schließt Nullung ab | `proof_crash_resume_matrix::test_crash_at_purging_vector` |
| `Measured` | Nach Zeugenerfassung, vor Bericht | Abbruch vor Bericht-Signatur | Zeugen bleiben valide im Journal; Replay signiert und verankert im Ledger | `proof_crash_resume_matrix::test_crash_at_measured_phase` |
| `Completed` | Nach Ledger-Verankerung | Wiederholter Aufruf / Replay | Idempotent: Keine Doppelwirkung, identischer Ledger-Anchor | `proof_crash_resume_matrix::test_replay_idempotency_after_completed` |

### 1.2 Gate-Zustand × Test Matrix

| Gate-Zustand | Eingabe-Szenario | Erwartete Reaktion | Nachweis-Test |
|---|---|---|---|
| `Active` | Anfrage auf gefencten Bereich | `FenceGateError::Fenced` (Fail-Closed) | `p4_fence_tests.rs`, `proof_fail_closed_and_consistency.rs` |
| `Active` | Anfrage auf ungefencten Bereich | Erlaubt (`Ok(WriterGuard)`) | `p4_fence_tests.rs` |
| `Corrupted` | Beliebige Lese- oder Schreibanfrage | Sofortige bedingungslose Verweigerung | `proof_fail_closed_and_consistency.rs` |
| `Active` | Überlappender Fence-Erwerb (Präfix/Range/Global) | `FenceGateError::OverlapConflict` | `proof_fail_closed_and_consistency.rs` |
| `Active` | Freigabe ohne gültiges Owner-Token | Verweigerung / Fehler | `proof_fail_closed_and_consistency.rs` |

### 1.3 Journal-Operation × Idempotenztest Matrix

| Operation | Wiederholungsszenario | Erwartetes Verhalten | Nachweis-Test |
|---|---|---|---|
| `GateAcquire` | Mehrfaches Replay des Gate-Eintrags | ErasedSet bleibt unverändert idempotent | `proof_crash_resume_matrix.rs` |
| `KeyPurge` | Mehrfaches Replay der Nullung | DEK bleibt genullt, kein Fehler | `p1_key_hierarchy_and_purge.rs`, `proof_crash_resume_matrix.rs` |
| `SegmentSwap` | Mehrfaches Replay von `commit_swap` | Bereits gelöschte Alt-Dateien erzeugen keinen Fehler | `p2_vault_rewrite_tests.rs` |
| `LedgerAppend` | Mehrfacher Append desselben Commitments | Idempotent: Bereits verankerte Einträge werden dedupliziert | `proof_fail_closed_and_consistency.rs` |

## 2. Übergaben an ERASURE-A (src/ Bearbeiter)

Aufgrund der strikten Eigentumstrennung (ERASURE-B darf `src/` nicht editieren) sind am HEAD 882aada72 folgende 3 Zeilen in `src/` durch den Parallel-Auftrag ERASURE-A anzupassen:
1. `crates/contextra-erasure/src/ledger.rs`:
   - Zeile 6-7: Hinzufügen von `pub mod error;` und `pub mod record;`.
   - Zeile 13: Entfernen von `pub use persistent::ErasureLedger;` (da `ErasureLedger` bereits aus `durable` exportiert wird).
2. `crates/contextra-erasure/src/purge.rs`:
   - Zeile 13: Entfernen von `pub use phase::PurgePhase;` (da `PurgePhase` in Zeile 16 bereits aus `phase::{...}` importiert wird).
Sobald diese 3 Zeilen gesetzt sind, baut `contextra-erasure` vollkommen fehlerfrei.
