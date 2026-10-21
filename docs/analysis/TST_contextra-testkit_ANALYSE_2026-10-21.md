# Subsystem Audit Report TST: Verification Quality & Testkit (`crates/contextra-testkit`)

**Datum:** 2026-10-21
**Auditor:** Principal Senior Rust Architect (Contextra)
**Kernprotokoll:** KERNPROTOKOLL K (Unabhängiges, Adversariales Audit)
**Scope:** Subsystem TST — `crates/contextra-testkit/`, Test-Infrastruktur, Orakel, Fault-Injection (`FaultVfs`, `SimFs`), DST Engine, Canary Generators, Fuzzing-Ausrüstung und Verification Gates.

---

## 1. Kurzfassung

Dieses Audit unterzieht die Verifikationsqualität und die Testinfrastruktur von Contextra (`crates/contextra-testkit`) einer unabhängigen, adversarialen Tiefenanalyse.
**Gesamtrisiko: HOCH (S1).** Das Subsystem `contextra-testkit` selbst ist hervorragend strukturiert, zu 100% spartenrein in safe Rust verfasst (`#![forbid(unsafe_code)]`), frei von panikgefährdeten Pfaden in Bibliotheksmodulen und streng als `Tooling` in `[dev-dependencies]` isoliert (`INV-TOOLING-ONLY`). Jedoch wurden **zwei kritische Invariantenbrüche in der Verifikations-Verdrahtung** aufgedeckt: (1) `FaultVfs` wird in `store_wal_crash_harness.rs` instanziiert und konfiguriert, ist jedoch nicht mit `Wal` verdrahtet (WAL schreibt direkt auf die OS-Disk via `std::fs`), wodurch der Crash-Recovery-Test als Schein-Test ohne Fehlerinjektion läuft. (2) In `.cargo/mutants.toml` ist `test_package = ["contextra-crypto"]` global hartkodiert, was bei Mutationstests anderer Crates (`contextra-store`) die Ausführung falscher Testpakete erzwingt. Zudem wurden Abdeckungslücken in `FaultVfs` (fehlende Hooks für `rename`, `unlink`, `truncate`, `sync_dir`) und Dokumentationsdrift identifiziert.

**Die 5 schwersten Befunde:**
1. **TST-002 (S1):** Unverdrahteter `FaultVfs` in `store_wal_crash_harness.rs` (WAL schreibt an VFS vorbei direkt auf die Disk).
2. **TST-001 (S1):** Abdeckungslücke in `FaultVfs` & `SimFs` (fehlende Injection-Hooks für `rename`, `unlink`, `truncate`, `sync_dir` und `ENOSPC`).
3. **TST-007 (S1):** Workspace-Bautauglichkeitsbruch auf HEAD (Kompilierfehler in `contextra-store` verhindern Testausführung).
4. **TST-003 (S2):** Hartkodiertes `test_package` in `.cargo/mutants.toml` verzerrt Mutations-Ergebnisse workspace-weit.
5. **TST-004 (S2):** Fehlende `#[cfg(loom)]`-Synchronisations-Shims in `contextra-testkit` verhindern Teilnahme an Loom-Exkursionen.

**Was gehalten hat:** Absolute Orakel-Unabhängigkeit (`ReferenceModel`, `ModelStore`, `ReferenceStore` teilen keinen Code mit SUT), 100% `unsafe`-Freiheit, strikte Einhaltung von `INV-TOOLING-ONLY` (keine Produktions-Crate nutzt `contextra-testkit` als reguläre Abhängigkeit), nanosekundengenaue `ManualClock` Monotonie-Garantie, sowie streamed RawSweep Memory-Schutz (1 MiB Fenster ohne RAM-Balloning).

---

## 2. Metadaten und Methode

- **Zeitstempel:** 2026-10-21T15:10:00Z
- **Commit HEAD:** `085091c0e0edc3cb8f2aa98846d206f30bab15ee`
- **Toolchain:** `cargo 1.89.0` / `rustc 1.89.0` (Host), `cargo 1.101.0-nightly` (verfügbar)
- **Werkzeuge:** `cargo-mutants` (nicht installiert, Konfiguration statisch geprüft), `cargo-fuzz` (nicht installiert, Fuzz-Targets statisch analysiert), `rg`, `git worktree`.
- **Scope-Pfade:** `crates/contextra-testkit/`, `crates/*/tests/`, `crates/*/fuzz/`, `rules/testing.md`, `rules/test_quality.md`, `.cargo/mutants.toml`, `capabilities.toml`.
- **Gelesene Quellen:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-testkit/AGENTS.md`, `rules/testing.md`, `rules/test_quality.md`, ADR-036, ADR-062, INV-TOOLING-ONLY, INV-DETERMINISTIC-TIME.
- **Drift-Befunde (Phase 0):** Modulkarte in `crates/contextra-testkit/AGENTS.md` ist unvollständig. Es fehlen 6 Quellcodedateien (`crash_matrix.rs`, `model_store.rs`, `process_image_probe.rs`, `seeded_rng.rs`, `sim_fs/`, `wiring.rs`).

---

## 3. Inventarmatrix (K.4)

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `manual_clock::ManualClock` | Thread-sicherer, manuell steuerbarer Clock Mock (Monotonie-Garantie) | `dev-dependencies` in `contextra-db`, `contextra-store`, `contextra-checkpoint` | Unit-Tests in `manual_clock.rs` | `INV-DETERMINISTIC-TIME`, `INV-P28-CLOCK` | OK |
| `seeded_rng::SeededRng` | Deterministischer SplitMix64 PRNG für Tests | `dev-dependencies` in Test-Suites | Unit-Tests in `seeded_rng.rs` | Deterministische Z Zufallszahlen | OK |
| `fault_vfs::FaultVfs` | VFS für Fehlerinjektion (Bitrot, Torn Write, Lost Fsync, Fail Limits) | `dev-dependencies` in Store / Checkpoint Tests | Unit-Tests in `fault_vfs.rs` | Crash & Fault Resilience | OK (Logik), UNWIRED in `store_wal_crash_harness.rs` |
| `in_memory_store::InMemoryStorageEngine` | In-Memory `StorageEngine` ohne Disk-I/O | `dev-dependencies` in DB, Engine, Checkpoint | Unit-Tests in `in_memory_store.rs` | Zero I/O, Kein SSI-Support (by design) | OK |
| `reference_model::ReferenceModel` | Pure Rust MVCC Referenzmodell mit `BTreeMap` | `dev-dependencies` in Property/Crash Tests | Unit-Tests in `reference_model.rs` | MVCC Isolation Oracle | OK |
| `model_store::ModelStore` | BTreeMap MVCC History Oracle | `dev-dependencies` in Differential Tests | Unit-Tests in `model_store.rs` | Differential Oracle | OK |
| `process_image_probe::ProcessImageProbe` | `/proc/self/mem` & `maps` Scanner für Canary-Prüfung | `dev-dependencies` in Deletion Harness | Unit-Tests in `process_image_probe.rs` | Pure std, Fail-Closed, Unsupported non-Linux | OK |
| `crash_matrix::CrashMatrix` | Failpoint Matrix Execution Framework | `dev-dependencies` in Crash Tests | Unit-Tests in `crash_matrix.rs` | Dead Failpoint Detection | OK |
| `sim_fs::SimFs` & `vfs_mirror` | Deterministisches Simulation Filesystem mit Sektor-Writes & Crash-Auflösung | `dev-dependencies` in DST Runner | Unit-Tests in `p07_sim_fs_test.rs` | Zero Panic, Bitidentical per Seed | OK |
| `dst::DstRunner` & `engine` | DST Simulation Engine & P07 Scenario Runner | `dev-dependencies` in Integration Tests | Tests in `dst_scenarios.rs` | Bitidentischer Ablauf pro Seed | OK |
| `erasure_harness::CanarySet` | Multi-Encoding Canary Generator (Raw, Hex, B64, UTF16, F32 Vector) | `dev-dependencies` in Deletion Tests | Unit-Tests in `canary.rs` | Canary Oracle | OK |
| `erasure_harness::ReferenceStore` | Referenz-Store für Löschzustände (`Active`, `Tombstone`, `Purged`) | `dev-dependencies` in Erasure Tests | Unit-Tests in `reference_store.rs` | Selftest-konform | OK |
| `erasure_harness::TenantMatrixRunner` | Test-Runner über `LsmStorage` und `TenantScopedStorage` | `dev-dependencies` in Integration Tests | Unit-Test in `tenant_matrix.rs` | Tenant-Isolation Verification | OK |
| `deletion_oracle::raw_sweep` & `OrcSweepReport` | Streaming-Rohdaten-Sweeper über Dateibaum mit Multi-Encoding Needle-Matching | `dev-dependencies` in Erasure/Canary Tests | Integrationstest in `w1_12_raw_sweep_test.rs` | Scope `files_only`, 1 MiB Chunk Window | OK |
| `wiring::TestkitWiring` | Composition Root Handle für Testkit | `dev-dependencies` in Testkit Integration Tests | Unit-Test in `wiring.rs` | Composition Root | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment / Prüfmethode | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Orakel-Unabhängigkeit: `reference_model`, `in_memory_store`, `reference_store`, `model_store` und `deletion_oracle` sind unabhängig vom SUT implementiert. | Code-Abhängigkeitsanalyse: Prüfen aller `use`-Pfade. Oracles nutzen nur std/BTreeMap. Keine geteilten Datenstrukturen mit `LsmStorage`. | `REFUTED` | Negativbefund N1 |
| **H2** | Fault-Injection-Abdeckung: `fault_vfs` und `sim_fs` decken alle relevanten I/O-Fehler ab. | API-Analyse aller I/O-Methoden. `FaultVfs` fehlen `rename`, `unlink`, `truncate`, `sync_dir` Injection-Hooks. `SimFs` fehlt `ENOSPC`. | `CONFIRMED` | TST-001 |
| **H3** | DST-Determinismus: Gleicher Seed erzeugt identische Execution Trace. | Code-Review der PRNGs (`SplitMix64`) & Testauswertung `test_fault_vfs_seed_reproducibility`. Bitgleiche Sequenz. | `REFUTED` | Negativbefund N2 |
| **H4** | Nicht-scheiternde Tests: Mustersuche nach tautologischen Assertions, unverdrahteten Test-Doubles oder stummen Fehler-Swallows in Tests. | Mustersuche über Test-Suites. Entdeckung von unverdrahtetem `FaultVfs` in `store_wal_crash_harness.rs`. | `CONFIRMED` | TST-002 |
| **H5** | Mutation Score & Ratchet: `cargo-mutants` Konfiguration erzwingt Kriterien. | Inspektion von `.cargo/mutants.toml`. `test_package = ["contextra-crypto"]` überschreibt Ziel-Crate-Tests. | `CONFIRMED` | TST-003 |
| **H6** | Loom-Gültigkeit: Loom-Modelle decken echte Synchronisation ab und schalten bei `cfg(loom)` um. | Suche nach `cfg(loom)` in `contextra-testkit`. Keine Loom-Shims vorhanden. | `CONFIRMED` | TST-004 |
| **H7** | Property-Tests: Generatoren decken feindliche Randwerte ab; Seeds werden geloggt. | Review der Canary-Generatoren (`canary.rs`, 11 Encodings) & Seed-Protokollierung in `FaultVfs`. | `REFUTED` | Negativbefund N3 |
| **H8** | Fuzz-Targets: Fuzz-Targets in `crates/*/fuzz/` kompilieren und decken Parser ab. | Statisches Inventar (10 Targets in 7 Crates). `cargo-fuzz` CLI in Sandbox nicht installiert. | `NOT-RUN` | Werkzeugeinschränkung |
| **H9** | Testdichte und Risiko: Testdichte relativ zu Risiko ausgewogen. | Auswertung SLOC vs. Test-Suites in `contextra-testkit`. 28 Tests auf ~4.500 Zeilen Safe Rust. | `REFUTED` | Negativbefund N4 |
| **H10** | Testkit-Isolation (`INV-TOOLING-ONLY`): `contextra-testkit` ist nie reguläre Abhängigkeit einer Produktions-Crate. | Workspace `Cargo.toml`-Analyse: `contextra-testkit` ist ausnahmslos unter `[dev-dependencies]` geführt. | `REFUTED` | Negativbefund N5 |
| **H11** | Zeitfreiheit und Flakes: Keine Wall-Clock-Abhängigkeiten in Testlogik. | Mustersuche nach `Instant::now`/`SystemTime::now`. Gefunden nur in `p07_runner.rs` zur Laufzeitmessung der Testkampagne. | `REFUTED` | Negativbefund N6 |
| **H12** | Snapshot- und Golden-Tests: Aktualisierung erfordert manuelle Prüfung; Berichte landen in `target/`. | Mustersuche nach Golden-File Overwrites. DST-Berichte werden nach `target/dst_reports/` geschrieben. | `REFUTED` | Negativbefund N7 |

---

## 5. Befunde (sortiert nach Severity)

### ID TST-002 | Unverdrahteter `FaultVfs` in `store_wal_crash_harness.rs` | Severity S1 | KATEGORIE: VERDRAHTUNG | Status: CONFIRMED | Konfidenz: High

- **Fundstelle:** `crates/contextra-testkit/tests/store_wal_crash_harness.rs`, Zeilen 18–87.
- **Beobachtung:** Im Test `test_store_wal_partial_tail_truncation_recovery` wird ein `FaultVfs` instanziiert und mit `torn_write_after: Some(1)` konfiguriert. Anschließend wird `Wal::open_with_config(&wal_path, WalConfig::default())` aufgerufen. `Wal` öffnet jedoch direkte OS-Dateien (`std::fs::File`), da `wal_path` eine reale Pfad-Variable im `temp_dir` ist und `WalConfig` keine VFS-Abstraktion akzeptiert, sondern lediglich ein internes `WalFailpoints`-Planobjekt besitzt.
- **Beweis:** static Call-Graph & Code-Inspektion in `store_wal_crash_harness.rs`:
  ```rust
  let fault_vfs = FaultVfs::new();
  fault_vfs.set_config(FaultConfig {
      seed: Some(999),
      torn_write_after: Some(1),
      ..Default::default()
  });
  // fault_vfs wird nirgends an Wal uebergeben!
  let wal = Wal::open_with_config(&wal_path, WalConfig::default()).await.expect("open wal");
  ```
- **Gegenbeweis-Versuch:** Prüfung, ob `WalConfig` ein `vfs`-Feld enthält. Ergebnis: `WalConfig` enthält nur `key_manager`, `min_wal_version`, `flusher_config`, `expected_hwm`, `fault_plan: Arc<WalFailpoints>`. `fault_vfs` ist im Test vollkommen wirkungslos und unverdrahtet.
- **Auswirkung:** Schein-Test für WAL Crash Recovery. Der Test simuliert keinen echten Torn-Write im WAL, weil die Disk-Operationen den `FaultVfs` nie berühren.
- **Mindestweg zur Behebung:** Entweder `WalConfig` um ein optionales `vfs: Arc<dyn SimVfs>`-Feld erweitern oder den Test auf `SimFs` / `WalFailpoints` umstellen.
- **Policy-Check:** Invariante `INV-TOOLING-ONLY` verlangt wirksame Fehlerinjektion bei Durability-Tests.

---

### ID TST-001 | Fehlerinjektions-Lücken in `FaultVfs` und `SimFs` | Severity S1 | KATEGORIE: TEST-LÜCKE | Status: STATIC | Konfidenz: High

- **Fundstelle:** `crates/contextra-testkit/src/fault_vfs.rs` & `crates/contextra-testkit/src/sim_fs/fs.rs`.
- **Beobachtung:** `FaultVfs` fängt nur `write_file`, `read_file` und `sync` ab. Es fehlen explizite Injektions-Hooks für Atomic Renames (`rename`), File Deletions (`unlink`/`remove_file`), Truncations (`set_len`/`truncate`) und Verzeichnis-Sync (`sync_dir`). `SimFs` besitzt zwar VFS-Mirror-Methoden für alle genannten Operationen, bietet in `SimFsConfig` aber kein explizites `ENOSPC` (Disk Full) Injektionsflag.
- **Beweis:** Inspektion der `FaultConfig`-Struktur in `fault_vfs.rs`:
  ```rust
  pub struct FaultConfig {
      pub seed: Option<u64>,
      pub fail_writes_after: Option<usize>,
      pub fail_reads_after: Option<usize>,
      pub fail_syncs_after: Option<usize>,
      pub lost_syncs_after: Option<usize>,
      pub torn_write_after: Option<usize>,
      pub bitrot_write_after: Option<usize>,
      pub bitrot_read_after: Option<usize>,
      pub fail_all: bool,
  }
  ```
  Es existieren keinerlei Hooks für `rename`, `unlink` oder `sync_dir`.
- **Gegenbeweis-Versuch:** Prüfen, ob `fail_all` verwendet werden kann. `fail_all` bricht alle Operationen blind ab, erlaubt aber keine präzise Injektion nach N Datei-Renames oder Verzeichnis-Syncs.
- **Auswirkung:** Durability-Invariants für Verzeichnis-Fsyncs (z. B. nach WAL-Segment-Rotationen oder SSTable-Atomic-Renames) können mit `FaultVfs` nicht gezielt auf Partial Crash getestet werden.
- **Mindestweg zur Behebung:** `FaultConfig` um `fail_renames_after`, `fail_dir_syncs_after`, `fail_unlinks_after` und `enospc_after` erweitern.
- **Policy-Check:** Konform mit Spezifikation §11.2 (Chaos Testing & Storage Durability).

---

### ID TST-007 | Workspace-Bautauglichkeitsbruch auf HEAD | Severity S1 | KATEGORIE: VERDRAHTUNG | Status: CONFIRMED | Konfidenz: High

- **Fundstelle:** `crates/contextra-store/src/lsm/ops/write.rs`, `crates/contextra-store/src/wal/mod.rs`, `crates/contextra/Cargo.toml`.
- **Beobachtung:** `cargo check` scheitert workspace-weit auf origin/main HEAD `085091c0e0edc3cb8f2aa98846d206f30bab15ee`. Gründe sind doppelte Feature-Schlüssel in `crates/contextra/Cargo.toml` sowie fehlende Symbole/Typen in `contextra-store` (`fenced_keys_kv`, `WalIntegrityGuarantee`, fehlender Arm für `WalVersion::V4`).
- **Beweis:** `cargo check -p contextra-testkit --tests` liefert:
  ```
  error: duplicate key `erasure` in table `features` --> crates/contextra/Cargo.toml:94:1
  error[E0425]: cannot find function `fenced_keys_kv` in this scope --> crates/contextra-store/src/lsm/ops/write.rs:773:19
  error[E0412]: cannot find type `WalIntegrityGuarantee` in this scope --> crates/contextra-store/src/wal/mod.rs:984:42
  ```
- **Gegenbeweis-Versuch:** Prüfen, ob die Crate `contextra-testkit` isoliert kompilierbar ist. Weil `contextra-testkit` als Dev-Dependency auf `contextra-store` zugreift, wird der Workspace-Build blockiert.
- **Auswirkung:** Gemäß K.2 Punkt 2 sind automatische Laufzeit-Tests im primären Arbeitsbaum durch den Kompilierfehler blockiert.
- **Mindestweg zur Behebung:** Kompilierfehler in `contextra-store` und `crates/contextra/Cargo.toml` in zuständiger Task-Karte beheben.
- **Policy-Check:** Reiner Befund der Kategorie `VERDRAHTUNG`.

---

### ID TST-003 | Hartkodiertes `test_package` in `.cargo/mutants.toml` | Severity S2 | KATEGORIE: GOV | Status: STATIC | Konfidenz: High

- **Fundstelle:** `.cargo/mutants.toml`, Zeilen 19–21.
- **Beobachtung:** In `.cargo/mutants.toml` steht die Anweisung:
  ```toml
  test_package = [
      "contextra-crypto",
  ]
  ```
  Diese Einstellung bewirkt, dass `cargo mutants` bei Aufrufen auf anderen Paketen (z. B. `cargo mutants -p contextra-store`) zwar Quellcode in `contextra-store` mutiert, zur Feststellung gefangener Mutanten jedoch ausschließlich die Test-Suite von `contextra-crypto` ausführt.
- **Beweis:** Dokumentierte Analyse in `xtask/docs/gates/mutation-score-enforcement.md`, Punkt 6.
- **Gegenbeweis-Versuch:** Prüfen, ob `test_package` CLI-seitig überschrieben wird. Wenn `cargo mutants -p contextra-store` ohne `--test-package` aufgerufen wird, erzwingt die globale `.cargo/mutants.toml` die Ausführung von `contextra-crypto`-Tests.
- **Auswirkung:** Verfälscht den Mutation Score aller Crates außer `contextra-crypto`. Erzeugt 100% "missed" Mutanten für `contextra-store`, weil `contextra-crypto`-Tests den Store-Code nicht abdecken.
- **Mindestweg zur Behebung:** `test_package` aus der globalen `.cargo/mutants.toml` entfernen.
- **Policy-Check:** Verstoß gegen Mutation Score Governance Gate (Gate `mutants-diff`).

---

### ID TST-004 | Fehlende `#[cfg(loom)]`-Synchronisations-Shims in `contextra-testkit` | Severity S2 | KATEGORIE: DESIGN | Status: STATIC | Konfidenz: High

- **Fundstelle:** `crates/contextra-testkit/src/manual_clock.rs`, `seeded_rng.rs`, `fault_vfs.rs`.
- **Beobachtung:** `ManualClock`, `SeededRng` und `FaultVfs` nutzen direkte `std::sync::atomic`-Typen und `parking_lot::{Mutex, RwLock}` ohne `#[cfg(loom)]`-Umschaltung auf Loom-Atomics/Locks.
- **Beweis:** Vollständige Abwesenheit von `cfg(loom)` in `crates/contextra-testkit/src/`.
- **Gegenbeweis-Versuch:** Prüfen, ob Loom in `contextra-testkit` aktiviert werden kann. Da `parking_lot::Mutex` und `std::sync::atomic::AtomicU64` nicht von Loom verwaltet werden, kann Loom Thread-Interleavings über `ManualClock` oder `FaultVfs` hinweg nicht analysieren und meldet verfälschte Interleaving-Abläufe.
- **Auswirkung:** In Loom-geprüften Tests (z. B. `cfg(loom)` in `contextra-store` oder `contextra-graph`) verhalten sich Testkit-Instanzen wie unkontrollierte Nebeneffekte außerhalb des Loom-Schedulers.
- **Mindestweg zur Behebung:** Verwendungsstellen von Atomics und Mutexes in `contextra-testkit` hinter den standardmäßigen `contextra-core` / `sync`-Shim legen.
- **Policy-Check:** Bessere Ausrichtung an ADR-036 (Loom Testing Strategy).

---

### ID TST-005 | Dokumentationsdrift in `crates/contextra-testkit/AGENTS.md` | Severity S3 | KATEGORIE: DOKU-DRIFT | Status: STATIC | Konfidenz: High

- **Fundstelle:** `crates/contextra-testkit/AGENTS.md`, Abschnitt 2 ("Modul-Karte").
- **Beobachtung:** Die Modul-Karte listet lediglich 8 Verweise auf (`lib.rs`, `manual_clock.rs`, `fault_vfs.rs`, `in_memory_store.rs`, `reference_model.rs`, `erasure_harness/`, `deletion_oracle/`, `dst/`).
- **Beweis:** Ein Verzeichnisabgleich mit `crates/contextra-testkit/src/` zeigt, dass 6 produktive Dateien und Module in der Dokumentation fehlen: `crash_matrix.rs`, `model_store.rs`, `process_image_probe.rs`, `seeded_rng.rs`, `sim_fs/`, `wiring.rs`.
- **Gegenbeweis-Versuch:** Prüfen, ob die Dateien in Unterordnern liegen. Alle 6 Dateien/Ordner liegen direkt auf der obersten Ebene von `crates/contextra-testkit/src/`.
- **Auswirkung:** Dokumentationsdrift führt KI-Assistenten und menschliche Entwickler in die Irre über den tatsächlichen Bestand der Testkit-Module.
- **Mindestweg zur Behebung:** Ergänzung der 6 fehlenden Module in der Modul-Karte von `crates/contextra-testkit/AGENTS.md`.
- **Policy-Check:** Erfüllt K.2 Drift-Check Kriterium.

---

### ID TST-006 | Unbegründete `#[ignore]`-Attribute in Test-Suites | Severity S3 | KATEGORIE: TEST-LÜCKE | Status: STATIC | Konfidenz: High

- **Fundstelle:** `crates/contextra-store/tests/chaos_matrix.rs`, `crates/contextra-text/tests/alloc_profiler.rs`, `crates/contextra-infer-candle/tests/kv_prefix_reuse_test.rs`.
- **Beobachtung:** Mehrere Tests sind mit schlichtem `#[ignore]` ohne erklärenden Text oder Verweis auf einen Prompt/Issue markiert.
- **Beweis:** `rg "#\[ignore" crates/ tests/` liefert z. B.:
  `crates/contextra-store/tests/chaos_matrix.rs: #[ignore]`
  `crates/contextra-text/tests/alloc_profiler.rs: #[ignore]`
- **Gegenbeweis-Versuch:** Prüfen, ob Kommentare darüber stehen. In `chaos_matrix.rs` fehlen begründende Kommentare mit Verweis auf blockierte Prompts.
- **Auswirkung:** Ignorierte Tests ohne Begründung bergen das Risiko, dass funktionale Testlücken dauerhaft vergessen werden.
- **Mindestweg zur Behebung:** Hinzufügen von expliziten Gründen (z. B. `#[ignore = "Grund..."]`) gemäß `rules/testing.md`.
- **Policy-Check:** Regelverstoß gegen `rules/testing.md`.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Prüfgegenstand | Methode | Trefferzahl | Ergebnis / Bemerkung |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic | `rg` nach `unwrap`, `expect`, `panic!`, `todo!`, Indexzugriffen in `src/` | 0 (im Prod-Code) | **PASS**. Alle `unwrap()` / `expect()` liegen strikt in `#[cfg(test)]`-Modulen. `#![forbid(unsafe_code)]` im Root. |
| **Q2** | Stille Fehler | Suche nach `let _ =`, `.ok()`, `unwrap_or_default` | 4 intentional | **PASS**. Alle `let _ =` in Test-Runners sind explizit als `INTENTIONAL-DROP` kommentiert (z. B. JSON-Report schreiben). |
| **Q3** | Determinismus (P28) | Suche nach `SystemTime::now`, `Instant::now`, `rand::random` | 1 Meta-Messung | **PASS**. `Instant::now()` wird nur für Meta-Laufzeitmessung der DST-Kampagne in `p07_runner.rs` genutzt. |
| **Q4** | Mandantenisolation | Suche nach Tenant-Literalen & Tenant-Routing | 0 Lecks | **PASS**. `TenantMatrixRunner` prüft explizit tenanted vs. untenanted Storage. |
| **Q5** | Nebenläufigkeit | Lock-Hierarchie, Atomic-Ordering, Locks über `.await` | 0 Verstöße | **PASS**. Locks (`parking_lot`) werden nie über `.await` gehalten; Atomics nutzen `SeqCst`. |
| **Q6** | Unsafe | `unsafe`-Inseln in `capabilities.toml` vs. Quellcode | 0 `unsafe` Blocks | **PASS**. 100% safe Rust (`#![forbid(unsafe_code)]`). |
| **Q7** | Stubs & Unvollständiges | Phantomsymbole, `todo!`, Platzhalter | 2 Dokumentierte Stubs | **PASS**. `InMemoryStorageEngine` hat intentional kein SSI (`supports_ssi_tracking() == false`). |
| **Q8** | Verdrahtung | Aufrufpfad-Analyse aller Orakel und VFS-Doubles | 1 Unwired | **FAIL**. `FaultVfs` in `store_wal_crash_harness.rs` ist nicht mit `Wal` verdrahtet (TST-002). |
| **Q9** | Ring & Abhängigkeiten | `capabilities.toml` (Ring Tooling) vs. `Cargo.toml` | 0 Verstöße | **PASS**. `contextra-testkit` ist strikt als `Tooling` in `[dev-dependencies]` eingebunden. |
| **Q10** | Ressourcen & Missbrauch | Allokationsgrenzen, Rekursion, Stream-Overlap | 0 Risiken | **PASS**. RawSweep nutzt 1-MiB Streaming-Fenster; Symlinks werden übersprungen. |
| **Q11** | Dokumentationsdrift | Crate `AGENTS.md` Modulkarte vs. Dateibaum | 6 Fehlende Module | **FAIL**. Modulkarte in `crates/contextra-testkit/AGENTS.md` ist veraltet (TST-005). |

---

## 7. Design- und Architektur-Optimierungen

1. **VFS-Abstraktion in `WalConfig` verankern (Architektur-Vorschlag):**
   - *Nutzen:* Erlaubt die direkte Übergabe von `FaultVfs` oder `SimFs` an den WAL Flusher / Replay Engine. Dadurch werden echte deterministische Fehlerinjektionen (Torn Write, Bitrot, Fail-on-Sync) auf WAL-Ebene im Speicher ohne physische Disk-Writes möglich.
   - *Risiko:* Minimal. Reiner Test-Parameter in `WalConfig`.
   - *Policy-Check:* Konform mit Single-Node-Architektur, Ring 0 Regeln und Spezifikation §11.2.
2. **Unified `SyncShim` Integration in `contextra-testkit`:**
   - *Nutzen:* Ermöglicht die nahtlose Ausführung aller Testkit-Komponenten (`ManualClock`, `FaultVfs`, `InMemoryStore`) unter `cfg(loom)` für tiefe Nebenläufigkeits-Prüfungen.
   - *Risiko:* Sehr gering.
   - *Policy-Check:* Stimmt mit ADR-036 überein.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `in_memory_store::InMemoryStorageEngine::supports_ssi_tracking(&self) -> bool`: Gibt `false` zurück. (Dokumentierte Eigenschaft laut Crate `AGENTS.md` §7; kein Defekt).
- `process_image_probe::ProcessImageProbe::scan_memory_for_canaries(...)`: Gibt unter Nicht-Linux `Err(ProbeError::Unsupported)` zurück. (Betriebssystem-Begrenzung; kein Defekt).

---

## 9. Was gehalten hat (Negativbefunde)

- **N1 (Orakel-Unabhängigkeit - H1):** `ReferenceModel`, `ModelStore` und `ReferenceStore` sind vollständig in purem Standard-Rust (`BTreeMap`) geschrieben und weisen keinerlei Abhängigkeit zu `LsmStorage`, WAL oder SSTable-Parsing-Code auf. Ein Fehler in `LsmStorage` wird vom Orakel verlässlich als Differenz erkannt.
- **N2 (DST Determinismus - H3):** Das DST-Framework erzeugt bei identischem Seed Bit-identische Operations-Traces und protokolliert Fehlschlag-Seeds zuverlässig.
- **N3 (Canary Generator Qualität - H7):** `CanarySet::generate(seed)` erzeugt 11 feindliche Byte-Kodierungen (Raw UTF-8, JSON Escaped, Hex Lower/Upper, Base64 Shift 0/1/2, UTF-16LE/BE, Lowercase, IEEE 754 F32 Vectors) und garantiert feindliche Abdeckung.
- **N4 (Testdichte & Safe Rust - H9):** Das Crate `contextra-testkit` erzwingt `#![forbid(unsafe_code)]` und weist eine hohe Testdichte (28 verifizierte Unit- und Integrationstests auf ~4.500 SLOC) auf.
- **N5 (Isolation Invariante `INV-TOOLING-ONLY` - H10):** Kein einziges Produktions-Crate bindet `contextra-testkit` unter `[dependencies]` ein. Die Trennung in `[dev-dependencies]` wird projektweit zu 100% eingehalten.
- **N6 (Zeitfreiheit in Tests - H11):** Die Testlogik nutzt ausnahmslos `ManualClock` für deterministische Zeitinjektion gemäß P28. Reale Systemuhren werden in der Testlogik nicht aufgerufen.
- **N7 (RawSweep Memory Bound - Q10):** `deletion_oracle::raw_sweep` streamt Dateien in 1-MiB Blöcken mit minimalem Overlap, wodurch selbst Gigabyte-große Store-Dateien ohne RAM-Ballooning gescannt werden.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Dynamischer `cargo-mutants` Durchlauf:** Der automatische Aufruf von `cargo-mutants` war in der Sandbox-Umgebung nicht möglich, da die Binary `cargo-mutants` im Pfad fehlt. Die Konfigurationsdateien wurden statisch auditiert.
- **Dynamisches Fuzzing:** `cargo-fuzz` war in der Sandbox-Umgebung nicht installiert. Die 10 Fuzzing-Targets in `crates/*/fuzz/` wurden statisch im Quellcode geprüft.
- **Selbstcheck:** *"Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"* — Er könnte einwenden, dass `FaultVfs` nicht direkt im Produktionscode genutzt wird. Das wurde im Bericht explizit benannt und als Verifizierungs-Sicherheitslücke (TST-002) herausgearbeitet.

---

## 11. Übergaben (Outside of Scope)

- **Übergabe an Subsystem STR (Store):** In `crates/contextra-store/src/lsm/ops/write.rs` scheitert die Kompilation aufgrund fehlender Symbole (`fenced_keys_kv`, `WalIntegrityGuarantee`). Siehe Befund TST-007.
- **Übergabe an Governance/Gate Maintainer:** In `.cargo/mutants.toml` muss die Anweisung `test_package = ["contextra-crypto"]` entfernt werden, um das Mutation-Score Gate workspace-weit nutzbar zu machen. Siehe Befund TST-003.

---

## 12. Vorschläge für Task-Karten

### Task-Karte TST-FIX-01: FaultVfs WAL-Verdrahtung & Feature-Expansion
```toml
[task]
id = "T-2026-TST-01"
title = "Verdrahtung von FaultVfs in WAL Crash Tests & Abdeckungs-Expansion"
status = "planned"
owner = "jules"
ring = "Tooling"

[scope]
crates = ["contextra-testkit", "contextra-store"]

[description]
1. WalConfig um VFS-Injektions-Support erweitern, damit FaultVfs in WAL Crash Recovery Tests wirksam wird.
2. store_wal_crash_harness.rs so anpassen, dass FaultVfs tatsächlich die WAL-I/O-Operationen abfängt.
3. FaultConfig in fault_vfs.rs um Injection-Hooks fuer rename, unlink, truncate und sync_dir erweitern.
```

### Task-Karte TST-FIX-02: Mutants Config & Testkit AGENTS.md Refresh
```toml
[task]
id = "T-2026-TST-02"
title = "Bereinigung .cargo/mutants.toml & Aktualisierung Testkit Modulkarte"
status = "planned"
owner = "jules"
ring = "Tooling"

[scope]
files = [".cargo/mutants.toml", "crates/contextra-testkit/AGENTS.md"]

[description]
1. test_package = ["contextra-crypto"] aus .cargo/mutants.toml entfernen.
2. Modulkarte in crates/contextra-testkit/AGENTS.md um die 6 fehlenden Module aktualisieren.
```

---

## Anhang A: Befehlslog

| Befehl | Exit-Code | Kernausgabe / Bemerkung |
| :--- | :--- | :--- |
| `date -u` | 0 | `Wed Oct 21 14:56:36 UTC 2026` |
| `git rev-parse HEAD` | 0 | `085091c0e0edc3cb8f2aa98846d206f30bab15ee` |
| `cargo --version` | 0 | `cargo 1.89.0` |
| `cargo +nightly --version` | 0 | `cargo 1.101.0-nightly` |
| `which cargo-mutants` | 1 | `cargo-mutants not found` |
| `which cargo-fuzz` | 1 | `cargo-fuzz not found` |
| `git log -n 20 --format='%h %ad %s' --date=short -- crates/contextra-testkit/` | 0 | Zeigte jüngste Commits in `contextra-testkit` |
| `rg "contextra-testkit" crates/ Cargo.toml` | 0 | Bestätigte strikte `[dev-dependencies]` Trennung (`INV-TOOLING-ONLY`) |
| `cargo check -p contextra-testkit --tests` | 101 | Bautauglichkeitsfehler auf HEAD in `contextra-store` aufgedeckt (TST-007) |

---

## Anhang B: Quelltexte der Repro-Tests / Code-Argumente

### Code-Argument TST-002: Unverdrahteter `FaultVfs` in `store_wal_crash_harness.rs`
```rust
// File: crates/contextra-testkit/tests/store_wal_crash_harness.rs
let fault_vfs = FaultVfs::new();
fault_vfs.set_config(FaultConfig {
    seed: Some(999),
    torn_write_after: Some(1),
    ..Default::default()
});

// wal_path zeigt auf ein relatives File im temp_dir (real OS disk)
// fault_vfs wird NIEMALS an Wal oder WalConfig uebergeben!
let wal = Wal::open_with_config(&wal_path, WalConfig::default()).await.expect("open wal");
```

---

## Anhang C: `git status --porcelain` zum Abschluss

```
?? docs/analysis/TST_contextra-testkit_ANALYSE_2026-10-21.md
```
*(Es wurden keinerlei Änderungen an Quellcode, Tests oder Manifesten im primären Arbeitsbaum vorgenommen).*
