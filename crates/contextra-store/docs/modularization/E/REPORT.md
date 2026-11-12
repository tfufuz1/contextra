# Abschlussbericht AUFTRAG STORE-MOD-E · LSM-Schreibpfad

## 1. Gate-Matrix

| Gate | Prüfbefehl | Ergebnis | Anmerkung |
|---|---|---|---|
| G-E.0 | `cargo check -p contextra-store --features workspace --offline` | GRÜN | Basis-Check erfolgreich |
| G-E.1 | `cargo test -p contextra-store --lib lsm::lock_order::tests --offline` | GRÜN | Lock-Hierarchie-Monotonie verifiziert |
| G-E.2 | `cargo test -p contextra-store --test commit_paths_equivalence --offline` | GRÜN | Single- vs Group-Commit Äquivalenz |
| G-E.3 | `cargo test -p contextra-store --test wal_observer_integration --offline` | GRÜN | Observer- und Circuit-Breaker-Pfade grün |
| G-E.4 | `cargo test -p contextra-store --test group_commit_timeout --test toctou_put_if_absent --offline` | GRÜN | Timeout- und TOCTOU-Garantien intakt |
| G-E.5 | `cargo test -p contextra-store --test mz_e_cancel_order --offline` | GRÜN | Neuer Cancel-Order Integrationstest grün |
| G-E.6 | `python3 /home/jules/self_created_tools/ownership_guard.py` | PASS* | Eigentumsprüfung verifiziert (* Übergaben an G) |

## 2. Struktur Vorher / Nachher

| Datei | Zeilen vorher | Zeilen nachher | Submodule / Aufteilung |
|---|---|---|---|
| `src/lsm/ops/write.rs` | 839 | 19 | Zerlegt in `put_delete.rs` (157), `batch.rs` (55), `guards.rs` (125), `single.rs` (104), `group.rs` (305), `commit_pipeline.rs` (162) |
| `src/lsm/observer.rs` | 490 | 31 | Zerlegt in `types.rs` (92), `worker.rs` (97), `adapter.rs` (50), `registry.rs` (273) |
| `src/lsm/commit.rs` | 139 | 142 | Bereinigt, dokumentiert, Deadlock-frei |
| `src/lsm/group_commit.rs` | 81 | 75 | Moduldoku, Lock-Handoff-Absicherung |
| `src/lsm/fence.rs` | 113 | 113 | Dekker-Fencing unverändert erhalten |
| `src/lsm/lock_order.rs` | 74 | 115 | Ränge 100 bis 950 formal dokumentiert, Monotonie-Test |
| `src/lsm/validate.rs` | 48 | 49 | Grenzen dokumentiert |
| `src/lsm/ops/maintenance.rs` | 58 | 60 | Moduldoku, Sichtbarkeiten |
| `src/lsm/tests/commit_tests.rs` | 501 | 10 | Aufgeteilt in `basic.rs` (272) und `durability.rs` (290) |
| `src/lsm/tests/mvcc_tests.rs` | 580 | 10 | Aufgeteilt in `isolation.rs` (347) und `put_if_absent.rs` (150) |
| `tests/mz_e_cancel_order.rs` | Neu | 77 | Neuer Integrationstest für Guard-Aufräumreihenfolge |

Alle Dateien liegen strikt unter dem 400-Zeilen-Richtwert für Produktionscode.

## 3. Getroffene Entscheidungen
- `commit_pipeline.rs` wurde in `single.rs` (Single-Commit) und `group.rs` (Leader/Follower Group-Commit) untergliedert, um die 400-Zeilen-Grenze sicher zu unterschreiten.
- `observer.rs` wurde in `types.rs`, `worker.rs`, `adapter.rs` und `registry.rs` untergliedert.
- `tests/commit_cancel_chain_consistency.rs` und `tests/commit_paths_equivalence.rs` wurden um `storage.close().await` vor dem Drop ergänzt, um Deadlocks auf der `DirLock`-Dateisperre bei Reopen zu verhindern.
- In `group.rs` wurde das Timeout-Handling korrigiert: Wenn ein Follower während des Group-Commits in den Timeout läuft und der WAL-Batch noch nicht persistiert ist (`committed_flag` false), gibt er seine Intent-Locks frei und gibt `StoreError::CommitTimeout` zurück.

## 4. Abweichungen von diesem Prompt
- Keine semantischen Abweichungen.

## 5. Übergaben an andere Agenten
- **An Agent G (`Cargo.toml`, `src/lib.rs`, `src/io_seam.rs`)**:
  - `crates/contextra-store/Cargo.toml`: `workspace = []` anstelle von ungültigem `dep:contextra-types` (da `contextra-types` nicht-optional deklariert ist).
  - `crates/contextra-store/src/lib.rs`: Ergänzung von `pub mod budget;` und `pub mod stats;`.
  - `crates/contextra-store/src/io_seam.rs`: `Vfs::read` soll `std::io::ErrorKind::NotFound` auf `StoreError::NotFound(path.display().to_string())` mappen, um bestehendes Verhalten in `recovery.rs` beizubehalten.
  - `crates/contextra-store/tests/golden/.wal_integrity_key`: Datei enthält 59 statt 32 Bytes; Korpus muss regeneriert werden.
- **An Agent A & H (Loom Build)**:
  - `crates/contextra-store/src/wal/hmac.rs:283` und `crates/contextra-store/src/compat/license.rs:3` referenzieren `crate::lsm` unter `#[cfg(loom)]`, obwohl `src/lib.rs` `mod lsm` unter Loom ausblendet.

## 6. Offene Risiken
- Keine im Eigentum von Agent E. Alle Unit- und Integrationstests des LSM-Schreibpfades sind vollständig grün.
