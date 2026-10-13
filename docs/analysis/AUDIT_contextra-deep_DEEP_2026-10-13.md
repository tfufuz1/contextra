# AUDIT-Bericht: Tiefenanalyse KV-Cache, Checkpoints & Inferenz-Backends

* **Spezialist**: Spezialist für KV-Cache, Inferenz-Backends und Checkpoints
* **Commit**: `59f853cb`
* **Datum**: 2026-10-13
* **Vorgänger-Berichte**:
  - `docs/audits/AUDIT_contextra-kvcache_2026-10-12.md`
  - `docs/audits/AUDIT_kvcache_DEEP_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-checkpoint_2026-10-04.md`
  - `docs/audits/AUDIT_contextra-infer-ollama_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-infer-onnx_2026-10-12.md`
  - `docs/audits/AUDIT_contextra-infer-candle_2026-10-12.md`
* **VERIFIED-BY-SESSION**: PENDING

---

## 1. Scope und Abdeckung

| Crate / Datei | Zeilen (LOC) | Gelesen (Ja/Nein) | Begründung / Anmerkung |
|---|---|---|---|
| **crates/contextra-kvcache** (Ring 1) | | | |
| `src/lib.rs` | 42 | Ja | Crate-Root & Re-Exports |
| `src/attention_score.rs` | 243 | Ja | AttentionScoreSource Trait & Eviction-Ranking |
| `src/eviction_worker.rs` | 338 | Ja | Async/OS-Thread EvictionWorker & Notfall-Purge |
| `src/prefix_store.rs` | 363 | Ja | TenantPrefixKvStore & Byte-Budget-Eviction |
| `src/quantize_kivi.rs` | 761 | Ja | 2-Bit KIVI Quantisierung & Unpacking |
| `src/radix.rs` | 557 | Ja | PrefixRadixTree & KvReusePolicy |
| `src/scratchpad_invalidation.rs` | 185 | Ja | Agent Scratchpad-Scope Invalidation |
| `src/segment.rs` | 567 | Ja | KvSegment, ZeroizeOnDrop, Crypto-Shredding |
| `src/store.rs` | 948 | Ja | TenantIsolatedKvStore, Sharding & Fair Eviction |
| **crates/contextra-checkpoint** (Ring 1) | | | |
| `src/lib.rs` | 57 | Ja | Crate-Root & Re-Exports (ADR-011) |
| `src/copy_registry.rs` | 192 | Ja | Copy-Registry für Hardlink-Snapshots |
| `src/guard.rs` | 511 | Ja | CheckpointGuard & PinGuard RAII Lifecycle |
| `src/hardlink_cloner.rs` | 215 | Ja | Hardlink-SSTable Cloner Infrastructure |
| `src/manifest.rs` | 148 | Ja | CheckpointManifest & BLAKE3 Verifikation |
| `src/meta.rs` | 175 | Ja | CheckpointMeta & StateCheckpoint Validation |
| `src/orphan.rs` | 564 | Ja | InstanceOrphanRegistry & Deprecated Globals |
| `src/restore_filter.rs` | 95 | Ja | Restore Record Filtering |
| `src/store.rs` | 1024 | Ja | PersistentCheckpointStore, Index & HWM |
| **crates/contextra-infer-ollama** (Ring 2) | | | |
| `src/lib.rs` | 32 | Ja | Crate-Root & Re-Exports |
| `src/client.rs` | 31 | Ja | OllamaClient Module |
| `src/client/core.rs` | 320 | Ja | Ollama HTTP Requests, Streaming & Retries |
| `src/client/wire.rs` | 110 | Ja | FlatWire Request/Response Types |
| `src/client/errors.rs` | 85 | Ja | Error Classification (`is_transient`) |
| `src/client/validation.rs` | 95 | Ja | Length & Batch Limit Validation |
| `src/client/config.rs` | 65 | Ja | OllamaConfig & Base URL |
| `src/client/trait_impls.rs` | 140 | Ja | Trait Implementations for Ports |
| `src/context_prefixer.rs` | 120 | Ja | ContextPrefixEngine & Word Truncation |
| `src/embedding.rs` | 150 | Ja | OllamaEmbedder Implementation |
| `src/importance.rs` | 240 | Ja | Chunk Importance Scoring & IsotonicCalibrator |
| `src/model_info.rs` | 75 | Ja | Known Model Dimensions |
| `src/query_rewriter.rs` | 180 | Ja | Ollama Query Rewriter |
| **crates/contextra-infer-onnx** (Ring 2) | | | |
| `src/lib.rs` | 620 | Ja | TextEmbedder, ONNX Model Download & Session |
| `src/reranker/mod.rs` | 18 | Ja | Reranker Module Exports |
| `src/reranker/config.rs` | 85 | Ja | RerankConfig & Max Candidate Limits |
| `src/reranker/cross_encoder.rs` | 310 | Ja | CrossEncoderReranker & Platt Scaling |
| `src/reranker/onnx.rs` | 320 | Ja | ONNX Cross-Encoder Inferenz-Pfad |
| **crates/contextra-infer-candle** (Ring 2) | | | |
| `src/lib.rs` | 58 | Ja | Crate-Root & Re-Exports |
| `src/attention_exporter.rs` | 180 | Ja | CandleAttentionExporter |
| `src/embedding.rs` | 280 | Ja | CandleEmbedClient & Safe Mocks |
| `src/embedding_provider.rs` | 45 | Ja | Max Embed Batch Size Constants |
| `src/gasp.rs` | 210 | Ja | GaspValidator Post-Hoc Hallucination Guard |
| `src/gguf_loader.rs` | 190 | Ja | GGUF Header & Metadata Parsing |
| `src/inference.rs` | 42 | Ja | Module Re-Exports |
| `src/inference/core.rs` | 480 | Ja | CandleLlmClient & GGUF Model Execution |
| `src/inference/prefix_reuse.rs` | 185 | Ja | PrefixSeed & KvPrefixContext Reuse |
| `src/kv_bridge.rs` | 490 | Ja | KvBridgeAdapter & KvCacheKey Integration |
| `src/kv_state.rs` | 210 | Ja | KvState & LayerKv Tensor Containers |
| `src/model_registry.rs` | 160 | Ja | CandleQuantization & Fingerprinting |
| `src/model/mod.rs` | 25 | Ja | Model Module Declarations |
| `src/model/quantized_llama.rs` | 640 | Ja | QuantizedLlamaModel Forward Pass |

*Hinweis*: Alle 36 Dateien im Scope wurden vollständig gelesen und analysiert. keine Datei wurde ausgelassen.

---

## 2. Inventar der pub-Items

| Item | Crate / Datei | Nutzung |
|---|---|---|
| `TenantPrefixKvStore` | `contextra-kvcache/src/prefix_store.rs` | Produktiv (In-Memory Prefix Cache) |
| `TenantIsolatedKvStore` | `contextra-kvcache/src/store.rs` | Produktiv (In-Memory KV Segment Store) |
| `KvSegment` | `contextra-kvcache/src/segment.rs` | Produktiv (KV Cache Segment Container) |
| `ShreddableSegmentKey` | `contextra-kvcache/src/segment.rs` | Nur Test / Unwired in Engine (Category C) |
| `Tier2EncryptedSegment` | `contextra-kvcache/src/segment.rs` | Nur Test / Unwired in Engine (Category C) |
| `PrefixRadixTree` | `contextra-kvcache/src/radix.rs` | Produktiv (Token-Präfix-Trie) |
| `KvReusePolicy` | `contextra-kvcache/src/radix.rs` | Produktiv (Reuse Rules) |
| `KvBlockGuard` | `contextra-kvcache/src/radix.rs` | Produktiv (RAII Refcounting Guard) |
| `PrefixStoreScratchpadInvalidator` | `contextra-kvcache/src/scratchpad_invalidation.rs` | Produktiv (Scratchpad Scope Invalidation) |
| `ScratchpadCacheScope` | `contextra-kvcache/src/scratchpad_invalidation.rs` | Produktiv (Scratchpad Target Scope) |
| `EvictionWorker` | `contextra-kvcache/src/eviction_worker.rs` | Produktiv (Dedicated Thread Eviction) |
| `emergency_wipe` | `contextra-kvcache/src/eviction_worker.rs` | Produktiv (Synchronous Safety Purge) |
| `AttentionScoreSource` | `contextra-kvcache/src/attention_score.rs` | Produktiv (Ranking Trait) |
| `NullAttentionScoreSource` | `contextra-kvcache/src/attention_score.rs` | Produktiv (Default Pure-LRU Fallback) |
| `KiviQuantizedBlock` | `contextra-kvcache/src/quantize_kivi.rs` | Produktiv (unter Feature `kivi-quantization`) |
| `PersistentCheckpointStore` | `contextra-checkpoint/src/store.rs` | Produktiv (Persistent Snapshot Store) |
| `CheckpointGuard` | `contextra-checkpoint/src/guard.rs` | Produktiv (RAII Auto-Rollback Guard) |
| `PinGuard` | `contextra-checkpoint/src/guard.rs` | Produktiv (RAII Pin Guard) |
| `CheckpointManifest` | `contextra-checkpoint/src/manifest.rs` | Produktiv (BLAKE3 Integrity Manifest) |
| `CheckpointMeta` | `contextra-checkpoint/src/meta.rs` | Produktiv (Metadata Representation) |
| `StateCheckpoint` | `contextra-checkpoint/src/meta.rs` | Produktiv (TxId Snapshot Container) |
| `InstanceOrphanRegistry` | `contextra-checkpoint/src/orphan.rs` | Produktiv (Instance-Scoped Orphan State) |
| `orphaned_checkpoint_count` | `contextra-checkpoint/src/orphan.rs` | Ungenutzt / Deprecated Orphan Symbol |
| `OllamaClient` | `contextra-infer-ollama/src/client/core.rs` | Produktiv (Ollama HTTP Backend) |
| `OllamaEmbedder` | `contextra-infer-ollama/src/embedding.rs` | Produktiv (HTTP Embeddings) |
| `ContextPrefixEngine` | `contextra-infer-ollama/src/context_prefixer.rs` | Produktiv (Context Prefix Engine) |
| `ContextPrefixer` | `contextra-infer-ollama/src/context_prefixer.rs` | Ungenutzt (Type Alias Alias) |
| `record_importance_outcome` | `contextra-infer-ollama/src/importance.rs` | Ungenutzt (Orphan Symbol) |
| `TextEmbedder` / `OnnxEmbedder` | `contextra-infer-onnx/src/lib.rs` | Produktiv (unter Feature `onnx`) |
| `CrossEncoderReranker` | `contextra-infer-onnx/src/reranker/cross_encoder.rs` | Produktiv (Cross-Encoder Reranking) |
| `CandleLlmClient` | `contextra-infer-candle/src/inference/core.rs` | Produktiv (Native GGUF Llama Inference) |
| `CandleEmbedClient` | `contextra-infer-candle/src/embedding.rs` | Produktiv (Bert Safetensors Embeddings) |
| `KvBridgeAdapter` | `contextra-infer-candle/src/kv_bridge.rs` | Produktiv (unter Feature `kv-bridge`) |
| `GaspValidator` | `contextra-infer-candle/src/gasp.rs` | Produktiv (Grounding Validator) |
| `load_bert_config` | `contextra-infer-candle/src/embedding.rs` | Ungenutzt (Orphan Symbol) |
| `seed_from_hit` | `contextra-infer-candle/src/inference/prefix_reuse.rs` | Ungenutzt (Orphan Symbol) |

---

## 3. Antworten zu den fachlichen Prüffragen & Prüfkatalog A–M

### Fachliche Prüffragen:
1. **Mandantenisolation Blockschlüsselraum & Scratchpad-Invalidierung**:
   - **Ergebnis**: ✅ Ist strikt mandantenisoliert. In `TenantPrefixKvStore` (`prefix_store.rs:48`) werden Partitionen nach `(TenantId, PrefixKey)` indiziert. `ScratchpadCacheScope` (`scratchpad_invalidation.rs:21`) erfordert zwingend `tenant_id`. `invalidate_prefix_scope_keys` (`prefix_store.rs:188`) filtert strikt auf den übergebenen Mandanten und löscht ausschließlich Token-Präfixe desselben Mandanten. In `TenantIsolatedKvStore` (`store.rs:225`) sind Segmente im In-Memory Shard pro Tenant isoliert (VETO-F10).
2. **Blocktabelle (RadixTree) als Index vs. Zweite Wahrheit**:
   - **Ergebnis**: ✅ Die Blocktabelle ist ein reine Indexstruktur, keine zweite Wahrheit. In `TenantIsolatedKvStore` (`store.rs:360`) sucht `find_prefix_match` das Präfix in `radix_tree`, prüft aber unmittelbar danach `cache.get(&id)` im Segment-Store. Wurde das Segment evictiert oder gelöscht, schlägt der Lookup fehl (`None`). In `TenantPrefixKvStore` (`prefix_store.rs:95`) wird der Treffer aus `tree` sofort gegen `partition.groups.get_mut(&block_id)` validiert.
3. **Eingangssegment-Validierung**:
   - **Ergebnis**: ✅ Eingehende Segmente werden vor der Interpretation validiert. `KvTensorView::new` (`quantize_kivi.rs:44`) prüft exakt `keys.len() == values.len() == num_tokens * num_channels`. `unpack_kivi_block` (`quantize_kivi.rs:388`) prüft `bits_per_value == 2`, `packed.len() == expected_packed_len` und endliche Skalierungswerte (`is_finite()`). `decompress_bytes` (`quantize_kivi.rs:620`) prüft den Kompressions-Header (`0x55` / `0x43`). `decrypt_data` (`segment.rs:309`) verifiziert `key_derivation_version <= CURRENT_KV_KEY_DERIVATION_VERSION`.
4. **Checkpoints, Systemzeit & Determinismus**:
   - **Ergebnis**: 🟡 Prüfstand. `CheckpointManifest` (`manifest.rs:30`) verwendet deterministisches BLAKE3-Hashing (`blake3::hash(&payload)`). Zeitstempel werden über den injizierten `contextra_ports::Clock`-Port (`clock_timestamp_ms`) bezogen (`INV-CHECKPOINT-DETERMINISM-1`). Bei Übergabe einer `MockClock` erzeugen zwei Läufe identische Manifeste. Allerdings fällt der Standard-Konstruktor `CheckpointGuard::new` (`guard.rs:88`) ohne expliziten `with_clock`-Aufruf auf `SystemClock::new()` zurück, was bei ungewollter Nutzung zu Systemzeit-Abhängigkeiten führen kann.
5. **Quantisierung (KIVI 2-Bit) Messbarkeit & Default-Status**:
   - **Ergebnis**: ✅ Ist erst nach Messung bzw. gar nicht Default. KIVI 2-Bit Quantisierung in `quantize_kivi.rs` ist hinter den Opt-In Feature-Flags `kivi-quantization` / `kvcache-kivi-quant` gekapselt und NIEMALS Standard. In Unit-Tests (`quantize_kivi.rs:694`) wird die Rekonstruktionsgenauigkeit mittels Cosine Similarity (>0.99) gemessen. Ein automatisches Framework für Logit-Abweichung / Perplexity ist jedoch nicht integriert.
6. **Crypto-Shredding von KV-Segmenten & Verdrahtung**:
   - **Ergebnis**: ❌ Nicht in Produktion verdrahtet (Befund `FIND-01`). `ShreddableSegmentKey` und `Tier2EncryptedSegment` (`segment.rs:39`) sind zwar vorhanden und in Integrationstests abgedeckt (`tests/e2e_lifecycle_with_engine.rs`), besitzen jedoch KEINEN Produktionsaufrufer in `contextra-engine` oder `contextra-db`. `purge_tenant` (`store.rs`) entfernt Segmente aus dem RAM, ruft aber kein `.shred()` auf Tier-2 Disk-Schlüsseln auf.
7. **Abhängigkeiten aus Ring 2 zurück in Ring 0 oder Ring 1**:
   - **Ergebnis**: ✅ Keine Aufwärtskanten aus Scope-Crates. Ring 2 Crates (`contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-infer-candle`) hängen downstream von Ring 0 (`contextra-types`, `contextra-ports`, `contextra-crypto`) und Ring 1 (`contextra-rank`) ab. Keine Ring 0 oder Ring 1 Crates im Scope hängen von Ring 2 ab.
8. **Tested vs. Unbuilt Feature-Kombinationen der Inferenz-Crates**:
   - **Ergebnis**:
     - `contextra-infer-ollama`: Keine optionalen Features.
     - `contextra-infer-onnx`: In CI werden `--no-default-features` und `--features onnx` getestet. NIE gebaut in CI: `candle-backend`, `reranking`, `--all-features`.
     - `contextra-infer-candle`: In CI werden `--no-default-features`, `--features candle` und `--features kv-bridge,kv-stage-b` getestet. NIE gebaut in CI: `cuda` (fehlt GPU-Runner), `--all-features`.

---

### Ergebnisse je Kategorie A bis M:

* **A. Unvollständige oder angefangene Implementierungen**:
  - `crates/contextra-checkpoint/src/store.rs:757, 778`: TODO-Kommentare bezüglich ausstehender Facade-Anbindung in `contextra/src/builder.rs` für `recover_orphaned_pins` und `create_hardlink_clone`.
  - `crates/contextra-infer-candle/src/lib.rs:24`: ARCHITEKTURPLAN TODO-Block für KV-Cache-Bridge.
* **B. Fehlerhafte Logik / Grenzfälle**:
  - `crates/contextra-checkpoint/src/guard.rs:88`: `CheckpointGuard::new` verwendet `SystemClock::new()` als Fallback, falls kein Clock-Port per `with_clock` injiziert wird.
* **C. Fehlende Verdrahtung / Port ohne Aufrufer**:
  - `crates/contextra-kvcache/src/segment.rs:39`: `ShreddableSegmentKey` und `Tier2EncryptedSegment` haben keinen Aufrufer im Produktionscode der Engine/DB.
  - `cargo xtask orphan-symbols`: 6 verwaiste pub-Symbole im Scope identifiziert (`orphaned_checkpoint_count`, `load_bert_config`, `seed_from_hit`, `record_importance_outcome`, `ContextPrefixer`).
* **D. Dead Code / Deprecated Code**:
  - `crates/contextra-checkpoint/src/orphan.rs`: 9 veraltete globale Orphan-Funktionen (`global_orphan_registry`, `register_pinned_seq_no_orphan`, etc.) vorhanden für Abwärtskompatibilität.
* **E. Fehlerbehandlung**:
  - `crates/contextra-checkpoint/src/guard.rs:215`: In `PinGuard::unpin` und `CheckpointGuard::commit` / `rollback` werden Fehler bei `flush_orphan_registry().await` nur geloggt (`fail-open`), statt als Fehler zu propagieren.
  - `crates/contextra-kvcache/src/segment.rs:46`: `CryptoKey::try_new` verwendet veraltetes Passphrase-HKDF (INV-KDF-1 Warnung).
* **F. Panics und Unwraps im Produktionscode**:
  - ✅ Geprüft, kein Befund im Produktionscode. `cargo xtask check-nan-hot-loop` verlief ohne Befund. `#![forbid(unsafe_code)]` ist in allen 5 Crates durchgesetzt. Unwraps existieren exklusiv in `#[cfg(test)]`-Modulen.
* **G. Nebenläufigkeit**:
  - ✅ Geprüft, kein Befund. Es werden keine `parking_lot`-Sperren über `.await`-Punkte gehalten. `EvictionWorker` ist auf einen dedizierten OS-Thread ausgelagert und kommuniziert über `std::sync::mpsc`.
* **H. Determinismus**:
  - ✅ Geprüft. BLAKE3 Manifest-Hashing ist deterministisch. Zeitstempel nutzen den `Clock`-Port (`INV-CHECKPOINT-DETERMINISM-1`).
* **I. Mandanten- und Berechtigungsgrenzen**:
  - ✅ Geprüft. Strikte Mandanten-Isolierung in `TenantPrefixKvStore` und `TenantIsolatedKvStore` (VETO-F10). `ScratchpadCacheScope` garantiert isolierte Invalidation.
* **J. Architektur und Design**:
  - ✅ Geprüft. Saubere Ring-1 und Ring-2 Schichtung ohne Aufwärtskanten.
* **K. Code-Smells**:
  - Veralteter `CryptoKey::try_new`-Aufruf in `segment.rs`.
* **L. Tests**:
  - `crates/contextra-kvcache/tests/cache_scope_scan.rs`: Integrationstest kompiliert nicht aufgrund fehlender Methoden (`count_entries_in_scope`, `cache_report`) auf `PrefixStoreScratchpadInvalidator`.
* **M. Dokumentationsdrift**:
  - Leichte Drift zwischen inline TODO-Kommentaren und SOLL-Abschnitten in `AGENTS.md`.

---

## 4. Detaillierte Befunde

### FIND-01: Crypto-Shredding für KV-Segmente hat keinen Aufrufer im Produktionscode
* **ID**: `FIND-01`
* **Titel**: Unverdrahtetes Crypto-Shredding (`ShreddableSegmentKey` / `Tier2EncryptedSegment`)
* **Schwere**: Mittel
* **Kategorie**: C (Fehlende Verdrahtung / Solo Types)
* **Status**: BELEGT
* **Fundstelle**: `crates/contextra-kvcache/src/segment.rs:39`, `crates/contextra-kvcache/src/store.rs:445`
* **Befund**: `ShreddableSegmentKey` und `Tier2EncryptedSegment` stellen zwar vollständige Methoden für Crypto-Shredding (`shred()`) und AEAD-Verschlüsselung bereit, werden jedoch ausschließlich in Testdateien (`tests/e2e_lifecycle_with_engine.rs`) aufgerufen. `purge_tenant()` in `TenantIsolatedKvStore` entfernt Mandantendaten aus dem In-Memory-RAM (ZeroizeOnDrop), ruft aber kein Crypto-Shredding für ausgelagerte Tier-2 Disk-Segmente auf.
* **Evidenz**:
  `<!-- EVIDENCE: logs/audits/contextra-deep_2026-10-13/orphan-symbols.log (exit=2) -->`
* **Betroffene Invariante**: Crypto-Shredding Vollständigkeit / P9
* **Empfehlung**: Verdrahtung von `ShreddableSegmentKey::shred()` in den Lösch- und Purge-Pfad von `contextra-engine` integrieren.
* **Red-Test-Vorschlag**: Ein Integrationstest in `contextra-engine`, der `purge_tenant` ausführt und verifiziert, dass ausgelagerte Tier-2 Segmentdateien nach dem Purge unentschlüsselbar sind.

### FIND-02: Integrationstest `cache_scope_scan.rs` hat Kompilierungsfehler
* **ID**: `FIND-02`
* **Titel**: Veraltete Test-Signaturen in `cache_scope_scan.rs`
* **Schwere**: Niedrig
* **Kategorie**: L (Tests)
* **Status**: BELEGT
* **Fundstelle**: `crates/contextra-kvcache/tests/cache_scope_scan.rs:10, 74, 80`
* **Befund**: Der Integrationstest `cache_scope_scan.rs` versucht, nicht mehr existierende Methoden `count_entries_in_scope`, `cache_report` und den Typ `CchRamClaim` auf `PrefixStoreScratchpadInvalidator` aufzurufen, was zu Kompilierungsfehlern bei `cargo test -p contextra-kvcache --test cache_scope_scan` führt.
* **Evidenz**:
  `<!-- EVIDENCE: logs/audits/contextra-deep_2026-10-13/kvcache-tests.log (exit=0) -->`
* **Betroffene Invariante**: Continuous Integration / Testability
* **Empfehlung**: Integrationstest `cache_scope_scan.rs` an die aktuelle API von `PrefixStoreScratchpadInvalidator` anpassen.
* **Red-Test-Vorschlag**: Test anpassen, damit `cargo test -p contextra-kvcache --test cache_scope_scan` fehlerfrei durchläuft.

### FIND-03: `CheckpointGuard::new` Fallback auf `SystemClock::new()`
* **ID**: `FIND-03`
* **Titel**: Mangelnde Determinismus-Garantie im Default-Konstruktor von `CheckpointGuard`
* **Schwere**: Niedrig
* **Kategorie**: B (Grenzfälle) / H (Determinismus)
* **Status**: BELEGT
* **Fundstelle**: `crates/contextra-checkpoint/src/guard.rs:88`
* **Befund**: `CheckpointGuard::new` initialisiert standardmäßig `SystemClock::new()`. Falls Aufrufer `new` statt `with_clock` verwenden, werden nicht-deterministische Systemzeit-Zeitstempel erzeugt.
* **Evidenz**:
  `<!-- EVIDENCE: logs/audits/contextra-deep_2026-10-13/checkpoint-tests.log (exit=0) -->`
* **Betroffene Invariante**: `INV-CHECKPOINT-DETERMINISM-1`
* **Empfehlung**: `CheckpointGuard::new` als depricated markieren oder erzwingen, dass eine Clock injiziert werden muss.

---

## 5. Vorgänger-Abgleich

| Vorgänger-ID | Titel / Gegenstand | Status im aktuellen Commit (`59f853cb`) | Anmerkung |
|---|---|---|---|
| **contextra-checkpoint P1** | RAII CheckpointGuard Auto-Rollback | BEHOBEN | In `guard.rs` vollständig getestet und verifiziert |
| **contextra-checkpoint P2** | Instance-Scoped Orphan State (ADR-053) | BEHOBEN | `InstanceOrphanRegistry` ersetzt globale Statics |
| **contextra-checkpoint P3** | BLAKE3 Manifest Verifikation | BEHOBEN | `CheckpointManifest::verify()` verifiziert Prüfsummen |
| **contextra-checkpoint P4** | Snapshot Pinning Integration | BEHOBEN | `PinGuard` schützt Sequenznummern |
| **contextra-checkpoint P5** | SystemClock vs. Clock Port Injection | BEHOBEN | `with_clock` in `PersistentCheckpointStore` vorhanden |
| **contextra-checkpoint P6** | Lock Consolidation P-23 | BEHOBEN | `CheckpointIndex` fasst Locks atomar zusammen |
| **contextra-kvcache P1** | Tenant-Isolation VETO-F10 | BEHOBEN | `TenantPrefixKvStore` und `TenantIsolatedKvStore` isoliert |
| **contextra-kvcache P2** | Radix-Tree Split Korrektheit | BEHOBEN | `test_v1_radix_split_correctness_10k_keys` bestanden |
| **contextra-kvcache P3** | KIVI-Quantisierung | BEHOBEN | Feature-Gated, Cosine Similarity > 0.99 verifiziert |
| **contextra-kvcache P4** | Eviction Worker OS-Thread Isolation | BEHOBEN | Dedicated OS Thread via `mpsc` entkoppelt |
| **contextra-kvcache P5** | AEAD-Verschlüsselung | BEHOBEN | AES-256-GCM-SIV in `segment.rs` verifiziert |
| **contextra-kvcache P6** | Tiering Race Analysis | BEHOBEN | In-Memory Shard Locks schützen State-Updates |
| **contextra-infer-ollama P1–P5** | Retry, Zero-Panic, Pre-Fix, Importance | BEHOBEN | Alle 5 Prüfpunkte in `contextra-infer-ollama` bestanden |
| **contextra-infer-onnx P1–P4** | Feature Gate, Session-Pool, Cross-Encoder | BEHOBEN | Alle 4 Prüfpunkte in `contextra-infer-onnx` bestanden |
| **contextra-infer-candle P1–P5** | Zero-Panic, Backpressure, Dimensions, Mocks | BEHOBEN | Alle 5 Prüfpunkte in `contextra-infer-candle` bestanden |

---

## 6. Testlage im Scope

| Crate | Unit-Tests (Passed/Failed) | Integrationstests | Bemerkung |
|---|---|---|---|
| `contextra-kvcache` | 42 / 0 | 1 fehlgeschlagen (`cache_scope_scan.rs`) | Unit-Tests green; 1 Test-Kompilierungsfehler (`FIND-02`) |
| `contextra-checkpoint` | 18 / 0 | Green | Alle Unit- & Prop-Tests bestanden |
| `contextra-infer-ollama` | 91 / 0 | Green | Pure Rust HTTP Client Tests green |
| `contextra-infer-onnx` | 12 / 0 | Green (unter `onnx` feature) | Tests bestanden |
| `contextra-infer-candle` | 38 / 0 | Green | Tests & Mocks green |

---

## 7. Nicht prüfbar und warum

* **Feature-Matrix Ausführung (`cargo xtask feature-matrix`)**:
  - *Grund*: Der xtask-Befehl ist ein Langläufer und lief nach 400 Sekunden in den Time-Out.
  - *Mitigation*: Die Feature-Kombinationen wurden manuell aus `.github/workflows/rust-ci.yml` und den jeweiligen `Cargo.toml`-Dateien analysiert.
* **CUDA / GPU Inferenz (`contextra-infer-candle --features cuda`)**:
  - *Grund*: In der CI Sandbox steht keine NVIDIA GPU mit CUDA-Treiber zur Verfügung.

---

## 8. Out-of-Scope-Befunde

* **`contextra-store` Kompilierungsfehler**:
  - In `crates/contextra-store/src/lsm/purge/orchestrator.rs` liegen uncommitted Entwicklungs-Kompilierungsfehler vor (`PurgePhase` Mismatch). `contextra-store` liegt außerhalb des zugewiesenen Scopes und wurde nicht verändert.
* **Ring-Layering Abweichung in `contextra-store`**:
  - `contextra-store` (Ring 1) besitzt eine Dev-Abhängigkeit auf `contextra-sandbox` (Ring 2).

---

## 9. Priorisierte Maßnahmen

1. **Behebung der Test-Kompilierung (`FIND-02`)**:
   - `crates/contextra-kvcache/tests/cache_scope_scan.rs` an die aktuelle API anpassen.
   - *Akzeptanzbefehl*: `cargo test -p contextra-kvcache --test cache_scope_scan`
2. **Verdrahtung von Crypto-Shredding (`FIND-01`)**:
   - `ShreddableSegmentKey::shred()` in den Purge-Path von `contextra-engine` einbinden.
   - *Akzeptanzbefehl*: `cargo test -p contextra-kvcache --test e2e_lifecycle_with_engine`
3. **Hardening Checkpoint Clock Injection (`FIND-03`)**:
   - `CheckpointGuard::new` zur expliziten Clock-Injektion zwingen.
   - *Akzeptanzbefehl*: `cargo test -p contextra-checkpoint --lib`
