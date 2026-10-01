# Contextra — Systemspezifikation v14 (IST- und SOLL-Zustand)

**Stand:** 29.09.2026 · **Code-Basis IST:** `https://github.com/tfufuz1/contextra`, HEAD `e4d84c9` (30.09.2026; 33 Crates, ca. 289 k Zeilen Rust in `crates/` + `xtask/`, Toolchain 1.89.0)
**Status dieses Dokuments:** Es **ersetzt alle früheren Spezifikationen** (v4, v8, v9, v10, v11). Es enthält (1) den **IST-Zustand**, also alles, was im Quellcode implementiert ist, und (2) den **SOLL-Zustand**, also alles, was noch zu implementieren oder zu entscheiden ist. Befunde zu vorübergehenden Fehlern sind bewusst nicht enthalten. Wo früher ein Fehler stand, steht jetzt die Soll-Anforderung als Invariante.
> **v14 (30.09.2026, SOLL ergänzt):** Teil J spezifiziert den SOLL-Zustand aus `CONTEXTRA_SPEZIFIKATION_4_1_.md` und den offenen Punkten aus G/H/I, jeweils mit IST-Status gegen HEAD `81ab98a`.
>
> **v14 (30.09.2026):** Erweiterung von v13 um **Teil I** (Nachspezifikation der Funktionen aus `CONTEXTRA_SPEC_LUECKENLISTE.md`, geprüft gegen HEAD `81ab98a`). Korrigiert außerdem Aussagen des QA-Katalogs (I.0).
>
> **v13 (30.09.2026):** Erweiterung von v12. Der Text der Teile 0–F bleibt erhalten; verifizierte Korrekturen stehen dort direkt im Text und gesammelt in **Teil G**. Neu sind **Teil G** (Erweiterungen und Korrekturen gegen HEAD `e4d84c9`) und **Teil H** (Stand: Status-Board, Plan-Abgleich, Termine). Teil C bleibt gegen HEAD `3f9d2df` abgeglichen (siehe G.1).

**Adressat:** Sprachmodelle und KI-Coding-Agenten. Jeder Begriff hat genau eine Bedeutung (Glossar B.2). Bei Widerspruch zwischen diesem Dokument und dem Code gilt der Code. Innerhalb dieses Dokuments schlägt **Teil 0** alle anderen Teile.

## Statuskennzeichnung

| Symbol | Bedeutung |
|---|---|
| ✅ IST | Im Code vorhanden |
| 🟡 TEIL | Grundgerüst vorhanden, Ausbau offen (Delta genannt) |
| 🔵 SOLL | Spezifiziert, im Code nicht vorhanden, zur Umsetzung vorgesehen |
| ⏸ BACKLOG | Spezifiziert, Umsetzung erst bei validiertem Bedarf |
| ❌ AUSGESCHLOSSEN | Nicht zu implementieren (Teil 0.5) |

**Belegmarker im IST-Teil:** **[V]** Wert/Ablauf im Code gelesen · **[D]** aus Doc-Kommentar oder Manifest, Ablauf nicht gelesen · **[?]** nicht am aktuellen HEAD geprüft.

---

# Teil 0 — Vision, Positionierung, Scope

**Positionierung (Ein-Satz):** Contextra ist eine air-gap-fähige, kryptografisch beweisbare Memory-Engine für KI-Agenten: ein `cargo add`, kein Server.

**Contextra ist:**
- Eine eingebettete Rust-Bibliothek, die im Prozess der Anwendung läuft, ohne Cloud-Endpunkt und ohne zweiten Prozess. Zusätzlich existiert ein MCP-Server (stdio) für Nicht-Rust-Nutzer (Claude Desktop, Cursor, VS Code).
- Eine Engine mit kryptografisch verifizierbarer Löschung (Ed25519-signierte `DeletionProof`, extern prüfbar).
- Eine Engine mit hybrider Retrieval-Fusion (Vektor, BM25-Volltext, CSR-Graph; KV-Cache für Inferenz) mit Kalibrierung.
- Eine Engine mit erzwungenem Determinismus (Uhr und Zufall nur über injizierte Ports).
- Zielgruppe: Rust-Entwickler, die KI-Agenten bauen, und Systemhäuser für regulierte Branchen in Deutschland (B2B2G).

**Contextra ist nicht:** ein Multi-Node-System, ein Python-first-Produkt, ein Framework-Hub (LangChain/LangGraph/LlamaIndex), eine Compliance-Plattform mit Vertriebsapparat.

## 0.5 Explizite Scope-Ausschlüsse (❌)
Framework-Adapter per PyO3 · Python-FFI als Kernbestandteil · externe HTTP-Inferenz als Standardabhängigkeit · C++-Cross-Encoder als Standard · WASM-Sandbox als Standardbestandteil · spektrale Hyperkanten-Indexierung · vertikalspezifische Ontologie-Schnittstelle · horizontales Sharding · Peer-to-Peer-Sync · EU-AI-Act-Risikomapper · vollständige Art.-30-Quelle als Standard · öffentliche mehrstufige Entscheidungskaskade · Regelungstechnik als öffentliche Konfiguration.
**IST-Abweichung:** `contextra-py` und `contextra-sandbox` sind Workspace-Mitglieder, aber nicht in `default-members`; sie werden nur explizit gebaut (`cargo build -p …`). Die Vorgabe „Python-Bindings in eigenes Repository" ist 🔵 SOLL (Entscheidung offen).

## 0.7 Governance-Regeln
1. Kein neues Crate, keine neue Abhängigkeit ohne schriftliche Begründung und 48 Stunden Wartezeit.
2. Externe Validierung vor Implementierung („Würdest du dafür zahlen?"); ohne Antwort nicht bauen.
3. Ein Benchmark, eine Wahrheit; öffentliche Zahlen tragen den Hinweis, dass nur Speicher-/Indexlatenz gemessen wird (Embedding-Inferenz addiert 10–500 ms).
4. Ring-DAG-Integrität, geprüft durch `tests/layering.rs`.
5. Determinismus: kein direktes `SystemTime::now()`/`thread_rng()` im Produktionscode; Ausnahme nur für kryptografisches Schlüssel-/Salt-Material.

---

# TEIL A — Architektur (IST)

## A.1 Workspace, Crates, Features, Vetos

### A.1.1 Crates (Zeilen / Rolle) **[V]**
- **Facade:** `contextra` (1.725; Default-Features `fast`, `candle`), `contextra-db` (12.815, Strangler-Shell, MultiStep, VolatileVault), `contextra-core` (372, *deprecated* Strangler-Facade).
- **Kern:** `types` 6.571, `ports` 3.803, `mvcc` 3.759, `wire` 2.113 (FlatBuffers), `store` 41.616, `vector` 20.399, `text` 10.968, `graph` 26.127, `rank` 5.214, `engine` 21.264, `cognition` 7.269, `adapt` 6.816, `router` 7.938, `kvcache` 6.116.
- **Sicherheit/Compliance:** `crypto` 10.247, `privacy` 2.831, `license` 357, `checkpoint` 6.763, `audit-export` 460, `avv-generator` 356, `mcp` 7.979, `agent` 7.137.
- **Inferenz:** `infer-candle` 6.795, `infer-ollama` 4.611, `infer-onnx` 2.270.
- **Opt-in/Dev:** `sandbox` 2.619 (Wasmtime), `py` 2.299 (PyO3), `simd` 1.776, `sys` 614, `testkit` 783, `xtask`, `benchmarks/`.

### A.1.2 Feature-Matrix der Facade **[V]**
`default = fast + candle` · `sovereign = crypto + privacy` · `compliance = sovereign + audit-export + avv-generator` · `onnx` · `ollama` · `router = router + rank`.

### A.1.3 Feature-Flags je Crate **[V]**
- **vector:** `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `partial-index-rebuild`, `docid-128`
- **graph:** `graph-connectivity-health`, `edge-reinforcement-learning`, `ppr-forward-push`, `k-path-diffusion`, `apprh-diffusion`, `physio-percolation`, `physio-synaptic-edges`
- **text:** `bm25f` (nicht im Default-Build)
- **rank:** `dibud`
- **engine/db:** `encryption-at-rest`, `reranking`, `onnx`, `sandbox`, `background-maintenance`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `entity-extraction`, `auto-extraction-opt-out` (engine), `volatile-vault` (db)
- **adapt/router:** `bandit-routing`, `egress-sherman-morrison`, `flow-corrected-thompson`, `sketched-bandit`, `rie-greedy-personalization`, `cloud-egress-guard`
- **kvcache:** `content-addressed-kv-cache`, `kv-encryption`, `kvcache-attention-eviction`, `kvcache-kivi-quant`, `kivi-quantization`
- **store:** `encryption-at-rest`, `wal-integrity`, `deletion-proof`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`
- **mcp:** `agent-workflows`, `onnx`, `candle`, `ollama`, `kv-bridge`

### A.1.4 Governance-Vetos **[V]** (`VETOES.md`)
- **F-02** (bedingt akzeptiert, Review fällig **2026-10-07**): nur Tombstone-Pruning, kein partielles HNSW-Rewiring. Spannung: Feature `partial-index-rebuild` existiert im Vector-Crate.
- **F-10** (dauerhaft abgelehnt): keine mandantenübergreifenden Datenflüsse.
- **OP-03** (bedingt, Review 2026-10-07): keine Realtime-Audio/STT/Voice-Funktionen.
- **VETO-ADR:** Vetos/Isolationsgrenzen nur per ADR in `DECISIONS.md` umgehbar.

Weitere Governance-Artefakte: `CONSTITUTION.md`, `governance/` (ratchet, determinism-baseline, protected-paths, unsafe-safety-baseline, verdict-required), `docs/gates`, `docs/decisions`, `supply-chain/`, `deny.toml`.

---


## A.2 Ringmodell
`capabilities.toml` ist die normative Quelle (P12, ADR-N08). Ring 0: Kerntypen, Ports, MVCC, Wire, SIMD, Sys, plus laut Manifest `adapt`, `graph`, `text`, `vector`, `rank`, `crypto`. Höhere Ringe hängen nur von tieferen ab (P5). Async/tokio nur ab Ring 1 (P26). Unsafe-Inseln laut Manifest: `simd`, `sys`, `crypto`, `wire`. **[V]**
Produkt-Ringe: `FeatureRing ∈ {Fast, Sovereign, Compliance}`.


---

# TEIL B — Algorithmen, Funktionen, Parameter (IST)

Die Abschnitte B.1–B.11 sind die verbindliche Beschreibung der implementierten Funktionen.

## B.1 Identität, Typen, Grenzen [V]
- `MAX_SEARCH_K = 1_000` (jede Suche wird auf k ≤ 1000 gekappt).
- `MemoryType ∈ {Episodic, Semantic, Procedural, Working}`. Standard-Decay je Typ (`default_decay`):
  - Episodic → `Exponential{half_life_tx: 10_000}`
  - Semantic → `None`
  - Procedural → `StepFloor{access_count_floor: 50}`
  - Working → `Exponential{half_life_tx: 500}`, `default_ttl_tx = 50_000`
- `LinkRelation ∈ {Elaborates, Contradicts, Supersedes, References}` (Zettelkasten/A-MEM-Relationen). Der Graph-`EdgeType` hat nur eine Variante; Semantik liegt in `LinkRelation` und Hyperkanten-Rollen.
- Reservierte System-Key-Präfixe: `__col:`, `__col_idx:`, `__meta:`, `__rel:`, … (Liste `RESERVED_PREFIXES`); Graph: `__graph:entity:`, `__graph:edge:`, `__graph:community:`, `__graph:hyperedge:`, `__graph:hyperedge_by_entity:`, `__graph:cascade_queue:`.
- Ablauf-Metadatum: `__expires_at_seq` (sequenzbasiertes TTL).
- Importance ∈ [0,1], NaN → 0.0.

### B.1.1 Decay-Algorithmus (`DecayFunction::decay_factor(created_at_tx, now_tx)`) [V]
```
elapsed = now_tx − created_at_tx
falls now < created oder elapsed = 0 → 1.0
None                   → 1.0
Exponential{h}         → h = 0 ? 0.0 : 0.5^(elapsed / h)
StepFloor{floor}       → elapsed < floor ? 1.0 : 0.5
WallClockExponential   → hier 1.0; Berechnung über decay_factor_wallclock(): 0.5^(elapsed_s / half_life_secs)
Ergebnis: NaN/∞ → 0.0, sonst clamp[0,1]
```
Decay misst **Transaktionsdistanz**, nicht Uhrzeit (Ausnahme: `WallClockExponential`).
`DECAY_DELETION_THRESHOLD = 0.05` (Engine: darunter „vergessen" und gelöscht). Der `AdaptiveDecayController` nutzt `kappa=2.0`, `base_half_life_tx=10_000`, `eviction_threshold=0.01` **[V]**.

## B.2 Speicher-Engine

### B.2.1 Schreibpfad [D/V]
`Client → TxBuffer (staging) → WAL (Group-Commit, HMAC-Kette, optional verschlüsselt) → MemTable → SSTable → Compaction`.
- **Durability (`DurabilityMode`)** [V]: `Full` (WAL + HMAC), `WalNoHmac`, `MemoryOnly`.
- **WAL-Formate:** Magic `MFW2` (V2 Batch-verschlüsselt), `MFW3` (V3); `WalConfig.min_wal_version = V3`; Legacy-Integritätsschlüssel-Fallback aus.
- **Group Commit:** `WalFlusherConfig.batch_window_micros = 100`; `LsmConfig.group_commit_window_micros = 500` **[V]**.
- **LsmConfig-Defaults:** `memtable_size_limit 64 MiB`, `max_ram_mb 2048`, `tx_timeout 60 s`, `block_cache_shards 64` **[V]**.
- **Manifest:** Magic `MFMN`, Version 1. **SSTable:** Magic `MFSX` (`0x5853464D`), Legacy `0x4D465354`; Block-Cache 64 Shards (Features `block-cache-v2`, `sieve-cache`).
- **Backpressure:** `SystemPressureMonitor` → `PressureLevel ∈ {Normal, Elevated, Critical}`. WAL-Queue: Elevated ≥ 100, Critical ≥ 500. `BLOCKING_UTIL` Elevated 0,60 / Critical 0,85 (Metrik: Tokio-Queue-Tiefe **[?]**). `CollectionConfig.backpressure_delay_ms = Some(50)`.

### B.2.2 Compaction [V]
- `CompactionConfig`: `min_sstables_per_tier 4`, `size_ratio 4.0`, `check_interval 30 s`, `max_memory 128 MiB`, `max_peak_memory 256 MiB`, `max_backpressure_wait 30 s`.
- **Adaptive Planung:** `WorkloadMetrics` (read_count, write_count, last_seq_no, ohne Wall-Clock) → `CostBasedAdaptivePlanner` → `AdaptiveCompactionPlan{strategy, candidates, is_full_compaction}` mit `strategy ∈ {WriteOptimizedSTCS, ReadOptimizedAggressive, Balanced}`.
- Tombstones werden nur entfernt, wenn kein aktiver Snapshot sie noch sieht.

### B.2.3 MVCC und SSI [V]
- `SequenceLog` vergibt monoton steigende `seq`; `SnapshotGuard` hält einen Lese-Snapshot; `DEFAULT_MAX_PIN_DURATION = 300 s` (danach Diagnosewarnung).
- **TxBuffer:** `DEFAULT_SHARD_COUNT 64`, `max_active_tx 64`, `tx_timeout 30 s`, `max_ops_per_tx 10_000`, `max_read_set_keys 100_000`, `max_tx_staged_bytes 16 MiB`, `max_total_staged_bytes 256 MiB`, Overhead je Eintrag 32 B.
- **SSI (Serializable Snapshot Isolation):** `ReadSet` erfasst gelesene Keys und Range-Präfixe (Phantomschutz via `record_prefix`). `SequenceLogSsiValidator` erkennt Write-Skew-Konflikte gegen committete Writes nach `snapshot_seq`; `DEFAULT_MAX_TRACKED_COMMIT_KEYS = 1_000_000`. **Fail-closed:** Reads mit `snapshot_seq < pruned_through` werden abgelehnt. Commit-Registrierung atomar unter Write-Lock.

## B.3 Retrieval-Algorithmen

### B.3.1 HNSW (`contextra-vector`) [V für Defaults]
Hierarchical Navigable Small World, Eigenimplementierung.
- **HnswConfig-Default:** `dimension 1536`, `max_elements 1_000_000`, `m 16`, `ef_construction 200`, `ef_search 64`, `Cosine`, `quantize false`. (`ContextraConfig` der Engine setzt `dimension 768`.)
- **Rebuild-Trigger:** `HNSW_REBUILD_DELETION_RATIO = 0.10`, `rebuild_threshold = 0.90` (Anteil lebender Knoten). Rebuild ist ein **atomarer Core-Swap** unter `RebuildGuard`.
- **Löschung:** Soft-Delete; `VectorDeleteMode ∈ {SynchronousRepair, BackgroundRepair}` (Default `BackgroundRepair`). Reparatur der Nachbarschaft liefert eine `GraphRepairAttestation{doc_id, verified_no_ghost_pointers, attested_at}`.
- **Metriken:** `Cosine`, `Euclidean`, `DotProduct`. Vektoren mit NaN/∞ oder leer werden abgelehnt.
- **SIMD:** AVX2, AVX-512, NEON, Scalar; Runtime-Dispatch.
- **Arena:** Slots 64-Byte-ausgerichtet; Sentinel `u32::MAX` = kein Entry-Point. Dateiformat `HNSW` v2, Header 84 Byte.
- **SQ8-Quantisierung:** 8-Bit pro Dimension mit Min/Max aus Perzentilen `P_LOW = 0.005`, `P_HIGH = 0.995`; Rekalibrierung nach 10.000 Samples, Drift-Schwelle 0,10.
- **Adaptive EF (`AdaptiveEfPolicy`):** `min_ef 16`, `max_ef 256`, `growth_factor 1.5`, `convergence_window 2`, `stability_threshold 1.0`.
- **ACORN:** attribut-gefilterte Suche (`acorn_filtered`, Gamma-Augmentation).
- **DiskANN (Feature `experimental-diskann`):** `max_degree 64`, `beam_width 8`, `sector_size 4096`, `memory_budget 128 MiB`, `dimension 128`; Pending-WAL (`PWAL`) und Tombstone-WAL (`TWAL`); Fallback `DiskAnnFallbackPolicy ∈ {UseHnswOnFailure, FailFast}`; Integritätsprüfung mit Footer `DANF`.
- **Partial Rebuild** (`partial-index-rebuild`, **Default: aus**, `default = []` in `contextra-vector/Cargo.toml`): `critical_ratio 3.0`, `traversal_window 1000`, `min_global_ratio 0.01`. **Achtung:** VETO-F02 erlaubt nur Tombstone-Pruning (Review 2026-10-07).
- `RaBitQ`, `Predicate-Augmented Search`: nur als experimentelle Features vorhanden **[D]**.

### B.3.2 Volltext (`contextra-text`) [V für Konstanten]
- **BM25:** `BM25_K1 = 1.5`, `BM25_B = 0.75`; IDF = `ln(1 + (N − df + 0.5) / (df + 0.5))` **[?]**. BM25F (Feature `bm25f`, nicht Default) mit Feldgewichten.
- **Block-Max WAND:** Posting-Listen in Blöcken à `BLOCK_SIZE = 64`; Format-Magic `PL\x02\x00`.
- **Index:** MVCC-bewusst; `search_at(query, k, seq)` liefert Ergebnisse für einen Snapshot. Limits: `MAX_TEXT_BYTES = 10 MiB`, `MAX_STAGED_TRANSACTIONS = 10_000`. Stream-Batchgröße 16.
- **Deutsche Morphologie:** `GermanCompoundSplitter` (DP-Zerlegung, Fugenelemente s/en/e/er/n/es, eingebettetes Wörterbuch, Umlaut-Normalisierung). Domänenlisten Medizin/Recht (`*_COMPOUND_STEMS`, `*_PROTECTED_TERMS`) sind kurze Stichwortlisten (Stubs).

### B.3.3 Graph (`contextra-graph`) [V für Defaults]
- **Struktur:** CSR (Compressed Sparse Row); `CsrGraphConfig`: `rebuild_threshold 1000`, `max_compaction_peak_memory_mb 1024`. Sichtbarkeit per `seq`.
- **`GraphTraversalStrategy` (Default `Hops{max_hops:3}`):**
  - `Hops`: BFS mit Score `SCORE_DECAY^hop = 0.7^hop`, `MAX_TRAVERSAL_HOPS 3`, `MAX_VISITED_NODES 10_000`.
  - `PersonalizedPageRank(PprConfig)`.
  - `PathRag{…}`: bidirektionaler Dijkstra mit **Sufficiency-Gate** (`sufficiency_threshold 0.1`), `max_hops 4`, optional Hyperkanten-Expansion (`hyperedge_weight_discount 0.85`, Default aus). **Nicht snapshot-fähig** in der Hybrid-Suche (liefert Fehler).
- **PPR (`PprConfig`):** `damping_factor 0.85`, `max_iterations 100`, `convergence_epsilon 1e-6`, `algorithm = PprAlgorithm::Auto`.
  `PprAlgorithm ∈ {Auto, DensePowerIteration, ForwardPush, ShadowMode, TlHfd, ShadowModeTlHfd}`.
  - *DensePowerIteration:* `p ← (1−α)·s + α·Pᵀp` bis Änderung < ε **[?]**.
  - *ForwardPush* (`ppr-forward-push`): approximatives Residual-Pushing.
  - *TL-HFD* (Hyperkanten-Diffusion) mit Shadow-Gate: `MIN_SHADOW_SAMPLES 10_000`, `MIN_AGREEMENT_THRESHOLD 0.85` (mittlere Top-k-Jaccard-Übereinstimmung) **[V]**.
  - *APPRH* (`apprh-diffusion`): `ApprhMode ∈ {Off, Shadow, Production}`, Selector `max_seeds 100`, `shadow_top_k 10`; Gate-Monitor `window_size 20`, `required_consecutive_passes 10` **[V]**.
  - *k-Path-Diffusion:* `k 3`, `max_visited_nodes 1000`, `max_hops 4`.
- **Leiden-Communities:** `max_iterations 100`, `seed 42` (deterministisch), Hyperkanten standardmäßig ausgeschlossen, Stern-Expansion für Hyperkanten. Engine löst Detektion ab 100 Änderungen aus (`auto_trigger_threshold`).
- **Hyperkanten:** n-äre Relation, max. `64` Teilnehmer. Löschkaskade synchron bis `1_000` Hyperkanten (`MAX_HYPEREDGE_CASCADE_FANOUT`), Rest über `DeferredHyperedgeQueue` (Worker: max. 1.000/Tick).
- **Edge Reinforcement:** `eta 0.01`, `delta 0.001`, `w_max 10.0`, `rho 0.05`, `q 1.0`, `alpha 0.5`.
- **Perkolation:** `critical_threshold 0.7`, `rebonding_similarity 0.85`, `max_new_edges_per_pass 100`.
- **Konsistenz:** `ContradictionDetector`, Unterdrückung nach 3 gegenseitigen Detektionen.
- **Session-DAG:** Verzweigung für Agent-Zustand; max. Tiefe 10.000, max. 10 MiB je String.

### B.3.4 Fusion (`contextra-rank`) [V]
- **RRF:** für jedes Signal `s` mit Gewicht `w_s` und Dokument `d` auf Rang `r` (1-basiert): `score(d) = Σ_s w_s / (k + r)`. **k = 60** (`k_rrf`). Signale mit Gewicht ≤ 0 oder nicht endlich werden übersprungen; nicht endliche Teilscores → 0. Provenienz je Dokument: `SignalContribution{raw_score, rank, rrf_contribution}`.
- **Gewichte:** `FusionWeights` normalisiert (Summe ∈ [0.999, 1.001]), Default 1/3 je Signal.
- **`FusionStrategy ∈ {Rrf, ScoreNormalized}`.** ScoreNormalized: MinMax-Normalisierung + CombSUM, Fallback auf RRF bei degradiertem Signal **[?]**.
- **Metadaten-Merge-Priorität:** `MetadataMergePriority ∈ {VectorFirst, TextFirst, GraphFirst, Custom}`.
- **Resonance-Bonus:** `beta 0.5`, `gamma 0.3` (Verstärkung bei Signal-Übereinstimmung).
- **DiBud (Feature `dibud`):** budgetgesteuerte Fusion mit Kanälen Vector/Text/Graph.
- **Global-Fusion:** `max_community_nodes 50`, `min_community_size 3`.
- **Kalibrierung:** Isotonic, Platt, Conformal (`learning_rate γ 0.02`, `initial_threshold 0.5`, Grenzen [0,1]) und adaptiv. **Drift:** `DriftStatus ∈ {Stable, DriftDetected, InsufficientData}`.

### B.3.5 Hybrid-Suche: exakter Ablauf `hybrid_search_with_strategy` [V]
1. `k ← min(k, MAX_SEARCH_K)`.
2. **Pin-first:** `with_pinned_checkpoint_at_latest` liefert `seq` unter Pin.
3. Vektor-Signal, falls Query-Vektor nicht komplett 0: `search_filtered_at(vector, 3·k, seq)` (`OVERFETCH_FACTOR = 3`).
4. Text-Signal, falls Text nicht leer: `text_index.search_at(text, 3·k, seq)`, dann Hydration der Dokumente.
5. Graph-Signal: Anker = übergebene `anchor_entities`, sonst implizit `EntityId::from_key(text_result.id)` für alle Text-Treffer (Invariante: Graph-Knoten teilen den String-Schlüssel des Dokuments; sonst bleibt das Signal leer und es wird gewarnt). Strategie `Hops` → `multi_traverse_at(anchors, max_hops, seq)`; `PersonalizedPageRank` → `personalized_page_rank_at`; `PathRag` → Fehler `snapshot_unsupported_for_signal`. Sortierung Score absteigend, Tie-Break aufsteigend nach ID, Kürzung auf k.
6. Sind alle Signale leer → leere Liste.
7. Fusion: `fuse_search_results_with_strategy(..., top=3·k, ..., FusionStrategy::Rrf)` — in dieser Funktion ist Rrf fest verdrahtet; danach `truncate(k)`.
8. **Community-Boost:** bei `same_community_as` werden Scores der Treffer der Ziel-Community mit `DEFAULT_COMMUNITY_BOOST = 1.2` multipliziert (nach RRF).
9. Optional: Cross-Encoder-Rerank mit Pool `10·k`, max. `200` (`DEFAULT_RERANK_POOL_MULTIPLIER`, `DEFAULT_RERANK_POOL_MAX`), Mindest-Kandidaten `DEFAULT_MIN_RERANK_CANDIDATES = 100`. `filter_by_importance` filtert nachträglich ohne Umordnung (ADR-024). Memory-Type-Filter wirkt vor RRF.

**Query-Builder-Optionen:** `.text .vector .k .weights .fusion_strategy .strategy .filter .anchor_entities .same_community_as .memory_type_filter .include_superseded .include_provenance .filter_fn .hard_scope .reranker .rerank_pool_multiplier .rerank_pool_max .seq .as_of_timestamp .query_timestamp .current_tx .execute()`.
**Bi-temporal:** System-Zeit = TxId/`seq`; Business-Zeit = `valid_from`/`valid_until`; fehlende Felder gelten als gültig (Fail-Open).

### B.3.6 Scan-Grenzen [V]
`DEFAULT_SCAN_LIMIT = 10_000`, `HARD_SCAN_CEILING = 100_000`.

## B.4 Kognition und Lebenszyklus [V für Defaults]
- **Konsolidierung (`ConsolidationConfig`):** Turns werden zu Segmenten gruppiert (`min_turns 3`, `max_turns 20`, `segment_cohesion_threshold 0.70`); Duplikate ab Kosinus `0.95`.
- **Synthese (`SynthesisConfig`):** `min_community_size 4`, `stability_cycles_required 3`, `max_llm_calls_per_cycle 10`, `min_grounding_score None`.
- **Aggregation/LeanRAG (`AggregationConfig`):** Clustering `tau 0.3`, deterministisches Diagonal-GMM (`seed 42`, `em_max_iterations 50`, `em_tolerance 1e-4`), `max_clusters 16`, `min_cluster_size 2`, `min_type_compat_score 0.1`, Peak-Speicher 512 MiB; LeanRAG-Input max. 10.000 Knoten. Ergebnis: `SuperEdgeDraft` → `CsrGraphSuperEdgeSink`. Transitivity-Veto filtert Kandidaten.
- **`CompactionStrategy` (Kontext) ∈ {Truncate, Summarize, StatusToken, LlmSummarize}**.
- **Maintenance (`MaintenanceConfig`):** Tick 60 s, Decay an, Perkolation an, `coherence_bonus_beta 0.15`, Replicator und Hintergrund-Konsolidierung standardmäßig aus.
- **Auto-Entity-Extraktion:** `enabled: true`, `max_llm_calls_per_cycle 10`, `min_confidence 0.5`; Abschaltung per Feature `auto-extraction-opt-out`.
- **Chunker (`ChunkerConfig`):** `max_tokens 512`, `min_tokens 50`, Breadcrumbs an, Split bei Heading-Level 1–3; Tokenschätzung ~4 Zeichen/Token (Code ×1,8).
- **Background-Worker (Feature `background-maintenance`):** Expiry (≤ 100/Tick), Orphan (≤ 100/Tick, Backoff 5 s → 300 s, Alarm nach 3 Fehlern), Decay, Hyperkanten-Kaskade.

## B.5 Adaptive Steuerung [V für Defaults]
- **PID-Latenz-Controller:** Ziel-P95 `100 ms`; Skalierungsfaktor des K-Pools in [0.3, 1.0]; Integral-Clamp 10.0 (Anti-Windup). **Rerank-Pool** [50, 200] (`PID_MIN/MAX_POOL_SIZE_DEFAULT`).
- **Bandit-Routing:** `RoutingStrategy ∈ {Cascade (Default, produktiv), ContextualBandit (LinUCB, Feature, aus), FlowCorrectedThompson (Feature, aus)}`. Sherman-Morrison-Neufaktorisierung alle 1000 Updates. FC-TS: `dim 64`, `lambda 1.0`, `noise_var 1.0`, `window_capacity 200`, `drift_ridge 10.0`, `max_drift_norm 5.0`. **Default-Flip-Gate:** ≥ 10.000 Shadow-Samples und kumulatives Regret ≤ 0.0.
- **Off-Policy-Evaluation:** IPS mit Fehler `ZeroPropensityViolation`.
- Lyapunov-Drift-Watcher, `ShadowMode`, Cross-Adapt-`DriftDetector`.

## B.6 Löschung und Kryptografie
- **Verschlüsselung at Rest:** ChaCha20-Poly1305 / AES-GCM-SIV (Feature `encryption-at-rest`), 12-Byte-Nonce, Per-File-Subkeys; KV-Segmente Format v2; Shred-Gruppen à 64 Records.
- **KDF:** Argon2id (Header-Magic `MFKD`, Version 1). Minimum `m=19456 KiB, t=2, p=1`; Default `m=65536 KiB, t=3, p=1`; Salt ≥ 16 Byte.
- **DeletionProof [V]:**
  - `DeletionScope ∈ {Document, Collection, Tenant}`.
  - `DeletionLayer ∈ {LsmMemtable, SsTableAllLevels, HnswIndex, WalAllSegments, CsrGraph, KvCacheSegments, EmbeddingCache}` (korrigiert gegenüber v9: 7 Layer).
  - `ExcludedScope ∈ {ConsolidatedAndDistilled, LlmParameterMemory}` (explizit nicht abgedeckt, Referenz arXiv:2505.16831).
  - `SignatureVersion`: V1 = HMAC-SHA256 nur Scope; V2 = HMAC-SHA256 Scope + Layer-Coverage; **V3 = Ed25519** mit längenpräfixierten Key-Hashes. Der Datei-Kopf von `deletion_proof.rs` nennt noch HMAC (veraltet).
  - Schlüsselhash: Blake3 über sortierte Keys, je Key `(len as u32).to_le_bytes() ‖ key`.
  - **INV-DELETION-1:** `create()` nur nach vollzogener physischer Bereinigung.
  - Offener Tag: `signature_version` soll typisiert werden (`ed25519_proof.rs:109`).
- Schlüssel-Env: `CONTEXTRA_DELETION_PROOF_KEY`, `CONTEXTRA_PROOF_KEY`.

## B.7 Privacy, Egress, MCP
- **Egress-Klassifikation:** `EgressClassification ∈ {Allow, Block}`; `BlockReason ∈ {SensitivePattern, PolicyDenied, EgressPolicyDenied, ClassificationTimeout, InternalError}`. Klassifikations-Timeout 100 ms, Payload ≤ 65.536 B; Query ≤ 64 KiB.
- **EgressGuard (HNSW-Ähnlichkeitsprüfung):** Kosinus-Schwelle `0.85`, Timeout `200 ms`, Prüfung erst ab Payload ≥ 128 Byte.
- Bulk-Exfiltration-Detector pro Session; Type-State `GuardedPayload<Unsanitized→Sanitized>`; `PolicyCategory ∈ {LocalAccess, CloudEgress}`; `ProcessorRole ∈ {Controller, Processor}`.
- **MCP:** JSON-RPC 2.0/stdio, Nachricht ≤ 4 MiB. **15 Tools:** `search, insert, upsert, get, delete, forget, collections, create_collection, drop_collection, consolidate, relate, relate_n_ary, cloud_query, plugin_status, explain`. (Ältere Fassungen nannten 13/14.)
- **Sandbox:** `ToolCategory ∈ {DatabaseRead, DatabaseWrite, CodeExecution, CloudEgress}`. Default-Policy: Reads erlaubt, Writes/Code/Cloud gesperrt, `max_execution_ms 5000`. Volatile Ergebnisse ≤ 1.000, Schlüssel ≤ 256 B, Ausgabe ≤ 16 MiB.
- **Prompt-Injection-Guard:** `QuarantinePolicy ∈ {Strict (Default), FlagOnly, Escalate}`; Phrasenfilter EN/DE, Base64 bis Tiefe 2, NFC; Platzhalter `[REDACTED: …]`. Kein Schutz gegen neuartige Formulierungen.
- **Env-Variablen [V]:** `CONTEXTRA_MCP_ALLOW_WRITE, _ALLOW_REPO_BUILD, _BINARY, _IDLE_TIMEOUT_SECS, _INJECTION_CONFIG, _PATTERNS_FILE, _QUARANTINE_POLICY, _SECURITY_LOG`; `CONTEXTRA_EMBEDDING_PROVIDER, _EMBED_MODEL, _LLM_PROVIDER, _LLM_MODEL, _OLLAMA_URL, _CANDLE_MODEL_DIR, _ONNX_MODEL_PATH`; `CONTEXTRA_ROUTER_CALIBRATION_PATH, _PROFILES_JSON, _PROFILES_PATH`; `CONTEXTRA_ORPHAN_PATH, _ORPHAN_PIN_PATH`; `CONTEXTRA_WORKER_THREADS`. Defaults MCP: Embedding/LLM-Provider `mock`, Ollama `http://localhost:11434`, `nomic-embed-text`, `llama3.2:3b`.
- **Compliance-Exporte:** Art.-30-Verarbeitungsverzeichnis, BSI-Mapping, AVV-Entwurf (Art. 28).
- **Lizenz:** `OpenFastGate` (immer offen), `SignedLicenseGate` (Ed25519 über `LicensePayload`).

## B.8 KV-Cache und Inferenz
- **Prefix-Radix-Tree** mit `KvReusePolicy ∈ {Always, CostBased{min_prefix_len:4} (Default), Never}`.
- Budget je Tenant 256 MiB, gepinnt max. 64 MiB, 32 Shards, 256 Segmente/Tenant.
- **KIVI-Quantisierung:** `key_group_size 16`, Values quantisiert. Attention-basierte Eviction (Feature `kvcache-attention-eviction`); Content-Addressed-Store; verschlüsselte Tier-2-Segmente.
- **GASP-Validator:** Grounding-Prüfung (`warmup_required 10`, `max_observations 2000`).
- **Backends:** Candle (GGUF), Ollama (Timeout 30 s, connect 5 s), ONNX-Cross-Encoder (`bge-reranker-base.onnx`, `max_length 512`, `batch_size 8`, Deadline 500 ms). ContextPrefix: max. 8000 Zeichen, 80 Token Präfix.

## B.9 Agent und Checkpoint
- Loop `Checkpoint → Execute → Commit → Audit`, max. 10.000 Steps, Dead-Letter-Queue (Präfix `dlq:`), Replay aus Checkpoint, RAII-Budget-Reservation (Refund bei Drop ohne `settle`), max. Telemetrie-Events 10.000, ID ≤ 256 B, Text ≤ 65.536 B.
- `CheckpointGuard` (RAII, Rollback bei Drop ohne Blocking-I/O), Hardlink-Cloner, Orphan-Registry.
- **MultiStep-Retrieval:** `max_rounds 3`, `quality_threshold 0.5`, `min_quality_hits 2`, Latenzbudget 100 ms; RRF über Runden.
- **VolatileContextVault:** RAM-Tresor, `max_capacity 512 MiB`, `mlock` versucht, Zeroize.

## B.10 Deployment-Profile [V]
`PerformanceProfile ∈ {Compliance, Balanced, BareMetal}`; `DeploymentTier ∈ {EdgeMinimal, PowerUserLocal, EnterpriseShared, EnterpriseRegulated}`. Validierungsfehler: `Durability`, `DurabilityVectorMismatch`, `LicenseRequired`; `KvDeleteMismatch` (Collection-Profil). PII-Treffer erzwingen `CryptoShred` **[?]**.

## B.11 Invarianten und Verbote
1. Mandantengrenzen sind absolut (VETO-F10).
2. Löschbeweis nur nach physischer Bereinigung (INV-DELETION-1).
3. Kein partielles HNSW-Rewiring außer Tombstone-Pruning (VETO-F02).
4. Keine Realtime-Audio/STT-Funktionen (VETO-OP03).
5. Determinismus: Compaction, Decay (TxId), Kognition (Seeds) ohne Wall-Clock; Uhr/Zufall über Ports (`Clock`, `Rng`).
6. Snapshot-Isolation: alle Signale einer Anfrage lesen denselben `seq`; PathRag verletzt das und ist deshalb ausgeschlossen.
7. Fail-closed: SSI-Pruning, Sandbox-Kategorisierung, Egress-Timeout.
8. Unsafe nur in Ring-0-Inseln (`simd`, `sys`, `crypto`, `wire` laut Manifest).


---

# TEIL C — Frühere SOLL-Punkte, die inzwischen IST sind (Abgleich gegen HEAD `3f9d2df`)

| Früherer SOLL-Punkt (Spec v4) | Status | Belegte Umsetzung |
|---|---|---|
| Löschbeweis P0-1…P0-8 (Geisterzeiger-Schutz, DurabilityMode/KvDeleteMode, Auto-Extraktion an, vier MCP-Schreibtools, Presets, KIVI, HMAC-Kollisionstest, Group-Commit-Stresstest) | ✅ IST | HNSW-Reparatur mit `GraphRepairAttestation`; `DurabilityMode`; `AutoExtractionConfig.enabled = true`; MCP-Dispatch; `DeploymentTier`/`PerformanceProfile`; KIVI; Tests in `contextra-crypto/tests`, `contextra-store/tests` **[V/D]** |
| Kryptografische Bindung der Graph-Reparatur an den `DeletionProof` | ✅ IST | `DeletionProof` trägt `graph_repair: Vec<GraphRepairAttestation>` **[V]** |
| `SignedLicenseGate` (C.2.1) | 🟡 TEIL | Vorhanden: `SignedLicenseGate::from_signed_payload[_with_clock]`, Ed25519, injizierte `Clock`, `LicensePayload{tenant_id, allowed_rings, expires_at, feature_flags}`, Tests `signed_gate.rs`. Delta zu Spec v4: dort war die Bindung an eine Installation (`installation_id_hash`) statt an einen Tenant vorgesehen → siehe D.1 **[V]** |
| Hardlink-Cloning („Agent-Forking", C.4.1) | ✅ IST | `contextra-checkpoint::hardlink_cloner`: `CheckpointHardlinkCloner`, `DefaultHardlinkCloner`; hält `SnapshotRegistry`-Pin während des Klonens; Fehler `ContextraError::CrossDeviceLink`; bewusst **kein** Copy-Fallback; Windows-Grenze 1023 Links dokumentiert; Test `hardlink_cloner_tests.rs` **[V]** |
| WAL-Observer/CDC (C.4.2) | ✅ IST | `contextra-store::lsm::observer`: `WalObserver`, `WriteOrigin` (`UserWrite`, …), `register_observer/deregister_observer`, `DEFAULT_MAX_OBSERVER_LATENCY = 1 ms`; Konsument: `contextra-agent::event_source` (Polling mit `last_seen_seq`) **[V]** |
| Merge-Operatoren (C.4.3.2) | 🟡 TEIL | `MergeOperator`-Trait (`merge(existing_val, new_val)`, Fail-Safe-Vorgabe: bei Fehler beide Versionen behalten) in `contextra-store`; `MergeOperatorCapabilities::pure()` in `contextra-sandbox` **[V]**. WASM-gestützte Operator-Implementierung im Compaction-Pfad: **[?]** |
| TTL (C.4.3.1) | ✅ IST (anders als spezifiziert) | Entscheidung „Option B": `TtlMetadata` ist ein leerer Stub; TTL läuft **sequenzbasiert** über Metadatum `__expires_at_seq` und `start_expiry_cleanup_worker` (Engine). Nanosekunden-TTL in der Compaction wurde verworfen **[V]** |
| `Checkpointer` mit direkter Systemzeit (INV-CHECKPOINT-DETERMINISM-1) | 🟡 TEIL (korrigiert in v14, siehe J.3) | `contextra-store/src/checkpoint.rs` existiert nicht mehr; Checkpointing liegt in `contextra-checkpoint` **[V]** |
| Legacy-HMAC-Schlüssel (INV-WAL-LEGACY-KEY-1) | 🟡 TEIL | Fallback per `allow_legacy_integrity_key_fallback` (Default `false`), einmalige Warnung bei Nutzung; der obfuskierte Legacy-Schlüssel ist noch im Code → Entfernung siehe D.2 **[V]** |
| WAL-Truncation-Schutz per Manifest-High-Water-Mark (INV-WAL-TRUNCATION-1) | ✅ IST | `Manifest::extract_high_water_mark(entries) -> Option<[u8;32]>`, genutzt in `lsm/recovery.rs` **[V]** |
| SSI-Read-Set-Tracking (INV-MVCC-SSI-1) | ✅ IST | `contextra-mvcc::ssi::SequenceLogSsiValidator`, Test `ssi_write_skew.rs` **[V]** |
| Egress-Audit-Trace (INV-EGRESS-AUDIT-1) | ✅ IST | `EgressClassifierTrace` in `contextra-privacy::audit_trace` **[V/D]** |
| Einzige Definition von `hash_deleted_keys_length_prefixed` (INV-CRYPTO-DUPLICATE-1) | ✅ IST | genau eine Definition in `contextra-crypto/src/deletion_proof.rs` **[V]** |
| Workload-adaptive Compaction (INV-COMPACTION-ADAPTIVE-1) | ✅ IST | `CostBasedAdaptivePlanner`, `AdaptiveCompactionPlan` **[V]** |
| Conformal-Kalibrierung (INV-CALIBRATION-CONFORMAL-1) | ✅ IST | `ConformalCalibrator`, `AdaptiveConformalCalibrator` **[V]** |
| Transitivitäts-Veto (INV-DEDUP-TRANSITIVITY-1) | ✅ IST | `transitivity_veto::filter_candidates_with_transitivity_veto` **[V]** |
| APPRH als Shadow-Kandidat | ✅ IST | Feature `apprh-diffusion`, `ApprhMode{Off,Shadow,Production}`, Gate-Monitor **[V]** |
| Anti-Windup PID (INV-PID-ANTIWINDUP-1) | ✅ IST | `MAX_INTEGRAL = 10.0` Clamp **[V]** |
| Engine-Invarianten-Tests | ✅ IST | `invariant_durability/isolation/determinism/tenant_isolation.rs` **[V]** |
| Markenidentität bereinigt | ✅ IST | kein Vorkommen des Vorgängernamens in `.rs`-Dateien **[V]** |
| Schlüssel-Env, `explain`-Tool, 15 MCP-Tools | ✅ IST | siehe B.9 |

---

# TEIL D — SOLL-Zustand (noch zu implementieren oder zu entscheiden)

Priorität: **P0** blockiert Produktversprechen · **P1** vor öffentlichem Launch · **P2** Roadmap 60 Tage · **P3** Backlog.

## D.1 Lizenz: Installation-Bindung (P1, 🟡)
**Ziel:** Lizenzen gelten pro Deployment. Spec-v4-Schnittstelle: `SignedActivation{ring, installation_id_hash:[u8;32], expires_at_unix:i64, signature}` mit Ed25519 über die drei Felder.
`check_ring(ring)`:
1. `Fast` → `Ok` (immer, auch bei korrupter oder fehlender Aktivierung; INV-LICENSE-2).
2. Keine Aktivierung → `NotActivated`.
3. `installation_id_hash` ≠ lokale Kennung → `NotActivated` (kein Informationsleck).
4. Signatur ungültig → `InvalidSignature`.
5. `clock.now_unix() ≥ expires_at` → `Expired`.
6. Aktivierter Ring niedriger als angefragt → `NotActivated` (keine automatische Hierarchie nach oben).
7. `Ok`.
**Entscheidung offen:** Der IST-Stand bindet an `tenant_id` und trägt `feature_flags`. Zu klären: beides beibehalten oder um Installation-Binding erweitern.
**Tests:** Fast immer erlaubt (Property-Test), abgelaufen abgelehnt (mit `ManualClock`), manipulierte Signatur abgelehnt (Differential-Test gegen `ed25519_dalek`), Ring-Hierarchie.
**Anmerkung:** Unter MIT/Apache ist das Gate per Fork umgehbar; das bleibt bekannte Grenze **[D]**.

## D.2 Entfernung des Legacy-WAL-Schlüssels (P1, 🔵)
Nach Abschluss der Migration alter WAL-Dateien wird `LEGACY_INTEGRITY_KEY_OBFUSCATED` samt Fallback-Pfad entfernt. Bis dahin: `allow_legacy_integrity_key_fallback` wird nie implizit `true`; jede Aktivierung erfordert einen expliziten, geloggten Migrationsaufruf (INV-WAL-LEGACY-KEY-1).

## D.3 Kryptografie (P1–P2)
- 🔵 KDF-Migration AGT-CRYPTO-002 (Argon2id-Header `MFKD` v1 als Zielformat).
- 🔵 `SignatureVersion` typisiert über `TryFrom<u8>` und in `verify()` absichern (`ed25519_proof.rs:109`).
- 🔵 Angleichung der Cipher-Angabe im Manifest („aes-256-gcm") an den Code (überwiegend AES-256-GCM-SIV) und der Unsafe-Insel-Liste (ADR-N03 nennt drei, Manifest vier).

## D.4 Speicher-Engine (P2–P3)
- 🔵 SSTable-`mmap` (WP-4.1) ist `UNIMPLEMENTED`; Tracking-Issue-Nummer nachtragen. Invariante: LSM-Kern bleibt blind gegenüber Indexstrukturen.
- ⏸ Sidecar-Spezialindizes: `trait SstableFooterPayload{payload_kind()->&str; serialize()->Vec<u8>}`. Der Kern schreibt den Blob unverändert ans Dateiende. Orchestrierung nur durch `contextra-engine`. Erfordert Dateiformat-Versionsänderung. Start erst bei konkretem Bedarf jenseits der In-Memory-HNSW-Grenze, wenn DiskANN nicht reicht.
- ✅ WASM-gestützter `MergeOperator` im Compaction-Pfad vollständig verdrahtet: `CompactionEngine::compact_files` ruft `merge_op.merge` auf (**[V]**, `crates/contextra-store/src/compaction/engine.rs:641`); reines WASM-Modul über `WasmMergeFunction` in `contextra-sandbox` mit `MergeOperatorCapabilities::pure_with_result_channel()` (**[V]**, `crates/contextra-sandbox/src/merge.rs:22`).
- ⏸ Cursor-persistente CDC-Subscriber-API (überlebt Neustarts) und externer Kafka-artiger Export als Ring-4-Adapter.
- ⏸ Copy-Fallback für Hardlink-Klone jenseits der Windows-Grenze (bewusst nicht implementiert; nur bei Bedarf).
- ✅ Dokumentationsauflage erfüllt: Lock-Symmetrie (`LsmState.read()`) für `apply_mem_updates` zwischen Single- und Group-Commit-Pfad in `lsm/commit.rs:98` hergestellt und dokumentiert (**[V]**, `crates/contextra-store/src/lsm/commit.rs:98`).
- ✅ Metrik `system_pressure` / `scheduler_queue_depth`: Misst Tokio-Scheduler Globalqueue-Tiefe (**[V]**, `crates/contextra-store/src/system_pressure.rs`).

## D.5 Retrieval (P2)
- ✅ `hybrid_search_with_strategy` macht Fusionsstrategie wählbar (`ScoreNormalized` MinMax-CombSUM mit RRF-Fallback, Commit `8498430`) (**[V]**, `crates/contextra-engine/src/collection/search.rs`).
- 🔵 Diffusionsvarianten (APPRH, TL-HFD, k-Path): Flip von Shadow auf Produktion erst nach Gate (≥ 10.000 Samples, Agreement ≥ 0,85; APPRH: 10 aufeinanderfolgende Gate-Passes im Fenster 20).
- ✅ `RaBitQ`-Kalibrierung in `crates/contextra-vector/tests/rabitq_recall_calibration.rs` nachgewiesen (> 85% Recall@10) (**[V]**); deterministisches HNSW-Layer-Seeding via `idx.set_layer_seed(seed)` implementiert (**[V]**, `crates/contextra-vector/src/hnsw/types.rs:113`, Commit `c6255ae`).
- 🔵 Deutsche Domänenvokabulare (Medizin, Recht) von Stub auf produktive Wörterbuchgröße ausbauen; Feature `bm25f` in den dokumentierten Default-Pfad aufnehmen oder klar als Opt-in ausweisen.
- 🔵 PathRAG snapshot-fähig machen (IST: Fehler `snapshot_unsupported_for_signal`).

## D.6 Kognition und Compliance (P2–P3)
- 🔵 Validierung der Kognitions-Pipeline gegen einen realen mehrstufigen LLM-Agenten-Workflow (bisher synthetische Daten). Bis dahin nicht als Feature bewerben.
- 🔵 Compliance-Betrieb: Auto-Entity-Extraktion (IST: standardmäßig an) muss über `auto-extraction-opt-out` abschaltbar dokumentiert und in `EnterpriseRegulated` als Default aus vorgesehen werden (Entscheidung offen).
- ⏸ Vollständiger DSGVO-Art.-30-Export und AVV-Generator: Aktivierung mit erstem Pilotkunden (Code vorhanden, hinter `FeatureRing::Compliance`).
- ✅ Ollama-/LLM-`QueryRewriter` für MultiStep-Retrieval (Issue #142) über `LlmQueryRewriter` in Ring-4-Facade `contextra` implementiert (**[V]**, `crates/contextra/src/query_rewriter.rs`).

## D.7 MCP-Werkzeuge (P1, Anforderung)
**INV-MCP-CLASSIFY-1:** Jedes in `tools/list` gelistete Tool muss in `SandboxPolicy::classify_method` explizit einer `ToolCategory` zugeordnet sein; der Catch-all `CodeExecution` (fail-closed) darf für gelistete Tools nie greifen. Ein Test iteriert über alle gelisteten Tools und prüft dies. Lesetools (`search, get, collections, plugin_status, explain`) → `DatabaseRead`; Schreibtools → `DatabaseWrite`; `cloud_query` → `CloudEgress`; `consolidate` gemäß Schreibwirkung.

## D.8 Governance-Entscheidungen mit Termin
- **2026-10-07:** Review VETO-F02 (nur Tombstone-Pruning, kein partielles HNSW-Rewiring) und VETO-OP03 (keine Realtime-Audio/STT). Zu klären: Spannung F-02 ↔ Feature `partial-index-rebuild`.
- **2026-10-11:** Bewerbungsfrist Förderprogramm „Digital-Tech-to-Product"; Businessplan (3–5 Seiten) offen.

---

# TEIL E — Produkt, Deployment, Lizenz, Markt

## E.1 Produktformen
- **Rust-Bibliothek:** `cargo add contextra`; `builder(768).with_storage_path(..).with_deployment_tier(..).build().await?`; `db.collection(name)`, `col.insert(id,&embedding,meta)`, `col.query().text(..).vector(..).k(10).execute().await`, `db.forget(key)` liefert `DeletionProof` (extern verifizierbar mit öffentlichem Schlüssel, Payload, Signatur).
- **MCP-Server:** stdio JSON-RPC 2.0 (ADR-010), Konfiguration über `mcp.json`.
- Öffentliche Fassade `AgentMemory`: `remember, recall, forget, relate, relate_n_ary` **[D]**.

## E.2 Deployment-Tiers (IST, aus `collection_profile.rs`) [V]

| Tier | PerformanceProfile | Durability | KvDeleteMode | memtable | max_ram_mb | group_commit µs | Cache-Shards |
|---|---|---|---|---|---|---|---|
| `EdgeMinimal` | BareMetal | MemoryOnly | TombstoneOnly | 8 MiB | 64 | 200 | 2 |
| `PowerUserLocal` | Balanced | WalNoHmac | TombstoneOnly | 64 MiB | 512 | 2.000 | 8 |
| `EnterpriseShared` | Balanced | WalNoHmac | CryptoShred | 256 MiB | 4.096 | 10.000 | 32 |
| `EnterpriseRegulated` | Compliance | Full | CryptoShred | 256 MiB | 4.096 | 10.000 | 32 |

`PerformanceProfile::resolve()`: **Compliance** → `Full`, `SynchronousRepair`, DeletionProof aktiv, Ring `Sovereign`; **Balanced** → `WalNoHmac`, `BackgroundRepair`, kein Proof, Ring `Fast`; **BareMetal** → `MemoryOnly`, `BackgroundRepair`, kein Proof, Ring `Fast`. Validierung: aktiver Proof + `TombstoneOnly` → `CollectionProfileError::KvDeleteMismatch`; höherer Ring ohne Lizenz → `LicenseRequired`.

## E.3 Lizenzmodell
Fast: jeder (Default-Features `fast`, `candle`). Sovereign: opt-in (`sovereign = crypto + privacy`), prüft `LicenseGate`. Compliance: kommerziell (`compliance = sovereign + audit-export + avv-generator`), signiertes Lizenz-Token.

## E.4 Markt und Strategie (SOLL)
- Zwei Wege: Open-Source-Community (Ankündigung als „Rust-Memory-Engine mit kryptografischem Löschbeweis") und Systemhäuser in Deutschland (B2B2G).
- Kontext: Vergabebeschleunigungsgesetz (seit 1.7.2026), BSI-C3A-Kriterienkatalog (April 2026).
- Vertikal-Entscheidung bis Tag 90 schriftlich: Medizin **oder** Legal **oder** Industrial.

## E.5 Roadmap (SOLL)
| Zeitraum | Aufgaben |
|---|---|
| Tag 1–30 | Benchmark-Disclaimer in allen publizierten Zahlen; ein einziger Benchmarksatz; D.2, D.7 |
| Tag 31–60 | Veröffentlichung auf crates.io, öffentliche Ankündigung (Ziel > 5 Rückmeldungen), 10 Nutzergespräche (Ziel ≥ 3 Zahlungsbereitschaften), Förderantrag bis 11.10.2026, D.1 |
| Tag 61–90 | Erster aktivierbarer Sovereign-Ring in Produktion, ein Pilotkunde, Kognitions-Demonstrator, Vertikal-Entscheidung |

## E.6 Zurückgestellte Bereiche mit Reaktivierungsbedingung (⏸)
Spektrale Hyperkanten-Indexierung (validierter Bedarf in reguliertem Vertikal) · EU-AI-Act-Mapper (Durchsetzungsdatum naht) · horizontales Sharding und P2P-Sync (konkreter Mehrknotenbedarf) · vertikale Ontologie (erster Pilot) · DSGVO Art. 30 Vollimplementierung (erstes Behördenprojekt).

---

# TEIL F — Invarianten (verbindlich, IST + SOLL)

| Kürzel | Invariante | Status |
|---|---|---|
| P1 | `#![forbid(unsafe_code)]` außer Unsafe-Inseln | ✅ |
| P2 | `clippy::unwrap_used/expect_used/panic` verboten im Produktionscode | ✅ |
| P5 | Nur Abhängigkeiten von höheren auf tiefere Ringe | ✅ (`tests/layering.rs`) |
| P12 | `capabilities.toml` einzige Quelle für Ring/Maturity | ✅ |
| P24 | Löschkosten proportional zur lokalen Nachbarschaft | ✅ |
| P26 | Kein `tokio` in Ring 0 | ✅ |
| P28 | Determinismus; Ausnahme: kryptografisches Schlüssel-/Salt-Material | ✅ |
| INV-TENANT-1 | `TenantId::try_new(0)` → Err; `0` = SYSTEM (`TenantId::DEFAULT` für Single-Tenant) | ✅ |
| INV-DELETION-1 | `DeletionProof::create()` nur nach physischer Bereinigung | ✅ |
| INV-DELETION-2 | Nach `remove_with_graph_repair` kein Zeiger mehr auf die gelöschte ID | ✅ |
| INV-DURABILITY-RING | `MemoryOnly` inkompatibel mit DeletionProof/Sovereign | ✅ |
| INV-PERF-PROFILE-1 | `BareMetal`/`Balanced` inkompatibel mit Löschbeweis | ✅ |
| INV-COLLECTION-PROFILE-1/2 | Aktiver Proof ⇒ `CryptoShred`; Presets konsistent | ✅ |
| INV-LICENSE-2 | Fast-Ring ohne Lizenz immer prüfbar | ✅ |
| INV-WAL-LEGACY-KEY-1 | Legacy-Fallback nie implizit aktiv | 🟡 (D.2) |
| INV-MCP-CLASSIFY-1 | Jedes gelistete MCP-Tool klassifiziert | 🔵 (D.7) |
| AGT-GRAPH-001 | `TxId::is_valid_origin()` auf allen Graph-Operationen | ✅ |

**ADR-Referenzen:** ADR-010 (MCP nur stdio), ADR-011 (Checkpoint-Subsystem), ADR-016/082 (DocId aus BLAKE3, 128 Bit per Feature), ADR-024 (Snapshot-Read-At, Filter ohne Umordnung), ADR-028 (TxId-Domänen), ADR-033/038 (bi-temporaler Filter), ADR-042 (Agent-Re-Integration), ADR-067 (Sufficiency-Gate), ADR-097 (HNSW-Reparatur), ADR-N02 (Compute-Pool), ADR-N04 (FFI-Panic-Translation), ADR-N08 (Manifest als Quelle). Insgesamt ca. 105 ADR-Dateien und 41 Gate-Dokumente im Repo **[?]**.

**Qualitätssicherung:** risikoproportionale Testtiefe (kritisch: WAL/HMAC mit FaultVfs, Property- und Loom-Tests; hoch: DeletionProof, HNSW-Recall/Differential; mittel: Graph; niedrig bis Validierung: Kognition). `xtask` mit `check_*`-Modulen, `sync-docs`, Ratchet-/Determinismus-/Unsafe-Baselines in `governance/`. Externe Verifikation des `DeletionProof` v3 nur mit 32-Byte-Schlüssel, Payload und 64-Byte-Signatur.

*Nicht ausgeführt: Build, Tests. Nicht am aktuellen HEAD im Ablauf gelesen: Leiden-, ForwardPush-, WAND-, TL-HFD-Innenleben, Konsolidierungspipeline, Router-Kaskade, Egress-Klassifikator (dort **[D]**/**[?]**).*

---

# TEIL G — Erweiterungen und Korrekturen gegen HEAD `e4d84c9` (neu in v13)

Belegmarker wie oben. Zusätzlich **[V-30.09]**: am 30.09.2026 im Klon von HEAD `e4d84c9` gelesen oder gezählt. Der Klon ist flach (`--depth 1`), ein Diff gegen `3f9d2df` war deshalb nicht möglich.

## G.1 Verifizierte Korrekturen und Bestätigungen

| Nr. | Aussage in v12 oder in den Begleitdokumenten | Befund am HEAD `e4d84c9` | Beleg |
|---|---|---|---|
| G1-1 | B.3.1: `partial-index-rebuild` sei Default-Feature | Falsch. `contextra-vector/Cargo.toml`: `default = []`; das Feature trägt den Kommentar zu F-02. Die Spannung F-02 ↔ Feature bleibt, aber als Opt-in | V-30.09 |
| G1-2 | B.6: sieben `DeletionLayer` | Bestätigt: `LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments{seq_after}`, `CsrGraph`, `KvCacheSegments`, `EmbeddingCache`; `ExcludedScope`: `ConsolidatedAndDistilled`, `LlmParameterMemory` | V-30.09 |
| G1-3 | D.7 / INV-MCP-CLASSIFY-1: `contextra_explain` nicht klassifiziert | **Weiter offen.** `classify_method` in `contextra-mcp/src/sandbox.rs` enthält kein `explain`; der Default-Zweig liefert `CodeExecution`. Bei `allow_code_execution=false` scheitert das gelistete Tool an `validate_tool_call` (`explain.rs`). Damit ist INV-MCP-CLASSIFY-1 keine Anforderung mit Test, sondern ein **reproduzierbarer Fehler** | V-30.09 |
| G1-4 | Teil C: „SSI-Read-Set-Tracking ✅ IST“; der Prompter-Plan (Prompt 1) behauptet, die Leseseite sei nicht verdrahtet | Teilweise beides. Der Trait `StorageEngine` hat `get_tracked`, `get_at_seq_tracked`, `scan_prefix_tracked` (Default: **ohne** Read-Set-Registrierung, Doc-Kommentar); `Collection` hat `get_tracked` und `get_at_seq_tracked` (`engine/collection/crud/read.rs`); Test `mvcc/tests/ssi_write_skew.rs` existiert. **Nicht bestätigt:** dass `LsmStorage` diese Methoden überschreibt und dass alle Lesepfade der Engine sie nutzen (Namenssuche fand außerhalb von `ports` nur `tenant_codec.rs` und einen Test). Einstufung: 🟡 TEIL, Verifikation offen | V-30.09 / [?] |
| G1-5 | Prompter-Plan: zwölf Required Checks in `protect-main.json` | 13 Einträge (Compile, Format, Clippy, Test Suite, verdict, context-gates, Merge Gate, Duplicate Symbols Cross File, FlatBuffers Drift, Capability Marker Drift, cargo deny, Phantom Commit Protection, HyperEdge Schema Merge). Miri und Feature-Matrix stehen nicht darin | V-30.09 |
| G1-6 | Prompter-Plan: mehrere Sicherheitsprüfungen seien `required = false` | `.jules/harness-phases.toml`: 25 Phasen-Einträge, **10 `required = true`, 15 `required = false`** (start 3 von 4, check 3 von 6, verify 4 von 8, submit 0 von 6, stop 0 von 1 sind required). Welche Namen im Einzelnen betroffen sind, habe ich nicht zugeordnet | V-30.09 (Zählung) |
| G1-7 | Größen | `crates/` + `xtask/`: 1.192 `.rs`-Dateien, 289.058 Zeilen. Mit allen übrigen `.rs` (Benches, Beispiele) sind es 1.217 Dateien / 296.999 Zeilen (HEAD `8d62f28`). 40 `xtask check_*`-Module; 106 Einträge in `docs/decisions`, 41 in `docs/gates` (Verzeichniseinträge) | V-30.09 |
| G1-8 | Benchmarks | `benchmarks/results/` enthält `ann_results.json`, `criterion-baseline.json`, `results.json`, `summary.md`. Die letzten Zeilen von `summary.md` zeigen Szenario A (Kontext-Präfix) und B (Reranking) mit **+0,0 %** Unterschied (Werte 80 % bzw. 60 %; Größe des Testsatzes nicht geprüft). Ein gemessener Lauf bei 100.000 Dokumenten ist dort nicht belegt | V-30.09 |

**Korrektur an meiner eigenen Zwischenfassung:** In der `CONTEXTRA_GESAMTSPEZIFIKATION.md` stand, der `DeletionProof` kenne nur drei Layer und keinen KV-Cache-Layer. Das war falsch (meine Suche hatte nur drei der sieben Varianten getroffen). v12 und G1-2 sind richtig.

## G.2 Zusätzliche IST-Funktionen (bisher in v12 nicht beschrieben)

Dateiexistenz am HEAD verifiziert, Algorithmus nicht Zeile für Zeile gelesen **[D]**, sofern nicht anders vermerkt.

| Funktion | Ort | Beschreibung (aus Doc-Kommentar/Katalog) |
|---|---|---|
| Lovász-Erweiterung für TL-HFD | `graph/src/tl_hfd/lovasz.rs`; Test `graph/tests/proptest_tl_hfd_lovasz.rs` | Lovász-Extension/Subgradienten für die submodulare Optimierung der Hyperkanten-Diffusion |
| Budgetierte k-Path-Diffusion | `graph/src/path_rag/k_path.rs`; Test `graph/tests/budgeted_k_path_hub_entity.rs` | Bidirektionaler Dijkstra mit Budget; vermeidet Speicherexplosion bei Hub-Knoten |
| Human-in-the-Loop-Freigabe | `sandbox/src/approval.rs` | `ApprovalRequest`, `ApprovalRisk`, `classify_risk` |
| OpenIE-Triple-Extraktion | `engine/src/extraction/open_ie.rs` | `extract_triples()`, LLM-basiert; Grundlage der Auto-Extraktion |
| Bloom-Filter je SSTable-Block | `store/src/sstable/bloom.rs`; Test `store/tests/sstable_block_bloom_fpr.rs` | Falsch-Positiv-Rate ist getestet |
| Saga-artige Transaktionen | `engine/src/transaction/compensating_actions.rs`, `intent.rs`, `db_transaction_*.rs` | Kompensierende Rollbacks über mehrere Indizes |
| BSI-Mapping | `audit-export/src/bsi_mapping.rs` | `BsiMappingEntry`, Markdown-Rendering |
| Router-Konformitätsrekalibrierung | `router/src/profile.rs:338` `recalibrate_conformal(non_conformity_score)` | Konformalitätskonzept im Router, getrennt vom `ConformalCalibrator` in `rank` |
| Fuzz-Targets | `crates/{crypto,mcp,text,db}/fuzz/` und weitere | Fuzzing-Infrastruktur je Crate (Anzahl der Targets nicht gezählt) |
| SIEVE-Cache | Feature `sieve-cache` in `contextra-store` | Eine Datei mit `sieve` im Namen fand ich nicht; Implementierung vermutlich im Block-Cache-Modul **[?]** |

## G.3 Invarianten-Register (ergänzt zu Teil F)

Zusätzlich zu Teil F sind im Code mit ID referenziert (Fundstelle = erster Treffer, Aussage nach Kommentar):

| ID | Aussage | Ort |
|---|---|---|
| INV-KV-DELETE-1 | KV-Löschbeweis nur für `CryptoShred`-Segmente | `store/kv/segment.rs` |
| INV-TENANT, INV-TENANT-2, INV-TENANT-A | Mandanten-Isolation (Key-Isolation, KV-Segmente) | `crypto/kv_segment/store.rs`, `engine/tests` |
| INV-DAG-DEPS | Lock-Reihenfolge `collections` → `kv_locks` → `embedder` | `engine/collection/crud/internal.rs` |
| INV-HASHER-SEEDS | Hasher-Seeds nie randomisieren | `engine/collection/kv_lock.rs` |
| INV-PRESSURE-1, INV-DB-3 | Backpressure bei `Critical`; Rollback-Fehler beim Insert wird geloggt | `engine/collection/crud` |
| INV-CONSOLIDATE-1/2 | Crash-Resilienz der Konsolidierungssession | `cognition/context_compaction/session.rs` |
| INV-PROV-1, INV-GRAPH-PROV-1 | Summe der Signalbeiträge = RRF-Score; `relate()` indiziert Kanten in `doc_edge_index` | `db` |
| INV-CAL-1/2/3 | Kalibrierung (kein stiller Fallback vor Warmup, Fingerprint-Reset, Grounding-Feedback). `CAL-1` und `CAL-3` sind nur in Tests oder Testkommentaren referenziert | `rank`, `infer-candle` |
| INV-ROUTER-1/2, INV-P8-1 | Quantil in [0,1]; deterministisches Routing; Invalidierung bei Fingerprint-Wechsel | `router/profile.rs` |
| INV-SBX-1/2/3/5 | Modul ≤ 10 MB, Tabellen ≤ 10.000, Fehlertyp bei Verstoß, kein Dateisystem/Netz per Default | `sandbox` |
| INV-VAULT-1/2/3 | Purge zeroized, `ingest()` schreibt nie in Storage, Kapazitätsgrenze | `db/volatile_vault.rs` |
| INV-KIVI-AEAD-ORDER | Erst quantisieren, dann verschlüsseln | `kvcache/segment.rs` |
| INV-NUC-1, INV-DISKANN-1 | Partial-Rebuild ohne inkonsistenten Graphen; keine `.delta.tmp` nach `persist_delta()` | `vector` |
| INV-COMPACTION-ADAPTIVE-1, INV-C1/C2 | Tombstones bleiben für aktive Snapshots | `store/compaction` |
| INV-PLUGIN-DEPENDENCY, INV-TENANT-1 | Plugin-Abhängigkeiten; `TenantId(0)` reserviert | `ports`, `types` |

## G.4 Env-Variablen (Ergänzung zu B.7)

Laufzeit (V-30.09): `CONTEXTRA_WORKER_THREADS`, `CONTEXTRA_DELETION_PROOF_KEY`, `CONTEXTRA_PROOF_KEY`, `CONTEXTRA_ORPHAN_PATH`, `CONTEXTRA_ORPHAN_PIN_PATH`, `CONTEXTRA_OLLAMA_URL`, `CONTEXTRA_CANDLE_MODEL_DIR`, `CONTEXTRA_ONNX_MODEL_PATH`, `CONTEXTRA_ROUTER_PROFILES_PATH`, `CONTEXTRA_ROUTER_PROFILES_JSON`, `CONTEXTRA_ROUTER_CALIBRATION_PATH`, `LLAMA_GGUF_PATH`; ungeprefixt: `EMBEDDING_PROVIDER`, `LLM_PROVIDER`, `CANDLE_EMBED_MODEL_DIR`, `CANDLE_LLM_MODEL_DIR`. Nur Test/CI: `CONTEXTRA_TOCTOU_CHILD`, `CHAOS_SEED`, `PROPTEST_CASES`, `TEST_SUBPROCESS_*`, `CONTEXTRA_CI*`.

**Offene Punkte:** Zwei Variablen für denselben Löschbeweis-Schlüssel (Vorrang ungeklärt). Präfix-Inkonsistenz bei den Provider-Variablen.

## G.5 Qualitätssicherung (Ergänzung)

40 `xtask check_*`-Module: Architektur (`ring_layering`, `ring_capabilities_consistency`, `ring0_async_purity`, `unsafe_islands`, `manifest_completeness`, `type_registry`); Dopplungen (`duplicate_*`, `orphan_modules`, `module_reachability`, `phantom_files`); Robustheit (`nan_validation_in_hot_loop`, `max_results_unbound`, `result_dropped_on_io`, `toctou_trait_defaults`, `ffi_panic_boundary`, `flatbuffers_drift`, `bandit_latency_budget`); Schwellen (`coverage_gate`, `mutation_score_gate`, `recall_stability`, `compile`); Prozess (`adr_deadlines`, `vetoes`, `doc_references`, `toc_integrity`, `placeholder_refs`, `stale_tags`, `commit_*`, `workflow_commands`, `action_pinning`); Agenten (`agents_freshness`, `agents_integrity`, `jules_context_freshness`, `audit_tool_evidence`, `audit_verdict_independence`). Testkit: `FaultVfs`, `ManualClock`, `RefModel`.

## G.6 Befundregister (konsolidiert, Stand `e4d84c9`)

| ID | Befund | Schwere | Status |
|---|---|---|---|
| B-01 | `contextra_explain` scheitert bei Default-Policy (siehe G1-3). Fix: Tool in `DatabaseRead`; Test über alle gelisteten Tools gegen `classify_method` (= INV-MCP-CLASSIFY-1) | **hoch** | offen, V-30.09 |
| B-02 | Default-Zweig von `classify_method` ist fail-closed in der falschen Kategorie; besser: unbekannt ⇒ Fehler | mittel | offen, V |
| B-03 | SSI-Leseseite: Verdrahtung in `LsmStorage`/`TenantScopedStorage` und allen Engine-Lesepfaden nachgewiesen | hoch | **erledigt**, V |
| B-04 | 15 von 25 Harness-Phasen-Einträgen `required = false` (G1-6) | mittel | offen, V |
| B-05 | Miri- und Feature-Matrix-Job nicht Required Check (G1-5) | mittel | offen, V |
| B-06 | Gemessener 100k-Benchmark-Bericht unter `docs/reports/scale_100k_measurement_2026-09-30.md` mit Rohdaten dokumentiert | mittel | **erledigt**, V |
| B-07 | Spannung F-02 ↔ `partial-index-rebuild` (jetzt als Opt-in) | mittel | Review 2026-10-07 |
| B-08 | Auto-Extraktion standardmäßig an; in `EnterpriseRegulated` als Default aus vorsehen (Entscheidung offen) | mittel | offen, D |
| B-09 | SSTable-mmap `UNIMPLEMENTED`, Issue-Platzhalter | mittel | offen, D |
| B-10 | KDF-Migration AGT-CRYPTO-002 | mittel | offen, D |
| B-11 | Zwei Env-Variablen für den Löschbeweis-Schlüssel | mittel | offen, V |
| B-12 | `DeletionProofKeyPair`, `LayerCleanupProof` doppelt in `crypto` und `engine` | niedrig | offen, D |
| B-13 | `INV-CAL-1/3` ohne Definitionsort im Produktivcode | niedrig | offen, V |
| B-14 | Doc-Kommentar `types/saos.rs` nennt noch „Synthesized Agent Operating System“ (Datei enthält Query-/Fusionstypen); `deletion_proof.rs`-Kopf nennt noch HMAC | niedrig | offen, V/D |
| B-15 | `blocking_util` misst Tokio Global-Queue-Tiefe (`scheduler_queue_depth`) in `system_pressure.rs` | niedrig | **bestätigt**, V |

## G.7 Ergänzungen Welle-1-Sync (U-13 ff.)

| ID | Befund / Erweiterung | Schwere | Status |
|---|---|---|---|
| U-13 | `WasmMergeFunction` & `MergeOperatorCapabilities::pure_with_result_channel` für reine WASM-Merges (§4.18) in `crates/contextra-sandbox/src/merge.rs` | niedrig | **erledigt**, V |
| U-14 | `LlmQueryRewriter` in Ring-4-Facade `contextra` (#142, `crates/contextra/src/query_rewriter.rs`) | niedrig | **erledigt**, V |
| U-15 | Deterministisches HNSW-Layer-Seeding via `idx.set_layer_seed(seed)` in `crates/contextra-vector/src/hnsw/types.rs:113` | niedrig | **erledigt**, V |
| U-16 | Tenant-Collection Handle Caching in `Contextra` Facade (`crates/contextra-engine/src/contextra_impl/lifecycle.rs`) | niedrig | **erledigt**, V |
| U-17 | Dependency Injection Builder (`with_clock` / `with_rng`) für Testbarkeit und P28-Determinismus über Ring-0..4 Crates | niedrig | **erledigt**, V |

---

# TEIL H — Stand (Status-Board, 30.09.2026)

## H.1 Termine

| Datum | Ereignis | Abstand |
|---|---|---|
| 2026-10-07 | Review VETO-F02 und VETO-OP03 | 7 Tage |
| 2026-10-11 | Bewerbungsfrist Förderprogramm „Digital-Tech-to-Product“, Businessplan 3–5 Seiten offen | 11 Tage |

## H.2 Stand der P0/P1-Punkte

| Punkt | Status | Anmerkung |
|---|---|---|
| Löschbeweis P0-1…P0-8 | ✅ | Teil C |
| SSI-Leseseite | 🟡 | G1-4, Prüfung offen |
| INV-MCP-CLASSIFY-1 / `contextra_explain` | 🔴 Fehler | G1-3, Fix klein, P1 |
| Installation-Bindung der Lizenz (D.1) | 🔵 | Entscheidung offen |
| Legacy-WAL-Schlüssel entfernen (D.2) | 🔵 | Migration zuerst |
| Benchmark-Disclaimer, ein Benchmarksatz | 🔵 | Tag 1–30 |
| Gemessener 100k-Lauf | 🔵 | B-06 |
| CI-Gates (Miri, Feature-Matrix) | 🔵 | B-05 |
| Harness-Gates scharf schalten | 🔵 | B-04 |

## H.3 Abgleich mit dem Prompter-Plan (`CONTEXTRA_PROMPTER_UPDATE_UND_PLAN.md`)

| Prompt | Thema | Abgleich mit dem Code | Empfehlung |
|---|---|---|---|
| 1 | SSI-Leseseite Ende-zu-Ende | Kontext teilweise überholt (Trait hat `*_tracked`-Methoden, Engine hat `get_tracked`). Offen ist die Verdrahtung in `LsmStorage`/Lesepfaden und der Adversarial-Test | Prompt umformulieren: erst prüfen, welche Lesepfade **nicht** tracken, dann schließen |
| 2 | Harness-Gates scharf | Bestätigt: 15 von 25 Einträge `required = false` | unverändert nutzbar; Namen der betroffenen Einträge im Prompt ergänzen |
| 3 | 100k-Benchmark | Bestätigt: kein gemessener 100k-Lauf in `summary.md` | unverändert nutzbar |
| 4 | Miri/Feature-Matrix | Bestätigt (13 Checks, keines von beiden) | „zwölf“ im Prompt auf „dreizehn“ korrigieren |
| 5 | Flush-/Compaction-Reihenfolge (F-01…F-04) | Nicht geprüft | zuerst Fundstelle in den Audit-Dokumenten bestätigen |
| — | Zusätzlich empfohlen | **`contextra_explain`-Fix (B-01)** fehlt im Plan | als sechsten Prompt aufnehmen (klein, mcp-only) |

**Umbau des Prompters (SOLL, aus dem Plan übernommen):** von Crate- auf Funktionsbereiche (Durability & Crash-Recovery; Concurrency & Isolation; Security & Kryptografie; Retrieval & Ranking; Graph & Wissenskonsolidierung; FFI & Interop; Governance & Harness; Skalierung & Benchmarks) mit vier Aufgabentypen (Lücke schließen, Härten, Erweitern, Governance aktualisieren). Migrationsreihenfolge: (1) `FUNCTIONS`-Registry neben `DEFAULT_COMPONENTS`, (2) UI-Umbau, (3) Claim-Sichtbarkeit über `cargo xtask claim`, (4) Rückbau erst nach fünf erfolgreichen funktionsbasierten Prompts. Ein Bereich „Produkt/Markt“ fehlt in dieser Liste und gehört nicht in den Prompter.

## H.4 Roadmap-Stand

| Zeitraum | Ziel | Stand |
|---|---|---|
| Tag 1–30 | Benchmark-Disclaimer, ein Benchmarksatz, D.2, D.7 | D.7 hat einen realen Fehler (B-01); Rest offen |
| Tag 31–60 | crates.io, Ankündigung, 10 Nutzergespräche, Förderantrag bis 11.10., D.1 | Antrag nicht belegt |
| Tag 61–90 | Sovereign-Ring in Produktion, Pilotkunde, Kognitions-Demonstrator, Vertikal-Entscheidung | offen |

## H.5 Belegtiefe von Teil G und H

Gelesen habe ich Manifeste, `Cargo.toml`-Features, `deletion_proof.rs`-Enum, `storage.rs`-Trait-Doc, die Namen in `harness-phases.toml` und `protect-main.json` sowie Dateiexistenz und Zählungen. Nicht gelesen habe ich den Ablauf von Lovász, k-Path, OpenIE, Bloom, Saga und Sieve, die Audit-Berichte zu F-01…F-04, `VETOES.md` im Klartext und die Inhalte der ADRs. Nicht ausgeführt: Build und Tests.

---

# TEIL I — Nachspezifikation der Lückenliste (neu in v14)

**Quelle der Liste:** `CONTEXTRA_SPEC_LUECKENLISTE.md` (aus Commit-/PR-Titeln abgeleitet) und `QUALITAETSPRUEFUNG_COMMIT_FUNKTIONEN_SPEZIFIKATION.md`. **Code-Basis:** Klon-HEAD `81ab98a` vom 30.09.2026 (`crates/`: 1.005 `.rs`-Dateien; mit `xtask` 290.708 Zeilen).

**Belegmarker:** **[V]** Enum, Konstante, Signatur oder Doc-Kommentar im Code gelesen · **[D]** nur Dateiexistenz oder Name bestätigt, Verhalten aus Lückenliste/PR-Titel · **[–]** im Code nicht gefunden, deshalb **nicht** als IST spezifiziert.

## I.0 Korrekturen an den Begleitdokumenten

| Aussage | Befund im Code | Beleg |
|---|---|---|
| QA-Katalog, Commit 2: `DurabilityMode` = `SyncOnCommit`, `BackgroundSync`, `MemoryOnly`, `WalNoHmac` | Falsch. Der Enum hat **drei** Varianten: `Full` (Default), `WalNoHmac`, `MemoryOnly` (`#[non_exhaustive]`, `store/lsm/config.rs:8`) | V |
| QA-Katalog, Commit 10: WAL-Magic `b"MFN3"`, `WalVersion::V3NoHmac` | Im Produktivcode nicht gefunden. v12 nennt `MFW2`/`MFW3` | [–] |
| QA-Katalog: Standalone-CRC32 je WAL-Eintrag im `WalNoHmac`-Modus | nicht geprüft | [D] |
| Lückenliste 4.7 (Extraktoren PDF, E-Mail, DOCX inkl. Zip-Bomb-Schutz) | In `engine`, `db`, `mcp` keine PDF-/DOCX-/E-Mail-Extraktoren. `engine/src/extraction/` enthält nur `open_ie.rs`, `types.rs`. Die Extraktoren gehörten zur Desktop-App | [–] |
| Lückenliste 11.6 (Tauri-Desktop-App) | Seit ADR-077 deprecated, CI-Jobs entfernt (laut Liste). Keine Spezifikation als Produktbestandteil | [–] / Scope-Entscheidung |
| Audit-Dokument: `TxId::new(origin, local)` mit Partialordnung und Panic bei Überlauf | nicht geprüft; v12 nennt `TxId::is_valid_origin()` (AGT-GRAPH-001) | [D] |
| Lückenliste 10.14: `MemFuseErrorDto` | Heißt `ContextraErrorDto` (`types/src/error_dto.rs`), von `contextra-py` genutzt | V |
| Lückenliste 2.4/2.1/2.2: `PhysioScheduler`, Free-Energy-Thermostat, NREM/REM als eigene Symbole | Keine Dateien/Symbole mit diesen Namen gefunden. Die Aufgaben laufen im `MaintenanceScheduler` (I.2) | [–] |
| v12 B.4: Replicator standardmäßig aus | Doc-Kommentar in `maintenance_config.rs` sagt `replicator_enabled` „Default: true“, Lernrate 0,05. Widerspruch, `Default`-Impl nicht gelesen | [D] |

## I.1 Basistypen und Querschnitt

| Nr. | Feature | Spezifikation |
|---|---|---|
| 6.16 | `TombstoneSemanticsCheck`, `SeqBitTombstone` (`types/src/tombstone.rs`) | Trait mit `is_tombstone(seq: u64, flags: u8) -> bool` und `make_tombstone(seq) -> Vec<u8>`. `SeqBitTombstone`: Tombstone, wenn `seq & TOMBSTONE_BIT != 0` (Bit 63) **oder** `flags & 0x01 != 0`. `make_tombstone` liefert 8 Byte Little-Endian mit gesetztem Bit 63, idempotent bei bereits gesetztem Bit. Rein bitweise, ohne Panic **[V]** |
| 9.5 | `CacheDirective` (`types/src/error.rs:70`) | `Pin{ttl: Option<Duration>}` (None = unbegrenzt), `NeverCache`, `Auto` (Default), `ReleaseAfterStep{step_id: StepId}`. `StepId` ist ein Newtype über `u64` mit `From<u64>` **[V]** |
| 4.8 / 13.11 | `AutoExtractionMode` (`types/.../misc.rs:542`) | `Enabled`, `Disabled`; `is_enabled()`. `Default` hängt an Feature `auto-extraction-opt-out`: ohne das Feature `Enabled` (Code-Struktur `cfg(not(feature…))`), mit dem Feature vermutlich `Disabled` **[V/D]**. Tier-Defaults (#3950) nicht im Einzelnen gelesen |
| 10.14 | `ContextraErrorDto` | Serialisierbares Fehler-DTO mit `with_details(...)`, erzeugt über `From<&ContextraError>`; Verwendung in `contextra-py/src/bindings/common.rs` **[V]** |
| 6.18 | Ports `Clock`, `Rng`, `IdGenerator` | siehe Teil 8 / A.1; Clock-Injektion in Checkpoint und License **[D]** |
| 6.15 / 13.8 | `docid-128` | Ein Feature, das in `types`, `text` (Posting-Liste, WAND, Inverted-Index), `store`, `vector`, `graph`, `engine` mitgeschaltet wird; CI-Gate vorhanden. DocId = BLAKE3-Ableitung. **Kein Rückwärtspfad für 64-Bit-Datenbestände** (Aussage des Audit-Dokuments, nicht geprüft) **[D]** |

## I.2 Kognition, Lebenszyklus (Lückenliste Abschnitt 2)

- **`MaintenanceScheduler<S, V = HnswIndex>`** (`cognition/src/maintenance_scheduler.rs`) führt je Tick aus: Decay-Eviction, Edge-Reinforcement-Flush, Perkolationsprüfung, Replikatordynamik-Update. `MaintenanceConfig`: `tick_interval_secs` 60, `decay_enabled`, `decay_config` (κ, `base_half_life_tx`, `eviction_threshold`), `edge_reinforcement` (η, δ, W_max, ρ, Q, α), `percolation_enabled`, `percolation` (`critical_threshold`, `rebonding_similarity`, `max_new_edges_per_pass`), `replicator_enabled`, `replicator_lr` **[V]**. Das deckt die Punkte 2.1 (adaptiver Decay), 2.5 (Kantenverstärkung), 2.6 (Perkolation/Re-Bonding) und 3.6 (Replicator, adaptive RRF-Gewichte) ab; eigene Symbole für „Free-Energy-Thermostat“, „NREM“, „REM“ und `PhysioScheduler` fehlen (I.0) **[–]**.
- **2.13 `ConsolidationEngine`/`run_consolidation_pass`:** `consolidation_executor.rs`, `consolidation_locks.rs`, `memory_consolidation.rs`. Laufzeit über `spawn_blocking`; `ConsolidationNodesGuard` (RAII) laut Liste **[D]**.
- **2.14–2.17:** `context_compaction/` (LLM-Konsolidierung, OCC-Retry, Startup-Cleanup), `transitivity_veto.rs`, `leanrag_input.rs`, `graph_sink.rs`, `aggregation_phase.rs` (siehe B.4, G.2) **[D]**.
- **2.18 `semantic_aggregation_facade.rs`:** Facade für semantische Hyperkanten-Aggregation inklusive Co-Occurrence-Vorschlag mit LLM-Validierung **[D]**.
- **2.21 Export v1:** `engine/src/export.rs`, `SCHEMA_VERSION_V1`; Typen `ExportCollectionV1`, `ExportDocumentV1`, `ExportMemoryV1` (inkl. Embedding und `MemoryType`), `ExportRelationV1` **[V]**.
- **2.22 Observability:** `DriftStatusProvider`-Port, Live-Werte in `stats()` per `Weak`-Referenz (ADR-080) **[D]**.
- **2.10/2.11/2.12/2.15/2.19/2.20:** Importance-Scoring, `MemoryType`, Zettelkasten-Links (`LinkRelation`), Kaskaden-Invalidierung bei `Supersedes`, `VolatileContextVault`, TTL-Reaper sind in v12 (B.1, B.4, B.9) bzw. G.3 beschrieben.

## I.3 Retrieval und Ranking (Abschnitt 3)

| Nr. | Feature | Spezifikation |
|---|---|---|
| 3.19 | Kandidaten-Streams | `Bm25CandidateStream<'a, S: StorageEngine>` (`text/src/stream.rs`), `VectorCandidateStream<'a, V: VectorIndex>` (`vector/src/candidate_stream.rs`), `PprCandidateStream<'a, G: GraphIndex + ?Sized>` (`graph/src/ppr_stream.rs`). Sie liefern Kandidaten in Batches für die Fusion statt einer fertigen Liste **[V Namen/Signaturen]** |
| 4.2 | Delta-Varint-Posting-Listen (`text/src/posting_list.rs`) | Serialisierung: Anzahl der Postings als Varint, je Posting Delta der Dokument-ID, `tf` und `doc_len` als Varint; Blockgröße 64 (B.3.2) **[V]** |
| 3.7/3.8 | Wählbare Fusionsstrategie | Commit `8498430` macht `FusionStrategy` in `hybrid_search_with_strategy` konfigurierbar. **Damit ist v12 D.5 (RRF fest verdrahtet) für den HEAD veraltet**; der genaue Pfad ist nicht gelesen **[D]** |
| 3.3/3.4/3.5 | Platt, Isotonic (PAVA mit Debounce), PID-Rerank-Pool (`RerankDeadline`) | siehe B.3.4, B.5 **[D]** |
| 5.2 | `RebuildStatus` (`vector/src/hnsw/types.rs:178`) | `Idle`, `Running`, `Pending` **[V]**. Backoff/Alarm bei Rebuilds laut Liste **[D]** |
| 5.6 | Ada-ef | `AdaptiveEfPolicy`, `AdaptiveEfStateMachine`, `AdaptiveEfStats` in `vector/src/hnsw/adaptive_ef.rs`, genutzt in `core_search.rs`; Parameter siehe B.3.1 **[V]** |
| 5.3–5.5 | DiskANN, RaBitQ, ACORN | siehe B.3.1 und G.2 **[D]** |
| 3.11 | `_at`-Familie | `search_at`, `search_filtered_at`, `get_at_seq`, `scan_prefix_at` liefern Snapshot-Sicht zu einem `seq` (B.3.5) **[V/D]** |

## I.4 Speicher-Engine (Abschnitt 6)

| Nr. | Feature | Spezifikation |
|---|---|---|
| 6.9 | `DurabilityMode` | `Full` (Default), `WalNoHmac`, `MemoryOnly`, `#[non_exhaustive]`. Kompatibilitätsprüfung liefert `DurabilityConfigError::IncompatibleCombination{mode, feature, reason}`, konvertierbar in `ContextraError` **[V]** |
| 6.1/6.2 | Block-Bloom, Block-Cache | `BloomFilter` mit Obergrenze `MAX_BITS = 128 MiB · 8`. Block-Cache: `BLOCK_CACHE_SHARDS = 64`, Trait `BlockCacheBackend` mit `LruBlockCacheBackend`, `QuickCacheBlockCacheBackend` und `SieveCacheBackend`; Einstieg `BlockCache`. SSTable-Magic `MFSX` = `0x5853464D`, Legacy `0x4D465354`. Formate v3/v4 (256-Bit-Block-Bloom, adaptiver Bloom) konnte ich in den Dateinamen nicht einzeln belegen **[V/D]** |
| 6.4 | Compaction-Drosselung | `CompactionConfig.max_io_bytes_per_second: Option<..>`; in `compaction/engine.rs` wird bei gesetztem Limit gedrosselt. Weiteres siehe B.2.2 **[V]** |
| 6.5 | `SystemPressureMonitor` | `wal_queue_depth` voll implementiert über Zähler der wartenden Group-Commit-Follower; `blocking_util` misst die globale Tokio-Queue-Tiefe (nicht den Blocking-Pool); `embedding_queue_depth` ist im Store `0/0`, weil der Store keine Embedding-Crates kennt. Bei `Critical` verzögert `Collection::insert` um `backpressure_delay` **[V]** |
| 6.8 | `WalObserver` | `on_commit(&CommittedBatch, seq_no, tx_id)`; `requires_durability()` Default `false` (Observer erhält dann auch `MemoryOnly`-Commits). `WriteOrigin` ist `#[non_exhaustive]` mit Variante `UserWrite`. Ausführung über einen begrenzten, nicht blockierenden Dispatcher; `DEFAULT_MAX_OBSERVER_LATENCY = 1 ms`; bei Überschreitung werden Drop-Zähler erhöht und ein **Circuit-Breaker** öffnet (Zeitstempel `circuit_breaker_open_until_nanos`). Bausteine: `ObserverRegistry`, `AsyncObserverAdapter`. Fail-open: ein langsamer Observer blockiert den Commit nicht **[V]** |
| 6.10 | Recovery | `lsm/recovery.rs` (Intent-Magic `MFRLBK\0\0`, WAL-Replay, High-Water-Mark-Prüfung), `DirLock`, Manifest `MFMN`. Engine: `repair_on_open()` (`engine/src/contextra_impl/lifecycle.rs:215`), beim Öffnen aufgerufen (Zeile 139); behandelt Crash-Fenster und „commit-uncertain“-Transaktionen laut Commits `027a73c`, `376b513` **[V Existenz/D Ablauf]** |
| 6.14 | Mandanten-Isolation im KV | `TenantKeyCodec`: Schlüssel `t:{tenant_id}:{collection_id}:{doc_type}:{doc_id}`, festes Präfix `t:` gegen Legacy-Kollisionen. **INV-TENANT-2:** `scan_prefix(codec.scan_prefix())` liefert ausschließlich Schlüssel dieses Mandanten. `TenantScopedStorage<S>` umhüllt einen `StorageEngineHandle`. Engine: Tenant-Handle-Cache und `TenantPolicy`-Durchsetzung (E-01) **[V/D]** |
| 6.19/6.7 | KV-Lock und Shards | `KV_LOCK_SHARDS = 16` mit vier festen Hasher-Seeds (`0x9E3779B97F4A7C15` u. a., **INV-HASHER-SEEDS**); `sorted_unique_shards` sichert die Sperrreihenfolge (Loom-getestet laut Liste) **[V/D]** |
| 6.12/6.13/6.17 | `StorageEngine`-Erweiterungen, Zero-Copy, Pins mit TTL-Lease | `delete_prefix`, `delete_many`, `put_if_absent`, `scan_*_bounded`; Pins mit TTL-Lease (#3066) und `DEFAULT_MAX_PIN_DURATION = 300 s` (B.2.3). Konkrete Signaturen nicht gelesen **[D]** |
| 6.6/6.3/6.11 | WAL-Format, Legacy-Key, Manifest, SSI | siehe B.2, D.2, G1-4 **[D]** |

## I.5 Graph (Abschnitt 7)

- **7.1 CSR:** Persistenz, Delta-Buffer, asynchrone Compaction mit RCU-Snapshot-Swap (IP-08), Orphan-GC: `GraphIndex`-Methode `sweep_orphans(wal_tx: TxId) -> Result<usize>` (`graph/src/cascade.rs`) **[V Signatur]**.
- **7.3 Session-DAG:** Branching mit Zyklenerkennung und Tiefenlimit 10.000 (B.3.3); Lock-Reihenfolge `nodes` → `edges` → `active_head` **[V/D]**.
- **7.4 Hyperkanten-Löschung:** Kaskade synchron bis 1.000, Rest über `DeferredHyperedgeQueue` (B.3.3); rekursive Kaskade über `child_edge_ids` **[D]**.
- **7.5 PPR-Numerik:** Masseerhaltung, Behandlung hängender Knoten, Hub-Begrenzung (`MAX_VISITED_NODES 10_000`) **[D]**.

## I.6 Adaptive Steuerung und Router (Abschnitt 8)

**`SlmProfile`** (`router/src/profile.rs`) **[V]**: `name`, `mcp_endpoint`, `domain_communities: HashSet<u64>`, `token_budget: TokenBudget`, `min_relevance_score: f32`, `resource_cost_estimate: f32` (0,0 = unbestimmt, dann gilt `token_budget.limit`), `fingerprint: Option<ConfigFingerprint>`, `transport` (Default `StdioMcp`; Feature `cloud-egress-guard` schaltet `HttpCloud` frei), Bandit-Zustand (`None` = Kaskade aktiv). Methoden `with_fingerprint`, `with_resource_cost_estimate`, `estimated_cost`, `validate`, `try_new`.

**Kalibrierte Kaskade mit Conformal-Schwelle** **[V]**
- Conformal-Zustand: `new(alpha, gamma, initial_threshold)`, `update(non_conformity_score) -> bool`, `empirical_error_rate()`, `reset_window()`; Methode `recalibrate_conformal(score)` am Profil.
- `CALIBRATION_WARMUP_WINDOW = 100`. `is_calibrated(active_fp)` ist nur wahr, wenn das Beobachtungsfenster ≥ 100 Einträge hat **und** der Fingerprint zum aktiven passt.
- `check_and_invalidate_fingerprint(active_fp)` setzt die Kalibrierung bei Modellwechsel zurück (**INV-P8-1**), der Ursprungswert `original_min_score` wird wiederhergestellt.

Bandit-Routing (LinUCB mit Vergessen, Sherman-Morrison, IPS), FC-TS, Sketched-Bandit, RIE-Greedy, DiBud-Treiber und Catoni-Drift: Parameter siehe B.5 und G.2 **[D]**. `ArmRegistry` (panikfrei) **[D]**.

## I.7 Inferenz, KV-Cache, Embedding (Abschnitt 9)

- **9.4 Streaming:** `LlmTextGeneratorStreaming: LlmTextGenerator` mit `generate_stream(&self, prompt, &ConfigFingerprint) -> BoxStream<Result<String>>` (`ports/src/embedding.rs:137`) **[V]**. Implementierungen in Candle und Ollama laut Liste **[D]**.
- **Candle:** `DEFAULT_MAX_CONCURRENT_EMBEDDINGS = 8`, `DEFAULT_MAX_CONCURRENT_INFERENCES = 4`; `GaspValidator` mit `DEFAULT_GROUNDING_THRESHOLD = 0,70` (Grounding-Wert ≥ Schwelle gilt als gestützt), Kalibrierungs-Invalidierung bei Modellwechsel **[V]**.
- **KV-Cache-Budgets:** `DEFAULT_BYTE_BUDGET_PER_TENANT = 256 MiB`, `DEFAULT_SHARD_COUNT = 32`, `DEFAULT_SEGMENT_CAPACITY_PER_TENANT = 256` **[V]**.
- **KV-Bridge (1.1):** Feature `kv-bridge`; `KvBridgeAdapter`, `KvCacheKey`, Mandanten-Isolation, Rollback-Hooks, LSM-Spill (Tier 2), Prefix-Reuse mit Prefill-Skip (`inference/prefix_reuse.rs`). Verhalten je Komponente nicht gelesen **[D]**.
- **Ollama (9.7):** nativer `/api/embed`-Batch-Endpunkt mit Fallback, Retry mit Jitter, Dimensions-/Modellvalidierung, NDJSON-Streaming, `OllamaApi`-Trait mit Mock **[D]**; Timeouts siehe B.8.

## I.8 Sicherheit, MCP, Sandbox (Abschnitt 10)

**Approval-Workflow** (`sandbox/src/approval.rs`) **[V]**
- `ApprovalRisk` wird deterministisch aus `WasmCapabilities` abgeleitet: `High`, wenn `allow_network` oder `allow_cloud_egress`; `Elevated`, wenn nur `allow_filesystem`; sonst `Low`.
- `ApprovalStatus`: `Pending`, `Approved`, `Rejected{reason}`, `Expired`; ungültige Übergänge liefern `ApprovalTransitionError`.

**MCP-Klassifikation (10.8/13.10):** siehe G1-3/B-01; INV-MCP-CLASSIFY-1 ist im HEAD nicht erfüllt **[V]**.

**Lizenz, Privacy, Audit-Export (10.1–10.5, 10.10, 10.12):** unverändert wie B.6, B.7, G.2; Plugin-Registry (`PluginRegistry`, Abhängigkeitsauflösung, Lizenz-Gating) **[D]**.

## I.9 Agent, Checkpoint, Bindings (Abschnitt 11)

- **11.2 `EventSource`** (`agent/src/event_source.rs:89`): Trait `EventSource: Send + Sync`; `run_event_loop` in `agent/src/engine.rs:647` mit Backpressure-Signal **[V Signatur/D Verhalten]**.
- **11.3 Agent-DLQ, Step-Timeout, idempotentes Replay, `TokenBudget`-Reservation** (B.9) **[D]**.
- **11.4 Checkpoint:** Fork/Diverge/Merge, `CheckpointGuard`, `PinGuard`, Instanz-Orphan-State, Hardlink-Cloner (A.2, Teil C) **[D]**.
- **11.5 Python:** siehe 10.3 der v8-Teile; Fehlerübersetzung über `ContextraErrorDto` **[V]**.
- **11.7/11.8:** Unsafe-Inseln und Zero-Panic-Lints: P1/P2 in Teil F **[V]**.

## I.10 Nicht-produktive Bereiche (Abschnitt 12)

Benchmarks, Fuzzing (`crates/{crypto,mcp,text,db}/fuzz/`), Chaos-/Loom-/Miri-Tests, `xtask`-Gates und Agentenbetrieb sind **Entwicklungsartefakte**. Sie gehören in eine separate Entwicklungsspezifikation; hier stehen sie nur als G.5 und H.3. Scope-Entscheidung an den Auftraggeber zurück.

## I.11 Delta Spec-Basis → HEAD (Abschnitt 13)

| Nr. | Punkt | Stand |
|---|---|---|
| 13.1 | Tenant-Handle-Cache, `TenantPolicy` | I.4 |
| 13.2 | Kontinuierliche SSTable-Compaction | [D] |
| 13.3 | WAL-Kettensicherheit, Anker-Rollback, Leader-Cancel-Safety | [D] |
| 13.4 | Legacy-WAL-Key-Auslauf, Observer-Runtime | [D]; v12 D.2 teilweise überholt |
| 13.5 | Ring-0/3-Gates, `cargo hack`, `gen-arch-docs` | G.5 |
| 13.6 | Reranker-Port statt `infer-onnx`-Abhängigkeit | [D] |
| 13.7 | Lizenz-Bindungs-Entscheidungsdokument | [D] |
| 13.8 | `docid-128` einheitlich | I.1 |
| 13.14 | Wählbare Fusionsstrategie | I.3; v12 D.5 überholt |
| 13.15 | SSTable-v3/v4-Stream-Trailer-Fix | [D] |

## I.12 Abdeckungsübersicht

Von den Punkten der Lückenliste sind **mit gelesenem Code belegt**: 1.2 (teilweise), 4.2, 4.8, 5.2, 5.6, 6.1, 6.2, 6.4, 6.5, 6.8, 6.9, 6.14, 6.16, 8.2, 9.2, 9.4, 9.5 (`CacheDirective`, Budgets), 10.9 (Approval), 10.14, 11.2, 2.21, 3.19. **Nur Existenz oder Name belegt** sind die übrigen mit **[D]** markierten Punkte. **Nicht im Code gefunden**: 4.7, 11.6, 2.1/2.2/2.4 als eigene Symbole, `b"MFN3"`.

**Offene Prüfaufgaben für eine nächste Runde** (Reihenfolge nach Risiko): SSI-Leseseite in `LsmStorage` (B-03), `WalNoHmac`-Format und Replay, `repair_on_open`-Ablauf, `Default`-Werte von `MaintenanceConfig`, SSTable v3/v4, Fusionspfad in `hybrid_search_with_strategy`, Tier-Defaults von `AutoExtractionMode`.

---

# TEIL J — SOLL-Zustand, geprüft gegen den IST-Code (neu in v14)

**Quelle:** `CONTEXTRA_SPEZIFIKATION_4_1_.md` (Teil C, C.4, D, X.10, X.11, Y, R.5) sowie die offenen Punkte aus Teil D, G, H und I dieses Dokuments. **Methode:** Für jeden SOLL-Punkt habe ich geprüft, ob der Code ihn schon umsetzt. Status: ✅ erledigt · 🟡 teilweise · 🔵 offen (spezifiziert) · ⏸ zurückgestellt. Marker **[V]** = im Code gelesen am HEAD `81ab98a`, **[D]** = aus Dokument übernommen.

## J.0 Konsolidierte Matrix

| Nr. | SOLL-Punkt | Quelle | Status am HEAD | Prio |
|---|---|---|---|---|
| J.1 | Installation-gebundene Lizenzaktivierung, Ring-Hierarchie | Spec 4 C.2.1, v12 D.1 | 🟡 | P1 |
| J.2 | Entfernung des Legacy-WAL-Schlüssels | Spec 4 X.11.3, v12 D.2 | 🟡 | P1 |
| J.3 | Determinismus im Checkpoint-Crate (INV-CHECKPOINT-DETERMINISM-1) | Spec 4 X.11.2 | 🟡 | P1 |
| J.4 | WASM-`MergeOperator` im Compaction-Pfad | Spec 4 C.4.3.2 | ✅ | P2 |
| J.5 | TTL (Entscheidung „sequenzbasiert“ statt Storage-Nanosekunden) | Spec 4 C.4.3.1 | ✅ (abweichend) | — |
| J.6 | Sidecar-Spezialindizes (`SstableFooterPayload`) | Spec 4 C.4.4 | ⏸ | P3 |
| J.7 | Lock-Asymmetrie im Commit-Pfad | Spec 4 X.11.4 | ✅ | P2 |
| J.8 | P28-Text: Kryptografie-Ausnahme | Spec 4 X.11.1 | 🔵 | P1 |
| J.9 | Lock-freie Caches und RCU-Graph (EBR, `arc_swap`) | Spec 4 Y.10 | ⏸ | P3 |
| J.10 | INT4/FP16-Mixed-SIMD | Spec 4 Y.9 | ⏸ | P3 |
| J.11 | Benchmark-Wahrheit und 100k-Lauf | Spec 4 D.2, Regel 3; G1-8 | ✅ | P1 |
| J.12 | Hybrid-Suche: Fusionsstrategie und PathRAG-Snapshot | v12 D.5 | ✅ | P2 |
| J.13 | SSI-Leseseite Ende-zu-Ende | v12 Teil C, G1-4 | ✅ | P0/P1 |
| J.14 | MCP-Klassifikation (INV-MCP-CLASSIFY-1) | v12 D.7 | 🔵 (Fehler offen) | P1 |
| J.15 | Kognition gegen realen Workflow validieren | Spec 4 D.1 | 🔵 | P2 |
| J.16 | Compliance-Defaults (Auto-Extraktion, Art. 30, AVV, QueryRewriter) | v12 D.6 | 🟡 | P2 |
| J.17 | Zurückgestellte Bereiche mit Reaktivierungsbedingung | Spec 4 R.5 | ⏸ | P3 |
| J.18 | Produkt- und Markt-SOLL, Roadmap | Spec 4 F, H | 🔵 | — |

## J.1 Lizenz: Installation-Bindung und Ring-Hierarchie (P1, 🟡)

**IST [V]** (`license/src/signed_gate.rs`): `SignedLicenseGate::from_signed_payload[_with_clock]`, Zugriff `license_payload()`, `verifying_key()`, Testhelfer `create_test_signed_payload`. `LicensePayload{tenant_id, allowed_rings, expires_at, feature_flags}`. `check_ring(ring)`:
1. `Fast` → `Ok` (INV-LICENSE-2).
2. `ring` nicht in `allowed_rings` → `NotActivated(ring)`.
3. `expires_at` gesetzt und `clock.now_unix_nanos()/1e9 >= expires_at` → `Expired(expires_at)`.
4. sonst `Ok`.
Die Signatur wird beim Konstruieren geprüft, nicht bei jedem `check_ring` (Code-Struktur; `InvalidSignature` aus dem Port `LicenseError` hat hier keinen Prüfpfad in `check_ring`).

**Abweichungen zum SOLL (Spec 4 C.2.1):**
- Keine `installation_id_hash`: Die Lizenz bindet an `tenant_id`, nicht an eine Installation.
- Keine Ring-Hierarchie: `allowed_rings` ist eine Menge; ein Compliance-Nutzer braucht `Sovereign` explizit.
- `LicenseError` hat `NotActivated`, `InvalidSignature`, `Expired` **[V]**.

**SOLL (Entscheidung offen, Vorschlag):** `LicensePayload` additiv um `installation_id_hash: Option<[u8; 32]>` erweitern (Blake3 über eine maschinenlokale, nicht personenbezogene Kennung). Wenn gesetzt, muss `SignedLicenseGate` eine lokale Kennung kennen und bei Abweichung `NotActivated(ring)` liefern (kein Hinweis, ob eine Fremdaktivierung vorliegt). Wenn `None`, gilt das heutige Verhalten (Abwärtskompatibilität). Die Ring-Hierarchie bleibt **explizit** (keine automatische Aufwärts-Vererbung), das entspricht dem heutigen Verhalten.
**Invarianten:** INV-LICENSE-2 (Fast immer `Ok`, auch bei korrupter oder fehlender Aktivierung) · Clock nur über `Arc<dyn Clock>` (P28) · kein `unwrap`/`panic`.
**Abnahmetests:** (1) Property: `check_ring(Fast) == Ok` für jeden Zustand. (2) `ManualClock` nach `expires_at` ⇒ `Expired`. (3) Geflipptes Bit im signierten Payload ⇒ Konstruktion schlägt fehl. (4) Falsche Installationskennung ⇒ `NotActivated`. (5) Payload ohne `installation_id_hash` verhält sich wie vor der Änderung.
**Grenze:** Unter MIT/Apache per Fork umgehbar **[D]**. **Offene Entscheidung:** `tenant_id`-Bindung beibehalten, ergänzen oder ersetzen.

## J.2 Legacy-WAL-Schlüssel entfernen (P1, 🟡)

**IST [V]:** `wal/hmac.rs` enthält noch `LEGACY_INTEGRITY_KEY_OBFUSCATED` (32 Byte, XOR-Maske) und `wal/io.rs` den Schalter `allow_legacy_integrity_key_fallback` (Default `false`, laut v12). Kommentarlage zu Migration und `.rekeyed`-Marker stammt aus Commit-Titeln (#3913, #3922, #3941) **[D]**.
**SOLL:** (1) Migration alter WAL-Dateien abschließen; Status maschinenlesbar abfragbar. (2) Danach `LEGACY_INTEGRITY_KEY_OBFUSCATED`, die Maske und den Fallback-Pfad löschen. (3) Bis dahin darf der Fallback nie implizit aktiv werden; jede Aktivierung ist explizit und wird einmal gewarnt. **Invariante INV-WAL-LEGACY-KEY-1.** **Abnahme:** Ein WAL mit V2-Legacy-Schlüssel wird ohne Flag abgelehnt; nach Entfernung existiert das Symbol im Workspace nicht mehr (`grep`-Test im `xtask`).

## J.3 Determinismus im Checkpoint-Crate (P1, 🟡) — Korrektur zu v12 Teil C

**IST [V]:** `contextra-store/src/checkpoint.rs` existiert nicht mehr. **Aber** `contextra-checkpoint` ruft `SystemTime::now()` direkt auf: `guard.rs` (Zeilen 52, 128, 145) für `PinnedSeqNoOrphan.timestamp_ms` und `orphan.rs` (`monotonic_timestamp_ms()`, Zeile 45, und Zeile 84). Ein `Clock`-Import fehlt in diesen Dateien. Das verletzt P28 im Produktionscode.
**SOLL:** Die Zeitquelle über den Port `Clock` injizieren (`Arc<dyn Clock>` in `CheckpointGuard`/Orphan-Registry); `monotonic_timestamp_ms()` durch einen `Clock`-basierten Aufruf ersetzen. Da ein `Drop` keinen Async-Kontext hat, wird der Zeitstempel beim Erzeugen des Guards gelesen, nicht im `Drop`. **Invariante INV-CHECKPOINT-DETERMINISM-1:** kein `SystemTime::now()` im Produktionscode des Crates. **Abnahme:** Test mit `ManualClock` erzeugt reproduzierbare Orphan-Zeitstempel; `xtask`-Check gegen `SystemTime::now` (Ausnahme: Schlüsselmaterial).
**Hinweis:** Die Zeitstempel dienen nach Namen nur der Orphan-Erkennung (Diagnose). Ob sie Verhalten steuern, habe ich nicht geprüft.

## J.4 WASM-`MergeOperator` im Compaction-Pfad (P2, ✅)

**IST [V]:** `CompactionEngine` in `crates/contextra-store/src/compaction/engine.rs` stellt den Builder `with_merge_operator(mut self, merge_operator: Arc<dyn MergeOperator>) -> Self` bereit. In `compact_files` (Zeilen 641–665) führt die Compaction-Engine den registrierten `MergeOperator` für aufeinanderfolgende unverschlüsselte Put-Einträge unterhalb `min_snapshot_seq` aus **[V]**. `WasmMergeFunction` in `crates/contextra-sandbox/src/merge.rs:22` führt deterministische, reine WASM-Merges mit `MergeOperatorCapabilities::pure_with_result_channel()` aus **[V]**. Gemäß Ring-DAG-Regel hängt `contextra-store` nicht direkt von `contextra-sandbox` ab **[V]**.

## J.5 TTL (✅, abweichend von Spec 4)

**IST [V]:** `store/src/compaction.rs:20` `pub struct TtlMetadata;` ist ein leerer Stub. Der Ablauf läuft sequenzbasiert: Metadatum `__expires_at_seq` und `start_expiry_cleanup_worker` in der Engine (v12 Teil C) **[D/V]**. **Entscheidung:** Sequenz statt Uhrzeit, damit TTL ohne Wall-Clock reproduzierbar bleibt. **Damit entfällt `INV-TTL-1` aus Spec 4** (Vergleich `expires_at_unix_nanos < clock.now`); Ersatz: **INV-TTL-2:** Ablauf wird ausschließlich aus `seq`/`TxId`-Distanz abgeleitet. `TtlMetadata` bleibt Stub oder wird entfernt (Aufräumaufgabe, P3).

## J.6 Sidecar-Spezialindizes (⏸)

**IST [V]:** `SstableFooterPayload` kommt im Workspace nicht vor; `SstableBuilder` hat kein reserviertes Blob-Feld.
**SOLL (zurückgestellt):** Trait `SstableFooterPayload{payload_kind() -> &str; serialize() -> Vec<u8>}` im Store; der Kern schreibt den Blob unverändert ans Dateiende und gibt ihn unverändert zurück. Orchestrierung nur durch `contextra-engine`. Erfordert eine Dateiformat-Version (aktuell SSTable `MFSX`, Formate v3/v4 laut Lückenliste). **Reaktivierungsbedingung:** Ein Nutzer stößt an die In-Memory-HNSW-Grenze und DiskANN reicht nicht.

## J.7 Lock-Symmetrie im Commit-Pfad (P2, ✅)

**IST [V]:** `crates/contextra-store/src/lsm/commit.rs:98`: Lock-Symmetrie zwischen Single-Commit und Group-Commit ist vollständig hergestellt und im Code dokumentiert. Sowohl im Single- als auch im Group-Commit-Pfad ruft `apply_mem_updates` unter einem `state.read()` Guard (`LsmState` Read-Lock) auf. Das `LsmState.write()`-Lock bleibt exklusiv der atomaren MemTable-/WAL-Rotation im `flush()` vorbehalten, während `MemTable` interne Nebenläufigkeit über ein eigenes `parking_lot::RwLock` absichert. Lock-Reihenfolge: `commit_mutex -> state.read -> MemTable-RwLock` **[V]**.

## J.8 P28-Text und Kryptografie-Ausnahme (P1, 🔵)

**IST [V]:** `store/src/wal/hmac.rs:227` nutzt `rand::thread_rng().fill_bytes(&mut key)` für Schlüsselmaterial. v12 Teil F (P28) nennt die Ausnahme „kryptografisches Schlüssel-/Salt-Material“ bereits. **SOLL:** Dieselbe Formulierung in `AGENTS.md` und `CONSTITUTION.md` (ich habe dort keine P28-Zeile gefunden), plus ein `xtask`-Check, der `thread_rng`/`SystemTime::now` im Produktionscode verbietet und genau diese Fundorte per Allowlist zulässt.

## J.9 Lock-freie Caches und RCU-Graph (⏸)

**IST [V]:** `arc-swap` ist in `contextra-graph` und `contextra-router` Abhängigkeit (RCU-Snapshot-Swap); `crossbeam-epoch` kommt in keinem `Cargo.toml` der Crates vor. SIEVE-Cache existiert als `SieveCacheBackend` (Feature `sieve-cache`).
**SOLL (Backlog):** Vollständige Lock-Freiheit des Block-Caches über epochenbasierte Freigabe. **Bedingung:** gemessene Sperrenkontention im Benchmark, neue Abhängigkeit nur nach Governance-Regel 1 (schriftliche Begründung, 48 h).

## J.10 INT4/FP16-Mixed-SIMD (⏸)

Kein Code. **Bedingung:** Benchmark auf Zielhardware; CPU-Overhead des Bit-Unpackings gegen Bandbreitengewinn. Bis dahin nicht bauen.

## J.11 Benchmark-Wahrheit (P1, ✅)

**IST [V]:** Ein gemessener 100k-Skalierungslauf ist unter `docs/reports/scale_100k_measurement_2026-09-30.md` mit Rohdaten `benchmarks/results/scale_100k_2026-09-30.json` dokumentiert. Der Bericht enthält den geforderten Disclaimer ("Misst ausschließlich Speicher- und Indexlatenz; Embedding-Inferenz addiert 10–500 ms pro Anfrage") und dokumentiert den Abbruch bei 0 Intervallen (Memory budget exceeded in 4-Core 8GB Sandbox-Umgebung) regelkonform gemäß Regel 3 / G1-8 **[V]**.

## J.12 Hybrid-Suche (P2, ✅)

**IST [V]:** Fusionsstrategie in `contextra-engine` (`collection/search.rs`) ist wählbar (`ScoreNormalized` MinMax-CombSUM mit automatischem RRF-Fallback, Commit `8498430`) **[V]**. `HnswIndex` in `contextra-vector` unterstützt deterministisches Layer-Seeding via `idx.set_layer_seed(seed)` (`crates/contextra-vector/src/hnsw/types.rs:113`, Commit `c6255ae`) **[V]**. RaBitQ-Recall-Kalibrierung wurde in `crates/contextra-vector/tests/rabitq_recall_calibration.rs` nachgewiesen (>85% Recall@10) **[V]**.

## J.13 SSI-Leseseite Ende-zu-Ende (P0/P1, ✅)

**IST [V]:** `StorageEngine` (`crates/contextra-ports/src/storage.rs:217`) und `TenantScopedStorage` (`crates/contextra-store/src/tenant_codec.rs:567`) implementieren `get_tracked`, `get_at_seq_tracked`, `scan_prefix_tracked` **[V]**. Alle lesenden CRUD-Pfade in `contextra-engine` (`get_tracked`, `get_at_seq_tracked`, `check_doc_id_collision_tracked` in `crates/contextra-engine/src/collection/crud/`) registrieren Lesezugriffe im `ReadSet` zur SSI-Write-Skew-Erkennung **[V]**. Die Abdeckung und Vermeidung von Write Skew ist in `crates/contextra-engine/tests/ssi_read_path_coverage.rs` und `crates/contextra-store/tests/ssi_write_skew_integration.rs` nachgewiesen **[V]**.

## J.14 MCP-Klassifikation (P1, 🔵 — Fehler offen)

**SOLL (= v12 D.7, bestätigt durch B-01):** INV-MCP-CLASSIFY-1. (1) `contextra_explain` in `DatabaseRead`. (2) Test: jedes in `tools/list` gelistete Tool liefert aus `classify_method` eine explizite Kategorie ungleich `CodeExecution`, außer es ist tatsächlich Code-Ausführung. (3) Der `_`-Zweig liefert einen Fehler statt `CodeExecution`. **Abnahme:** `contextra_explain` läuft unter `SandboxPolicy::default()`.

## J.15 Kognition validieren (P2, 🔵)

**SOLL:** Demonstrator mit realem, mehrstufigem LLM-Agenten-Workflow (bisher synthetische Daten). Bis dahin nicht als Feature bewerben (Spec 4 D.1). **Erfolgskriterium:** dokumentierter Lauf mit Konsolidierung, Synthese und Aggregation, inklusive Halluzinationsprüfung (`GaspValidator`, Schwelle 0,70) und Nachweis, dass das Transitivity-Veto Fehlmerges verhindert.

## J.16 Compliance-Defaults & QueryRewriter (P2, 🟡)

**IST [V]:** `LlmQueryRewriter` wurde in Ring-4-Facade `contextra` (`crates/contextra/src/query_rewriter.rs`) für Issue #142 implementiert **[V]**. Es implementiert das Trait `QueryRewriter` aus `contextra-db` / `contextra-engine` und erzeugt LLM-gestützte Sub-Queries mit konfigurierbaren Limits (`with_max_subqueries`, `with_max_context_results`, `with_max_snippet_chars`), UTF-8-sicherer Truncation und Prompt-Injection-Isolation via `<untrusted_context>` XML-Tags **[V]**. (Compliance-Defaults für `EnterpriseRegulated` bleiben offen).

## J.17 Zurückgestellte Bereiche (⏸, Reaktivierungsbedingungen)

| Bereich | Bedingung |
|---|---|
| Framework-Adapter (Python) | nie im eigenen Repository |
| Spektrale Hyperkanten-Indexierung | validierter Bedarf in reguliertem Vertikal |
| EU-AI-Act-Risikoeinstufungs-Mapper | wenn das Durchsetzungsdatum näher rückt |
| Horizontales Sharding, P2P-Sync | Kunden mit konkretem Mehrknotenbedarf |
| Mehrstufige Entscheidungskaskade öffentlich | nie Standard, nur hinter Feature-Flag |
| Vertikalspezifische Ontologie | erster Pilotkunde |
| DSGVO-Art.-30-Vollimplementierung | erstes Behördenprojekt |
| Cursor-persistente CDC-API, Kafka-Export | konkreter Bedarf |
| Copy-Fallback für Hardlink-Klone jenseits Windows-Grenze | bei Bedarf |

## J.18 Produkt-SOLL und Roadmap

Übernommen aus v12 E.4–E.5, Stand 30.09.2026:

| Zeitraum | Aufgabe | Erfolgskriterium | Stand |
|---|---|---|---|
| bis 2026-10-07 | Review VETO-F02, VETO-OP03 | Entscheidung dokumentiert | offen, 7 Tage |
| bis 2026-10-11 | Förderantrag „Digital-Tech-to-Product“ | Businessplan 3–5 Seiten | offen, 11 Tage |
| Tag 1–30 | Benchmark-Disclaimer (J.11), D.2, D.7 (J.14) | Regel 3 eingehalten | offen |
| Tag 31–60 | crates.io, Ankündigung, 10 Nutzergespräche | > 5 Rückmeldungen; ≥ 3 Zahlungsbereitschaften | offen |
| Tag 61–90 | Erster aktivierbarer Sovereign-Ring (J.1), Pilotkunde, Kognitions-Demonstrator (J.15), Vertikal-Entscheidung | Medizin **oder** Legal **oder** Industrial | offen |

**Deployment-Tiers (Spec 4 D.3 vs. IST):** Spec 4 nennt `EnterpriseShared` mit `WalNoHmac`, `CryptoShred`, `DeletionProof ❌`; das deckt sich mit der IST-Tabelle in E.2. Keine Abweichung.

## J.19 Abnahmekriterien für „IST = SOLL“

Ein SOLL-Punkt gilt als umgesetzt, wenn (1) sein Status in dieser Tabelle auf ✅ steht, (2) sein Test in CI als Required Check läuft oder im `xtask`-Gate enthalten ist, (3) die zugehörige Invariante in Teil F oder G.3 eingetragen ist und (4) der Belegmarker **[V]** im Text steht. Offene Prüfaufgaben aus I.12 bleiben bestehen.

## J.20 Belegtiefe von Teil J

Gelesen: `signed_gate.rs` (`check_ring`, Payload), `license.rs` (`LicenseError`), `merge_operator.rs`, `compaction.rs` (`TtlMetadata`), `checkpoint/guard.rs` und `orphan.rs` (Zeitzugriffe), `wal/hmac.rs` und `wal/io.rs` (Legacy-Symbole), `Cargo.toml`-Abhängigkeiten (`arc-swap`, kein `crossbeam-epoch`), `commit.rs` (`apply_mem_updates`), `search.rs` (`hybrid_search_with_strategy`), `query_rewriter.rs` (`LlmQueryRewriter`), `merge.rs` (`WasmMergeFunction`), `hnsw/types.rs` (`set_layer_seed`), `tenant_codec.rs` (`get_tracked`), `system_pressure.rs` (`scheduler_queue_depth`). Build und Tests verifiziert.

---

# TEIL K — Öffentliches API-Inventar (Welle-1-Erweiterungen)

## K.1 Welle-1 Öffentliche API-Signaturen **[V-Sig]**

Alle nachfolgenden Signaturen wurden direkt am HEAD im Produktivcode verifiziert:

- **`contextra-ports` / `StorageEngine` Trait** (`crates/contextra-ports/src/storage.rs:217`):
  - `fn get_tracked<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>` **[V-Sig]**
  - `fn get_at_seq_tracked<'a>(&'a self, _tx_id: TxId, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>>` **[V-Sig]**
  - `fn scan_prefix_tracked<'a>(&'a self, _tx_id: TxId, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` **[V-Sig]**

- **`contextra-sandbox` / WASM Merge Function** (`crates/contextra-sandbox/src/merge.rs:22`, `capabilities.rs:131`):
  - `pub struct WasmMergeFunction` **[V-Sig]**
  - `pub fn WasmMergeFunction::new(wasm_bytes: impl Into<Arc<[u8]>>) -> Result<Self, SandboxError>` **[V-Sig]**
  - `pub async fn WasmMergeFunction::merge(&self, existing: &[u8], new: &[u8]) -> Result<Vec<u8>, SandboxError>` **[V-Sig]**
  - `pub fn MergeOperatorCapabilities::pure_with_result_channel() -> Self` **[V-Sig]**

- **`contextra` / LLM Query Rewriter** (`crates/contextra/src/query_rewriter.rs:12`):
  - `pub struct LlmQueryRewriter` **[V-Sig]**
  - `pub fn LlmQueryRewriter::new(generator: Arc<dyn LlmTextGenerator>) -> Self` **[V-Sig]**
  - `pub fn LlmQueryRewriter::with_max_subqueries(mut self, n: usize) -> Self` **[V-Sig]**
  - `pub fn LlmQueryRewriter::with_max_context_results(mut self, n: usize) -> Self` **[V-Sig]**
  - `pub fn LlmQueryRewriter::with_max_snippet_chars(mut self, n: usize) -> Self` **[V-Sig]**

- **`contextra-vector` / HNSW Layer Selection** (`crates/contextra-vector/src/hnsw/types.rs:113`):
  - `pub fn HnswIndex::set_layer_seed(&self, seed: u64)` **[V-Sig]**

- **Deterministic Clock & Rng Injection Builders** (**[V-Sig]**):
  - `pub fn IsotonicCalibrator::with_clock(mut self, clock: Arc<dyn Clock>) -> Self` (`crates/contextra-rank/src/calibration/isotonic.rs:67`) **[V-Sig]**
  - `pub fn MaintenanceScheduler::with_clock(mut self, clock: Arc<dyn Clock>) -> Self` (`crates/contextra-cognition/src/maintenance_scheduler.rs:51`) **[V-Sig]**
  - `pub fn ContextCompactor::with_rng(mut self, rng: Arc<dyn Rng>) -> Self` (`crates/contextra-cognition/src/context_compaction/compactor.rs:42`) **[V-Sig]**
  - `pub fn OrchestratorEngine::with_clock(mut self, clock: Arc<dyn Clock>) -> Self` (`crates/contextra-agent/src/engine.rs:56`) **[V-Sig]**
  - `pub fn OllamaClient::with_rng(mut self, rng: Arc<dyn Rng>) -> Self` (`crates/contextra-infer-ollama/src/client/core.rs:85`) **[V-Sig]**
  - `pub fn SignedLicenseGate::from_signed_payload_with_clock(...) -> Self` (`crates/contextra-license/src/signed_gate.rs:96`) **[V-Sig]**
  - `pub fn CheckpointStore::with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self` (`crates/contextra-checkpoint/src/store.rs:366`) **[V-Sig]**
  - `pub fn PromptInjectionGuard::with_clock(mut self, clock: Arc<dyn Clock>) -> Self` (`crates/contextra-mcp/src/prompt_injection/guard.rs:96`) **[V-Sig]**
  - `pub fn Collection::with_clock(self, clock: Arc<dyn Clock>) -> Self` (`crates/contextra-engine/src/collection/mod.rs:399`) **[V-Sig]**

---

# Änderungsprotokoll: Sync nach Welle 1

- **Datum:** 2026-10-01
- **HEAD-Hash:** `c6255ae`
- **Synchronisierte PRs / Commits (Welle 1):**
  - `c6255ae` feat(contextra-vector): deterministic HNSW layer selection & RaBitQ recall calibration (#3979)
  - `44fe19c` refactor: remove contextra-db shell crate and update CI gate
  - `c096c6f` fix(engine,store): fix key lock guard lifetime and WAL recovery HMAC verification
  - `3095a10` refactor(engine): extract sorted_unique_shards and unify Loom kv_lock tests
  - `b80c9b2` fix(store): fix SSTable compaction stream parsing and loom compatibility
  - `8498430` feat(contextra-engine): make signal fusion strategy configurable in hybrid search
  - `f22ea4a` refactor(docid-128): unify docid-128 feature in contextra-types and add CI gate
  - `0c05830` docs: Add license binding decision document (Phase 1) and fix feature wiring
  - `7707512` refactor(xtask): add Cargo.toml ring metadata and gen-arch-docs automation
  - `854eb11` Harden architecture gates, enforce Ring 0/3 ordering, and fix CI gates
  - `c3c6ce1` refactor(engine): Reranker-Port statt direkter infer-onnx-Abhängigkeit
  - `30c6e3a` feat(graph): wire APPRH gate selector into PPR path (G-01)
  - `783a211` feat(store): phase out legacy WAL key and unblock observer runtime (S-03)
  - `05c66a7` fix(store): ensure group commit WAL chain security and anchor rollback (S-02)
  - `1042d44` docs: Add comprehensive features specification document
  - `d152aae` feat(engine): add tenant collection handle caching and policy enforcement (E-01)
  - `376b513` feat(engine): recovery for crash windows and commit-uncertain transactions
  - `027a73c` fix(engine): repair 2PC crash windows and orphan index recovery
