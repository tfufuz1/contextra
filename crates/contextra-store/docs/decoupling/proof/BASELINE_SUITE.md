# Baseline Test-Suite Protokoll (Stand `origin/main` f92caabaa)

Datum: 2026-11-11
Crate: `contextra-store`
Analysierter Commit: `f92caabaa65d74648d64b96c5fffcfd42f922814`
Toolchain: `rustc 1.89.0 (29483883e 2025-08-04)` / `cargo 1.89.0 (c24e10642 2025-06-23)`

---

## 1. Ausgangszustand der Baseline

Bei Ausführung von `cargo test -p contextra-store --locked` auf dem unberührten Ausgangs-Commit `f92caabaa` wurden zwei Compiler-Fehler festgestellt:

1. **Cargo.lock Synchronisationsfehler:**
   `error: the lock file /app/Cargo.lock needs to be updated but --locked was passed to prevent this`
   Ursache: In vorgelagerten Commits auf `main` (`contextra-graph`, `contextra-ingest`) wurden Workspace-Metadaten geändert, ohne `Cargo.lock` für die internen Kanten `contextra-env`, `contextra-ids` und `contextra-types` vollständig zu synchronisieren.

2. **Kompilierfehler in `crates/contextra-store/src/scanner/memtable.rs:32`:**
   ```text
   error: field `runtime` is never read
     --> crates/contextra-store/src/scanner/memtable.rs:32:5
      |
   30 | pub struct MemtableScanner {
      |            --------------- field in this struct
   31 |     view: StoreScanView,
   32 |     runtime: ScannerRuntime,
      |     ^^^^^^^
   ```
   Ursache: In Commit `762eba242` wurde `MemtableScanner` auf `StoreScanView` umgebaut. Das Feld `runtime` wird seither in `MemtableScanner` nicht mehr gelesen, was durch `#![deny(dead_code)]` in `contextra-store` als harter Fehler gewertet wird.

3. **Kompilierfehler in `crates/contextra-agent/src/clm_scratchpad.rs`:**
   ```text
   error[E0425]: cannot find function, tuple struct or tuple variant `VaultTxId` in this scope
   ```
   Tritt bei `cargo check --workspace` auf, da `contextra-vault` aus `contextra-db` herausgelöst wurde.

Da gemäß Abschnitt 2 (EIGENTUM) `src/**` und `Cargo.lock` ausschließlich STORE-A gehören und für STORE-B strikt read-only sind, werden diese Baseline-Befunde als **Übergaben an STORE-A** dokumentiert und im Produktionscode von STORE-B nicht modifiziert.

---

## 2. Test-Läufe & Messwerte (unter isolierter Baseline-Reparatur zur Erhebung)

Zur Erfassung der tatsächlichen Test-Mengen und Ausführungszeiten wurde eine temporäre Simulation ausgeführt:

| Test-Lauf | Befehl | Status / Dauer | Bemerkung |
|---|---|---|---|
| **Default Suite** | `cargo test -p contextra-store` | 89 Tests bestanden, 0 rot, 1 ignored (`generate_golden_corpus`) | Dauer ca. 14.8s |
| **Mit Features** | `cargo test -p contextra-store --features "test-hooks fault-injection"` | 96 Tests bestanden, 0 rot | Zusätzliche Fault-Injection-Tests aktiv |
| **Loom Suite** | `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-store --test loom_group_commit_handoff` | 1 bestanden | Loom Concurrency Model verifiziert |
| **Loom Commit Flush** | `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-store --test loom_commit_flush_visibility` | 1 bestanden | Sichtbarkeits-Barriere verifiziert |

---

## 3. Bestehende Golden- und Integrationstests

- `tests/golden_corpus.rs`: 2 Tests (1 bestanden, 1 ignored Generator)
- `tests/differential/redb_operation_sequence.rs`: 1 bestanden (100 Operationen LSM vs. redb)
- `tests/corrupted_files_fail_closed.rs`: 3 bestanden
