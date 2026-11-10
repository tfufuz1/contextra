# Entkopplungsbericht: contextra-mvcc (Baustein-Kern ohne types/ports/testkit)

## 1. Baseline (V0)
Ausgangsstand auf `main` (vor Änderungen an `contextra-mvcc`):
- `cargo check -p contextra-mvcc --no-default-features`: GRÜN (0 Fehler)
- `cargo check -p contextra-mvcc --all-targets`: GRÜN (0 Fehler)
- `cargo test -p contextra-mvcc --lib`: GRÜN (52 Tests bestanden, 0 fehlgeschlagen)
- `cargo test -p contextra-mvcc`: GRÜN (52 lib-Tests, 20 Integrationstest-Suiten bestanden, 1 ignored: `generate_golden_mvcc`)
- `cargo clippy -p contextra-mvcc --all-targets -- -D warnings`: GRÜN (0 Warnungen)
- `cargo tree -p contextra-mvcc -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u`:
  - `contextra-env v0.1.0`
  - `contextra-ids v0.1.0`
  - `contextra-mvcc v0.1.0`
  - `contextra-ports v0.1.0`
  - `contextra-types v0.1.0`
  *(Hinweis: contextra-ports und contextra-types waren bisher harte Abhängigkeiten)*

## 2. Arbeitspakete & Commits
- WP-1..3: Entkopplung des Kerns von `contextra-types`, `contextra-ports` und `contextra-testkit`.
  - Feature `workspace = ["dep:contextra-types", "dep:contextra-ports"]` mit `default = ["workspace"]`.
  - Interne Kern-Abhängigkeiten beschränkt auf `contextra-ids` und `contextra-env`.
  - Re-Exports von Typen und Kompatibilitätsbrücke unter `src/compat/` hinter `#[cfg(feature = "workspace")]`.
  - Pfad-Ersetzungen: `Clock`, `SystemClock`, `MetricsSink` auf `contextra-env`, `TOMBSTONE_BIT`, `DocId`, `TenantId`, `TxId` auf `contextra-ids`.
  - Unit-Tests in `tx_buffer.rs`, `seq_log.rs`, `snapshot.rs` auf `contextra_env::ManualClock` umgestellt.
  - Clippy-Härtung ohne Typ-Truncation (`advance_nanos`).

## 3. Verifikation (V1 bis V7) & Loom
- **V1**: `cargo check -p contextra-mvcc --no-default-features`: **GRÜN**
- **V2**: `cargo check -p contextra-mvcc --all-targets`: **GRÜN**
  - `cargo check -p contextra-mvcc --no-default-features --features workspace --all-targets`: **GRÜN**
- **V3**: `cargo test -p contextra-mvcc --lib`: **GRÜN** (52/52 Tests bestanden)
  - `cargo test -p contextra-mvcc`: **GRÜN** (alle Unit- und Integrationstests bestanden)
- **V4**: `cargo clippy -p contextra-mvcc --all-targets -- -D warnings`: **GRÜN** (0 Warnungen)
- **V5 (Null-Prüfung)**:
  `grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-mvcc/src --include=*.rs | grep -v '/compat/'`
  Ergebnis: **LEER** (0 Treffer außerhalb von `src/compat/`)
- **V6 (Kern-Closure)**:
  `cargo tree -p contextra-mvcc -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u`
  Ergebnis:
  ```
  contextra-env v0.1.0 (/app/crates/contextra-env)
  contextra-ids v0.1.0 (/app/crates/contextra-ids)
  contextra-mvcc v0.1.0 (/app/crates/contextra-mvcc)
  ```
  Exakte Teilmenge der erlaubten Kern-Abhängigkeiten!
- **V7 (Workspace-Probe)**:
  Fremdnutzer (`contextra-store`, `contextra-vector`, `contextra-engine`, `contextra-checkpoint`) kompilieren unverändert mit Default-Features. Ein vorbestehender Syntaxfehler in `contextra-store/src/sstable/builder.rs` auf `main` liegt außerhalb von BESITZ.
- **Loom Concurrency Tests**:
  - `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-mvcc --test loom_snapshot_registry`: **GRÜN**
  - `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-mvcc --test loom_sequence_log`: **GRÜN**
  - `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-mvcc --test loom_tx_buffer`: **GRÜN**

## 4. Fassaden- und T2-Liste
Keine T2-Fassaden notwendig, da alle Typen via Typidentität (`contextra-ids` und `contextra-env`) bit- und typgleich sind und existierende öffentliche Pfade über `pub use contextra_ids::*` und `pub mod types` erhalten blieben.

## 5. Fremdnutzer-Inventar & Späterer Default-Flip
- `crates/contextra-store`
- `crates/contextra-vector`
- `crates/contextra-engine`
- `crates/contextra-checkpoint`
Wenn später `default = []` gesetzt wird, können diese Crates entweder `features = ["workspace"]` setzen oder direkt die entkoppelten Ports verwenden.
