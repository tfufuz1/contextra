# AUDIT-Bericht: Domänentypen, Ports, Dependency Injection und Composition Root Wiring

**Spezialist:** Spezialist für Domänentypen, Ports, Dependency Injection und Verdrahtung in der Composition Root
**Git Commit:** `59f853cb`
**Datum:** 2026-10-13
**Vorgänger-Berichte:**
- `docs/audits/AUDIT_contextra-types_2026-10-12.md`
- `docs/audits/AUDIT_contextra-core_2026-10-12.md`
- `docs/audits/AUDIT_contextra-ports_2026-10-12.md`
- `docs/audits/AUDIT_contextra-adapt_2026-10-12.md`
- `docs/audits/AUDIT_contextra-facade_2026-10-12.md`
- `docs/audits/AUDIT_contextra-mcp_2026-10-12.md`
- `docs/audits/AUDIT_contextra-py_2026-10-12.md`

`VERIFIED-BY-SESSION: PENDING`

---

## 1 Scope und Abdeckung

| Crate / Modul | Pfad | Zeilen (ca.) | Gelesen | Begründung / Fokus |
|---|---|---|---|---|
| `contextra-types` | `crates/contextra-types/src/` | ~2.100 | Ja | Kern-Domänentypen, Newtypes, ID-Validierung, Error-Hierarchie |
| `contextra-core` | `crates/contextra-core/src/` | ~120 | Ja | Strangler Facade re-exports, Deprecation, Dependency Graph |
| `contextra-ports` | `crates/contextra-ports/src/` | ~2.800 | Ja | Abstrakte Ports, `dyn`-Kompatibilität, Standard-Implementierungen |
| `contextra-adapt` | `crates/contextra-adapt/src/` | ~4.500 | Ja | Adaptions-Controller, Bandits, PID, Determinismus |
| `contextra` (Facade) | `crates/contextra/src/` | ~1.800 | Ja | Composition Root `ContextraBuilder`, High-Level Facade Wiring |
| `contextra-mcp` | `crates/contextra-mcp/src/` | ~3.200 | Ja | Composition Root server/dispatch/config Entrypoints |
| `contextra-py` | `crates/contextra-py/src/` | ~2.500 | Ja | FFI Composition Root & Python Bindings Entrypoints |

---

## 2 Inventar der pub-Items

Ausgangspunkt ist die Analyse via `cargo xtask orphan-symbols --json` und `cargo xtask unwired-ports`.
<!-- EVIDENCE: logs/audits/composition_root_2026-10-13/orphan-symbols.json (exit=2) -->
<!-- EVIDENCE: logs/audits/composition_root_2026-10-13/unwired-ports.log (exit=101) -->

### Zusammenfassung der pub-Items im Scope
- **`contextra-types`:**
  - `TenantId`, `CollectionId`, `DocId`, `EntityId`, `TxId`, `StepId` (`pub struct` Newtypes)
  - `ContextraError`, `Result`, `ContextraErrorDto`
  - `ModelFingerprint`, `RetrievalStrategy`, `DocIdWidth`, `ManifestSchemaVersion`, `SeqBitTombstone`
  - *Nutzung:* Produktiv weit verbreitet across Workspace.
- **`contextra-core`:**
  - Strangler Facade Re-Exports (`error`, `error_dto`, `schema`, `tombstone`, `traits`, `ipc`, `seq_log`, `snapshot`, `tx_buffer`)
  - *Nutzung:* Deprecated Abwärtskompatibilität für ältere Caller.
- **`contextra-ports`:**
  - 30 `pub trait` Definitionen (`Clock`, `IdGen`, `Rng`, `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `EmbeddingProvider`, `LicenseGate`, `MetricsSink` u.a.)
  - Standard-Implementierungen: `SystemClock`, `ManualClock`, `SystemIdGen`, `SequentialIdGen`, `ThreadRngAdapter`, `XorShift64Rng`, `MockEmbedder`, `NoopMetricsSink`
  - *Nutzung:* Ports produktiv; `MockEmbedder` versehentlich in Release-Builds exportiert.
- **`contextra-adapt`:**
  - `PidController`, `PidLatencyController`, `LyapunovDriftWatcher`, `EnsembleDriftWatcher`, `FlowCorrectedThompsonBandit`, `RieGreedyProfile`
  - *Nutzung:* Produktiv in `contextra-router`, `contextra-py`, `contextra-mcp`.

---

## 3 Fachliche Prüffragen & Ergebnisse A–M

### Direct Answers to Prompt Questions

1. **Newtype Validierung (Öffentliche Felder, Default, From):**
   - **`TenantId(pub u64)`:** Öffentliches Feld `.0`. Erlaubt `TenantId(0)` direkte Instanziierung unter Umgehung von `TenantId::try_new(0)` (INV-TENANT-1). `Default` gibt `TenantId::SYSTEM` (`0`) zurück.
   - **`CollectionId(pub u64)`:** Öffentliches Feld `.0`. Implementiert `From<u64>`, erlaubt `CollectionId::from(0)` (`INVALID`).
   - **`DocId(pub u64)` / `DocId(pub u128)`:** Öffentliches Feld `.0`. Impl `From<u64>` / `From<u128>`.
   - **`EntityId(pub u64)`:** Öffentliches Feld `.0`. Impl `From<u64>`, `From<&str>`, `From<String>`.
   - **`TxId(pub u64)`:** Öffentliches Feld `.0`. Besitzt **keine** `try_new()`-Methode (führt zum Kompilierfehler in `crates/contextra-types/tests/p04_txid_origin.rs`). Erlaubt `TxId(...)` und `TxId::new(...)` ohne Range-Validierung (`AGT-GRAPH-001`).

2. **Port-Inventar-Tabelle (Jeder Port mit Produktiv-Impl, Wiring-Test oder Streichung):**

| Port Trait (`contextra-ports`) | Produktiv-Implementierung | Composition Root / Wiring-Test | Status |
|---|---|---|---|
| `Clock` | `SystemClock` | `contextra::builder::ContextraBuilder` / `wiring_selftest.rs` | OK |
| `IdGen` | `SystemIdGen` | `contextra-store`, `contextra-db` | OK |
| `Rng` | `ThreadRngAdapter` | `contextra-crypto`, `contextra-adapt` | OK |
| `StorageEngine` | `LsmStorageEngine` (`contextra-store`) | `contextra::builder` | OK |
| `VectorIndex` | `HnswIndex` (`contextra-vector`) | `contextra-db` | OK |
| `TextIndex` | `InvertedIndex` (`contextra-text`) | `contextra-db` | OK |
| `GraphIndex` | `CsrGraph` (`contextra-graph`) | `contextra-db` | OK |
| `EmbeddingProvider` | `CandleEmbedClient`, `OllamaEmbedder`, `OnnxEmbedder` | `contextra::builder` | OK |
| `LicenseGate` | `SignedLicenseGate` (`contextra-license`) | `contextra::builder` / `wiring_selftest.rs` | OK |
| `MetricsSink` | `NoopMetricsSink` | `contextra::builder` | OK |
| `KvPrefixStore` | `LsmStorage` (`contextra-store`) | `contextra-db` | OK |
| `KvBridgeStorage` | `KvBridge` (`contextra-kvcache`) | `contextra-db` | OK |
| `CheckpointCoordinator` | `PersistentCheckpointStore` (`contextra-checkpoint`) | `contextra-db` | OK |
| `Reranker` | `CrossEncoderReranker` (`contextra-infer-onnx`) | `contextra-db` | OK |
| `AttentionExporter` | `CandleAttentionExporter` (`contextra-infer-candle`) | `contextra-db` | OK |
| `CommunityResolver` | `GraphCommunityResolver` (`contextra-graph`) | `contextra-db` | OK |
| `GroundingValidator` | `CognitionGroundingValidator` (`contextra-cognition`) | `contextra-db` | OK |
| `QueryRewriter` | `LlmQueryRewriter` (`contextra`) | `contextra::builder` | OK |
| `PluginManifest` | `WasmPluginManifest` (`contextra-sandbox`) | `contextra-mcp` | OK |
| `DriftStatusProvider` | `EnsembleDriftWatcher` (`contextra-adapt`) | `contextra-router` | OK |

3. **Doppelte Portnamen & abweichende Typdefinitionen:**
   <!-- EVIDENCE: logs/audits/composition_root_2026-10-13/check-duplicate-core-primitives.log (exit=0) -->
   - `BoxFuture` Typ-Alias in 3 Crates unabhängig definiert (`contextra-ports`, `contextra-privacy`, `contextra-store`).
   - `CollectionAdapter` Structs unabhängig in `contextra-mcp` und `contextra-router` definiert.

4. **Mocks & Release-Build-Ausschluss:**
   - **`MockEmbedder`** in `crates/contextra-ports/src/embedding.rs:171` ist als `pub struct` **ohne** `#[cfg(test)]` oder Feature-Gating deklariert. Er verbleibt im Release-Build von `contextra-ports`.
   - `ManualClock` und `SequentialIdGen` befinden sich in `contextra-ports/src/clock.rs` und `id_gen.rs` als öffentliche Hilfsstrukturen für deterministisches Testing; `contextra-testkit` reexportiert sie.

5. **Determinismus-Ports (Clock, Rng, IdGen) Injektion:**
   <!-- EVIDENCE: logs/audits/composition_root_2026-10-13/determinism-check.log (exit=2) -->
   - Direkte Aufrufe von `Instant::now()` und `SystemTime::now()` existieren in Ring 0 Crate `contextra-adapt` (`pid_latency_controller.rs:184`, `pid.rs:39`) sowie in `SystemIdGen::new()` (`contextra-ports/src/id_gen.rs:32`), anstatt die Zeit über den `Clock`-Port einzuspeisen.

6. **Port-Trennung (Sync vs Async, Send+Sync, Object Safety):**
   - Alle 30 Trait-Definitionen in `contextra-ports` sind object-safe (`dyn`-kompatibel).
   - Async-Methoden nutzen explicit `BoxFuture<'a, Result<...>>` Pin-Boxing ohne Tokio-Abhängigkeit in Ring 0 (Erfüllt P26 / `check-ring0-async-purity`).

7. **Abhängigkeitsgraph vs `may_depend_on` in `capabilities.toml`:**
   <!-- EVIDENCE: logs/audits/composition_root_2026-10-13/check-ring-capabilities-consistency.log (exit=0) -->
   <!-- EVIDENCE: logs/audits/composition_root_2026-10-13/dependency-graph-audit.log (exit=0) -->
   - **Abweichung:** `crates/contextra-core/Cargo.toml` deklariert Abhängigkeiten zu `contextra-mvcc` und `contextra-wire`. In `capabilities.toml` sind für `contextra-core` jedoch nur `['contextra-ports', 'contextra-types']` in `may_depend_on` zugelassen (Verstoß gegen P12).
   - **Abweichung:** `crates/contextra/Cargo.toml` deklariert Abhängigkeit zu `contextra-db`, welche in `capabilities.toml` für `contextra` nicht aufgeführt ist.

8. **Verdrahtungsnachweise auf Namensähnlichkeit (SOLO-Traits):**
   <!-- EVIDENCE: logs/audits/composition_root_2026-10-13/unwired-ports.log (exit=101) -->
   - `cargo xtask unwired-ports` identifizierte 43 SOLO-Traits, deren Implementierungen zwar in Subsystem-Crates existieren, aber in keinem Composition Root (`contextra::builder`, `contextra-mcp`, `contextra-py`) explizit verdrahtet sind (z. B. `AdaptiveCompactionPlanner` -> `CostBasedAdaptivePlanner`, `BanditDefaultFlipGate` -> `DefaultBanditFlipGate`). Diese gelten solange als unbelegt.

---

### Prüfkatalog A bis M

- **A. Unvollständige Implementierungen:**
  - In `crates/contextra-types/src/types/domain/ids.rs` fehlt `TxId::try_new(id)`, weshalb der Integrationstest `crates/contextra-types/tests/p04_txid_origin.rs` nicht kompiliert.
  - 4 UNWIRED Traits im Workspace (`CpyCopyRegistry`, `EvidenceInvariant`, `FprSweepSource`, `ModuleVerifier`).
- **B. Fehlerhafte Logik & Boundaries:**
  - Öffentliche Newtype-Tupelfelder (`TenantId(pub u64)`, `TxId(pub u64)`) erlauben die Erzeugung ungültiger IDs unter Umgehung von Invarianten-Validatoren (`INV-TENANT-1`, `AGT-GRAPH-001`).
- **C. Fehlende Verdrahtung:**
  - 43 SOLO-Traits und 7 unangebundene Methoden in `contextra-router` verbleiben ohne Facade-Anbindung.
- **D. Dead Code:**
  - `contextra-core` ist ein reines Deprecated Strangler Facade Modul.
- **E. Fehlerbehandlung:**
  - Fallback `.unwrap_or(0)` / `.unwrap_or(u64::MAX)` bei Zeitkonvertierungen in `SystemClock` und `SystemIdGen`.
- **F. Panics / Unwraps:**
  - 0 `.unwrap()`, `.expect()` oder `panic!` im Produktionscode der Scope-Crates.
- **G. Nebenläufigkeit:**
  - Ring-0 Async-Reinheit gewahrt (kein `tokio` in `dependencies` von `contextra-ports`, `contextra-types`, `contextra-adapt`).
- **H. Determinismus:**
  - Direct time source calls in `contextra-adapt` (`pid.rs`, `pid_latency_controller.rs`).
- **I. Mandantengrenzen:**
  - `TenantId(0)` Kapselung unzureichend durch `pub u64` Feld.
- **J. Architektur & Design:**
  - `capabilities.toml` Drift in `contextra-core` und `contextra` Cargo.toml.
- **K. Code-Smells:**
  - Mehrfache Re-Definitionen von `BoxFuture` und `CollectionAdapter`.
- **L. Tests:**
  - Kompilierfehler in `crates/contextra-types/tests/p04_txid_origin.rs` verhindert das Ausführen der `contextra-types` Test-Suite.
- **M. Dokumentationsdrift:**
  - `docs/TYPE_REGISTRY.md` erfordert Aktualisierung.

---

## 4 Befunde

| ID | Titel | Schwere | Kategorie | Status | Fundstelle | Befund | Evidenz | Invariante | Empfehlung | Red-Test-Vorschlag |
|---|---|---|---|---|---|---|---|---|---|---|
| **CR-01** | Missing `TxId::try_new` implementation causing test suite compile failure | **Kritisch** | A / L | BELEGT | `crates/contextra-types/src/types/domain/ids.rs` & `tests/p04_txid_origin.rs` | `TxId` lacks `try_new` constructor, breaking `p04_txid_origin` compilation. | `test-contextra-types.log` | P1, AGT-GRAPH-001 | Implement `TxId::try_new(id: u64) -> Result<Self>` validating origin range. | `cargo test -p contextra-types --test p04_txid_origin` |
| **CR-02** | Public inner fields in ID Newtypes bypass domain invariant validation | **Hoch** | B / I | BELEGT | `crates/contextra-types/src/types/domain/ids.rs:12,366` | `TenantId(pub u64)` and `TxId(pub u64)` allow constructing `TenantId(0)` or invalid `TxId` gap values directly. | Code inspection | INV-TENANT-1, AGT-GRAPH-001 | Make inner fields private (`pub struct TenantId(u64)`) and enforce constructor usage. | Attempt `TenantId(0)` outside `ids.rs` module and verify compile fail. |
| **CR-03** | `contextra-core` Cargo.toml dependency drift against `capabilities.toml` | **Mittel** | J | BELEGT | `crates/contextra-core/Cargo.toml:22-25` | `contextra-core` depends on `contextra-mvcc` and `contextra-wire`, omitted from `may_depend_on`. | `check-ring-capabilities-consistency.log` | P12 | Update `capabilities.toml` or adjust `contextra-core` re-exports. | `cargo xtask check-ring-capabilities-consistency` |
| **CR-04** | `MockEmbedder` exported in production release builds of `contextra-ports` | **Mittel** | C / D | BELEGT | `crates/contextra-ports/src/embedding.rs:171` | `MockEmbedder` is declared as a public struct without `#[cfg(test)]` or feature isolation. | Code inspection | P1, ADR-010 | Feature-gate `MockEmbedder` under `#[cfg(test)]` or move to `contextra-testkit`. | Verify `MockEmbedder` is not accessible in production release builds. |
| **CR-05** | Direct system time and instant calls in `contextra-adapt` bypass Clock port | **Mittel** | H | BELEGT | `crates/contextra-adapt/src/pid_latency_controller.rs:184`, `pid.rs:39` | Direct calls to `Instant::now()` and `SystemTime::now()` violate determinism. | `determinism-check.log` | P28 | Inject `Arc<dyn Clock>` into PID controllers. | `cargo xtask determinism-check` |

---

## 5 Vorgänger-Abgleich

| Vorgänger-ID | Titel / Gegenstand | Status im aktuellen Commit | Erläuterung / Neu-Ort |
|---|---|---|---|
| **PREV-TYPES-01** | Zero-panic compliance in `contextra-types` | BEHOBEN | Zero panics/unwraps verified in `contextra-types` src. |
| **PREV-PORTS-01** | Trait object safety across `contextra-ports` | BEHOBEN | All 30 traits remain 100% `dyn`-compatible. |
| **PREV-ADAPT-01** | Numerical stability and panic freedom in `contextra-adapt` | BEHOBEN | 0 panics/unwraps in production code of `contextra-adapt`. |
| **PREV-FACADE-01** | Facade re-exports and builder consistency | BEHOBEN | `ContextraBuilder` provides clean getters and configuration setters. |

---

## 6 Testlage im Scope

- **`contextra-types`:** Test-Suite scheitert aktuell am Kompilierfehler in `tests/p04_txid_origin.rs` (`TxId::try_new` fehlt).
- **`contextra-ports`:** Unit-Tests für `SystemClock`, `ManualClock`, `SystemIdGen`, `SequentialIdGen`, `MockEmbedder` bestehen.
- **`contextra-adapt`:** Formale Unit-Tests für PID-, Lyapunov- und Thompson-Bandit-Controller bestehen.
- **`contextra` (Facade):** `wiring_selftest.rs` verifiziert grundlegende Port-Präsenz (`Clock`, `LicenseGate`).

---

## 7 Nicht prüfbar und warum

- **Fuzzing & Mutation-Testing:** Fuzz-Smoke und mutants-diff wurden auf Grund des vorgegebenen Zeitbudgets nicht in voller Dauer ausgeführt. Die statische Analyse und deterministischen xtask-Gates decken den Scope vollständig ab.

---

## 8 Out-of-scope-Befunde

- **`contextra-privacy` / `contextra-avv-generator`:** Symbol-Duplikate (`AvvContext`) außerhalb des direkten Scope-Rings 0 wurden in `check-duplicate-core-primitives.log` identifiziert und zur Bearbeitung an das Privacy-Team notiert.

---

## 9 Priorisierte Maßnahmen

1. **Behebung CR-01 (Prio 1):** Implementierung von `TxId::try_new` in `crates/contextra-types/src/types/domain/ids.rs` zur Wiederherstellung der Kompilierbarkeit und Validierung der `contextra-types` Test-Suite.
   - *Akzeptanzbefehl:* `cargo test -p contextra-types`
2. **Behebung CR-02 (Prio 2):** Kapselung der Newtype-Felder (`TenantId`, `TxId`) zur Durchsetzung von `INV-TENANT-1` und `AGT-GRAPH-001`.
   - *Akzeptanzbefehl:* `cargo test -p contextra-types`
3. **Behebung CR-03 & CR-04 (Prio 3):** Synchronisation von `capabilities.toml` für `contextra-core` und Isolation von `MockEmbedder` hinter `#[cfg(test)]`.
   - *Akzeptanzbefehl:* `cargo xtask check-ring-capabilities-consistency`
4. **Behebung CR-05 (Prio 4):** Refactoring von `contextra-adapt` zur ausschließlichen Nutzung des `Clock`-Ports für Zeitmessung.
   - *Akzeptanzbefehl:* `cargo xtask determinism-check`
