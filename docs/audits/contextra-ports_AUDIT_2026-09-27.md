# Audit-Bericht: `contextra-ports` (Ring 0 Ports & Adapters)

**Datum:** 2026-09-27
**Crate:** `contextra-ports` v0.1.0 (`crates/contextra-ports/src/`)
**Auditor:** Principal Senior Rust Architect (Jules)
**Klassifizierung:** Ring 0 Core Port-Abstraktionsschicht
**Sicherheitseinstellung:** `#![forbid(unsafe_code)]`

---

## Executive Summary

Der Crate `contextra-ports` dient als zentrale Entkopplungsschicht (Ports-and-Adapters-Muster) im Ring 0 der Contextra-Architektur. Er definiert Interfaces für Storage, Vektorindizes, Textindizes, Graphindizes, KV-Caches, Embedding-Provider, Zeit-/RNG-Quellen und das Lizenz-Gate.

Der vollständige Audit-Zyklus umfasst die Prüfpunkte **P1** (dyn-Kompatibilität), **P2** (Clock-Port-Vollständigkeit & Monotonie), **P3** (RNG-Compliance nach ADR-098/P28), **P4** (Trait-Vertragsdokumentation), **P5** (Lizenz-Gate Fail-Closed-Nachweis) und **P6** (Ring-Reinheit).

---

## (1) Trait-Inventar-Tabelle (P1 & P4)

Alle 33 Traits in `crates/contextra-ports/src/` wurden auf dyn-Kompatibilität, Implementoren-Count im Workspace und Vollständigkeit der Fehler- und Vertragsspezifikationen untersucht:

| Trait | dyn-kompatibel? | Implementoren-Count | Fehlerverhalten / Vertrags-Dokumentation | Modul |
| :--- | :---: | :---: | :--- | :--- |
| `AttentionExporter` | **Ja** | 0 (Ring 2 backend interface) | **Spezifiziert:** Gibt `Option<Vec<f32>>`, `None` bei fehlenden Weights. | `attention.rs` |
| `Checkpoint` | **Ja** | 0 (in `contextra-checkpoint`) | **Spezifiziert:** `Result<WorkflowState>`. | `checkpoint.rs` |
| `CheckpointCoordinator` | **Nein** (`type Meta`, AFIT) | 0 (in `contextra-checkpoint`) | **Spezifiziert:** Reintensioniert statischen Dispatch via `Meta`. | `checkpoint.rs` |
| `Snapshot` | **Ja** | 0 | **Spezifiziert:** Non-failing `seq_no(&self) -> u64`. | `checkpoint.rs` |
| `Clock` | **Ja** | 5 (`SystemClock`, `Arc<T>`, `TestClock`, `FixedClock`, `ManualClock`) | **Spezifiziert:** Non-failing, `now_unix_nanos` gibt `0` vor Epoch zurück. | `clock.rs` |
| `EmbeddingProvider` | **Ja** | 1 in ports (`MockEmbedder`) + ONNX/Ollama | **Spezifiziert:** Detaillierte `EmbeddingError`-Enum (`Unavailable`, `ComputationFailed`, `InputTooLong`). | `embedding.rs` |
| `TextGenerator` | **Ja** | 0 | **Spezifiziert:** `crate::Result<String>`. | `embedding.rs` |
| `LlmTextGenerator` | **Ja** | 1 in tests (`MockDefaultLlm`) | **Spezifiziert:** Default-Konkatenation für Context-Segments. | `embedding.rs` |
| `LlmTextGeneratorStreaming` | **Ja** | 0 | **Spezifiziert:** `BoxStream<'a, crate::Result<String>>`. | `embedding.rs` |
| `GraphCollectionMutation` | **Ja** | 1 in tests (`MockGraphMutation`) | **Spezifiziert:** Detaillierte `GraphMutationError`-Enum. | `graph.rs` |
| `GraphIndex` | **Ja** (`BoxFuture`) | 2 in tests + `CsrGraph` | **Teilweise:** TxId-Invariante (AGT-GRAPH-001) & `CapabilityUnsupported` dokumentiert; methodenlokale `# Errors`-Blöcke fehlen. | `graph_index.rs` |
| `CommunityResolver` | **Ja** | 0 | **Spezifiziert:** `Result<Option<u64>>`. | `graph_index.rs` |
| `IdGen` | **Ja** | 2 (`SequentialIdGen`, `Arc<T>`) | **Spezifiziert:** Atomic non-failing monotonic sequence. | `id_gen.rs` |
| `KvPrefixStore` | **Ja** | 1 in tests (`MockKvStore`) | **Spezifiziert:** `Option<KvPrefixHit>`, `Result<(), ContextraError>`. | `kv.rs` |
| `KvBridgeStorage` | **Ja** | 0 (`LsmStorage` in `contextra-store`) | **Spezifiziert:** `BoxFuture` vtable-Layout. | `kv_bridge_port.rs` |
| `LicenseGate` | **Ja** | 1 in tests (`TestGate`) + `SignedLicenseGate` | **Spezifiziert:** Strict `LicenseError`-Enum (`NotActivated`, `InvalidSignature`, `Expired`). | `license.rs` |
| `DistanceCalculator` | **Ja** | 1 (`DistanceMetric` impl) | **Spezifiziert:** `Result<f32>` / `Result<u32>`. | `lifecycle.rs` |
| `MemoryLifecycleManager` | **Nein** (AFIT) | 0 | **Spezifiziert:** `Result<LifecycleSweepReport>`, `Result<Vec<ConsolidationAction>>`. | `lifecycle.rs` |
| `GroundingValidator` | **Ja** | 0 | **Spezifiziert:** `Result<GroundingAssessment>`. | `lifecycle.rs` |
| `ResponseGroundingValidator` | **Ja** | 0 | **Spezifiziert:** `Result<f32>`. | `lifecycle.rs` |
| `ContextPreparer` | **Ja** | 0 | **Spezifiziert:** `Result<ContextWindow>`. | `lifecycle.rs` |
| `MetricsSink` | **Ja** | 2 (`NoopMetricsSink`, `MockMetricsSink`) | **Spezifiziert:** Discard/Record void methods. | `metrics.rs` |
| `DriftStatusProvider` | **Ja** | 0 | **Spezifiziert:** String-Status ("stabil", "warnung", "kritisch", "unbekannt"). | `observability.rs` |
| `PluginManifest` | **Ja** | 1 in tests (`SimplePlugin`) | **Spezifiziert:** Detailliertes `PluginError`-Enum. | `plugin.rs` |
| `Rng` | **Ja** | 2 (`SeededRng`, `Arc<T>`) | **Spezifiziert:** Lock-free SplitMix64 atomic implementation. | `rng.rs` |
| `StorageRead` | **Ja** | 1 in tests (`MockReadStore`) | **Teilweise:** Methodenlokale `# Errors`-Blöcke, Verhalten bei leeren Mengen und `k==0` nicht in rustdoc dokumentiert. | `storage.rs` |
| `StorageWrite` | **Ja** (`BoxFuture`) | 0 | **Spezifiziert:** Transaktionale `BoxFuture`-Rückgaben. | `storage.rs` |
| `StorageEngine` | **Ja** (`BoxFuture`) | 0 | **Spezifiziert:** Backward-compatibility facade. | `storage.rs` |
| `TextEmbeddingEngine` | **Ja** | 1 blanket impl für `T: EmbeddingProvider` | **Spezifiziert:** `Result<Vec<f32>>`. | `text_index.rs` |
| `SegmentSynthesizer` | **Ja** | 0 | **Spezifiziert:** `Result<String>`. | `text_index.rs` |
| `TextIndex` | **Nein** (AFIT) | 2 in tests (`TextIndexPlaceholder`, `MockTextIndex`) | **Teilweise:** `CapabilityUnsupported` dokumentiert; methodenlokale `# Errors` & `k==0`-Dokumentation fehlen. | `text_index.rs` |
| `VectorIndex` | **Nein** (AFIT) | 2 in tests (`VectorIndexPlaceholder`, `MockIndex`) | **Teilweise:** `CapabilityUnsupported` dokumentiert; methodenlokale `# Errors`, `k==0` & leere Mengen fehlen. | `vector_index.rs` |
| `HybridSearchProvider` | **Ja** | 0 | **Spezifiziert:** `Result<Vec<ContextChunk>>`. | `vector_index.rs` |

### Detailerkenntnis zu P1 (dyn-Kompatibilität):
- **BoxFuture / Sync-Methoden Traits (29 Traits):** Vtable-kompatibel (`dyn Trait`), z.B. `Box<dyn StorageRead>`, `Box<dyn GraphIndex>`, `Box<dyn LicenseGate>`.
- **AFIT-Traits (4 Traits):** `VectorIndex`, `TextIndex`, `MemoryLifecycleManager`, `CheckpointCoordinator` nutzen native Async-in-Trait (`impl Future<Output = ...> + Send`) für Nativer-Performance-Static-Dispatch. Sie sind bewusst **nicht** dyn-kompatibel als `dyn VectorIndex` ohne Typ-Löschungs-Wrapper.

---

## (2) Clock/Rng-Compliance-Sektion (P2 & P3)

### P2: Clock-Port-Vollständigkeit & Monotonie
grep-Ergebnis für Implementoren des `Clock`-Traits:
```
crates/contextra-ports/src/clock.rs:39:impl Clock for SystemClock
crates/contextra-ports/src/clock.rs:54:impl<T: Clock + ?Sized> Clock for Arc<T>
crates/contextra-license/tests/signed_gate.rs:18:impl Clock for TestClock
crates/contextra-privacy/tests/audit_trace_tests.rs:20:impl Clock for FixedClock
crates/contextra-testkit/src/manual_clock.rs:58:impl Clock for ManualClock
```
- **Vorhandene Implementoren:**
  - `SystemClock` (Produktions-Default in `contextra-ports`)
  - `ManualClock` (Deterministischer Test-Clock in `contextra-testkit`)
  - `TestClock` / `FixedClock` (Testspezifische Mocks)
- **Monotonie-Analyse von `SystemClock`:**
  - `now_unix_nanos(&self)` verwendet `SystemTime::now().duration_since(UNIX_EPOCH)`. Wall-Clock-Zeit ist **NICHT** streng monoton, da NTP-Anpassungen, Schaltsekunden oder manuelle Systemzeitänderungen zu rückwärts springenden Zeitstempeln führen können.
  - `monotonic_nanos(&self)` verwendet `Instant::now().saturating_duration_since(self.start_instant)`. Diese Methode ist **streng monoton**.

### P3: RNG-Port-Compliance (ADR-098 / P28)
Vollständige grep-Ausgabe für `thread_rng()` und `OsRng` im Workspace:
```
crates/contextra-ports/src/rng.rs:26:/// Note that non-deterministic entropy seeding (e.g., via `OsRng` or system entropy)
crates/contextra-store/src/lsm/recovery.rs:105:            rand::thread_rng().fill(&mut buf);
crates/contextra-store/src/wal/hmac.rs:112:            rand::thread_rng().fill_bytes(&mut key);
crates/contextra-store/src/wal/hmac.rs:117:                rand::thread_rng().next_u64()
crates/contextra-store/src/wal/hmac.rs:215:                rand::thread_rng().next_u64()
crates/contextra-privacy/src/egress_vault.rs:61:        rand::rngs::OsRng.fill_bytes(&mut salt);
crates/contextra-privacy/src/audit_trace.rs:4:// Kein thread_rng(), kein Random-Nonce, Zero-Panic.
crates/contextra-mcp/src/sandbox.rs:116:        rand::thread_rng().fill_bytes(&mut salt);
crates/contextra-mcp/src/sandbox.rs:118:        rand::thread_rng().fill_bytes(&mut passphrase);
crates/contextra-vector/src/quantize/tests.rs:171:    let mut rng = rand::thread_rng();
crates/contextra-vector/src/hnsw/core_rebuild.rs:40:        let mut rng = rand::thread_rng();
crates/contextra-vector/benches/sq8_bench.rs:8:    let mut rng = rand::thread_rng();
crates/contextra-vector/benches/distance_bench.rs:10:    let mut rng = rand::thread_rng();
crates/contextra-vector/benches/hnsw_bench.rs:18:    let mut rng = rand::thread_rng();
crates/contextra-vector/benches/flush_threshold_amplification.rs:63:    let mut rng = rand::thread_rng();
crates/contextra-vector/benches/hnsw_delete_search_bench.rs:15:    let mut rng = rand::thread_rng();
crates/contextra-vector/benches/audit_benchmarks.rs:46:    let mut rng = rand::thread_rng();
```

**Kategorisierung und Audit-Bewertung:**
1. **Dokumentation / Kommentare (2 Treffer):** `rng.rs:26`, `audit_trace.rs:4` — Unbedenklich.
2. **Test- & Benchmark-Code (7 Treffer):** `quantize/tests.rs`, `sq8_bench.rs`, `distance_bench.rs`, `hnsw_bench.rs`, `flush_threshold_amplification.rs`, `hnsw_delete_search_bench.rs`, `audit_benchmarks.rs` — Unbedenklich.
3. **Produktions-Code P28/ADR-098 Abweichungen (7 Treffer):**
   - `contextra-store/src/lsm/recovery.rs:105`: Salt-Generierung bei LSM Recovery via `thread_rng()`.
   - `contextra-store/src/wal/hmac.rs:112, 117, 215`: HMAC key/nonce Generation via `thread_rng()`.
   - `contextra-privacy/src/egress_vault.rs:61`: Egress-Vault Salt via `OsRng`. (Gemäß ADR-098 Ausnahme nur in `contextra-crypto` erlaubt!).
   - `contextra-mcp/src/sandbox.rs:116, 118`: Sandbox Salt/Passphrase via `thread_rng()`.
   - `contextra-vector/src/hnsw/core_rebuild.rs:40`: Random Perturbation in core rebuild via `thread_rng()`.

*Empfehlung:* Diese 7 Produktionscode-Treffer sollten in Folgetickets auf injizierte `Rng`-Ports umgestellt werden, um vollständige Determinismus-Isolation (P28) außerhalb von `contextra-crypto` zu gewährleisten.

---

## (3) Lizenz-Gate Fail-Closed-Nachweis (P5)

Der Lizenz-Gate-Mechanismus ist in `crates/contextra-ports/src/license.rs` und `plugin.rs` verankert:

```rust
pub trait LicenseGate: Send + Sync {
    fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError>;
}
```

### Fail-Closed-Beweisführung:
1. **Typ-Vertrag:** `check_ring` gibt ein `Result<(), LicenseError>` zurück. Jede denkbare Fehlerbedingung (Netzwerkfehler, korrupte Ed25519-Signatur, abgelaufener Key, nicht aktivierter Ring) resultiert in `Err(LicenseError::...)`.
2. **Plugin-Aktivierungs-Gate (`PluginRegistry::activate_all`):**
   ```rust
   for (p_name, plugin) in &plugin_map {
       let required_ring = plugin.feature_ring_required();
       if self.gate.check_ring(required_ring).is_err() {
           return Err(PluginError::LicenseDenied(p_name.clone()));
       }
   }
   ```
   Bei *jedem* Fehler wird die Aktivierung des betreffenden Plugins unverzüglich abgelehnt (`PluginError::LicenseDenied`). Es existiert kein permissiver Fallback.
3. **Feature-Ring-Bestimmung (`PluginRegistry::current_feature_ring`):**
   ```rust
   pub fn current_feature_ring(&self) -> FeatureRing {
       if self.gate.check_ring(FeatureRing::Compliance).is_ok() {
           FeatureRing::Compliance
       } else if self.gate.check_ring(FeatureRing::Sovereign).is_ok() {
           FeatureRing::Sovereign
       } else {
           FeatureRing::Fast
       }
   }
   ```
   Schlägt die Prüfung für höhere Ringe (`Compliance`, `Sovereign`) fehl, fällt die Registrierung automatisch auf den restriktivsten Ring (`Fast` - Basis-Open-Source-Core) zurück.

**Ergebnis:** Das Lizenz-Gate verhält sich in allen Fehlerfällen **strictly Fail-Closed**.

---

## (4) Ring-Reinheit (P6)

Prüfung mit `cargo xtask check-ring-layering-full`:
- `contextra-ports` befindet sich in **Ring 0**.
- `Cargo.toml` deklariert ausschliesslich Abhängigkeiten zu `contextra-types` (Ring 0) und reinen Drittanbieter-Crates (`futures-util`, `bytes`, `serde`, `serde_json`, `ahash`, `thiserror`).
- `check-ring-layering-full` liefert 0 Verstöße für `contextra-ports`.

---

## Test & Lint Ausführung

- **Unit- & Integrationstests:**
  Command: `cargo test -p contextra-ports --locked -- --nocapture`
  Resultat: **38 passed; 0 failed; 0 ignored**
  Log: `/tmp/audit-ports-test.log`

- **Clippy-Sanitizing:**
  Command: `cargo clippy -p contextra-ports --all-targets -- -D warnings`
  Resultat: **0 errors, 0 warnings** (nach Behebung von `expect()`-Invocations und Redundant Lifetimes in `plugin.rs` & Tests)
  Log: `/tmp/audit-ports-clippy.log`

---

## VERDICT

**VERDICT: PASSED_WITH_OBSERVATIONS**

*Begründung & Beobachtungen:*
1. `contextra-ports` erfüllt alle strukturellen und architekturellen Anforderungen an Ring 0 (`#![forbid(unsafe_code)]`, Ring-Reinheit, Zero-Panic, Fail-Closed Lizenz-Gate).
2. AFIT-Traits (`VectorIndex`, `TextIndex`) sind bauartbedingt für statischen Dispatch ausgelegt und nicht direkt dyn-kompatibel.
3. 7 Stellen im Produktionscode außerhalb von `contextra-ports`/`contextra-crypto` nutzen direkte Entropy (`thread_rng()` / `OsRng`) anstelle des injizierten `Rng`-Ports und sind als P28-Abweichungen nachzuverfolgen.

VERIFIED-BY-SESSION: PENDING (TS: 2026-09-27T20:25:00Z)
