# contextra-ports — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Status:** Normativ, abgeleitet aus `docs/SPEC.md` v4.1
**Version:** 4.1
**Geltungsbereich:** `crates/contextra-ports`
**Normative Quellen:** SPEC 3.2 (R1, R4, R9), 3.3, 3.4 (Besitz der Zustände und Port-Implementierungen), 4.4 (Traits), 6.2, 6.6, 6.8, 6.9, 7.4, 7.13, 15, ADR-08, ADR-24, ADR-25, ADR-32, ADR-33, ADR-38, ADR-51, ADR-54, G-03, G-06, G-07, REM-03, REM-10, REM-14, REM-19, REM-25, REM-27, REM-36.

---

## 1. Zweck und Ebene

`contextra-ports` definiert die **Traits** und **Datentransferobjekte** (DTOs), über die Crates miteinander sprechen, ohne einander zu kennen (SPEC ADR-32). Das Crate implementiert nichts. Die Implementierung jedes Ports liegt im Crate, das den Zustand besitzt (R9, 3.4).

Das Crate hängt nach dem Umbau nur von `contextra-ids` und `roaring` ab. Es ist die einzige Stelle, an der Verträge zwischen E1-Crates stehen. Die Regel „ein Port genau einmal definiert“ (R4, R9) wird hier geprüft.

### 1.1 Ist-Befund (gemessen am HEAD `f8b44b2`)

| Messgröße | Wert | Befund |
|---|---|---|
| Quelldateien | 46 `.rs` (inkl. `scanner/`, Tests, Beispiel), 7.208 Zeilen | Großteil ist nicht SPEC-Bestand |
| Abhängigkeiten | `contextra-env`, `contextra-types`, `blake3`, `futures-util`, `bytes`, `serde`, `serde_json`, `ahash`, `thiserror` | Nach SPEC nur `contextra-ids`, `roaring`, `thiserror`. Sechs Abhängigkeiten sind Verstöße gegen R1 oder ADR-33 |
| `contextra-types`-Re-Exporte | `lib.rs` Zeilen 23–29: 25 Namen, darunter `TxId`, `EntityId`, `FileId`, `StoreId`, `ErasureId`, `OperatorRef`, `Seq`, `Timestamp`, `TenantScoped` | Ports ist Drehscheibe für Typen, die SPEC auflöst (ADR-33). Der Umbau entfernt die Re-Exporte |
| Features | `default = []`, `conformance`, `contract-tests`, `testkit` | `default` ist leer (R3 eingehalten). Die drei anderen Features sind Testwerkzeuge im Produktbau, verboten nach R3 und ADR-19 |
| Trait-Bestand | ca. 30 Traits. Davon SPEC-konform: `ReadGate` (mit anderer Signatur), `RecordSealer` (anders), `VectorIndex`, `TextIndex`, `GraphIndex`, `EmbeddingProvider`, `Reranker`, `Checkpoint` | Die meisten anderen sind Altmodell (`ErasureLayer`, `LayerScanner`, `Purger`, `WitnessLayer`, `CommitBarrier`, `ErasureObserver`, `ReportSigner`, `LicenseGate`, `PluginRegistry`, `AttentionExporter`) |
| Duplikate | `StorageEngine` (309 Zeilen, `StorageStats`) und `KvPrefixStore` (`kv.rs`, 274 Zeilen) | Zwei KV-Abstraktionen. SPEC kennt `KvStore` (3.4). Verbraucher von `StorageEngine` in 60 Dateien (Wortsuche) |
| Duplikate Uhr, RNG, Metriken, IDs | `clock.rs`, `rng.rs`, `metrics.rs`, `id_gen.rs`, `env.rs` (je 3 Zeilen, Re-Export aus `env`) | Reine Durchleitung. Entfällt, Verbraucher importieren aus `contextra-env` |
| Duplikat Dateisystem | `vfs.rs` (403 Zeilen, `Vfs`, `VfsFile`, `VfsOpenOptions`), `fs.rs` (11 Zeilen) | Gehört `contextra-durable-fs` (E-PRT-x, siehe 5) |
| Async | `BoxFuture`, `BoxStream` in `lib.rs`. Verbrauch in 85 Dateien außerhalb | Architekturentscheidung, siehe Abschnitt 9 (O-PRT-1) |
| Test-Konformität im Produktbau | `conformance.rs` (278), `contract_tests.rs` (632), `testkit`-Feature | ADR-19 und 11: Prüfwerkzeuge gehören `contextra-testkit` |
| Tests | 36 bestanden (inkl. `plugin_registry_dependency_cycle.rs`, `golden_env_parity.rs`) | Grün. Das Plugin-Test-Target testet einen zu entfernenden Port |

Ergebnis: Das Crate ist die größte Ansammlung von Altmodellen im E0-Bereich. Der Soll-Bestand ist kleiner als die Hälfte und besteht vor allem aus SPEC-Traits, die heute mit anderer Signatur existieren oder fehlen.

---

## 2. Vertrag (öffentliche API)

Alle Traits sind objektsicher (`dyn Trait` kompiliert, T-PRT-01). Alle Methoden sind synchron (Entscheidung O-PRT-1, Abschnitt 9). Alle Fehler sind `ContextraError` (SPEC 6.8).

### 2.1 Fehler (`error.rs`)

```rust
/// Einzige Fehlerart der öffentlichen API (SPEC 6.8). Keine Daten in der Anzeige (INV-6).
#[non_exhaustive]
pub enum ContextraError {
    Config(Vec<String>),
    Refused(String),
    Backpressure,
    DeadlineExceeded,
    NotFound,
    Erased,
    HistoryNotIndexed,
    SubjectSuppressed,
    ModelMismatch,
    IndexUnavailable,
    ReadOnly(ReadOnlyReason),
    Quarantine(TenantId),
    Integrity,
    Unsupported(&'static str),
    Crypto,
    Io(std::io::ErrorKind),
}
pub enum ReadOnlyReason { FsyncFailed, NoSpace, IoError }
```

`Refused(String)` trägt eine Begründung ohne Nutzdaten. Der Aufrufer prüft das über Tests (T-LOG-01).

### 2.2 Erasure und Gate (`erasure.rs`)

```rust
pub struct SealOrd(pub u32);      // Re-Export aus contextra-ids, nicht neu definiert
pub enum SealState { Open, Restricted, Erased }
pub enum RetireReason { RetentionExpired }

/// Mandantenweite Lesesperre (SPEC 4.4). Besitzer: contextra-erasure. O(1), keine E/A.
pub trait ReadGate: Send + Sync {
    fn state(&self, s: SealOrd) -> SealState;
    fn seq(&self) -> u64;
}

/// Ein Scrub-Ziel (SPEC 4.4). Besitzer jeweils vector, text, graph, store.
pub trait ScrubTarget: Send + Sync {
    fn name(&self) -> &'static str;
    fn plan(&self, dead: &roaring::RoaringBitmap) -> Result<ScrubPlan, ContextraError>;
    fn execute(&self, plan: &ScrubPlan, io: &mut IoBudget) -> Result<ScrubProgress, ContextraError>;
}

/// Zwischenspeicher mit Klartext oder abgeleitetem Inhalt entfernen Seals (ADR-39).
pub trait SealScoped: Send + Sync {
    fn invalidate_seals(&self, seals: &roaring::RoaringBitmap);
}

/// Protokoll-Senke für Löschereignisse (REQ-BAK-2, optional). Besitzer: erasure.
pub trait ErasureSink: Send + Sync {
    fn append(&self, record: &[u8]) -> Result<(), ContextraError>;
}

pub struct ScrubPlan { /* Werttyp, Felder durch erasure festgelegt */ }
pub struct ScrubProgress { /* Werttyp */ }
pub struct IoBudget { /* Werttyp: Bytes pro Sekunde, Restbudget */ }
```

`ReadGate` ersetzt die alte Form aus `gate.rs` (`KeyBound`, `KeyRange`, `RangeGate`, `GateDecision`). Diese Typen entfallen (E-PRT-03).

### 2.3 Version (`version.rs`)

```rust
/// Sichtbarkeit nach Versionsstand (SPEC 4.4). Besitzer: contextra-version.
pub trait Visibility: Send + Sync {
    fn is_current(&self, c: ChunkSlot) -> bool;
    fn visible_at(&self, c: ChunkSlot, seq: u64) -> bool;
}

/// Einbahn-Ereignis Version → Löschung (SPEC 4.3). Der Sender ist `version`, der Empfänger `db`.
pub trait ContentLifecycle: Send + Sync {
    fn content_retired(&self, s: SealOrd, why: RetireReason);
}
```

### 2.4 Kryptografie (`crypto.rs`)

```rust
pub enum RecordKind { ChunkText, VectorFull, DocBlob, SealedMeta, GraphLabel }
pub enum OpenError { KeyAbsent, Integrity }

/// Verschlüsselung je Seal (SPEC 4.4). Besitzer: contextra-crypto.
pub trait RecordSealer: Send + Sync {
    fn seal(&self, s: SealOrd, kind: RecordKind, aad: &[u8], pt: &[u8], out: &mut Vec<u8>) -> Result<(), ContextraError>;
    fn open(&self, s: SealOrd, kind: RecordKind, aad: &[u8], ct: &[u8], out: &mut Vec<u8>) -> Result<(), OpenError>;
}

/// Verschlüsselung je Segment (SPEC 5.8, ADR-30). Besitzer: contextra-crypto.
pub trait SegmentSealer: Send + Sync {
    fn new_segment(&self) -> Result<SegOrd, ContextraError>;
    fn seal_chunk(&self, seg: SegOrd, chunk_index: u32, pt: &[u8], out: &mut Vec<u8>) -> Result<(), ContextraError>;
    fn open_chunk(&self, seg: SegOrd, chunk_index: u32, ct: &[u8], out: &mut Vec<u8>) -> Result<(), OpenError>;
}

/// Schlüsselvernichtung in Stapeln (SPEC 4.4, REQ-KEY-4). Besitzer: contextra-crypto.
pub trait KeyDestroyer: Send + Sync {
    fn destroy_seals(&self, seals: &[SealOrd]) -> Result<(), ContextraError>;
    fn destroy_segments(&self, segs: &[SegOrd]) -> Result<(), ContextraError>;
    fn destroy_tenant(&self) -> Result<(), ContextraError>;
}

/// Wurzelschlüssel für den Anker (REQ-KEY-7). Implementierungen: Passphrase, KeyFile, VaultTransit (M6).
pub trait RootKeyProvider: Send + Sync {
    fn root_key(&self) -> Result<Zeroizing<[u8; 32]>, ContextraError>;
}
```

Hinweis: `Zeroizing` kommt aus `zeroize`. Die Abhängigkeit wird ergänzt. `zeroize` ist eine Zeile in `Cargo.toml` und keine Logik.

### 2.5 Speicher und Index (`kv.rs`, `vector_index.rs`, `text_index.rs`, `graph_index.rs`)

```rust
/// Schlüssel-Wert-Speicher mit atomaren Schreibstapeln (SPEC 3.4, REQ-CON-6). Besitzer: contextra-store.
pub trait KvStore: Send + Sync {
    fn get(&self, family: Family, key: &[u8]) -> Result<Option<Vec<u8>>, ContextraError>;
    fn write_batch(&self, ops: &[WriteOp]) -> Result<(), ContextraError>;   // atomar
    fn scan_prefix(&self, family: Family, prefix: &[u8]) -> Result<Box<dyn KvIter + '_>, ContextraError>;
}
pub trait SegmentStore: Send + Sync { /* Segment-Lesen und -Schreiben über Kopf nach SPEC 5.8 */ }

`Family`, `WriteOp` und `KvIter` sind Werttypen in `kv.rs`. Ihre Felder legt der PLAN von `store` fest (SPEC 5.10 sowie die KV-Familien `reg/`, `sub/`, `doc/` aus 5.7).

/// Vektorindex (SPEC 6.3). Besitzer: contextra-vector.
pub trait VectorIndex: Send + Sync {
    fn search(&self, q: &[f32], k: usize, filter: &RoaringBitmap, ef: usize) -> Result<Vec<Candidate>, ContextraError>;
    fn insert(&self, slot: ChunkSlot, vec: &[f32]) -> Result<(), ContextraError>;
    fn mark_dead(&self, slots: &RoaringBitmap) -> Result<(), ContextraError>;
}

/// Textindex (SPEC 6.4). Besitzer: contextra-text.
pub trait TextIndex: Send + Sync {
    fn search(&self, terms: &[String], k: usize, filter: &RoaringBitmap) -> Result<Vec<Candidate>, ContextraError>;
    fn stats(&self) -> TextIndexStats;
}

/// Entitätsgraph (SPEC 6.10, optional). Besitzer: contextra-graph.
pub trait GraphIndex: Send + Sync {
    fn expand(&self, seeds: &[ChunkSlot], max_hops: u8) -> Result<GraphPaths, ContextraError>;
}

pub struct Candidate { pub slot: ChunkSlot, pub score: f32 }
pub struct TextIndexStats { pub docs: u64, pub avg_len: f32 }
```

`VectorIndex::search` nimmt einen `RoaringBitmap` als Filter, damit die Bitmap-Mechanik aus SPEC 4.6 an allen Stellen gleich ist. `HybridSearchProvider` und `CommunityResolver` entfallen (E-PRT-06, E-PRT-07). Die Fusion (SPEC 6.6) liegt in `db`.

### 2.6 Einbettung, Reranker, Checkpoint (`embedding.rs`, `reranker.rs`, `checkpoint.rs`)

```rust
/// Embedding-Backend (SPEC 6.2, REQ-EMB-1). Besitzer: contextra-embed (ONNX), contextra-embed-http (HTTP).
pub trait EmbeddingProvider: Send + Sync {
    fn dim(&self) -> usize;
    fn model_id(&self) -> &str;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, ContextraError>;
}

/// Reranker (SPEC 6.2, REQ-EMB-5). Besitzer: contextra-embed.
pub trait Reranker: Send + Sync {
    fn rerank(&self, query: &str, candidates: &[&str]) -> Result<Vec<f32>, ContextraError>;
}

/// Checkpoint-Erzeugung und -Wiederherstellung (SPEC 7.11, ADR-42). Besitzer: contextra-checkpoint.
pub trait Checkpointer: Send + Sync {
    fn create(&self, data_dir: &std::path::Path, target: &std::path::Path) -> Result<(), ContextraError>;
    fn restore(&self, source: &std::path::Path, data_dir: &std::path::Path) -> Result<(), ContextraError>;
}
```

### 2.7 DTOs (`dto.rs`, SPEC 15)

Die Typen aus SPEC 15 liegen hier, ohne Serialisierung und ohne Logik: `SourceDoc`, `SearchQuery`, `SearchResponse`, `ContextPack`, `ErasureRequest`, `ScrubPlan`, `ScrubProgress`, `IoBudget`. Felder und Semantik ergeben sich aus den REQ-Abschnitten der SPEC. Dieser PLAN legt nur fest, welche Typen dort liegen. `ErasureRequest` und `Scope` werden in `erasure` benötigt, `ScrubPlan`/`ScrubProgress`/`IoBudget` in `erasure` und `vector`/`text`/`graph`/`store`.

---

## 3. Abhängigkeiten

| Crate | Status | Begründung |
|---|---|---|
| `contextra-ids` | **behalten** | Ausschließlich für Typen (`SealOrd`, `SegOrd`, `ChunkSlot`, `TenantId`, `TicketId`, `SubjectRef`, `PrincipalRef`, `DocId`, `CollectionId`) |
| `roaring` | **hinzufügen** | `RoaringBitmap` in Signaturen (SPEC 4.4) |
| `zeroize` | **hinzufügen** | `Zeroizing` in `RootKeyProvider` |
| `thiserror` | behalten | `ContextraError` |
| `contextra-types` | **entfernen** (REM-27) | ADR-33: aufgelöst |
| `contextra-env` | **entfernen** | Ports braucht keine Uhr und keinen RNG. Verbraucher importieren direkt |
| `blake3` | **entfernen** | keine Digests in Ports |
| `futures-util` | **entfernen** | O-PRT-1 |
| `bytes` | **entfernen** | `Vec<u8>` genügt |
| `serde`, `serde_json` | **entfernen** | Serialisierung in `contextra-mcp` |
| `ahash` | **entfernen** | Standard-HashMap genügt, Ports speichern keine Maps |

Interne Abhängigkeiten nach dem Umbau: `ids`. Das entspricht SPEC 3.3 (`ports = [ids]`).

---

## 4. Module

```
src/
  lib.rs              # Re-Exports der Typen aus contextra-ids (nur die, die Ports selbst nutzt), #![forbid(unsafe_code)]
  error.rs            # ContextraError, ReadOnlyReason
  erasure.rs          # SealState, RetireReason, ReadGate, ScrubTarget, SealScoped, ErasureSink, ScrubPlan, ScrubProgress, IoBudget
  version.rs          # Visibility, ContentLifecycle
  crypto.rs           # RecordKind, OpenError, RecordSealer, SegmentSealer, KeyDestroyer, RootKeyProvider
  kv.rs               # KvStore, SegmentStore, WriteOp, Family, KvIter
  vector_index.rs     # VectorIndex, Candidate
  text_index.rs       # TextIndex, TextIndexStats
  graph_index.rs      # GraphIndex, GraphPaths
  embedding.rs        # EmbeddingProvider
  reranker.rs         # Reranker
  checkpoint.rs       # Checkpointer
  dto.rs              # SourceDoc, SearchQuery, SearchResponse, ContextPack, ErasureRequest
tests/
  object_safety.rs    # T-PRT-01
  error_display.rs    # T-PRT-03
  deps.rs             # T-PRT-02 (cargo metadata)
```

Entfallen: alle nicht genannten Dateien, namentlich `attention.rs`, `barrier.rs`, `clock.rs`, `conformance.rs`, `contract_tests.rs`, `env.rs`, `erasure_legacy.rs`, `fs.rs`, `gate.rs` (ersetzt durch `erasure.rs`), `graph.rs`, `id_gen.rs`, `index.rs`, `inference.rs`, `keys.rs` (ersetzt durch `crypto.rs`), `kv_bridge_port.rs`, `key_material.rs`, `license.rs`, `ledger.rs` (ersetzt durch `erasure::ErasureSink`), `lifecycle.rs`, `metrics.rs`, `observability.rs`, `observer.rs`, `plugin.rs`, `policy.rs`, `query_rewriter.rs`, `report_signer.rs`, `rng.rs`, `scanner/`, `storage.rs`, `vfs.rs`.

---

## 5. Entfernungen

| ID | Entfernen | Grund | Ersatz | Verbraucher (Dateien, grep -w) |
|---|---|---|---|---|
| E-PRT-01 | `contextra-types`-Re-Exporte in `lib.rs` (25 Namen) | ADR-33, REM-27 | direkter Import aus `contextra-ids` oder `ports::dto` | 201 Dateien importieren aus Ports (Umstellung mechanisch) |
| E-PRT-02 | Re-Export-Module `clock.rs`, `env.rs`, `rng.rs`, `metrics.rs`, `id_gen.rs` (je 3 Zeilen) | Duplikat zu `contextra-env`, R4 | `contextra_env::...` | Clock 89, MetricsSink 23, IdGen 9 (Wortsuche) |
| E-PRT-03 | `gate.rs`: `KeyBound`, `KeyRange`, `RangeGate`, `GateDecision`, `Seq` | Altes Schlüssel-Bereichs-Modell. SPEC 4.4 kennt nur `ReadGate` auf Seal-Ordinal | `erasure::ReadGate` | RangeGate 4, GateDecision 8, KeyRange 1 |
| E-PRT-04 | `storage.rs` (`StorageEngine`, `StorageStats`, `MAX_SCAN_MERGE_ACCUMULATOR`) und `kv.rs` (`KvPrefixStore`, `KvLayout`, `RopeConfig`, `PrefixKey`, `KvBlock`, `KvPrefixHit`) | Zwei KV-Abstraktionen. SPEC kennt `KvStore` (3.4). Rope- und Prefix-Konfiguration ohne SPEC-Bezug | `kv::KvStore` | StorageEngine 60, KvPrefixStore 4, KvBlock 4 |
| E-PRT-05 | `keys.rs` (`FileId`, `FileKind`, `FileKey`, `SealCtx`, `Envelope`, `RecordSealer` alt, `FileKeyring`) | Altes Dateischlüssel-Modell (`DocDek`, Erfüllungsklassen). SPEC 5.4 kennt `DocKey`/`SegKey` im Crate `crypto` | `crypto::RecordSealer`, `SegmentSealer` | Envelope 2, FileKeyring 0, FileKey 0 |
| E-PRT-06 | `vector_index.rs`: `HybridSearchProvider` | Fusion gehört `db` (SPEC 6.6). Kein zweiter Fusionsort | Orchestrierung in `db` | 4 |
| E-PRT-07 | `graph_index.rs`: `CommunityResolver` | ADR-24 (Community-Erkennung entfernt) | keine | 4 |
| E-PRT-08 | `text_index.rs`: `SegmentSynthesizer` | Kein Verbraucher, altes Segmentmodell | keine | 0 |
| E-PRT-09 | `embedding.rs`: `TextEmbeddingEngine`, `TextGenerator`, `LlmTextGenerator`, `LlmTextGeneratorStreaming`, `ContextSegment` | ADR-24, REM-25: keine LLM-Schnittstellen im Kern | keine | TextEmbeddingEngine 6, LlmTextGenerator 7, TextGenerator 2 |
| E-PRT-10 | `inference.rs`, `query_rewriter.rs` | REM-25 | keine | QueryRewriter 2 |
| E-PRT-11 | `erasure.rs` (alt: `LayerId`, `EraseTarget`, `LayerScope`, `PurgeCx`, `PurgeOutcome`, `Armed`, `Scanner`, `KeyCoverage`, `ErasureLayer`, `StorageBackend`, `ErasureTarget`), `erasure_legacy.rs`, `scanner/` (`LayerScanner`, `Purger`, `WitnessLayer`, `ScanSession`, `LayerWitness`), `observer.rs` | REM-10, ADR-02, ADR-19: Saga, Zeugen, Prüfer sind Altmodell | neue `erasure.rs` (Abschnitt 2.2) | ErasureLayer 7, LayerScanner 4, Purger 5, WitnessLayer 4, ScanSession 2, ErasureObserver 0 |
| E-PRT-12 | `barrier.rs` (`CommitBarrier`, `BarrierExclusive`, `CompactionPermit`, `PurgePermit`, `PurgeBarrier`, `BarrierShared`) | Sperren liegen in `db` (SPEC 8.4 REQ-CON-1a und 1b). Ports besitzen keine Sperren | `db` | CommitBarrier 3, BarrierExclusive 1 |
| E-PRT-13 | `ledger.rs` (`LedgerSink`, `LedgerEntry`, `RangeSet`, `Timestamp`) | Ersetzt durch `erasure::ErasureSink` (BAK-2, SPEC 7.11). `Timestamp` entfällt (`u64`-Nanosekunden aus `Clock`) | `ErasureSink` | LedgerSink 0, LedgerEntry 0 |
| E-PRT-14 | `vfs.rs` (`Vfs`, `VfsFile`, `VfsOpenOptions`) und `fs.rs` | Dateisystem gehört `contextra-durable-fs` (dort `Vfs`, Abschnitt U-DFS-4) | `durable_fs::Vfs` | Vfs 33, VfsFile 14, VfsOpenOptions 11 |
| E-PRT-15 | `key_material.rs` (`KeyMaterialSource`) | Ersetzt durch `RootKeyProvider` (REQ-KEY-7) | `crypto::RootKeyProvider` | 5 |
| E-PRT-16 | `policy.rs` (`EgressPolicy`, `EgressDecision`) | Egress-Guard liegt in `privacy` (SPEC 6.9, REQ-PRV-3). Ports besitzt keine Richtlinie | `privacy::EgressGuard` | 8 |
| E-PRT-17 | `report_signer.rs` (`ReportSigner`, `ReportSignature`, `SignerId`) | ADR-02: keine Eigennachweise | keine | 0 |
| E-PRT-18 | `license.rs` (`LicenseGate`, `FeatureRing`, `AuthorizedRing`, `VectorDeleteMode`, `OpenFastGate`) | REM-03 | keine | LicenseGate 3 |
| E-PRT-19 | `plugin.rs` (`PluginRegistry`, `PluginManifest`, `PluginCapability`) und Test `plugin_registry_dependency_cycle.rs` | Nicht SPEC-Bestand | keine | PluginRegistry 1 |
| E-PRT-20 | `attention.rs` (`AttentionExporter`, `RequestId`), `observability.rs` (`DriftStatusProvider`) | Nicht SPEC-Bestand. Drift (REQ-EMB-4) ist eine Funktion von `embed` | `embed` | AttentionExporter 4, DriftStatusProvider 5 |
| E-PRT-21 | `lifecycle.rs` (`ContextPreparer`, `GroundingValidator`, `ResponseGroundingValidator`, `GroundingAssessment`) | Kontextzusammenstellung ist `rank`/`contextra` (SPEC 6.7), kein Port | `rank`/`contextra` | ContextPreparer 4 |
| E-PRT-22 | `graph.rs` (`RoleId`, `RoleBinding`, `HyperEdgeId`, `GraphMutationError`, `GraphCollectionMutation`) | ADR-24: Hyperkanten entfernt | keine | RoleBinding/HyperEdgeId (Wortsuche, meist im `graph`-Crate, das neu geschrieben wird) |
| E-PRT-23 | `kv_bridge_port.rs` (`KvBridgeStorage`) | Brücke zwischen alten Ports, ADR-51 | keine | 0 |
| E-PRT-24 | `conformance.rs`, `contract_tests.rs`, Features `conformance`, `contract-tests`, `testkit` | ADR-19, 11. Prüfwerkzeuge in `contextra-testkit` | `testkit` | 0 |
| E-PRT-25 | Abhängigkeiten `contextra-env`, `contextra-types`, `blake3`, `futures-util`, `bytes`, `serde`, `serde_json`, `ahash` | Abschnitt 3 | keine | |
| E-PRT-26 | `BoxFuture`, `BoxStream` in `lib.rs` | O-PRT-1 | synchrone Traits | 85 Dateien (siehe 9) |
| E-PRT-27 | `text_index.rs`-Kommentare, ANCHOR- und LINK-Zeilen in allen Dateien | ADR-19, REM-14 | Dokumentation | 0 |

Die Tabelle ist nicht abschließend. Vor dem Umbau wird `grep -rn "pub trait\|pub struct\|pub enum" crates/contextra-ports/src` gegen Abschnitt 2 abgeglichen. Alles Nichtgelistete wird entfernt.

---

## 6. Umbau (je Schritt ein Commit, Workspace grün)

Dieser PLAN hat die meisten Verbraucher im Workspace. Die Schritte sind deshalb nach dem Prinzip „additiv zuerst, Entfernung danach“ geordnet. Ein Port wird zuerst neu eingeführt, dann implementiert der Besitzer, zuletzt verschwindet der alte Port. Das hält den Workspace in jedem Commit grün.

| Schritt | Inhalt | Abhängigkeit | Prüfung |
|---|---|---|---|
| U-PRT-1 | Re-Export-Module löschen (E-PRT-02). Verbraucher mechanisch auf `contextra_env::...` umstellen | keine | `cargo check --workspace` |
| U-PRT-2 | `contextra-types`-Re-Exporte entfernen (E-PRT-01). Verbraucher importieren IDs aus `contextra_ids`, DTOs aus `ports::dto` | `contextra-ids` mit den neuen Typen (U-IDS-1 bis U-IDS-4) | `grep -rn "contextra_ports::.*TxId\|contextra_ports::.*EntityId" crates` leer |
| U-PRT-3 | **Additiv:** neue Module `erasure.rs`, `version.rs`, `crypto.rs`, `kv.rs` (`KvStore`), `dto.rs`, `checkpoint.rs` mit den Traits aus Abschnitt 2 einführen, alte Traits bleiben | keine | `cargo check -p contextra-ports` |
| U-PRT-4 | Entfernen der Testfeatures und Test-Module (E-PRT-24). `plugin_registry_dependency_cycle.rs` löschen (E-PRT-19) | keine | `cargo test -p contextra-ports` |
| U-PRT-5 | Entfernen der LLM- und Lizenz-, Plugin-, Attention-, Lifecycle-Ports (E-PRT-09, E-PRT-10, E-PRT-18 bis E-PRT-21). Verbraucher: `contextra` (llm-Rewriter-Tests), `contextra-mcp` | keine Abhängigkeit auf neue Traits | `grep -rn "LlmTextGenerator\|QueryRewriter\|LicenseGate"` leer |
| U-PRT-6 | Entfernen der Ports `report_signer`, `key_material`, `policy`, `kv_bridge_port`, `barrier`, `ledger`, `graph.rs` (E-PRT-12, E-PRT-13, E-PRT-15 bis E-PRT-17, E-PRT-22, E-PRT-23). Verbraucher sind die Crates, die den neuen Port implementieren (`erasure`, `crypto`, `privacy`, `db`) | **Abhängig** von den PLANs von `erasure`, `crypto`, `privacy`, `db` | `cargo check --workspace` |
| U-PRT-7 | Entfernen des Altmodells `erasure.rs`, `erasure_legacy.rs`, `scanner/`, `observer.rs` (E-PRT-11). Der neue `erasure`-Port ist mit U-PRT-3 vorhanden | **Abhängig** von REM-10 (`contextra-erasure`) | `grep -rn "ErasureLayer\|LayerScanner" crates` leer |
| U-PRT-8 | Entfernen von `gate.rs` (E-PRT-03). `ReadGate` wird über `erasure.rs` implementiert | **Abhängig** von `erasure` | `grep -rn "RangeGate\|KeyBound" crates` leer |
| U-PRT-9 | Entfernen von `keys.rs` (E-PRT-05). `RecordSealer` und `Envelope` werden über `crypto` ersetzt | **Abhängig** von `crypto` | `grep -rn "FileKeyring\|DocDek" crates` leer |
| U-PRT-10 | Entfernen von `vfs.rs`, `fs.rs` (E-PRT-14). Verbraucher importieren `Vfs` aus `contextra-durable-fs` | **Abhängig** von U-DFS-4 | `grep -rn "contextra_ports::.*Vfs" crates` leer |
| U-PRT-11 | `StorageEngine` entfernen (E-PRT-04 erster Teil). Verbraucher (60 Dateien) auf `KvStore` aus `kv.rs` umstellen. Die Implementierung liegt in `store` | **Abhängig** von `store` (`KvStore` implementiert) | `grep -rn "StorageEngine" crates` leer |
| U-PRT-12 | `KvPrefixStore`, `KvLayout`, `RopeConfig`, `KvBlock` entfernen (E-PRT-04 zweiter Teil) | keine weiteren Verbraucher nach U-PRT-11 | `grep -rn "KvPrefixStore\|RopeConfig" crates` leer |
| U-PRT-13 | Entfernen von `vector_index.rs`-Teil `HybridSearchProvider`, `graph_index.rs`-Teil `CommunityResolver`, `text_index.rs`-Teil `SegmentSynthesizer` (E-PRT-06 bis E-PRT-08) | keine | `cargo check --workspace` |
| U-PRT-14 | Abhängigkeiten `contextra-env`, `contextra-types`, `blake3`, `futures-util`, `bytes`, `serde`, `serde_json`, `ahash` entfernen. `roaring`, `zeroize` hinzufügen (E-PRT-25) | Verbraucher nutzen nur noch die Typen aus Abschnitt 2 | `cargo tree -p contextra-ports` zeigt nur `contextra-ids`, `roaring`, `thiserror`, `zeroize` |
| U-PRT-15 | `BoxFuture`, `BoxStream` entfernen (E-PRT-26), **nach** Entscheidung O-PRT-1 | siehe Abschnitt 9 | `grep -rn "BoxFuture\|BoxStream" crates` leer |

---

## 7. Tests

| ID | Prüfung | Ort |
|---|---|---|
| T-PRT-01 | Jeder Trait aus Abschnitt 2 ist objektsicher: `fn _assert(_: &dyn Trait) {}` kompiliert (Compile-Test) | `tests/object_safety.rs` |
| T-PRT-02 | `cargo metadata` für `contextra-ports` listet als Workspace-Abhängigkeiten nur `contextra-ids` (G-03, R1) | `tests/deps.rs` (ruft `cargo metadata` auf) |
| T-PRT-03 | `Display` von `ContextraError` enthält keine Nutzdaten. Canary-Pfad und Canary-Schlüsselbytes in den Varianten `Refused` und `Io` werden nicht ausgegeben | `tests/error_display.rs` |
| T-PRT-04 | `SealState` und `RetireReason` haben die Varianten aus SPEC 4.4 und keine weiteren | `tests/object_safety.rs` |
| T-PRT-05 | Die Konstanten und Typen, die in Ports vorkommen, sind dieselben wie in `contextra-ids` (keine Neudefinition, Typ-Identität über `TypeId`-Vergleich in einem Test) | `tests/object_safety.rs` |

---

## 8. Abnahme

| Gate | Prüfung für `contextra-ports` |
|---|---|
| G-01 | `cargo check`, `cargo test`, `cargo clippy -- -D warnings` (Toolchain 1.89.0) |
| G-02 | `#![forbid(unsafe_code)]` in `src/lib.rs` |
| G-03 | Abhängigkeiten `ids` (intern), `roaring`, `thiserror`, `zeroize`. Keine weiteren Workspace-Crates |
| G-06 | `default = []`, Features `conformance`, `contract-tests`, `testkit` entfernt |
| G-07 | `grep -rn "contextra_types\|contextra-types\|ContextraError::Ledger\|LedgerSink\|StorageEngine\|HybridSearchProvider\|CommunityResolver" crates/contextra-ports` leer |
| G-13 | keine `unwrap`/`expect`/`panic!` (in Ports ohnehin keine Logik) |
| R9 | `grep -rn "pub trait" crates --include=*.rs \| sort` zeigt jeden Port genau einmal, in `contextra-ports` (Ausnahme: Implementierungen mit `impl Trait for`) |
| R1 | keine Workspace-Abhängigkeit außer `contextra-ids` |

---

## 9. Offene Entscheidung O-PRT-1: Async im Kern

Der Bestand verwendet `BoxFuture` und `BoxStream` in 85 Dateien außerhalb des Crates. SPEC 4.4 und 6.x beschreiben alle Schnittstellen synchron. SPEC 6.1 verlangt Backpressure ohne Blockieren. Das kann mit Warteschlangen und Rückmeldung `Backpressure` gelöst werden, ohne `async`.

**Vorschlag dieses PLAN:** Die Kern-Traits bleiben synchron. Asynchronie wird nur in `contextra-server` (M6) über eine Brücke aus Tokio-Tasks auf den synchronen Kern gelegt. Begründung: Die Löschkette braucht eine feste Reihenfolge mit `fdatasync`-Punkten (SPEC 7.4), die sich in einem Async-Executor nur schwer prüfen lässt. Der Schreibpfad ist pro Shard seriell (REQ-CON-1). Ein Async-Vertrag würde diese Garantie nicht stärken.

**Gegenargument:** Das Bestandsmaterial (85 Stellen) ist ein größerer Umbau. Wer U-PRT-15 vorzieht, muss die Stellen auf Synchronität umbauen.

Die Entscheidung ist zu bestätigen, bevor U-PRT-15 ausgeführt wird. Bis dahin bleiben `BoxFuture` und `BoxStream` bestehen und werden nicht in neue Traits übernommen.

---

**Reihenfolge im Repository:** Nach dem `durable-fs`-PLAN als sechster Commit. `ports` ist der letzte E0-PLAN. Die Umsetzung beginnt mit U-PRT-1, das additive U-PRT-3 folgt danach, weil die Besitzer-Crates die neuen Traits implementieren müssen, bevor die alten Traits entfallen.
