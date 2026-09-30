# Contextra — Vollständiger Commit- & PR-Funktionskatalog zur Qualitätssicherung & Nachspezifikation

**Dokument-Version:** 1.0.0
**Datum:** 2026-09-30
**Autor:** Principal Senior Rust Architect für Contextra
**Status:** Verbindliches Prüfdokument für die Qualitätsprüfung (QA & Audit)
**Repository-HEAD:** Commit `b81e14a` (30 Commits Gesamtverlauf)

---

## 1. Executive Summary & Anwendungsbereich

Im Zuge der Weiterentwicklung des Contextra Multi-Crate-Ökosystems wurden zahlreiche Kernfunktionalitäten, Sicherheitsmechanismen, Algorithmen und Optimierungen implementiert. Das System umfasst 33 Workspace-Crates, aufgeteilt in ein striktes 5-Ring-Architekturmodell (Ring 0 bis Ring 4) sowie Tooling/Harness-Module.

Dieses Dokument dient als **zentrale Referenz für den Qualitätsprüfer (QA/Auditor)**. Es analysiert systematisch und lückenlos den **gesamten Commit- und PR-Verlauf von Anfang bis Ende** und führt alle enthaltenen, teilweise undokumentierten oder opt-in Schnittstellen, Fehlerpfade und Algorithmen mit konkreten Prüfkriterien auf.

---

## 2. Vollständige Chronologische Commit- & PR-Analyse (30 Commits)

### Commit 1: `46f0a2c` — docs: complete algorithm criticality analysis and SOTA integration research report
- **Betroffene Komponenten:** `docs/`, `contextra-rank`, `contextra-adapt`, `contextra-mvcc`
- **Spezifizierte Funktionen & Features:**
  - **SOTA-Algorithmen-Kritikalitätsanalyse:** Dokumentation der mathematischen Garantien und Performanzgrenzen aller Kernalgorithmen.
  - **Adaptive Konforme Kalibrierung (`INV-CALIBRATION-CONFORMAL-1`):** Dynamic Score Recalibration in `contextra-rank` für konforme Wahrscheinlichkeitsgrenzen bei der Ähnlichkeitssuche.
  - **PID-Anti-Windup-Regelung (`INV-PID-ANTIWINDUP-1`):** Akkumulationsbegrenzung im PID-Latenz-Controller in `contextra-adapt`.
  - **MVCC SSI Write-Skew Validation (§B.2):** Mathematisches Fundament zur Erkennung von Schreib-Schiefe-Anomalien in `contextra-mvcc`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Verfiziere, dass `PlattScaledSigmoid` und Isotonische Kalibrierung Monotonie wahren.
  - [ ] Teste PID-Regler unter extremer Systemlast auf Sättigung ohne Anti-Windup-Explosion.

---

### Commit 2: `ec64513` — feat(store): implement explicit DurabilityMode and feature compatibility matrix
- **Betroffene Komponenten:** `contextra-store` (`src/config.rs`, `src/lib.rs`, `src/durability.rs`)
- **Spezifizierte Funktionen & Features:**
  - **`DurabilityMode` Enum:** Explizite Unterscheidung von `SyncOnCommit` (Full Fsync), `BackgroundSync` (Async Flush), `MemoryOnly` (RAM) und `WalNoHmac` (Optimierter Durchsatz ohne HMAC).
  - **Feature-Kompatibilitätsmatrix:** Automatische Konfigurations-Validierung gegen widersprüchliche Einstellungen (z. B. `DurabilityMode::MemoryOnly` mit aktiver `deletion_proof`-Anforderung).
  - **WAL-Header-Format-Validierung:** Striktes Parsen der Durability-Header beim Initialisieren von `LsmStorage`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Teste ungültige Einstellungskombinationen (z. B. `MemoryOnly` + `CryptoShred`) auf Erzeugung von `ContextraError::DurabilityVectorMismatch`.
  - [ ] Verifiziere das Umschalten von `DurabilityMode` beim Neustart.

---

### Commit 3: `2e336c1` — feat(mvcc): implement SSI read-set tracking and write-skew validation
- **Betroffene Komponenten:** `contextra-mvcc` (`src/ssi.rs`), `contextra-types` (`src/error.rs`)
- **Spezifizierte Funktionen & Features:**
  - **`SsiReadSet` Struktur:** Erfassung aller gelesenen Schlüssel und ihrer Snapshot-Sequenznummern (`reads: AHashMap<Vec<u8>, SeqNo>`).
  - **`SsiValidator` Trait & `SsiValidatorImpl`:** Thread-sichere Commit-Validierung mit `parking_lot::RwLock<AHashMap<Vec<u8>, SeqNo>>`.
  - **Fehlervariante `ContextraError::WriteSkewDetected { tx_id }`:** Typisierter Abbruch von Transaktionen bei erkannter Schlüsselmutation seit Snapshot-Erstellung.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe nebenläufige Transaktions-Tests mit überlappenden Lese- und Schreibmengen aus und erzwinge `WriteSkewDetected`.
  - [ ] Prüfe DTO-Konvertierung von `WriteSkewDetected` in `contextra-wire` und `contextra-engine`.

---

### Commit 4: `bed8f21` — feat(mvcc): add SSI forget_from, capacity bounds, phantom protection, and TxBuffer byte budget
- **Betroffene Komponenten:** `contextra-mvcc` (`src/ssi.rs`, `src/sequence_log.rs`), `contextra-core` (`src/tx_buffer.rs`)
- **Spezifizierte Funktionen & Features:**
  - **`forget_from` Sequenz-Pruning:** Speicherschonende Bereinigung historischer Lese-Sets vor `bound_seq`.
  - **Read-Set Kapazitätsbegrenzung:** O(1)-Schutz gegen RAM-Explosion via `DEFAULT_MAX_READ_SET_KEYS` (100.000 Schlüssel).
  - **Phantom-Read-Schutz:** Bereichsschlüssel-Prüfung bei Range-Scans zur Verhindung von Phantom-Inserts.
  - **`TxBuffer` Byte-Budget-Enforcement:** Obergrenze für uncommitteten Puffer-Speicher pro Transaktion.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Überschreite 100.000 gelesene Keys in einer Transaktion und verifiziere Kapazitätsfehler.
  - [ ] Teste Range-Query gefolgt von Fremd-Insert zur Bestätigung der Phantom-Read-Interception.

---

### Commit 5: `67950e4` — fix(engine): consolidate KvKeyLocks to Tokio Mutex to fix E0277 Send blocker
- **Betroffene Komponenten:** `contextra-engine` (`src/collection/crud/locks.rs`)
- **Spezifizierte Funktionen & Features:**
  - **`KvKeyLocks` Tokio Mutex Konsolidierung:** Ersetzung synchroner/nicht-`Send` Sperren durch `tokio::sync::Mutex` zur Beseitigung von Async-Task-Migration-Blockern.
  - **Granulare Schlüssel-Sperrung:** Parallele Zeilen-/Schlüssel-Sperren ohne globale Collection-Mutex-Sperre.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe `cargo check -p contextra-engine` unter `tokio` Multi-Thread Executor aus.
  - [ ] Verifiziere Stress-Tests auf parallele Schlüssel-Writes ohne Deadlocks.

---

### Commit 6: `238ea0e` — fix(engine): make multi-index transaction commit 2PC crash-safe
- **Betroffene Komponenten:** `contextra-engine` (`src/collection/crud/db_transaction_commit.rs`)
- **Spezifizierte Funktionen & Features:**
  - **Multi-Index Two-Phase Commit (2PC):** Atomare Transaktions-Commits über LSM-Store, Vector (HNSW), Text (BM25) und Graph (CSR/Hyperedge).
  - **`CommitIntent` Zustands-Engine:** `CommitIntent::Pending` vor Index-Commits, gefolgt von Storage-Commit und `CommitIntent::Committed`.
  - **`CommitLedger` Kompensations-Aktionen:** LIFO-Rollback-Kompensationen (`CompensateHnswAction`, `CompensateTextAction`, `CompensateGraphAction`, `CompensateLsmAction`).
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Injiziere Crashs/Failures zwischen Phase 1 und Phase 2 und verifiziere den Rollback via `CommitLedger`.

---

### Commit 7: `39389a7` — I have hardened the harness verification gate and associated tools
- **Betroffene Komponenten:** `xtask` (`src/harness/`, `tests/`)
- **Spezifizierte Funktionen & Features:**
  - **Tree-Hash-Berechnung über Git-Index:** Manipulationssichere Hash-Erstellung über ein temporäres Git-Index-Arbeitsfenster.
  - **`ledger` Harness-Modul:** Protokollierung von Audit-Entscheidungen und Session-Verlauf.
  - **`env-attest` Harness-Modul:** Attestierung der Build- und Ausführungsumgebung.
  - **`loop-guard` Harness-Modul:** Automatische Schleifenerkennung für CI/Agenten.
  - **`session-report` Harness-Modul:** Detaillierte Metrik-Aggregierung von CI-Testläufen.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe `cargo test -p xtask --test harness_*` aus.

---

### Commit 8: `e3ff2f4` — Remove #[allow(dead_code)] suppressions in 10 crates (J11)
- **Betroffene Komponenten:** `contextra-checkpoint`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-mcp`, `contextra-router`
- **Spezifizierte Funktionen & Features:**
  - **Dead-Code Cleanout:** Entfernung aller unterdrückten tot-Quellcode-Blöcke.
  - **`#[cfg(test)]` Gating:** Strikte Beschränkung von Hilfsfunktionen auf Testumgebungen.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Prüfe mit `cargo clippy --workspace --all-targets` auf verbleibende ungenutzte Symbole.

---

### Commit 9: `881dee2` — feat(ci): implement verdict harness module, workflows, rulesets and governance
- **Betroffene Komponenten:** `xtask/src/harness/verdict.rs`, `.github/workflows/verdict.yml`, `governance/verdict-required.toml`, `.github/rulesets/`
- **Spezifizierte Funktionen & Features:**
  - **`verdict` Single Merge Engine:** Zentrale, nicht-umgehbare Merge-Entscheidungslogik.
  - **`verdict-required.toml` Governance Rules:** Regelset für Pflicht-Checks vor Main-Merges.
  - **Branch Protection Rulesets:** `protect-main.json` und `protect-main.solo.json`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Simuliere fehlgeschlagene CI-Pipelines und verifiziere, dass `verdict` den Merge hart blockiert.

---

### Commit 10: `5f123e0` — feat(wal): close WAL integrity gaps and implement WalNoHmac mode
- **Betroffene Komponenten:** `contextra-store` (`src/wal/`)
- **Spezifizierte Funktionen & Features:**
  - **`DurabilityMode::WalNoHmac` (ADR-100):** Format `b"MFN3"`, `WalVersion::V3NoHmac` und Standalone-CRC32-Checksummen pro WAL-Eintrag.
  - **Atomare WAL Legacy-Key Migration:** Dualer `fsync` (Datei und Parent Directory) vor dem Schreiben des `.rekeyed`-Markers.
  - **Dauerhafte Fallback-Sperre:** Wenn `.rekeyed` existiert, wird jeglicher unsichere Obfuscation-Fallback garantiert abgelehnt.
  - **Atomare Sidecar-Erstellung (`.wal_integrity_key`, `.uuid`):** Vermeidung von Race Conditions via `hard_link`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Unterbrich den Legacy-Key Migration Process mitten im Schreiben und verifiziere Crash-Safety.
  - [ ] Teste Replay von `WalNoHmac`-Dateien unter Korruptionsinjektion.

---

### Commit 11: `109e545` — test(store): add MVCC reference model, crash harness, and property tests (J12)
- **Betroffene Komponenten:** `contextra-testkit`, `contextra-store`, `docs/spec-sync/J12.md`
- **Spezifizierte Funktionen & Features:**
  - **`ReferenceModel` in `contextra-testkit`:** Deterministisches `std`-only In-Memory-Referenzmodell zur Validierung von Speicher-Zuständen.
  - **WAL Truncation Crash Recovery Test Harness:** Simulation teilweiser WAL-Schreibabbrüche an beliebigen Byte-Grenzen.
  - **Modellbasierte MVCC Property Tests:** Automatisierte Vergleiche zwischen `LsmStorage` und `ReferenceModel`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe Property-Tests mit `quickcheck` / `proptest` für `LsmStorage` aus.

---

### Commit 12: `ea7a489` — feat(store): harden WAL legacy migration crash safety, sidecars and WalNoHmac replay
- **Betroffene Komponenten:** `contextra-store` (`src/wal/replay.rs`, `src/wal/migration.rs`)
- **Spezifizierte Funktionen & Features:**
  - **Härtung der Replay-Engine:** Robuste Fehlerbehandlung bei abgeschnittenen oder korrupten `WalNoHmac`-Dateien.
  - **Sidecar-Integritätsprüfung:** Validierung der UUID und Integrity-Keys beim Systemstart.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Prüfe Replay-Verhalten mit manipulierter UUID-Sidecar-Datei.

---

### Commit 13: `addd337` — feat(engine): wire tenant-scoped storage and startup SSI checks
- **Betroffene Komponenten:** `contextra-engine` (`src/contextra.rs`, `src/lifecycle.rs`), `contextra-store` (`src/tenant_scoped.rs`)
- **Spezifizierte Funktionen & Features:**
  - **`collection_for_tenant` & `collection_with_language_and_tenant`:** Mandanten-isolierter Zugriff auf Sammlungen unter Nutzung von `TenantScopedStorage`.
  - **`drop_collection` Isolation:** Kaskadierende Bereinigung mandantenspezifischer Key-Ranges.
  - **Startup SSI Capability Check (`supports_ssi_tracking`):** Validierung der SSI-Fähigkeit der Storage-Engine beim Systemstart.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Teste Zugriffsversuche von Tenant A auf Daten von Tenant B und verifiziere `ContextraError::KeyNotFound` / Isolation.

---

### Commit 14: `ea70ccf` — refactor(contextra-py): sanitize panic test hooks and narrow clippy attributes
- **Betroffene Komponenten:** `contextra-py` (`src/bindings/`)
- **Spezifizierte Funktionen & Features:**
  - **FFI Panic Isolation (`run_blocking_ffi`):** Fälltsichere Kapselung von Rust-Panics an der PyO3-FFI-Grenze.
  - **Fehlergruppen-Säuberung:** Präzise Abbildung von Rust-Fehlern auf Python-Exceptions (`RuntimeError`, `ValueError`).
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Erzwinge ein Panic innerhalb einer Python-Binding-Methode und verifiziere sauberes Python-Exception-Handling ohne Absturz des Interpreter-Prozesses.

---

### Commit 15: `aafe145` — feat(graph): implement ApprhGateMonitor for shadow comparison evaluation
- **Betroffene Komponenten:** `contextra-graph` (`src/apprh/monitor.rs`)
- **Spezifizierte Funktionen & Features:**
  - **`ApprhGateMonitor` Shadow Evaluation:** Paralleles Ausführen von APPRH (Approximate Personalized PageRank Hypergraph) gegen Standard-PPR.
  - **Divergenz-Metriken:** Erfassung von Präzisions- und Recall-Abweichungen sowie Latenz-Differenzen im Shadow-Betrieb.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Aktiviere Shadow-Modus und prüfe Log-Ausgaben/Metriken auf Abweichungsmeldungen.

---

### Commit 16: `2e763ba` — feat(store): legacy HMAC key migration path and status tracking (F-30)
- **Betroffene Komponenten:** `contextra-store` (`src/manifest/`, `src/wal/`)
- **Spezifizierte Funktionen & Features:**
  - **Migrationspfad-Statusüberwachung (F-30):** Systemweite Abfragbarkeit des Rekeying-Status in Manifest und WAL.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Fragt `storage.migration_status()` ab und validiert den Fortschrittsindikator.

---

### Commit 17: `027a73c` — fix(engine): repair 2PC crash windows and orphan index recovery
- **Betroffene Komponenten:** `contextra-engine` (`src/collection/crud/maintenance.rs`, `src/collection/crud/db_transaction_commit.rs`)
- **Spezifizierte Funktionen & Features:**
  - **`CompensateGraphAction` in 2PC Ledger:** Erfassung von Graph-Kanten-Aktionen im 2PC Rollback-Protokoll.
  - **`Collection::repair()` Verwaiste Index-Kompensation:** Automatische Erkennung und Entfernung verwaister Einträge in HNSW, BM25 und Graph CSR, falls Datensätze im Storage fehlen.
  - **Crash-Window Integration Tests:** Abdeckung der Crash-Punkte (a) bis (d).
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Erzeuge künstliche verwaiste HNSW-/BM25-Einträge und starte `Collection::repair()`. Verifiziere vollständige Bereinigung.

---

### Commit 18: `376b513` — feat(engine): recovery for crash windows and commit-uncertain transactions
- **Betroffene Komponenten:** `contextra-engine/tests/engine_recovery_crash_windows.rs`
- **Spezifizierte Funktionen & Features:**
  - **Prüfsuite für unentschiedene Transaktionen:** Simulation von Systemabstürzen direkt während des `CommitIntent::Pending`-Zustands.
  - **Frühzeitige LSM-Kompensations-Registrierung:** Registrierung von `CompensateLsmAction` zu Beginn der Commit-Phase.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe `cargo test -p contextra-engine --test engine_recovery_crash_windows` aus.

---

### Commit 19: `d152aae` — feat(engine): add tenant collection handle caching and policy enforcement (E-01)
- **Betroffene Komponenten:** `contextra-engine` (`src/contextra.rs`, `src/config.rs`)
- **Spezifizierte Funktionen & Features:**
  - **Mandanten-Handle-Caching:** Thread-sichere Wiederverwendung von Collection-Handles in `RwLock<TenantCollectionMap>` via Double-Checked Locking per `(TenantId, String)`.
  - **`TenantPolicy` Enforcement:** Konfigurierbarer Modus (`TenantPolicy::Optional` vs. `TenantPolicy::Required`). Bei `Required` führt un-mandanteneinflussreicher Zugriff sofort zu `ContextraError::InvalidInput`.
  - **Automatische Handle-Eviction:** Sichere Entfernung aus dem Cache bei `drop_collection` und `wait_shutdown`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Setze `TenantPolicy::Required` und verifiziere die Blockade unangemeldeter Aufrufe.

---

### Commit 20: `1042d44` — docs: Add comprehensive features specification document
- **Betroffene Komponenten:** `docs/FEATURES_SPECIFICATION.md`, `docs/SOURCE_OF_TRUTH.md`, `docs/spec/INDEX.md`
- **Spezifizierte Funktionen & Features:**
  - **Gesamtsystem-Feature-Katalog:** Vollständige Spezifikation aller sichtbaren, opt-in und versteckten Funktionen über 33 Crates.
  - **Bugfixes in Harness & Quellcode:** Behebung von TOCTOU-Glitches in `put_if_absent` und Kaskaden-Invalidierungs-Tests.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Gleiche `docs/FEATURES_SPECIFICATION.md` mit dem vorliegenden Commit-Katalog ab.

---

### Commit 21: `05c66a7` — fix(store): ensure group commit WAL chain security and anchor rollback (S-02)
- **Betroffene Komponenten:** `contextra-store` (`src/wal/group_commit.rs`)
- **Spezifizierte Funktionen & Features:**
  - **Group Commit HMAC-Ketten-Sicherheit:** Lückenlose Validierung der HMAC-Verkettung bei Batch-Writes mehrerer Threads.
  - **Anker-Rollback-Mechanismus:** Zurücksetzen des WAL-Schreibzeigers auf den letzten gesicherten Anker bei Fehlschlagen eines Gruppen-Flushes.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Injiziere I/O-Fehler während eines Group-Commits und verifiziere die Integrität der verbleibenden WAL-Kette.

---

### Commit 22: `783a211` — feat(store): phase out legacy WAL key and unblock observer runtime (S-03)
- **Betroffene Komponenten:** `contextra-store` (`src/wal/observer.rs`, `src/wal/mod.rs`)
- **Spezifizierte Funktionen & Features:**
  - **Phase-Out des Legacy WAL Keys:** Ausmusterung alter Obfuscation-Schlüsselstrukturen.
  - **`WalObserver` Laufzeit-Entkopplung:** Synchrones Aufrufen von `on_commit`-Callbacks mit `catch_unwind`-Kapselung. Panikbehaftete Observer werden automatisch ohne Transaktionsabbruch evakuiert.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Registriere einen fehlerhaften/panikenden `WalObserver` und verifiziere, dass Transaktionen erfolgreich durchlaufen und der Observer isoliert/evakuiert wird.

---

### Commit 23: `30c6e3a` — feat(graph): wire APPRH gate selector into PPR path (G-01)
- **Betroffene Komponenten:** `contextra-graph` (`src/apprh/selector.rs`, `src/ppr.rs`), `docs/spec-sync/G-01.md`
- **Spezifizierte Funktionen & Features:**
  - **`ApprhSelector` Modi:** `Off`, `Shadow` und `Production`.
  - **Hysterese-Promotion/Demotion:** Dynamische, schwellwertbasierte Herabstufung in den Shadow-Modus bei Qualitätsabfall.
  - **`forward_push_apprh` Algorithmus (Spec Y.1.2):** Panikfreie Implementierung mit deterministischer IEEE-754 Aufsummierung via Sortierung nach `EntityId`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Teste Sortier-Determinismus bei Fließkomma-Summierung über k-Knoten-Hyperkanten.

---

### Commit 24: `c3c6ce1` — refactor(engine): Reranker-Port statt direkter infer-onnx-Abhängigkeit
- **Betroffene Komponenten:** `contextra-ports`, `contextra-types`, `contextra-infer-onnx`, `contextra-engine`, `contextra-db`
- **Spezifizierte Funktionen & Features:**
  - **`Reranker` Trait in Ring 0 (`contextra-ports`):** Abstrakte Definition für Cross-Encoder Reranking.
  - **Entkopplung von `contextra-engine`:** Injektion via `Option<&dyn Reranker>`, vollständige Entfernung direkter `infer-onnx`-Abhängigkeiten aus Ring 3.
  - **`MockReranker` für Unit-Tests:** Leichtgewichtiger Mock-Provider für deterministiches Testing.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Verifiziere das Fehlen von `contextra-infer-onnx` im Dependency-Graph von `contextra-engine`.

---

### Commit 25: `854eb11` — Harden architecture gates, enforce Ring 0/3 ordering, and fix CI gates
- **Betroffene Komponenten:** `xtask/src/check_ring_layering.rs`, `.github/workflows/rust-ci.yml`
- **Spezifizierte Funktionen & Features:**
  - **Strikte Ring-Schichtung:** Automatische CI-Sperre bei zyklischen Abhängigkeiten oder Verkäufen höherer Ringe an niedere Ringe.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe `cargo run --manifest-path xtask/Cargo.toml -- check-ring-layering` aus.

---

### Commit 26: `7707512` — refactor(xtask): add Cargo.toml ring metadata and gen-arch-docs automation
- **Betroffene Komponenten:** `Cargo.toml` aller Crates, `xtask/src/gen_arch_docs.rs`, `ARCHITECTURE.md`
- **Spezifizierte Funktionen & Features:**
  - **Ring-Metadaten im Manifest:** `[package.metadata.contextra] ring = "..."` für alle Crates.
  - **`xtask gen-arch-docs` Automation:** Dynamische Generierung der Crate-Inventar-Tabelle und Mermaid-Architektur-Diagramme in `ARCHITECTURE.md`.
  - **`TextEmbeddingEngine` Umzug:** Platzierung in `contextra-ports/src/embedding.rs`.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe `cargo run --manifest-path xtask/Cargo.toml -- gen-arch-docs --check` aus.

---

### Commit 27: `0c05830` — docs: Add license binding decision document (Phase 1) and fix feature wiring
- **Betroffene Komponenten:** `docs/decisions/`, `contextra-license`
- **Spezifizierte Funktionen & Features:**
  - **Lizenzbindungs-Architekturentscheidung:** Spezifikation der kryptographischen Lizenzüberprüfung für Enterprise-Ringe.
  - **Feature-Wiring Korrektur:** Abstimmung der Feature-Flags zwischen `contextra-license` und der Facade.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Prüfe `SignedLicenseGate` mit abgelaufener und gültiger Signatur.

---

### Commit 28: `f22ea4a` — refactor(docid-128): unify docid-128 feature in contextra-types and add CI gate
- **Betroffene Komponenten:** `contextra-types`, `workspace Cargo.toml`, `.github/workflows/rust-ci.yml`
- **Spezifizierte Funktionen & Features:**
  - **Zentralisiertes `docid-128` Feature:** Durchschleifen des Features über `contextra-types/docid-128` für 128-Bit Dokument-IDs.
  - **`docid-128-check` CI Gate:** Dedizierter CI-Prüfschritt zur Sicherstellung der Kompilierbarkeit unter 128-Bit Dokument-IDs.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe `cargo check --workspace --features docid-128` aus.

---

### Commit 29: `8498430` — feat(contextra-engine): make signal fusion strategy configurable in hybrid search
- **Betroffene Komponenten:** `contextra-engine` (`src/collection/search.rs`, `src/query/builder.rs`)
- **Spezifizierte Funktionen & Features:**
  - **Konfigurierbare `FusionStrategy`:** Unterstützung von `Rrf` (Reciprocal Rank Fusion, Default), `ScoreNormalized` und `ConstantScoreFallback`.
  - **Post-Fusion Community Boosting (`apply_community_boost_post_fusion`):** Dynamische Aufwertung von GraphRAG-Community-Ergebnissen nach der Signal-Fusion.
  - **100-Run Determinisierungs-Garantie:** Exakte Wiederholbarkeit von Hybrid-Suchergebnissen.
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe 100 identische Hybrid-Suchen durch und verifiziere Byte-für-Byte Identität der Score-Reihenfolge.

---

### Commit 30: `b81e14a` — test(contextra-store): verify F-10, F-05/06/08, F-07 store scenarios and fix SSTable v3/v4 stream trailer parsing (#3954)
- **Betroffene Komponenten:** `contextra-store` (`src/lsm/sstable.rs`, `src/lsm/stream.rs`)
- **Spezifizierte Funktionen & Features:**
  - **SSTable v3/v4 Stream Trailer Parsing Fix:** Behebung von Trailer-Offset-Berechnungsfehlern beim Lesen von SSTable-Streams.
  - **Loom Concurrency Kompatibilität:** Absicherung des Concurrent Read Paths unter Deterministic Loom Testing.
  - **Szenarien-Verifikation:**
    - `F-10`: Mandanten-Storage-Isolation
    - `F-05/06/08`: WAL Replay / Recovery Abdeckung
    - `F-07`: Stream-Trailer-Leserobustheit
- **Prüfkriterien für den Qualitätsprüfer:**
  - [ ] Führe `cargo test -p contextra-store --test sstable_stream` unter Loom aus.

---

## 3. Thematische Zuordnung allerpezifizierten Funktionen nach Architektur-Ringen

### 3.1 Ring 0 — Primitives, Math, Crypto, MVCC, Ports
1. **MVCC SSI Write-Skew Protection (`contextra-mvcc`):**
   - `SsiReadSet` mit Max-Key Bound (100.000).
   - `SsiValidatorImpl` mit `WriteSkewDetected`-Meldung.
   - Range-Key Phantom Protection.
2. **Kryptographisches Crypto-Shredding & DeletionProof (`contextra-crypto`):**
   - HKDF-SHA256 Subkey-Derivation & O(1) Key Revocation in `KeyRegistry`.
   - Ed25519 External Verification (`DeletionProof::verify_external`) für auditor-unabhängige Löschnachweise.
3. **Typensystem & Fast-Ports (`contextra-types`, `contextra-ports`):**
   - Unifiziertes `docid-128` Feature.
   - Decoupled `Reranker` & `TextEmbeddingEngine` Port Traits.

### 3.2 Ring 1 — Storage, Checkpoint & KV-Cache
1. **Durability & WAL Integrity (`contextra-store`):**
   - Explicit `DurabilityMode` Enum (`WalNoHmac` mit CRC32).
   - Atomic WAL Migration mit dualem `fsync` und `.rekeyed`-Marker-Garantie.
   - Group Commit WAL Chain HMAC Protection & Anchor Rollback.
   - SSTable v3/v4 Stream Trailer Parsing.
2. **Tenant-Isolated KV-Cache (`contextra-kvcache`):**
   - `TenantPrefixKvStore` mit `PrefixRadixTree` und per-Tenant LRU Byte Budgeting.
   - `AttentionExporter`-gewichte Eviction.

### 3.3 Ring 2 — Privacy, Sandbox & Inference Providers
1. **WASI Preview 1 Standalone Boundary (`contextra-sandbox`):**
   - Standalone WASI Host Boundary in `wasi.rs` ohne `wasmtime-wasi`.
   - Stream Limits (1 MiB) und PRNG Seed Injection.
2. **In-Process Inference & Candle Bridge (`contextra-infer-candle`):**
   - Candle GGUF Inference Engine mit Metal/CUDA Support.
   - Prefill-Attention Weight Adapter für Ring-0 Eviction.

### 3.4 Ring 3 — Engine, Graph, Router & Agent
1. **Multi-Index Transaction Commit & Recovery (`contextra-engine`):**
   - 2PC Commit Protocol mit `CommitIntent` Log und `CommitLedger` Compensation.
   - `Collection::repair()` zur Bereinigung verwaister Indexe bei fehlenden Storage-Records.
2. **Hybrid Search Signal Fusion & Community Boost (`contextra-engine`):**
   - Configurable `FusionStrategy` (`Rrf`, `ScoreNormalized`, `ConstantScoreFallback`).
   - Post-Fusion Community Boost für GraphRAG.
3. **Graph APPRH Shadow Gate (`contextra-graph`):**
   - `ApprhSelector` (`Off`, `Shadow`, `Production`) mit Hysterese-Promotion/Demotion.
   - `ApprhGateMonitor` für Schatten-Vergleiche gegen PPR.
4. **Agentic & Contextual Bandit Routing (`contextra-router`, `contextra-adapt`):**
   - LinUCB Sherman-Morrison $O(d^2)$ Updates.
   - Flow-Corrected Thompson Sampling.

---

## 4. Prüfplan & Akzeptanzkriterien für die Qualitätsprüfung

| Prüffeld | Befehl / Testpfad | Invariante / Akzeptanzkriterium |
| :--- | :--- | :--- |
| **Ring-Schichtung** | `cargo run --manifest-path xtask/Cargo.toml -- check-ring-layering` | Keinerlei zyklische oder abwärtsgerichtete Abhängigkeiten zwischen Ring 0..4. |
| **SSTable Stream Parsing** | `cargo test -p contextra-store --lib lsm::stream` | Fehlerfreies Lesen von v3/v4 Stream-Trailern ohne Offset-Panics. |
| **2PC Crash Safety** | `cargo test -p contextra-engine --test engine_recovery_crash_windows` | 100% konsistente Rollbacks verwaister Indexe nach unvollständigen Commits. |
| **SSI Write-Skew Interception** | `cargo test -p contextra-mvcc --lib ssi` | Erzeugung von `WriteSkewDetected` bei überlappenden Concurrent Commits. |
| **Hybrid Search Determinismus**| `cargo test -p contextra-engine --test hybrid_search` | Identisches Ergebnis über 100 aufeinanderfolgende Suchläufe. |
| **WASI Sandbox Limits** | `cargo test -p contextra-sandbox` | Strikter Abbruch mit `OutputLimitExceededError` bei Überschreiten von 1 MiB. |
| **Merge Verdict Enforcement** | `cargo run --manifest-path xtask/Cargo.toml -- verdict` | Verweigerung des Merge-Status, wenn relevante Gates nicht bestanden wurden. |

---
**Ende des Dokuments**
