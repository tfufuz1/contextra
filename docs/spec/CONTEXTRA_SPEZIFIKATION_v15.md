# Contextra — Systemspezifikation v15 (IST- und SOLL-Zustand)

**Stand:** 29.09.2026 · **Code-Basis IST:** `https://github.com/tfufuz1/contextra`, HEAD `e4d84c9` (30.09.2026; 33 Crates, ca. 289 k Zeilen Rust in `crates/` + `xtask/`, Toolchain 1.89.0)
**Status dieses Dokuments:** Es **ersetzt alle früheren Spezifikationen** (v4, v8, v9, v10, v11). Es enthält (1) den **IST-Zustand**, also alles, was im Quellcode implementiert ist, und (2) den **SOLL-Zustand**, also alles, was noch zu implementieren oder zu entscheiden ist. Befunde zu vorübergehenden Fehlern sind bewusst nicht enthalten. Wo früher ein Fehler stand, steht jetzt die Soll-Anforderung als Invariante.
> **v15 (30.09.2026, vollständiges Inventar):** Neu sind **Teil K** (API- und Feature-Inventar aller 33 Crates, Tool- und Env-Verzeichnis, Aktualisierungen U-01…U-12 gegen HEAD `b564e830`) und **Teil L** (Verhaltensspezifikation bisher unbeschriebener Subsysteme). Die Teile 0–J bleiben unverändert; Widersprüche löst K.0.1.
>
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
- 🔵 WASM-gestützter `MergeOperator` im Compaction-Pfad vollständig verdrahten: rein (kein I/O, keine `Clock`, kein `Rng`), `max_fuel` (Default 10.000.000) und Wall-Clock-Timeout; bei `FuelExhausted` beide Versionen behalten.
- ⏸ Cursor-persistente CDC-Subscriber-API (überlebt Neustarts) und externer Kafka-artiger Export als Ring-4-Adapter.
- ⏸ Copy-Fallback für Hardlink-Klone jenseits der Windows-Grenze (bewusst nicht implementiert; nur bei Bedarf).
- 🔵 Dokumentationsauflage: asymmetrische Lock-Stärke (`write` vs. `read` auf `LsmState`) für `apply_mem_updates` zwischen Single- und Group-Commit-Pfad in `lsm/commit.rs` begründen oder angleichen.
- 🔵 Metrik `blocking_util` soll die Auslastung des Blocking-Pools messen (IST: Queue-Tiefe **[?]**).

## D.5 Retrieval (P2)
- 🔵 `hybrid_search_with_strategy` verwendet fest RRF. `FusionStrategy::ScoreNormalized` (MinMax-CombSUM mit RRF-Fallback) soll dort wählbar werden.
- 🔵 Diffusionsvarianten (APPRH, TL-HFD, k-Path): Flip von Shadow auf Produktion erst nach Gate (≥ 10.000 Samples, Agreement ≥ 0,85; APPRH: 10 aufeinanderfolgende Gate-Passes im Fenster 20).
- 🔵 `RaBitQ`-Kalibrierung dokumentieren und gegen Recall benchmarken; INT4/FP16-Mixed-SIMD nur nach Zielhardware-Benchmark (⏸).
- 🔵 Deutsche Domänenvokabulare (Medizin, Recht) von Stub auf produktive Wörterbuchgröße ausbauen; Feature `bm25f` in den dokumentierten Default-Pfad aufnehmen oder klar als Opt-in ausweisen.
- 🔵 PathRAG snapshot-fähig machen (IST: Fehler `snapshot_unsupported_for_signal`).

## D.6 Kognition und Compliance (P2–P3)
- 🔵 Validierung der Kognitions-Pipeline gegen einen realen mehrstufigen LLM-Agenten-Workflow (bisher synthetische Daten). Bis dahin nicht als Feature bewerben.
- 🔵 Compliance-Betrieb: Auto-Entity-Extraktion (IST: standardmäßig an) muss über `auto-extraction-opt-out` abschaltbar dokumentiert und in `EnterpriseRegulated` als Default aus vorgesehen werden (Entscheidung offen).
- ⏸ Vollständiger DSGVO-Art.-30-Export und AVV-Generator: Aktivierung mit erstem Pilotkunden (Code vorhanden, hinter `FeatureRing::Compliance`).
- 🔵 Ollama-`QueryRewriter` für MultiStep-Retrieval (Issue #142); Sub-Queries laufen IST nur über BM25 **[D]**.

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
| INV-MCP-CLASSIFY-1 | Jedes gelistete MCP-Tool klassifiziert | ✅ (K.0.1 U-01; `TOOL_REGISTRY`) |
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
| G1-4 | Teil C: „SSI-Read-Set-Tracking ✅ IST“; der Prompter-Plan (Prompt 1) behauptet, die Leseseite sei nicht verdrahtet | Teilweise beides. Der Trait `StorageEngine` hat `get_tracked`, `get_at_seq_tracked`, `scan_prefix_tracked` (Default: **ohne** Read-Set-Registrierung, Doc-Kommentar); `Collection` hat `get_tracked` und `get_at_seq_tracked` (`engine/collection/crud/read.rs`); Test `mvcc/tests/ssi_write_skew.rs` existiert. **In v14 nachgeprüft (J.13):** `LsmStorage` überschreibt alle drei Methoden (`store/src/lsm/ops.rs:21-46`), und die schreibenden Engine-Pfade nutzen sie. **Aber:** Sie registrieren Lesezugriffe mit `snapshot_seq = u64::MAX`, was die Konfliktprüfung für diese Schlüssel wirkungslos machen kann (Befund B-16) | V-30.09 |
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
| B-01 | `contextra_explain` scheitert bei Default-Policy (siehe G1-3). Fix: Tool in `DatabaseRead`; Test über alle gelisteten Tools gegen `classify_method` (= INV-MCP-CLASSIFY-1) | **hoch** | **behoben** (K.0.1 U-01) |
| B-02 | Default-Zweig von `classify_method` ist fail-closed in der falschen Kategorie; besser: unbekannt ⇒ Fehler | mittel | offen, V |
| B-03 | SSI-Leseseite: Verdrahtung in `LsmStorage` bestätigt (v14); Engine-Lesepfade teils ungetrackt (Suche, Wartung); Rest siehe B-16 | mittel | geprüft, V |
| B-04 | 15 von 25 Harness-Phasen-Einträgen `required = false` (G1-6) | mittel | offen, V |
| B-05 | Miri- und Feature-Matrix-Job nicht Required Check (G1-5) | mittel | offen, V |
| B-06 | Benchmark-Beleg endet bei 10.000 Dokumenten; Skalierungsaussagen darüber sind Extrapolation (aus dem Plan, `summary.md` zeigt keinen 100k-Lauf) | mittel | offen |
| B-07 | Spannung F-02 ↔ `partial-index-rebuild` (jetzt als Opt-in) | mittel | Review 2026-10-07 |
| B-08 | Auto-Extraktion standardmäßig an; in `EnterpriseRegulated` als Default aus vorsehen (Entscheidung offen) | mittel | offen, D |
| B-09 | SSTable-mmap `UNIMPLEMENTED`, Issue-Platzhalter | mittel | offen, D |
| B-10 | KDF-Migration AGT-CRYPTO-002 | mittel | offen, D |
| B-11 | Zwei Env-Variablen für den Löschbeweis-Schlüssel | mittel | offen, V |
| B-12 | `DeletionProofKeyPair`, `LayerCleanupProof` doppelt in `crypto` und `engine` | niedrig | offen, D |
| B-13 | `INV-CAL-1/3` ohne Definitionsort im Produktivcode | niedrig | offen, V |
| B-14 | Doc-Kommentar `types/saos.rs` nennt noch „Synthesized Agent Operating System“ (Datei enthält Query-/Fusionstypen); `deletion_proof.rs`-Kopf nennt noch HMAC | niedrig | offen, V/D |
| B-15 | `blocking_util` misst Queue-Tiefe; Domänenvokabulare nur Stubs; `bm25f` nicht im Default | niedrig | offen |
| B-16 | **SSI-Lesepfade der Engine mit `snapshot_seq = u64::MAX`.** `insert.rs:321/325`, `update.rs:63/67`, `delete.rs:42/46`, `links.rs:65/139` und `relate.rs:31` rufen `get_at_seq_tracked(tx, key, u64::MAX)`. `ReadSet::record_read` behält pro Schlüssel den **niedrigsten** `snapshot_seq`; die Validierung meldet einen Konflikt nur bei `commit_seq > snapshot_seq` (`mvcc/src/tx_buffer.rs:379`). Für einen Schlüssel, der in der Transaktion zuerst so gelesen wird, kann das nie zutreffen. Schützt nur, wenn derselbe Schlüssel vorher mit echtem Snapshot gelesen wurde (so im Test `write_skew_via_engine_api.rs`). **Nicht ausgeführt**, deshalb Verdacht, kein bewiesener Fehler | hoch (bis geprüft) | V (Code), ungetestet |

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
| INV-MCP-CLASSIFY-1 / `contextra_explain` | ✅ behoben | K.0.1 U-01; offen nur B-02 |
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

**Offene Prüfaufgaben für eine nächste Runde** (Reihenfolge nach Risiko; SSI-Leseseite in `LsmStorage` ist seit v14 geprüft, siehe J.13 und B-16): `WalNoHmac`-Format und Replay, `repair_on_open`-Ablauf, `Default`-Werte von `MaintenanceConfig`, SSTable v3/v4, Fusionspfad in `hybrid_search_with_strategy`, Tier-Defaults von `AutoExtractionMode`.

---

# TEIL J — SOLL-Zustand, geprüft gegen den IST-Code (neu in v14)

**Quelle:** `CONTEXTRA_SPEZIFIKATION_4_1_.md` (Teil C, C.4, D, X.10, X.11, Y, R.5) sowie die offenen Punkte aus Teil D, G, H und I dieses Dokuments. **Methode:** Für jeden SOLL-Punkt habe ich geprüft, ob der Code ihn schon umsetzt. Status: ✅ erledigt · 🟡 teilweise · 🔵 offen (spezifiziert) · ⏸ zurückgestellt. Marker **[V]** = im Code gelesen am HEAD `81ab98a`, **[D]** = aus Dokument übernommen.

## J.0 Konsolidierte Matrix

| Nr. | SOLL-Punkt | Quelle | Status am HEAD | Prio |
|---|---|---|---|---|
| J.1 | Installation-gebundene Lizenzaktivierung, Ring-Hierarchie | Spec 4 C.2.1, v12 D.1 | 🟡 | P1 |
| J.2 | Entfernung des Legacy-WAL-Schlüssels | Spec 4 X.11.3, v12 D.2 | 🟡 | P1 |
| J.3 | Determinismus im Checkpoint-Crate (INV-CHECKPOINT-DETERMINISM-1) | Spec 4 X.11.2 | 🟡 | P1 |
| J.4 | WASM-`MergeOperator` im Compaction-Pfad | Spec 4 C.4.3.2 | 🟡 | P2 |
| J.5 | TTL (Entscheidung „sequenzbasiert“ statt Storage-Nanosekunden) | Spec 4 C.4.3.1 | ✅ (abweichend) | — |
| J.6 | Sidecar-Spezialindizes (`SstableFooterPayload`) | Spec 4 C.4.4 | ⏸ | P3 |
| J.7 | Lock-Asymmetrie im Commit-Pfad | Spec 4 X.11.4 | 🔵 | P2 |
| J.8 | P28-Text: Kryptografie-Ausnahme | Spec 4 X.11.1 | 🔵 | P1 |
| J.9 | Lock-freie Caches und RCU-Graph (EBR, `arc_swap`) | Spec 4 Y.10 | ⏸ | P3 |
| J.10 | INT4/FP16-Mixed-SIMD | Spec 4 Y.9 | ⏸ | P3 |
| J.11 | Benchmark-Wahrheit und 100k-Lauf | Spec 4 D.2, Regel 3; G1-8 | 🔵 | P1 |
| J.12 | Hybrid-Suche: Fusionsstrategie (✅ wählbar, K.0.1 U-02) und PathRAG-Snapshot (offen) | v12 D.5 | 🟡 | P2 |
| J.13 | SSI-Leseseite Ende-zu-Ende (Verdacht B-16: `u64::MAX`) | v12 Teil C, G1-4 | 🟡 | **P0** |
| J.14 | MCP-Klassifikation (INV-MCP-CLASSIFY-1) | v12 D.7 | ✅ behoben am HEAD `b564e830` (K.0.1 U-01); B-02 offen | P1 |
| J.15 | Kognition gegen realen Workflow validieren | Spec 4 D.1 | 🔵 | P2 |
| J.16 | Compliance-Defaults (Auto-Extraktion, Art. 30, AVV) | v12 D.6 | 🔵 | P2 |
| J.17 | Zurückgestellte Bereiche mit Reaktivierungsbedingung | Spec 4 R.5 | ⏸ | P3 |
| J.18 | Produkt- und Markt-SOLL, Roadmap | Spec 4 F, H | 🔵 | — |

## J.1 Lizenz: Installation-Bindung und Ring-Hierarchie (P1, 🟡)

**IST [V]** (`license/src/signed_gate.rs`): `SignedLicenseGate::from_signed_payload[_with_clock]`, Zugriff `license_payload()`, `verifying_key()`, Testhelfer `create_test_signed_payload`. `LicensePayload{tenant_id, allowed_rings, expires_at, feature_flags}`. `check_ring(ring)`:
1. `Fast` → `Ok` (INV-LICENSE-2).
2. `ring` nicht in `allowed_rings` → `NotActivated(ring)`.
3. `expires_at` gesetzt und `clock.now_unix_nanos()/1e9 >= expires_at` → `Expired(expires_at)`.
4. sonst `Ok`.
**Nachgeprüft in v14 [V]:** `from_signed_payload_with_clock(payload_bytes, signature: &[u8;64], verifying_key, clock)` prüft die Ed25519-Signatur über die rohen Payload-Bytes und deserialisiert danach per `bincode`. **Beide** Fehlerfälle (Signatur ungültig, Payload nicht lesbar) liefern `InvalidSignature`. `check_ring` prüft die Signatur nicht erneut; der verifizierte Payload ist unveränderlich im Gate gespeichert. Ohne Clock-Argument nutzt `from_signed_payload` `SystemClock::new()`. Ein Payload ohne `installation_id_hash` ist damit nicht an eine Installation gebunden.

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
**Nachgeprüft in v14 [V]:** Die Zeitstempel sind **nicht nur Diagnose.** `CheckpointStore::monotonic_timestamp_ms()` (`store.rs:362`) liest die Wandzeit über `orphan::monotonic_timestamp_ms()` und macht sie per `fetch_max` auf `checkpoint_counter` monoton. Das Ergebnis landet in `timestamp_ms` (`store.rs:373-376`) und in `created_at` (`store.rs:406`) der Checkpoint-Metadaten. Die Orphan-Zeitstempel (`guard.rs`, `orphan.rs`) werden in `PinnedSeqNoOrphan.timestamp_ms` gespeichert; in der Codesuche fand ich nur Zuweisungen und Tests, keinen Vergleich im Produktivpfad. **Folge:** Zwei Läufe mit identischer Eingabe erzeugen unterschiedliche Checkpoint-Metadaten, Replay ist dort nicht byte-deterministisch. Die monotone Zählung per `fetch_max` bleibt als Schutz gegen Zeitrückläufe sinnvoll und wird mit dem `Clock`-Port beibehalten. Ob die Metadaten persistiert werden, habe ich nicht geprüft.

## J.4 WASM-`MergeOperator` im Compaction-Pfad (P2, 🟡)

**IST [V]:** `store/src/compaction/merge_operator.rs`: Trait `MergeOperator: Send + Sync` mit `merge` und Fail-Safe-Vorgabe (bei Fehler oder Fuel-Erschöpfung beide Versionen behalten, `trace`-Log). `sandbox/src/capabilities.rs:81`: `MergeOperatorCapabilities`. Im Store und in der Engine gibt es **keine** Verwendung des Traits außer Export: `CompactionEngine` ruft keinen `MergeOperator` auf, und es existiert keine `impl MergeOperator`.
**SOLL (Spec 4 C.4.3.2):** Nutzerdefinierte Merge-Logik nur als WASM-Modul über `contextra-sandbox`. Schnittstelle: `merge(key, old, new) -> Result<Vec<u8>, SandboxError>`. Grenzen: `max_fuel` Default `10_000_000`, Wall-Clock-Timeout, `MergeOperatorCapabilities::pure()` (kein I/O, keine `Clock`, kein `Rng`). Verdrahtung: Die Engine (Ring 3) übergibt den Operator an `CompactionEngine`, der Store kennt die Sandbox nicht (Ring-Regel: `store` darf nicht von `sandbox` abhängen). **Invarianten:** deterministisch, rein; bei `FuelExhausted` Fail-Safe ohne Datenverlust. **Abnahme:** (1) Ein Zähler-Merge über zwei SSTable-Ebenen liefert die Summe. (2) Endlosschleife im Modul ⇒ beide Versionen bleiben. (3) Zwei Läufe mit gleichem Modul und Eingaben ⇒ byte-gleiches Ergebnis.

## J.5 TTL (✅, abweichend von Spec 4)

**IST [V]:** `store/src/compaction.rs:20` `pub struct TtlMetadata;` ist ein leerer Stub. Der Ablauf läuft sequenzbasiert: Metadatum `__expires_at_seq` und `start_expiry_cleanup_worker` in der Engine (v12 Teil C) **[D/V]**. **Entscheidung:** Sequenz statt Uhrzeit, damit TTL ohne Wall-Clock reproduzierbar bleibt. **Damit entfällt `INV-TTL-1` aus Spec 4** (Vergleich `expires_at_unix_nanos < clock.now`); Ersatz: **INV-TTL-2:** Ablauf wird ausschließlich aus `seq`/`TxId`-Distanz abgeleitet. `TtlMetadata` bleibt Stub oder wird entfernt (Aufräumaufgabe, P3).

## J.6 Sidecar-Spezialindizes (⏸)

**IST [V]:** `SstableFooterPayload` kommt im Workspace nicht vor; `SstableBuilder` hat kein reserviertes Blob-Feld.
**SOLL (zurückgestellt):** Trait `SstableFooterPayload{payload_kind() -> &str; serialize() -> Vec<u8>}` im Store; der Kern schreibt den Blob unverändert ans Dateiende und gibt ihn unverändert zurück. Orchestrierung nur durch `contextra-engine`. Erfordert eine Dateiformat-Version (aktuell SSTable `MFSX`, Formate v3/v4 laut Lückenliste). **Reaktivierungsbedingung:** Ein Nutzer stößt an die In-Memory-HNSW-Grenze und DiskANN reicht nicht.

## J.7 Lock-Asymmetrie im Commit-Pfad (P2, 🔵)

**Befund aus Spec 4:** `apply_mem_updates` werde im Single- und im Group-Commit-Pfad mit unterschiedlicher Sperrstärke (`write` vs. `read` auf `LsmState`) aufgerufen. **Am HEAD nicht bestätigt:** `apply_mem_updates` steht in `lsm/commit.rs:93`; die Zeilenverweise aus Spec 4 (`:298`, `:456`) passen nicht mehr. **SOLL:** Sperrstärke im Doc-Kommentar begründen oder angleichen; Test, der beide Pfade unter Nebenläufigkeit gegen einen Referenzstand vergleicht (Loom).

## J.8 P28-Text und Kryptografie-Ausnahme (P1, 🔵)

**IST [V]:** `store/src/wal/hmac.rs:227` nutzt `rand::thread_rng().fill_bytes(&mut key)` für Schlüsselmaterial. v12 Teil F (P28) nennt die Ausnahme „kryptografisches Schlüssel-/Salt-Material“ bereits. **SOLL:** Dieselbe Formulierung in `AGENTS.md` und `CONSTITUTION.md` (ich habe dort keine P28-Zeile gefunden), plus ein `xtask`-Check, der `thread_rng`/`SystemTime::now` im Produktionscode verbietet und genau diese Fundorte per Allowlist zulässt.

## J.9 Lock-freie Caches und RCU-Graph (⏸)

**IST [V]:** `arc-swap` ist in `contextra-graph` und `contextra-router` Abhängigkeit (RCU-Snapshot-Swap); `crossbeam-epoch` kommt in keinem `Cargo.toml` der Crates vor. SIEVE-Cache existiert als `SieveCacheBackend` (Feature `sieve-cache`).
**SOLL (Backlog):** Vollständige Lock-Freiheit des Block-Caches über epochenbasierte Freigabe. **Bedingung:** gemessene Sperrenkontention im Benchmark, neue Abhängigkeit nur nach Governance-Regel 1 (schriftliche Begründung, 48 h).

## J.10 INT4/FP16-Mixed-SIMD (⏸)

Kein Code. **Bedingung:** Benchmark auf Zielhardware; CPU-Overhead des Bit-Unpackings gegen Bandbreitengewinn. Bis dahin nicht bauen.

## J.11 Benchmark-Wahrheit (P1, 🔵)

**SOLL:** (1) Ein öffentlicher Benchmarksatz, jede Zahl mit dem Hinweis „misst nur Speicher- und Indexlatenz; Embedding-Inferenz addiert 10–500 ms“ (Spec 4 D.2, Regel 3). (2) Gemessener Lauf bei 100.000 Dokumenten (Intervalle je 10.000, Einfügerate, Speicher, Latenz), nicht extrapoliert; bei Abbruch die Abbruchstelle dokumentieren. (3) `benchmarks/results/summary.md` nur ergänzen. **IST [V]:** Es liegen `ann_results.json`, `criterion-baseline.json`, `results.json`, `summary.md` vor; ein 100k-Lauf ist nicht belegt (G1-8).

## J.12 Hybrid-Suche (P2, 🟡)

**IST:** Commit `8498430` macht die Fusionsstrategie wählbar **[D]**; v12 B.3.5 beschreibt den früheren Zustand mit festem RRF. **SOLL:** (1) B.3.5 nach Sichtung des Codepfads aktualisieren. (2) `ScoreNormalized` (MinMax-CombSUM, Fallback RRF) mit Test, dass Degradation eines Signals auf RRF zurückfällt. (3) PathRAG snapshot-fähig machen (IST: Fehler `snapshot_unsupported_for_signal`). **Invariante:** alle Signale einer Anfrage lesen denselben `seq` (B.11 Nr. 6).

## J.13 SSI-Leseseite Ende-zu-Ende (P0, 🟡 — Befund B-16)

**IST, in v14 nachgeprüft [V]:**
- `StorageEngine` (`ports/src/storage.rs`) hat `get_tracked`, `get_at_seq_tracked`, `scan_prefix_tracked`; die Trait-Defaults registrieren **nichts**.
- `LsmStorage` überschreibt alle drei (`store/src/lsm/ops.rs`, Zeilen 21–46) und delegiert an `lsm/ops/read.rs`. `get_tracked` liest `snapshot_seq = last_applied_seq`, registriert den Schlüssel per `tx_buffer.register_read(tx, key, snapshot_seq)` und liest dann `get_at_seq`. `scan_prefix_tracked` registriert **jeden gescannten Schlüssel**, ruft aber `record_prefix` nicht auf. Der Doc-Kommentar in `lsm/engine.rs:177-181` sagt selbst: Nur getrackte Lesezugriffe erhalten SSI-Schutz, `get()` bleibt ungetrackt.
- `ReadSet::record_read` behält je Schlüssel den niedrigsten `snapshot_seq`; `record_prefix` existiert für Phantomschutz. Validierung (`mvcc/src/tx_buffer.rs` ab Zeile 366): `snapshot_seq < pruned_through` ⇒ Fehler „snapshot too old“; Konflikt, wenn `commit_seq > snapshot_seq`.
- **Engine, getrackt:** `Collection::get_tracked`/`get_at_seq_tracked` (`crud/read.rs:271,282`), `check_doc_id_collision_tracked` (`insert.rs:261`), Schreibpfade in `insert.rs`, `update.rs`, `delete.rs`, `links.rs`, `relate.rs:31`, `maintenance.rs:651`.
- **Engine, ungetrackt:** Suche und Hydration (`search/hydrate.rs:23,70`, `get_at_seq`), Wartung (`maintenance.rs`, `get`/`scan_prefix`). Suche ist lesend; ungetrackt heißt dort: keine Write-Skew-Validierung.
- **Tests:** `mvcc/tests/ssi_write_skew.rs`, `engine/tests/write_skew_via_engine_api.rs` (Arzt-Szenario mit `get_tracked`, `update_op`, Commit-Konflikt), `engine/tests/ssi_write_skew_integration.rs`, `store/tests/ssi_tracked_reads_via_trait.rs`.

**Befund B-16 (Verdacht, nicht ausgeführt):** Die Schreibpfade lesen den Vorwert mit `get_at_seq_tracked(tx, key, u64::MAX)`. Wird ein Schlüssel in der Transaktion zuerst so gelesen, steht `snapshot_seq = u64::MAX` im `ReadSet`, und `commit_seq > u64::MAX` ist nie wahr. Der Test `write_skew_via_engine_api.rs` ist grün, weil er die Schlüssel vorher mit `get_tracked` (echter Snapshot) liest; der niedrigere Wert gewinnt. Ein Read-Modify-Write ohne vorheriges `get_tracked` ist damit vermutlich nicht gegen einen parallelen Commit auf denselben Schlüssel abgesichert.

**SOLL:**
1. **Reproduktionstest zuerst:** Zwei Transaktionen rufen direkt `update_op` bzw. `insert` auf denselben Schlüssel auf, ohne vorheriges `get_tracked`; beide committen. Erwartung: genau eine scheitert mit Konflikt. Schlägt der Test fehl, ist B-16 bestätigt.
2. **Fix, falls bestätigt:** Die Engine übergibt den Begin-Snapshot der Transaktion statt `u64::MAX`; `get_at_seq_tracked` darf `u64::MAX` nicht als Registrierungs-Seq akzeptieren (Clamp auf `last_applied_seq` oder Fehler). Gelesen werden darf weiter der neueste Stand, registriert muss der Begin-Snapshot werden.
3. **Phantomschutz:** `scan_prefix_tracked` ruft zusätzlich `record_prefix` auf.
4. **Lesepfad-Tabelle** als Testliste pflegen: jeder lesende Engine-Pfad ist entweder „trackt“ oder mit Begründung „read-only“.
**Ring-Regel:** `contextra-ports` bekommt keine Abhängigkeit auf `mvcc` oder `store`. **Invariante INV-MVCC-SSI-1.**

## J.14 MCP-Klassifikation (P1, 🔵 — Fehler offen)

**SOLL (= v12 D.7, bestätigt durch B-01):** INV-MCP-CLASSIFY-1. (1) `contextra_explain` in `DatabaseRead`. (2) Test: jedes in `tools/list` gelistete Tool liefert aus `classify_method` eine explizite Kategorie ungleich `CodeExecution`, außer es ist tatsächlich Code-Ausführung. (3) Der `_`-Zweig liefert einen Fehler statt `CodeExecution`. **Abnahme:** `contextra_explain` läuft unter `SandboxPolicy::default()`.

## J.15 Kognition validieren (P2, 🔵)

**SOLL:** Demonstrator mit realem, mehrstufigem LLM-Agenten-Workflow (bisher synthetische Daten). Bis dahin nicht als Feature bewerben (Spec 4 D.1). **Erfolgskriterium:** dokumentierter Lauf mit Konsolidierung, Synthese und Aggregation, inklusive Halluzinationsprüfung (`GaspValidator`, Schwelle 0,70) und Nachweis, dass das Transitivity-Veto Fehlmerges verhindert.

## J.16 Compliance-Defaults (P2, 🔵)

**SOLL:** (1) In `EnterpriseRegulated` Auto-Extraktion standardmäßig aus (Entscheidung offen; heute `AutoExtractionMode::Enabled` ohne Feature `auto-extraction-opt-out`). (2) Art.-30-Export und AVV-Generator bei erstem Pilotkunden aktivieren (Code vorhanden, `FeatureRing::Compliance`). (3) Ollama-`QueryRewriter` (Issue #142).

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

Gelesen: `signed_gate.rs` (Konstruktor, `check_ring`, Payload), `license.rs`, `merge_operator.rs`, `compaction.rs` (`TtlMetadata`), `checkpoint/guard.rs`, `orphan.rs`, `store.rs` (Zeitzugriffe), `wal/hmac.rs` und `wal/io.rs` (Legacy-Symbole), `store/lsm/ops/read.rs`, `ops.rs`, `engine.rs` (Tracked-Reads), `mvcc/ssi.rs` und `tx_buffer.rs` (ReadSet, Validierung), Engine-Schreibpfade, `Cargo.toml`-Abhängigkeiten. **Nicht gelesen:** Lock-Stärke in `commit.rs` (J.7), `.rekeyed`-Migration (J.2), Fusionspfad der Hybrid-Suche (J.12), ob die Engine einen Begin-Snapshot der Transaktion bereitstellt. **Nicht ausgeführt:** Build und Tests, insbesondere der Reproduktionstest für B-16.

---

# TEIL K — Vollständiges API- und Feature-Inventar (neu in v15)

**Zweck:** Teil K schließt die Lücke zwischen den Teilen A–J (Kernverhalten, Invarianten, SOLL) und dem tatsächlichen Code. Er enthält für **alle 33 Crates** jede öffentliche Datei, jeden `pub`-Typ (struct, enum, trait, type), jede öffentliche Konstante und freie Funktion sowie Feature-Flags und Workspace-Abhängigkeiten.
**Code-Basis:** `https://github.com/tfufuz1/contextra`, HEAD `b564e830` (30.09.2026). 35 Workspace-Mitglieder = 33 Crates + `benchmarks/contextra-bench` + `xtask`. 1.193 `.rs`-Dateien, 290.708 Zeilen in `crates/` + `xtask/`.
**Vorrang:** Bei Widerspruch zwischen Teil K/L und den Teilen B–J gilt **K.0.1** (Aktualisierungen) und danach Teil K/L. Teil 0 schlägt weiterhin alles.

**Belegmarker in Teil K:** **[V-Sig]** = per Parser (tree-sitter-rust) aus dem Quelltext extrahiert: Name, Sichtbarkeit, Feld-/Variantennamen, Methodennamen, Konstantenwert und erster Satz des Doc-Kommentars. Verhalten wurde daraus **nicht** abgeleitet und nichts ausgeführt. Das Verhalten der wichtigsten bisher unbeschriebenen Subsysteme steht in **Teil L** mit **[V]** (Ablauf gelesen).
**Nicht enthalten:** Testcode (`tests/`, `#[cfg(test)]`, `fuzz/`, `benches/`), `pub(crate)`-Items, Re-Exports (`pub use`) und Methoden fremder Typen. Methoden von Typen erscheinen als Namensliste (max. 14), nicht mit Signatur. Doc-Kommentare sind unverändert aus dem Code übernommen (teils Englisch, teils Deutsch) und können veralten; bei Widerspruch gilt der Code.

## K.0.1 Aktualisierungen gegenüber Teil A–J (geprüft gegen HEAD `b564e830`)

| Nr. | Bisherige Aussage | Befund am HEAD | Wirkung |
|---|---|---|---|
| U-01 | B-01 / G1-3 / J.14 / D.7: `contextra_explain` ist nicht klassifiziert, scheitert unter Default-Policy | **Behoben.** `contextra-mcp/src/sandbox.rs` enthält `TOOL_REGISTRY` (Tool, Kategorie, Beschreibung, Schema an einer Stelle); `classify_method` sucht dort. `contextra_explain` → `DatabaseRead`. Test in `mcp/src/tests.rs` (Assertion auf `DatabaseRead`). Commit #3949 | INV-MCP-CLASSIFY-1 ✅. **B-02 bleibt offen:** unbekannte Namen liefern weiter `CodeExecution` statt Fehler |
| U-02 | B.3.5 Schritt 7 / D.5 / J.12(1): `hybrid_search_with_strategy` verwendet fest RRF | **Überholt.** Signatur hat den Parameter `fusion_strategy: Option<FusionStrategy>`. Die Methode ist `#[deprecated]` zugunsten `Collection::query()`. Der Builder kennt `SearchStrategy` (`query_builder/strategy.rs`) mit `Rrf` und `ScoreNormalized`. Der Community-Boost heißt `apply_community_boost_post_fusion` und wirkt nach beiden Strategien | D.5(1) ✅. Offen bleibt: Test für RRF-Fallback bei degradiertem Signal und PathRAG-Snapshot |
| U-03 | J.12(3): PathRAG nicht snapshot-fähig | **Unverändert.** `snapshot_unsupported_for_signal` wird in `search/hybrid/mod.rs:190` und `query.rs:292` geworfen | offen |
| U-04 | B-16 / J.13: Schreibpfade registrieren Lesezugriffe mit `snapshot_seq = u64::MAX` | **Unverändert vorhanden.** Acht Aufrufstellen: `insert.rs:321,325`, `update.rs:63,67`, `delete.rs:42,46`, `links.rs:65,139`. `store/src/lsm/ops/read.rs:18-29` (`get_at_seq_tracked`) registriert `snapshot_seq` ungeklemmt per `tx_buffer.register_read`. Nicht ausgeführt | B-16 bleibt Verdacht, **P0** |
| U-05 | I.0 / B.4: Widerspruch zum Replicator-Default | **Aufgelöst.** `MaintenanceConfig::default()` setzt `replicator_enabled: false`, `replicator_lr: 0.05`, `coherence_bonus_beta: 0.15`. Der Doc-Kommentar „Default: true“ am Feld ist falsch. B.4 ist korrekt | Doc-Kommentar korrigieren (P3) |
| U-06 | D.6 / J.16 / I.1: Tier-Defaults der Auto-Extraktion offen | `collection_profile.rs`: `DEFAULT_AUTO_EXTRACTION_MODE = Enabled`; `ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT = Enabled`, markiert `DECISION-PENDING (Spec D.6)`. `AutoExtractionMode::default()` = `Enabled`, mit Feature `auto-extraction-opt-out` = `Disabled` | Alle vier Tiers `Enabled`; Entscheidung weiter offen |
| U-07 | Lückenliste 10.6: MCP biete HTTP/Axum | **Falsch.** `contextra-mcp/Cargo.toml` verbietet Axum ausdrücklich (ADR-010); `lib.rs:55`: Transport ausschließlich stdin/stdout | B.7 ist richtig |
| U-08 | G.4: `LLAMA_GGUF_PATH`, `CANDLE_EMBED_MODEL_DIR`, `CANDLE_LLM_MODEL_DIR` als Laufzeitvariablen | Im Produktivcode von `crates/` und `xtask/` am HEAD **nicht gefunden** (Suche über alle `.rs` außerhalb `tests/`) | Vollständige Liste: K.35 |
| U-09 | MCP-Schreibtools | `contextra_drop_collection` verlangt wie `contextra_forget` den Parameter `confirm: true` (Pflichtparameter im Schema) | siehe K.34, L.7 |
| U-10 | Kopfzeile: HEAD `e4d84c9`, „33 Crates“ | HEAD `b564e830`. `docs/FEATURES_SPECIFICATION.md` spricht von „35 Crates“: Das sind die 35 Workspace-Mitglieder (33 Crates + `xtask` + `contextra-bench`), kein Widerspruch | Zählweise in A.1.1 beibehalten |
| U-11 | Repo-Datei `docs/spec/CONTEXTRA_SPEZIFIKATION_v14.md` | Ältere Fassung von v14: ohne B-16, J.13-Nachprüfung und G1-4-Korrektur | Diese Datei ersetzt sie |
| U-12 | `DurabilityMode` | Drei Varianten `Full`, `WalNoHmac`, `MemoryOnly` bestätigt (`store/lsm/config.rs:8-10`). `WalNoHmac` + aktiver Löschbeweis wird in `config.rs:45` abgelehnt | I.0 bestätigt |

## K.0.2 Abdeckung nach Teil K

| Größe | Vor v15 (Namensabgleich) | Nach v15 |
|---|---|---|
| Öffentliche struct/enum/trait im Produktivcode | 798, davon 588 (74 %) ohne Nennung | alle im Inventar (K.1–K.33) |
| Crates mit Kurznamen-Nennung, aber ohne Modul-/Typliste | 33 | 0 |
| Cargo-Features ohne Nennung | `kv-stage-b`, `cuda`, `candle-backend` (+ `test-utils`) | alle je Crate in K.1–K.33 |
| Env-Variablen (Produktiv + xtask) | 27 | 41 (K.35) |
| MCP-Tools mit Schema, Kategorie, Pflichtparametern | Namen | 15 (K.34) |

**Grenze:** „Vollständig“ gilt für die **Sichtbarkeit** (jedes `pub`-Item ist genannt), nicht für die **Tiefe**. Für Items ohne Eintrag in Teil L fehlt die Verhaltensbeschreibung; sie sind als **[V-Sig]** markiert. Ein Item in Teil K ist damit benannt und auffindbar, aber nicht verhaltensgeprüft.

## Inventar je Crate (K.1–K.33)

Alphabetisch nach Crate-Name, die Facade `contextra` zuletzt (K.33). Zeilenzahlen zählen nur Produktivcode ohne Tests und weichen deshalb von A.1.1 ab.

### K.1 `contextra-adapt` — 4.823 Zeilen (ohne Tests), 13 Dateien [V-Sig]

Adaptive controllers, bandits, and PID regulators for Contextra

**Features:** default = `bandit-routing, flow-corrected-thompson`; weitere: `egress-sherman-morrison`, `sketched-bandit`, `rie-greedy-personalization`

**Workspace-Abhängigkeiten:** `contextra-ports`, `contextra-types`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `bandit.rs` | 1180 | LinUCB Contextual Bandit für SLM-Profil-Routing (§13.2). |
| `decay_controller.rs` | 210 | Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (F-01). |
| `drift.rs` | 315 | Drift-Verdrahtung zwischen Lyapunov-Drift-Watcher und Bandit-Policy (§B.5.2.3 & §8.2). |
| `flow_thompson.rs` | 883 | Flow-Corrected Thompson Sampling (FC-TS) pro SLM-Profil-Arm (§21.3). |
| `homeostat.rs` | 231 | — |
| `lib.rs` | 36 | Adaptive controllers, PID regulators, bandits, and Lyapunov drift watchers for Contextra. |
| `lyapunov.rs` | 558 | — |
| `off_policy.rs` | 4 | Alias/Re-export-Modul für Off-Policy-Evaluation (§B.5.2.4 & §8.5). |
| `offpolicy.rs` | 303 | Off-Policy-Evaluation (IPS) für Bandit-Routing (§B.5.2.4 & §8.5). |
| `pid.rs` | 460 | — |
| `pid_latency_controller.rs` | 285 | — |
| `rie_greedy.rs` | 240 | Regularization-Induced Exploration (RIE) Greedy Mechanism (§10.4.1). |
| `shadow_mode.rs` | 118 | Generic shadow mode discrepancy logging framework (§12 Integrationsregel). |

**`bandit.rs`**

- `const SHERMAN_MORRISON_REFACTORIZATION_INTERVAL: u64 = 1000`
- `enum BanditError` ∈ {DimensionMismatch, InvalidConfig, NonFiniteVariance, PrecisionMatrixDriftDetected} — Fehlerzustände des Contextual Bandits.
- `struct AlignedF32Vec` — 64-Byte cache-aligned Wrapper um `Vec<f32>` zur Vermeidung von False Sharing. · Methoden: new, zeros · impl: Deref, DerefMut
- `enum BanditImplementation` ∈ {ShermanMorrison, DiagonalApproximation, FlowCorrectedThompson, SketchedProjection} — LinUCB-Implementierungsvariante (§13.2).
- `struct SketchMatrix` { data, projected_dim, original_dim, seed } — Projektionsmatrix $R \in \mathbb{R}^{k \times d}$ für Sketched Projection. · Methoden: derive_seed, from_seed, project
- `trait BanditPolicy` (apply_drift_penalty) — Trait für Bandit-Routing Policies mit Konzeptdrift-Anpassung (§5.2.3).
- `struct BanditProfileState` { theta, sigma_sq, inv_a, work_buf, alpha, alpha_base, alpha_max_multiplier, lambda, mu, gamma… } — Laufzeitzustand eines LinUCB-Bandits pro Profil. · Methoden: cold_start, ensure_sketched_state, expected_dim, score, update, on_drift_detected · impl: BanditPolicy

**`decay_controller.rs`**

- `struct DecaySignalInputs` { tombstone_ratio, query_load_inverse, w_tombstone, w_query } — Inputs for Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) aus bestehenden System-Metriken.
- `struct DecayControllerConfig` { kappa, base_half_life_tx, eviction_threshold } — Konfiguration für Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (via MaintenanceConfig).
- `struct AdaptiveDecayController` — Adaptive Decay Controller für time-weighted Cache-Eviction. · Methoden: new, with_defaults, system_temperature, effective_half_life, effective_score, should_evict

**`drift.rs`**

- `enum DriftSignal` ∈ {Stable, Detected} — Drift detection signal returned by a [`DriftDetector`].
- `trait DriftDetector` (observe, is_drifting) — Generic interface for sequential O(1) drift detectors.
- `struct CatoniDriftDetector` { catoni_mu, cusum_sum, threshold, alpha, baseline_reward } — Catoni-M-estimator based robust drift detector (arXiv:2505.20051 & arXiv:2501.10974). · Methoden: new, reset · impl: DriftDetector
- `struct EnsembleDriftWatcher` { lyapunov, catoni } — Ensemble combination of [`LyapunovDriftWatcher`] and [`CatoniDriftDetector`]. · Methoden: new, observe_and_decide, set_baseline
- `struct DriftPolicyBridge` { watcher, k_drift, alpha_max, gamma } — Adapter zur Kopplung des `LyapunovDriftWatcher` mit einer `BanditPolicy`. · Methoden: new, observe_and_react

**`flow_thompson.rs`**

- `enum FcTsError` ∈ {DimensionMismatch, NonFinite, InvalidConfig} — Fehlerzustände für Flow-Corrected Thompson Sampling (FC-TS).
- `trait FcTsRng` (next_u64, next_f32, next_standard_normal) — Trait für deterministischen Zufallszahlengenerator im Thompson-Sampling.
- `struct SplitMix64` — Deterministische `SplitMix64`-Referenzimplementierung des [`FcTsRng`]-Traits. · Methoden: new · impl: FcTsRng
- `struct FcTsConfig` { dim, lambda, noise_var, window_capacity, explore_scale, drift_ridge, max_drift_norm } — Konfigurationsparameter für Flow-Corrected Thompson Sampling (§21.3). · Methoden: validate
- `struct Ring3Token` — Privater Zero-Sized Token zur Erzwingung, dass Drift-Update-Aufrufe exklusiv aus Ring-3-Hintergrundtasks stammen (AK-18).
- `fn ring3_background_task_token() -> Ring3Token` — Erzeugt einen `Ring3Token` für den Aufruf in asynchronen Ring-3-Hintergrundtasks.
- `struct FlowCorrectedThompsonBandit` — Laufzeitzustand eines Arms im Flow-Corrected Thompson Sampling (§21.3). · Methoden: new, with_shadow_sink, shadow_sink, config, mu, drift_rate, inv_a, update_with_flow, sample_score, baseline_score, select_arm, recompute_drift_rate_from_window
- `struct DiagonalApproximationBandit` { propensities } — Referenzpolitik basierend auf Diagonal-Approximation für Off-Policy-Evaluation (§B.3.2). · Methoden: new, from_pairs, propensity
- `trait OffPolicyCompatibility` (verify_positivity) — Trait zur Überprüfung der Off-Policy-Kompatibilität zwischen FC-TS und einer Referenzpolitik (§B.3.2).
- `enum OffPolicyError` ∈ {ZeroPropensityViolation} — Fehlerzustände bei der Off-Policy-Kompatibilitätsprüfung.
- `struct FcTsSamplingDistribution` { probabilities } — Sampling-Wahrscheinlichkeitsverteilung über FC-TS-Arme/Strategien (§B.3.2). · Methoden: new, from_pairs, probability · impl: OffPolicyCompatibility
- `struct FcTsArmSet` { arms } — Menge von FC-TS-Armen zur Auswahl der optimalen Aktion. · Methoden: with_shadow_sink, select_arm · impl: OffPolicyCompatibility
- `const MIN_BANDIT_SHADOW_SAMPLES: u64 = 10_000` — Minimale Anzahl an Shadow-Samples für den Bandit-Default-Flip.
- `const MAX_CUMULATIVE_REGRET_THRESHOLD: f64 = 0.0` — Maximales zulässiges kumulatives Regret für den Bandit-Default-Flip (≤ 0.0).
- `struct BanditShadowReport` { sample_count, cumulative_regret, p99_latency_delta_us, reward_improvement_ratio } — Aggregierter Shadow-Mode-Bericht für den Bandit-Default-Flip.
- `trait BanditDefaultFlipGate` (should_flip) — Formales Bandit-Default-Flip-Gate nach analogem Muster wie `DefaultFlipGate` (AP-P15-01).
- `struct DefaultBanditFlipGate` — Standardimplementierung des [`BanditDefaultFlipGate`]. · impl: BanditDefaultFlipGate

**`homeostat.rs`**

- `struct RerankPidController` · Methoden: new, kp, ki, kd, target_p95_latency_ms, k_pool, k_min, k_max, update
- `fn pid_regulated_candidate_pool( controller: &mut RerankPidController, observed_p95_latency_ms: f32, ) -> usize`
- `struct RerankDeadline` { budget } — Hard deadline manager for candidate retrieval and Cross-Encoder reranking phases (P11 requirement). · Methoden: new, deadline_exceeded

**`lyapunov.rs`**

- `struct DriftReason` { kl_divergence, lyapunov_exponent } — Grund für erkannte Verteilungsverschiebung (Distributional Drift).
- `enum LyapunovResult` ∈ {Stable, DriftDetected, InsufficientData} — Ergebnis der Lyapunov-Drift-Analyse. · Methoden: is_drift_detected, apply_drift_if_detected
- `struct LyapunovDriftWatcher` { window_size, divergence_history, baseline_distribution, latest_result } — Proaktiver Drift-Wächter auf Basis diskreter Lyapunov-Exponenten über KL-Divergenzen. · Methoden: new, observe_score, analyze, latest_result, status_str, set_baseline, update

**`offpolicy.rs`**

- `struct RandomizedLoggingPolicy` { epsilon, num_actions } — Randomisierte Logging-Policy für unverzerrte Datensammlung zur Off-Policy-Evaluation (§B.5.2.4 & §8.5). · Methoden: new, compute_propensities, select_action
- `struct OffPolicyStats` { discarded } — Statistiken über verworrene/ungültige Samples der Off-Policy-Evaluation.
- `struct OffPolicyEvaluator` — Evaluator für Inverse Propensity Scoring (IPS) zur kontrafaktischen Off-Policy-Schätzung. · Methoden: new, observe, estimate, cumulative_ips, samples, discarded, stats

**`pid.rs`**

- `const PID_MIN_POOL_SIZE_DEFAULT: usize = 50` — Minimale Kandidaten-Pool-Größe (Hard Floor = 50, Quality Knee per arXiv:2604.01733).
- `const PID_MAX_POOL_SIZE_DEFAULT: usize = 200` — Maximale Kandidaten-Pool-Größe.
- `trait AntiWindupController` (update_with_anti_windup) — Erweitert PID-Regler um konditionelles Anti-Windup bei Aktuatorsättigung.
- `struct PidController` { kp, ki, kd, target_latency_ms, min_pool_size, max_pool_size, current_pool_size } — PID-Regler zur dynamischen Steuerung der Reranking-Kandidatenpool-Größe basierend auf Latenzmessungen. · Methoden: new, update, current_pool_size, reset · impl: AntiWindupController

**`pid_latency_controller.rs`**

- `const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0` — Konservativer Default für das P95-Latenzziel in Millisekunden (100ms).
- `const MIN_SCALING_FACTOR: f64 = 0.3` — Unterer Clamp für den K-Pool-Skalierungsfaktor (30% des konfigurierten K-Pools), um APM-RECALL-COLLAPSE in Standardlastszenarien zu verhindern.
- `const MAX_SCALING_FACTOR: f64 = 1.0` — Oberer Clamp für den K-Pool-Skalierungsfaktor (100% des konfigurierten K-Pools).
- `const MAX_INTEGRAL: f64 = 10.0` — Max-Grenze für den I-Anteil (Anti-Windup Clamping).
- `struct PidLatencyController` { kp, ki, kd, target_latency_ms } — Classical PID controller for dynamically scaling multi-step retrieval parameters (`k_pool`, `max_hops`). · Methoden: new, with_params, compute_adjustment, reset
- `struct LatencyBudgetGuard` — Simple, panic-free latency budget guard for iterative multi-step search loops. · Methoden: new, elapsed_ms, is_exceeded, budget_ms

**`rie_greedy.rs`**

- `enum RieGreedyError` ∈ {DimensionMismatch, NonFinite, InvalidConfig} — Fehlerzustände für den RIE-Greedy-Personalismus-Mechanismus (§10.4.1).
- `struct RieGreedyProfile` { precision_inv, info_vector, dim, lambda, gamma } — Laufzeitzustand eines RIE-Greedy-Profils (§10.4.1). · Methoden: new, mu_hat, predict, update, trace_precision_inv

**`shadow_mode.rs`**

- `struct ShadowDiscrepancy` { baseline, candidate, context_id } — Diskrepanz-Eintrag:
- `trait ShadowSink` (record) — Dyn-kompatibler Port (P27) für Diskrepanz-Senken (In-Memory für Tests, Tracing/Log für Produktion).
- `struct TracingShadowSink` — Referenzimplementierung über `tracing::debug!` — kein zusätzlicher I/O-Pfad, keine Rng/Clock-Portverletzung (P28), da rein diagnostisch. · impl: ShadowSink

### K.2 `contextra-agent` — 3.263 Zeilen (ohne Tests), 10 Dateien [V-Sig]

Persistent agent workflow engine for Contextra — checkpoint/execute/audit loop

**Features:** default = `—`; weitere: `test-utils`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-db`, `contextra-graph`, `contextra-checkpoint`, `contextra-store`, `contextra-router`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `audit.rs` | 730 | — |
| `budget.rs` | 90 | — |
| `context.rs` | 492 | — |
| `dlq.rs` | 263 | — |
| `engine.rs` | 832 | — |
| `engine/condition.rs` | 146 | — |
| `event_source.rs` | 290 | — |
| `graph.rs` | 234 | — |
| `step.rs` | 95 | — |

**`audit.rs`**

- `struct AuditEntry` { task_id, step_count, node_id, tokens_consumed, payload, error, tx_id } — Single immutable record of an agent step execution.
- `struct MigrationStats` { migrated, failed } — Summary statistics for legacy audit entry migration.
- `struct AuditLog` — Append-only audit log backed by a Contextra collection. · Methoden: new, append_to, append, replay_task
- `fn validate_audit_payload_and_error( payload: &serde_json::Value, error: Option<&str>, ) -> Result<()>` — Validates payload and optional error string of an audit entry for non-emptiness and absence of null bytes.
- `fn migrate_legacy_audit_entries( collection: &Collection<S, V>, ) -> Result<MigrationStats>` — Migrates legacy zero-vector audit entries from the HNSW vector index and doc_key mappings into the direct LSM KV store format (`put_kv`).

**`budget.rs`**

- `fn reserve_tokens( budget: &'a TokenBudget, amount: usize, ) -> contextra_types::Result<Reservation<'a>>` — Helper to reserve `amount` tokens from `budget`.

**`context.rs`**

- `const MAX_ID_LEN: usize = 256` — Maximum allowed length in bytes for task IDs and node IDs.
- `const MAX_TELEMETRY_EVENTS: usize = 10_000` — Maximum allowed telemetry events stored in memory history.
- `fn validate_task_id(task_id: &str) -> Result<()>` — Validates a task identifier to ensure it is non-empty, <= 256 bytes, and contains no null bytes.
- `fn validate_node_id(node_id: &str) -> Result<()>` — Validates a node identifier to ensure it is non-empty, <= 256 bytes, and contains no null bytes.
- `enum AgentStatus` ∈ {Idle, Running, Completed, Failed} — Operational status of an agent workflow.
- `struct AgentContext` { task_id, current_node, step_count, db, state_collection, budget, status, current_cache_directive, step_directives, memory… } — Operational context spanning the entire workflow execution. · Methoden: try_new, set_cache_directive, get_cache_directive, set_step_cache_directive, directive_for_node, attach_event, next_retry_step_count, try_attach_event
- `struct AgentEngine` { current_directive } — Agent execution engine helper managing per-step cache directives during workflow execution. · Methoden: new, directive_for_step, run

**`dlq.rs`**

- `struct DeadLetterQueue` — Persistente Dead-Letter-Queue für fehlgeschlagene Agent-Schritte. · Methoden: new, push, drain, list, remove, is_already_committed, allocate_tx

**`engine.rs`**

- `enum EventLoopExitReason` ∈ {Shutdown, SourceExhausted} — Reason for exiting `OrchestratorEngine::run_event_loop`.
- `const MAX_WORKFLOW_STEPS: u64 = 10_000` — Maximum allowed steps in a single workflow execution to prevent unbounded loops.
- `struct OrchestratorEngine` { tools, checkpoint_store, dead_letter_queue } — Async executor engine applying nodes in Sequence. · Methoden: try_new, try_from_db, new, from_db, try_register_tool, recover_orphans, run, replay_from, checkpoint, run_event_loop

**`engine/condition.rs`**

- `fn evaluate_condition_expr(expr: &str, ctx: &AgentContext) -> bool` — Evaluates a declarative condition expression against the provided [`AgentContext`].

**`event_source.rs`**

- `const MAX_EVENT_SOURCE_CAPACITY: usize = 10_000` — Maximum allowed events in pending event buffers to prevent memory exhaustion.
- `struct BackgroundEvent` { payload, source, observed_at_seq } — Background telemetry/trigger event delivered to the agent context. · Methoden: try_new, new
- `trait EventSource` (next_event, is_exhausted, wait_for_event) — Dynamic event source delivering continuous background events.
- `const MAX_PENDING_EVENTS_CAPACITY: usize = 10_000` — Maximum capacity for pending background telemetry events queue before dropping or rejecting.
- `struct PollingDocumentEventSource` — Concrete `EventSource` that periodically polls `Collection` storage sequence numbers, using `scan_prefix_at` for snapshot delta calculation to emit document changes. · Methoden: new, with_capacity, with_last_seen_seq, poll_interval · impl: EventSource
- `struct VecEventSource` — Trivial mock/static `EventSource` that reads from a fixed list of `BackgroundEvent`s. · Methoden: try_new · impl: EventSource

**`graph.rs`**

- `type NodeId = String`
- `const MAX_TEXT_LEN: usize = 65_536` — Maximum allowed length in bytes for descriptions and conditions.
- `enum NodeType` ∈ {Start, Task, Decision, End} — Type of node within the declarative agent state graph.
- `struct AgentNode` { id, description, node_type, handler } — Represents a single specialized node in the workflow.
- `struct WorkflowEdge` { from, to, condition, priority } — Represents a conditional transition between two graph nodes.
- `struct StateGraph` { nodes, edges } — Core declarative structure routing autonomous agent steps. · Methoden: new, try_add_node, add_node, try_add_edge, add_edge, get_node

**`step.rs`**

- `struct StepDeadLetter` { session_id, node_id, step_index, tx_id, failure_reason, input, attempt, failed_at_secs } — Ein fehlgeschlagener Agent-Schritt der für spätere Analyse und Idempotenz-Prüfung persistiert wird.
- `enum DeadLetterReason` ∈ {Timeout, BudgetExhausted, ToolError, MaxRetriesExceeded}
- `struct StepResult` { node_id, output, tokens_consumed, next_edge } — The explicit result of an agent step execution.
- `trait AgentTool` (name, estimated_cost, execute, timeout_ms, is_retriable, max_retries)

### K.3 `contextra-audit-export` — 382 Zeilen (ohne Tests), 5 Dateien [V-Sig]

GDPR Article 30 Processing Register export generator for Contextra

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `bsi_mapping.rs` | 144 | BSI Grundschutz / TR-02102 Cryptographic Mapping (Spec §2.4 Punkt 2, §16.3). |
| `error.rs` | 20 | Error types for the audit export crate. |
| `lib.rs` | 83 | `contextra-audit-export` — GDPR Article 30 Processing Register export generator. |
| `markdown_template.rs` | 52 | Markdown table renderer for GDPR Article 30 processing register entries. |
| `testkit.rs` | 83 | Test kit utilities for `contextra-audit-export`. |

**`bsi_mapping.rs`**

- `struct BsiMappingEntry` { primitive_name, code_location, bsi_reference, note } — A mapping entry linking a cryptographic primitive in the codebase to a BSI Technical Guideline (Spec §2.4 Punkt 2, §16.3).
- `fn bsi_mapping_table() -> Vec<BsiMappingEntry>` — Returns the static, code-maintained cryptographic mapping table for Contextra.
- `fn render_bsi_mapping_markdown(entries: &[BsiMappingEntry]) -> String` — Renders the BSI Grundschutz / TR-02102 cryptographic mapping table as Markdown.

**`error.rs`**

- `enum AuditExportError` ∈ {Serialization, SourceCollection, InvalidTenant} — Error type returned by operations in `contextra-audit-export`.

**`lib.rs`**

- `struct EgressEventSummary` { event_id, destination, timestamp_nanos, detail } — Summary of an egress gateway event recorded for audit verification.
- `struct DeletionProofSummary` { proof_id, scope, timestamp_nanos, verified } — Summary of a cryptographic deletion proof (GDPR Art.
- `struct ProcessingRegisterEntry` { tenant_id, processing_purpose, data_categories, legal_basis, egress_events, deletion_proofs, generated_at } — A record entry in the GDPR Article 30 processing register.
- `trait ProcessingRegisterSource` (collect_entries) — Abstract data source for retrieving processing register records.
- `fn render_register_json( entries: &[ProcessingRegisterEntry], ) -> Result<String, AuditExportError>` — Renders a list of processing register entries as a formatted JSON string.

**`markdown_template.rs`**

- `fn render_register_markdown( entries: &[ProcessingRegisterEntry], ) -> Result<String, AuditExportError>` — Renders a list of [`ProcessingRegisterEntry`] records as a formatted Markdown document containing a GDPR Article 30 processing register table.

**`testkit.rs`**

- `struct InMemoryProcessingRegisterSource` — In-memory implementation of [`ProcessingRegisterSource`] for testing purposes. · Methoden: new, with_sample_data_for, add_entry · impl: ProcessingRegisterSource

### K.4 `contextra-avv-generator` — 256 Zeilen (ohne Tests), 3 Dateien [V-Sig]

AVV (Auftragsverarbeitungsvertrag) template generator referencing technical guarantees for Contextra

**Workspace-Abhängigkeiten:** `contextra-types`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `error.rs` | 16 | Error types for AVV document generation. |
| `lib.rs` | 74 | `contextra-avv-generator` Generates AVV (Auftragsverarbeitungsvertrag) template documents referencing Contextra's technical guarantees (deletion proof |
| `template.rs` | 166 | Template rendering module for AVV (Auftragsverarbeitungsvertrag) according to Art. |

**`error.rs`**

- `enum AvvGeneratorError` ∈ {InvalidContext, RenderError} — Errors that can occur during AVV template rendering.

**`lib.rs`**

- `struct AvvContext` { controller_name, processor_name, tenant_id, technical_measures, subprocessors, deletion_sla_days } — Context parameter container for generating an AVV document.
- `struct TechnicalMeasure` { name, description, reference_article } — Description of a Technical and Organizational Measure (TOM).
- `fn render_avv_markdown(ctx: &AvvContext) -> Result<String, AvvGeneratorError>` — Renders the AVV document in Markdown format using the provided [`AvvContext`].
- `fn default_technical_measures() -> Vec<TechnicalMeasure>` — Returns the default list of technical measures derived from Contextra's product guarantees.

**`template.rs`**

- `fn render(ctx: &AvvContext) -> Result<String, AvvGeneratorError>` — Renders the AVV Markdown document from the provided [`AvvContext`].

### K.5 `contextra-checkpoint` — 2.527 Zeilen (ohne Tests), 7 Dateien [V-Sig]

Backup and snapshot management for Contextra storage

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-core`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `guard.rs` | 560 | — |
| `hardlink_cloner.rs` | 202 | Hardlink cloner for physical SSTable cloning with MVCC pin holding. |
| `lib.rs` | 53 | Checkpoint-Registry für Time-Travel und MVCC-basiertes Snapshotting (gemäß ADR-011). |
| `manifest.rs` | 165 | — |
| `meta.rs` | 147 | — |
| `orphan.rs` | 579 | — |
| `store.rs` | 821 | — |

**`guard.rs`**

- `struct PinGuard` — RAII-Guard für gepinnte Checkpoint-Sequenznummern (gemäß ADR-015). · Methoden: pin, defuse, unpin
- `struct CheckpointGuard` — RAII Guard, der explizit über [`commit`](Self::commit) oder [`rollback`](Self::rollback) finalisiert werden MUSS. · Methoden: new, with_registry, with_registry_and_counter, for_agent_step, for_agent_step_with_registry, checkpoint, commit, rollback, rollback_blocking

**`hardlink_cloner.rs`**

- `fs::fn create_dir_all(_path: P) -> std::io::Result<()>`
- `fs::fn read_dir(_path: P) -> std::io::Result<LoomReadDir>`
- `fs::fn metadata(_path: P) -> std::io::Result<LoomMetadata>`
- `fs::fn remove_file(_path: P) -> std::io::Result<()>`
- `fs::fn hard_link(_from: P, _to: Q) -> std::io::Result<()>`
- `struct fs::LoomReadDir` · Methoden: next_entry
- `struct fs::LoomDirEntry` · Methoden: path
- `struct fs::LoomMetadata`
- `struct HardlinkCloneResult` { source_seq_no, linked_files, wal_tail_offset } — Result structure returned upon completing an SSTable hardlink clone operation.
- `trait CheckpointHardlinkCloner` (clone_sstable_hardlinks) — Trait defining the physical hardlink cloning interface for SSTables.
- `struct DefaultHardlinkCloner` — Productive implementation of [`CheckpointHardlinkCloner`] using `tokio::fs`. · Methoden: new · impl: CheckpointHardlinkCloner

**`manifest.rs`**

- `struct CheckpointManifest` { meta, components, checksum } — AI-TAG\[PANIC-SAFETY\]\[CRITICAL\] RESOLVED: · Methoden: new, verify

**`meta.rs`**

- `struct CheckpointMeta` { name, collection_id, seq_no, tx_id, metadata, created_at } — Metadata for a persistent checkpoint. · Methoden: into_workflow_state
- `struct StateCheckpoint` { tx_id, timestamp_ms, namespace } — Point-in-Time Checkpoint representing an agent step or transaction boundary.

**`orphan.rs`**

- `type PinId = u64` — Type alias for sequence numbers managed as pinned checkpoint identifiers.
- `fn global_orphan_registry() -> &'static OrphanRegistry`
- `struct OrphanRegistry` · Methoden: new, register_orphan, get_orphans, clear_all, recover_and_clean
- `struct PinnedSeqNoOrphan` { seq_no, timestamp_ms } — Orphaned gepinnte Sequenznummer — wird beim Recovery verarbeitet.
- `fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan)`
- `struct OrphanState` { checkpoints, pinned_seq_nos, persist_path } — Instance-scoped orphan state for checkpoints and pinned sequence numbers. · Methoden: persist_sync, load_sync
- `struct InstanceOrphanRegistry` — Instance-scoped orphan registry for checkpoints and pinned sequence numbers (ADR-053). · Methoden: new, load_sync, persist_sync, register_orphan_sync, register_checkpoint_sync, flush_orphan_registry, drain_orphan_pins, get_orphan_pins, clear_orphan_pin, drain_orphaned_checkpoints, get_orphaned_checkpoints, clear_orphaned_checkpoint, clear_all
- `fn register_orphaned_checkpoint(cp: StateCheckpoint)`
- `fn get_orphaned_checkpoints() -> Vec<StateCheckpoint>`
- `fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint>` — Retrieves orphaned checkpoints registered for a specific namespace.
- `fn clear_orphaned_checkpoint(_tx_id: TxId)`
- `fn clear_all_orphaned_checkpoints()`
- `fn await_pending_rollbacks()` — Retained for backward compatibility.
- `fn pending_rollback_count() -> usize` — Retained for backward compatibility.
- `fn orphaned_checkpoint_count() -> usize` — Liefert die Anzahl der aktuell registrierten verwaisten ("orphaned") Checkpoints.

**`store.rs`**

- `trait CheckpointRegistry` (save_checkpoint, load_checkpoint, list_checkpoints, orphan_registry, recover_orphaned_pins, recover_orphaned_checkpoints, flush_orphan_registry) — Trait für die Checkpoint-Verwaltung.
- `struct PersistentCheckpointStore` — Registry für gespeicherte Checkpoints mit Thread-sicherem Zustand. · Methoden: open, open_with_orphan_registry, new, new_with_orphan_registry, allocate_tx, skipped_rollback_count, checkpoint_guard_skipped_rollback_count, checkpoint_counter, monotonic_timestamp_ms, create_guard, create_checkpoint, drop_checkpoint, list_checkpoints, get_checkpoint… · impl: CheckpointRegistry, CheckpointCoordinator, Checkpoint

### K.6 `contextra-cognition` — 5.240 Zeilen (ohne Tests), 18 Dateien [V-Sig]

Contextra — Memory consolidation, compaction, and context management

**Features:** default = `—`; weitere: `graph-connectivity-health`, `edge-reinforcement-learning`

**Workspace-Abhängigkeiten:** `contextra-engine`, `contextra-types`, `contextra-ports`, `contextra-store`, `contextra-vector`, `contextra-graph`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `aggregation_phase.rs` | 691 | Aggregation Phase (Consolidation Stage 3) — Spec §21.4 Implementiert Ring-3-Consolidation: |
| `consolidation_executor.rs` | 985 | — |
| `consolidation_locks.rs` | 105 | — |
| `context.rs` | 554 | — |
| `context_compaction/cleanup.rs` | 40 | — |
| `context_compaction/compactor.rs` | 257 | — |
| `context_compaction/session.rs` | 306 | — |
| `context_compaction/types.rs` | 42 | — |
| `graph_sink.rs` | 91 | Graph-Sink Implementation (`CsrGraphSuperEdgeSink`) for LeanRAG Stage 3 Aggregation (§21.4). |
| `leanrag_input.rs` | 113 | — |
| `maintenance_config.rs` | 93 | — |
| `maintenance_scheduler.rs` | 461 | — |
| `memory_consolidation.rs` | 967 | — |
| `semantic_aggregation_facade.rs` | 109 | — |
| `synthesis_phase.rs` | 207 | Generative Synthesis Pass — LLM-basierte, stochastische Wissenssynthese pro konsolidiertem Segment. |
| `transitivity_veto.rs` | 140 | — |

**`aggregation_phase.rs`**

- `struct AggregationConfig` { max_compaction_peak_memory_mb, clustering_tau_threshold, gmm_deterministic_seed, max_llm_calls_per_cycle, max_clusters, min_cluster_size, em_max_iterations, em_tolerance, min_type_compat_score, stability_cycles_required } — Konfiguration für die Aggregation-Phase. · Methoden: validate
- `struct AggregationPhaseResult` { raw_edges_tombstoned, abstract_hyperedges_created, peak_memory_used_mb, child_edge_ids_written } — Ergebnis der Aggregation-Phase.
- `struct ConsolidationPipelineResult` { structural, synthesis, aggregation } — Gesamtergebnis der Konsolidierungspipeline (Spec §21.4).
- `struct AggregationNode` { entity, embedding, type_id } — Eingabe-Knoten für die Aggregationsphase.
- `struct AggregationEdge` { id, predicate_type, participants } — Eingabe-Kante für die Aggregationsphase.
- `struct SuperEdgeDraft` { cluster_pair, participants, child_edge_ids, weight } — Entwurf einer abstrakten Super-Hyperkante.
- `struct AlphaNode` { cluster_id, members, summary } — Ein synthetisierter Alpha-Knoten (abstraktes Community-Konzept).
- `trait SuperEdgeSink` (tombstone_edge, write_super_edge, commit) — Trait zur Abstraktion des Speicherziels für Superkanten und Tombstones.
- `fn check_compaction_budget(peak_bytes: usize, cfg: &AggregationConfig) -> Result<()>` — Prüft, ob der geschätzte Compaction-Speicherbedarf das konfigurierte Budget überschreitet.
- `fn compute_entity_community_hash(members: &[EntityId]) -> u64` — Berechnet einen deterministischen Hash über eine Liste von `EntityId`s.
- `fn run_aggregation_pass( nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGenerator, tracker: &mut CommunityStabilityTracke` — Führt die Aggregation (Consolidation Stage 3) aus.

**`consolidation_executor.rs`**

- `struct ConsolidationLockGuard` — RAII Guard zur Koordination des Konsolidierungslaufs. · Methoden: try_acquire
- `fn execute_consolidation_pass( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Result<ConsolidationPhaseResult>` — Führt den Structural Consolidation Pass aus UND wendet die Ergebnisse an (Tombstones, Graph-Cascade).
- `fn execute_background_consolidation( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&Syn` — Führt die vollständige Hintergrund-Konsolidierung (Structural Consolidation Pass und optional Generative Synthesis Pass) aus.
- `fn execute_leanrag_aggregation_stage( collection: &Collection<S, V>, nodes: &[AggregationNode], edges: &[AggregationEdge], cfg: &AggregationConfig, llm: &dyn LlmTextGener` — Führt die Stage-3-Aggregation (LeanRAG) aus und wendet Kanten-Tombstones und Super-Hyperkanten atomar über [`CsrGraphSuperEdgeSink`] an.
- `fn execute_sleep_cycle( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], consolidation_config: &ConsolidationConfig, synthesis_config: Option<&SynthesisConfig>` — Deprecated legacy wrapper for `execute_background_consolidation`.
- `fn start_consolidation_worker( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Durat` — Starts a background task for periodic consolidation.
- `fn start_consolidation_reaper( collection: Arc<Collection<S, V>>, consolidation_config: ConsolidationConfig, synthesis_config: SynthesisConfig, interval: std::time::Durat` — Deprecated legacy alias for `start_consolidation_worker`.
- `struct ConsolidationEngine` — Tokio-Background-Task Engine für periodische Speicher-Konsolidierung und Wissenssynthese. · Methoden: new, with_llm, with_leanrag, with_validator, start, start_worker, run, run_cycle

**`consolidation_locks.rs`**

- `struct ConsolidationNodesGuard` — Guard zur Erzwingung der Lock-Reihenfolge und kanonischen Node-Sortierung waehrend der Konsolidierung. · Methoden: try_acquire, collection, cascade_invalidate_edges_ordered

**`context.rs`**

- `struct ContextManager` — Manages autonomous context preparation for LLM consumption. · Methoden: new, with_defaults, set_relevance_threshold, relevance_threshold, prepare_context, estimate_tokens
- `struct SpatialFence` { region, field_name } — Spatial fencing for geographically constrained context (optional). · Methoden: new, matches

**`context_compaction/cleanup.rs`**

- `fn cleanup_orphaned_consolidation_intents( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>` — Aufgerufen beim Öffnen einer Collection / DB.

**`context_compaction/compactor.rs`**

- `struct ContextCompactor` — Context Compaction Engine. · Methoden: new, compact, consolidate_via_llm, consolidate_with_retry

**`context_compaction/session.rs`**

- `struct ConsolidationSession` { collection, source_docs, intent_key, target_id, base_tx } — Optimistic Concurrency Control (OCC) Consolidation Session for Sleep-Cycle Memory Compaction. · Methoden: start, validate_occ, refresh, abort, execute, commit, commit_ref

**`context_compaction/types.rs`**

- `enum CompactionStrategy` ∈ {Truncate, Summarize, StatusToken, LlmSummarize} — Strategie für Context Compaction.
- `struct CompactedContext` { retained_chunks, status_tokens, tokens_used, source_doc_ids } — Kompaktierter Kontext für LLM-Übergabe.
- `struct StatusToken` { summary, replaced_tokens, replaced_doc_ids } — Kompakter Stellvertreter für einen oder mehrere kompaktierte Chunks.

**`graph_sink.rs`**

- `struct CsrGraphSuperEdgeSink` — Buffered sink implementation writing synthetic superedges and tombstone requests to [`CsrGraph`]. · Methoden: new · impl: SuperEdgeSink

**`leanrag_input.rs`**

- `const DEFAULT_MAX_LEANRAG_NODES: usize = 10_000` — Standardmäßige Obergrenze für die Anzahl extrahierter Knoten im LeanRAG-Input.
- `struct LeanRagInputs` { nodes, edges } — Extrahiertes Eingabepaket für die LeanRAG Stage-3-Aggregation.
- `fn build_leanrag_inputs( collection: &Collection<S, V>, turns: &[(DocId, Vec<f32>)], max_nodes: usize, ) -> LeanRagInputs` — Baut [`LeanRagInputs`] aus den Turns einer Collection und den assoziierten Graphendaten.

**`maintenance_config.rs`**

- `struct PercolationConfig` { critical_threshold, rebonding_similarity, max_new_edges_per_pass }
- `struct MaintenanceConfig` { tick_interval_secs, decay_enabled, decay_config, edge_reinforcement, percolation_enabled, percolation, replicator_enabled, replicator_lr, coherence_bonus_beta, background_consolidation_enabled… } — Zentrale Konfiguration für den `MaintenanceScheduler` (§10.2).

**`maintenance_scheduler.rs`**

- `struct MaintenanceScheduler` — Zentraler Scheduler für die Ausführung der Background-Maintenance-Prozesse (ADR-079). · Methoden: new, with_edge_reinforcement_buffer, active_agent_sessions, increment_active_sessions, decrement_active_sessions, start, run_tick

**`memory_consolidation.rs`**

- `struct ConsolidationConfig` { min_turns_per_segment, max_turns_per_segment, segment_cohesion_threshold, near_duplicate_cosine_threshold, aggregation_config } — Konfiguration für den Structural Consolidation Pass.
- `struct TurnSegment` { turn_ids, representative_embedding } — Repräsentiert ein semantisch zusammenhängendes Segment aus aufeinanderfolgenden Turns.
- `struct ConsolidationPhaseResult` { segments_created, duplicates_tombstoned, cascade_edge_tombstones_needed, cascade_errors } — Ergebnis des Structural Consolidation Pass.
- `fn group_turns_into_segments( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> Vec<TurnSegment>` — Gruppiert semantisch zusammenhängende, zeitlich benachbarte Turns via sequenziellem Sliding-Window-Clustering.
- `fn detect_near_duplicates(turns: &[(DocId, Vec<f32>)], threshold: f32) -> Vec<(DocId, DocId)>` — Führt einen paarweisen Cosine-Similarity-Vergleich INNERHALB eines Segments durch (O(n²) segmentlokal).
- `fn run_consolidation_pass( turns: &[(DocId, Vec<f32>)], config: &ConsolidationConfig, ) -> ConsolidationPhaseResult` — Orchestriert den Structural Consolidation Pass (Segmentierung & Near-Duplicate-Detection).
- `struct SynthesisConfig` { min_community_size, stability_cycles_required, max_llm_calls_per_cycle, min_grounding_score } — Konfiguration für den Generative Synthesis Pass.
- `struct CommunityStabilityTracker` — Verfolgt die Stabilität von Graph-Communities über aufeinanderfolgende Zyklen hinweg. · Methoden: new, observe, reset_if_absent
- `struct MetaChunk` { content, abstracts_from, source_community_hash, created_at_tx, llm_model_id } — Ein generativ synthetisierter Wissens-Chunk (MetaChunk) aus dem Generative Synthesis Pass.
- `struct SynthesisPhaseResult` { synthesized, deferred_community_hashes } — Ergebnis des Generative Synthesis Pass.
- `fn compute_community_hash(member_doc_ids: &[DocId]) -> u64` — Berechnet einen deterministischen 64-Bit-Hash für eine Liste von Member-DocIds.
- `fn run_structural_synthesis_pass( stable_communities: &[(u64, Vec<DocId>)], source_texts: &std::collections::HashMap<DocId, String>, llm: &dyn LlmTextGenerator, config: &` — Dies ist der DETERMINISTISCHE, LLM-FREIE Structural Consolidation Pass.
- `fn compact_segment_via_context_compactor( segment: &TurnSegment, compactor: &ContextCompactor, chunks: &[ContextChunk], ) -> CompactedContext` — Adapterfunktion zur Kompaktierung eines Segments via des bereits vorhandenen `ContextCompactor`.

**`semantic_aggregation_facade.rs`**

- `fn consolidate_semantic_hyperedges( collection: &Collection<S, V>, embedder: &dyn TextEmbeddingEngine, llm: &dyn LlmTextGenerator, config: &AggregationConfig, ) -> Result` — Ausführung der semantischen Hyperkanten-Konsolidierung (LeanRAG Stage 3) auf einer Collection.

**`synthesis_phase.rs`**

- `struct SegmentSynthesisResult` { synthesized_chunks, skipped_segments } — Ergebnis des Generative Synthesis Pass auf Segment-Ebene.
- `type SynthesisPhaseResult = SegmentSynthesisResult`
- `struct SynthesizedChunk` { content, source_turn_ids, model_id } — Ein generativ synthetisierter Wissens-Chunk mit Provenienz.
- `fn run_synthesis_pass( segments: &[TurnSegment], segment_texts: &[Vec<String>], // Texte der Turns pro Segment synthesizer: &dyn SegmentSynthesizer, min_turns_for_synthes` — Dies ist der LLM-BASIERTE Generative Synthesis Pass (erzeugt `SynthesizedChunk`s via `SegmentSynthesizer`-Trait).

**`transitivity_veto.rs`**

- `fn validate_transitivity_veto( node_a: &AggregationNode, node_b: &AggregationNode, node_c: &AggregationNode, threshold: f32, ) -> bool` — Reine, synchrone Funktion, die für drei `AggregationNode`-Kandidaten (A, B, C) und einen Ähnlichkeits-Schwellenwert prüft, ob eine transitive Verkettung (z.
- `fn filter_candidates_with_transitivity_veto( turns: &[(DocId, Vec<f32>)], threshold: f32, agg_cfg: &AggregationConfig, ) -> Vec<(DocId, DocId)>` — Orchestrierungsfunktion für die Transitivitätsprüfung unter Einhaltung der P24-Lokalität.

### K.7 `contextra-core` — 137 Zeilen (ohne Tests), 1 Dateien [V-Sig]

Deprecated Strangler Facade re-exporting Ring-0 types, traits, MVCC, and wire IPC for Contextra

**Features:** default = `—`; weitere: `test-utils`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `lib.rs` | 137 | `Contextra` Core — Types, traits, and error handling (Strangler Facade). |

### K.8 `contextra-crypto` — 6.257 Zeilen (ohne Tests), 15 Dateien [V-Sig]

Encryption at Rest and KV-Cache Security utilities for Contextra

**Features:** default = `—`; weitere: `test-utils`, `kv-encryption`, `cloud-egress-guard`

**Workspace-Abhängigkeiten:** `contextra-types`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `anti_tamper.rs` | 135 | — |
| `crypto.rs` | 900 | — |
| `deletion_proof.rs` | 1672 | — |
| `ed25519_proof.rs` | 209 | — |
| `error.rs` | 128 | Error types for `contextra-crypto`. |
| `kdf.rs` | 279 | — |
| `kv_cipher.rs` | 391 | — |
| `kv_segment/eviction_worker.rs` | 211 | — |
| `kv_segment/segment.rs` | 352 | — |
| `kv_segment/store.rs` | 841 | — |
| `kv_shredding.rs` | 210 | — |
| `wal_completeness.rs` | 58 | — |
| `wal_crypto.rs` | 810 | — |

**`anti_tamper.rs`**

- `struct VolatileEncryptionKey` — Defines a cryptographic key that is explicitly zeroed out when dropped or when an emergency trigger is activated, protecting against cold-boot attacks. · Methoden: new, emergency_wipe, as_bytes

**`crypto.rs`**

- `struct KeyManager` — Manager for encryption keys and block encryption. · Methoden: try_new, try_new_with_kdf, try_new_random_salt, derive_file_key, derive_segment_key, derive_kv_key, derive_kv_key_scoped, cipher_for, cipher_for_scoped, integrity_key, derive_deletion_proof_key, encrypt_auto_nonce, decrypt_auto_nonce, emergency_wipe · impl: KvCipher

**`deletion_proof.rs`**

- `struct GraphRepairAttestation` { doc_id, verified_no_ghost_pointers, attested_at } — Attestation confirming synchronous neighborhood graph repair for vector deletions.
- `fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32]` — Berechnet den Blake3-Hash einer deterministisch sortierten Liste gelöschter Schlüssel mit Längenpräfix.
- `struct DeletionProofKeyPair` { verifying_key } — KeyPair for Ed25519 signing and verification of DeletionProofs (version 3). · Methoden: generate, verifying_key_bytes, signing_key, verifying_key
- `enum VerificationKey` ∈ {Hmac, Ed25519} — Key parameter for verification (either HMAC-SHA256 byte slice or Ed25519 VerifyingKey).
- `struct LayerCleanupProof` — Beweis, dass ein bestimmter DeletionLayer physisch bereinigt wurde. · Methoden: new_after_verified_empty, verify_and_create, layer
- `enum DeletionLayer` ∈ {LsmMemtable, SsTableAllLevels, HnswIndex, WalAllSegments, CsrGraph, KvCacheSegments, EmbeddingCache} — Layer-explizite Coverage-Deklaration.
- `enum ExcludedScope` ∈ {ConsolidatedAndDistilled, LlmParameterMemory} — Explizite Nicht-Abdeckung — maschinenlesbar für Audit-Systeme.
- `enum DeletionScope` ∈ {Document, Collection, Tenant} — Target scope for physical deletion.
- `struct DeletionProof` { signature_version, scope, deleted_keys_hash, deleted_after_tx, timestamp, signature, covered_layers, excluded_scopes, graph_repair, wal_chain_receipt… } · Methoden: create, create_with_wal_receipt, create_v3, create_with_wal_receipt_v3, signature_version_typed, verify, verify_external, export_for_audit, tenant_id
- `fn compute_wal_delete_receipt( prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<[u8; 32]>` — Erzeugt eine WAL-Löschquittung H(hmac_prev \|\| delete_event) für den Nachweis auf WAL-Ebene gemäß DSGVO Art.
- `fn verify_wal_delete_receipt( receipt: &[u8; 32], prev_hmac: &[u8; 32], delete_event_payload: &[u8], integrity_key: &[u8], ) -> Result<bool>` — Verifiziert eine WAL-Löschquittung in O(1) konstanter Zeit ohne Klartextzugang.

**`ed25519_proof.rs`**

- `enum DeletionProofError` ∈ {InvalidVerifyingKey, InvalidSignature, UnsupportedVersion} — Fehlerzustände bei der Verifikation von Ed25519-Löschbeweisen.
- `struct DeletionProofKeyPair` { signing_key, verifying_key } — Schlüsselpaar für Ed25519-Löschbeweise.
- `fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8; 64]` — Signiert den kanonischen Byte-Payload eines Version-3-Löschbeweises mit Ed25519.
- `fn verify_deletion_proof_v3( verifying_key_bytes: &[u8; 32], proof_payload: &[u8], signature: &[u8; 64], ) -> Result<(), DeletionProofError>` — Verifiziert eine Ed25519-Signatur eines Version-3-Löschbeweises rein funktional.
- `enum SignatureVersion` ∈ {V1, V2, V3} — `SignatureVersion`-Enum für typsichere Versionierung von DeletionProof-Signaturen. · Methoden: as_u8 · impl: TryFrom

**`error.rs`**

- `type Result = std::result::Result<T, CryptoError>` — Result type alias for cryptographic operations in `contextra-crypto`.
- `enum CryptoError` ∈ {KeyDerivation, Encryption, Decryption, InvalidLength, IntegrityViolation, InvalidInput, Crypto, KvFormatVersionMismatch, WalCorruption, InvalidProofSignature, UnsupportedProofVersion, WalTruncationDetected} — Standalone error type for all operations within `contextra-crypto`. · Methoden: wal_corruption

**`kdf.rs`**

- `const KDF_HEADER_MAGIC: &[u8; 4] = b"MFKD"` — Magischer Header-Marker "MFKD" (Contextra Key Derivation)
- `const KDF_HEADER_VERSION_1: u8 = 1` — Aktuelle KDF Header Version
- `const KDF_ID_ARGON2ID: u8 = 1` — Identifikator für Argon2id
- `const MIN_SALT_LEN: usize = 16` — Min. empfohlener Salt-Umfang (16 Bytes)
- `const MIN_M_COST_KIB: u32 = 19456` — OWASP Mindestanforderungen für Argon2id
- `const MIN_T_COST: u32 = 2`
- `const MIN_P_COST: u32 = 1`
- `const DEFAULT_M_COST_KIB: u32 = 65536` — OWASP Standardempfehlungen für Argon2id
- `const DEFAULT_T_COST: u32 = 3`
- `const DEFAULT_P_COST: u32 = 1`
- `struct KdfParams` { m_cost_kib, t_cost, p_cost } — Konfiguration der Argon2id KDF-Kostenparameter. · Methoden: new
- `struct KdfHeader` { version, kdf_id, params, salt } — Versionierter KDF-Header, der alle Parameter und den Salt für die Argon2id-Ableitung speichert. · Methoden: generate_default, new, to_bytes, from_bytes
- `struct DerivedKey` — Wrapper für abgeleitetes 256-Bit Schlüsselmaterial, das bei Drop automatisch aus dem Speicher gelöscht wird.
- `fn derive_key_argon2id(passphrase: &str, header: &KdfHeader) -> Result<DerivedKey>` — Leitet einen 256-Bit (32 Byte) Schlüssel aus einer Passphrase und einem `KdfHeader` via Argon2id ab.

**`kv_cipher.rs`**

- `const CURRENT_KV_FORMAT_VERSION: u8 = 2` — Current serialized format version for [`EncryptedKvLayer`].
- `struct EncryptedKvLayer` { format_version, ciphertext, nonce, tenant_id, model_fingerprint } — Container for an encrypted KV-cache segment layer.
- `trait KvCipher` (seal, open) — Abstract cipher trait for KV block encryption and seal/open operations.
- `struct KvSegmentCipher` — High-level cipher engine for KV-cache segment encryption and decryption. · Methoden: new, encrypt, decrypt, derive_key_for_segment, encrypt_with_version, decrypt_with_version · impl: KvCipher

**`kv_segment/eviction_worker.rs`**

- `struct EvictionWorker` · Methoden: spawn, trigger_eviction, shutdown
- `fn emergency_wipe(store: &TenantIsolatedKvStore)` — SEPARATER Pfad für Notfall-Löschung (z.B Prozess-Shutdown, expliziter Sicherheits-Trigger).

**`kv_segment/segment.rs`**

- `const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1` — Aktuelle Version der KV-Segment-Schlüsselableitung.
- `struct EncryptedSegmentPayload` { layer } *[feature: kv-encryption]* — Encrypted layer representation stored inside a KvSegment when encryption is active.
- `struct KvSegment` { tenant_id, segment_id, key_derivation_version, encrypted, model_fingerprint, rope_offset } — Ein KV-Cache-Segment. · Methoden: new, new_with_metadata, new_encrypted, decrypt_data, to_spill_bytes, as_bytes, len, is_empty

**`kv_segment/store.rs`**

- `type SpillHandler = Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>` — Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache.
- `struct TenantIsolatedKvStore` — Tenant-isolierter KV-Segment-Store. · Methoden: new, with_shard_count, with_capacity, set_spill_handler, insert_segment, on_rollback, remove_segments_for_rollback, remove_segment, get_segment_bytes, insert_encrypted_segment, get_segments, get_decrypted_segment, get_tenant_segment_len, evict_lru_fair…

**`kv_shredding.rs`**

- `const DEFAULT_SHRED_KEY_GROUP_SIZE: usize = 64` — Default size of a shred key group (number of records sharing one sub-key).
- `struct SubKey` — Derived sub-key (256 bits) that is zeroized on drop.
- `fn derive_subkey(master_key: &KeyManager, group_id: u64) -> Result<SubKey>` — Derives a sub-key for a shred group from a master key (`KeyManager`) using HKDF-SHA256.
- `struct KeyRegistry` — In-memory thread-safe registry for shred group sub-keys. · Methoden: new, get_or_derive, revoke_subkey, is_key_active, encrypt_with_group, decrypt_with_group

**`wal_completeness.rs`**

- `fn verify_wal_chain_completeness( wal_tail_hmac: &[u8; 32], manifest_high_water_mark: &[u8; 32], ) -> Result<(), CryptoError>` — Verifiziert, dass der WAL-Tail nicht abgeschnitten wurde, indem der letzte bekannte WAL-HMAC mit der im Checkpoint-Manifest persistierten High-Water-Mark verglichen wird.

**`wal_crypto.rs`**

- `trait KmsProvider` (get_key) — Provides Key Management Strategy hooks.
- `struct EncryptedWal` — Encrypted WAL chunk provider that handles transparent encryption/decryption of WAL payloads. · Methoden: new, encrypt_chunk, decrypt_chunk
- `struct WalHmac` — Stateful wrapper around HMAC-SHA256 initialized with WAL domain separation. · Methoden: new, update, finalize
- `struct WalEntrySnapshot` { tx_id, seq_no, op_type, key, value, checksum, prev_hmac } — Immutable snapshot of a WAL entry used for cryptographic integrity verification.
- `struct IntegrityVerifier` — Stateful verifier for WAL HMAC-SHA256 hash chains. · Methoden: new, set_last_hmac, last_hmac_snapshot, verify_and_update_v3, verify_and_update_v2, skip_hmac_verify_legacy, verify_and_update

### K.9 `contextra-db` — 1.840 Zeilen (ohne Tests), 3 Dateien [V-Sig]

Contextra — Embedded hybrid-search for AI agents

**Features:** default = `—`; weitere: `bench`, `sandbox`, `reranking`, `onnx`, `experimental-diskann`, `background-maintenance`, `graph-connectivity-health`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `volatile-vault`, `edge-reinforcement-learning`

**Workspace-Abhängigkeiten:** `contextra-engine`, `contextra-cognition`, `contextra-sys`, `contextra-types`, `contextra-ports`, `contextra-crypto`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-checkpoint`, `contextra-graph`, `contextra-rank`, `contextra-adapt`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `multistep.rs` | 577 | — |
| `volatile_vault.rs` | 375 | Ephemerer RAM-Puffer für sensitive Kontexte (Safe-Modus). |

**`multistep.rs`**

- `struct MultiStepConfig` { max_rounds, quality_threshold, min_quality_hits, latency_budget_ms } — Konfiguration für Multi-Step Retrieval.
- `struct MultiStepResult` { results, rounds_executed, sub_queries } — Ergebnis einer Multi-Step-Suche mit Audit-Informationen.
- `struct MultiStepEngine` — Multi-Step Retrieval Engine. · Methoden: new, search
- `trait QueryRewriter` (rewrite) — Trait für Query-Rewriting (LLM-agnostisch).

**`volatile_vault.rs`**

- `enum SignalModality` ∈ {AudioTranscript, VisualCapture, TextInput, StructuredData} — Modalität eines Vault-Chunks — bestimmt Verarbeitungskontext beim Commit.
- `struct VaultChunk` { id, content, modality, captured_tx, label } — Ein einzelner Chunk im VolatileContextVault. · Methoden: new, with_label, size_bytes
- `struct VaultConfig` { max_capacity_bytes, attempt_mlock } — Konfiguration für den VolatileContextVault.
- `struct PurgeReceipt` { chunks_purged, bytes_zeroed, purged_at } — Quittung nach erfolgreichem purge().
- `struct CommitReceipt` { chunks_committed, bytes_committed } — Quittung nach erfolgreichem commit_to_storage().
- `enum VaultError` ∈ {CapacityExceeded, AlreadyConsumed} — Fehler des VolatileContextVault.
- `struct VolatileContextVault` — Ephemerer RAM-Puffer für sensitive Kontexte. · Methoden: open, ingest, purge, current_size_bytes, len, is_empty, preview_metadata
- `struct VaultChunkMetadata` { id, modality, size_bytes, label } — Metadaten-Ansicht eines VaultChunks ohne sensitiven Inhalt.

### K.10 `contextra-engine` — 12.102 Zeilen (ohne Tests), 56 Dateien [V-Sig]

Contextra — Core storage, index, and transaction orchestrator engine

**Features:** default = `—`; weitere: `encryption-at-rest`, `bench`, `sandbox`, `experimental-diskann`, `background-maintenance`, `graph-connectivity-health`, `edge-reinforcement-learning`, `coherence-bonus-fusion`, `adaptive-candidate-pool-sizing`, `entity-extraction`, `auto-extraction-opt-out`

**Workspace-Abhängigkeiten:** `contextra-sys`, `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-crypto`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-checkpoint`, `contextra-graph`, `contextra-rank`, `contextra-adapt`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `background_workers/config.rs` | 64 | — |
| `background_workers/expiry_workers.rs` | 114 | — |
| `background_workers/hyperedge_worker.rs` | 103 | — |
| `background_workers/orphan_workers.rs` | 177 | — |
| `chunker.rs` | 682 | — |
| `collection/crud/auto_extraction.rs` | 202 | — |
| `collection/crud/read.rs` | 297 | — |
| `collection/kv_lock.rs` | 224 | — |
| `collection/maintenance.rs` | 926 | — |
| `collection/mod.rs` | 698 | — |
| `collection/query_builder/builder.rs` | 285 | — |
| `collection/query_builder/scope.rs` | 59 | — |
| `collection/query_builder/strategy.rs` | 76 | — |
| `collection/query_builder/weights.rs` | 41 | — |
| `collection/search/checkpoint.rs` | 101 | — |
| `decay_controller.rs` | 205 | Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (F-01). |
| `export.rs` | 389 | — |
| `extraction/open_ie.rs` | 123 | — |
| `extraction/types.rs` | 45 | — |
| `filter.rs` | 323 | — |
| `fusion.rs` | 41 | Hybrid Search Signal Fusion implementations (Reciprocal Rank Fusion & Score Normalization). |
| `import.rs` | 127 | — |
| `lib.rs` | 607 | — |
| `temporal_filter.rs` | 353 | — |
| `transaction/cleanup.rs` | 90 | — |
| `transaction/compensating_actions.rs` | 336 | — |
| `transaction/db_transaction.rs` | 173 | — |
| `transaction/intent.rs` | 35 | — |

**`background_workers/config.rs`**

- `struct OrphanCleanupBackoffConfig` { base_delay, max_delay, alert_threshold } — Configuration parameters for HNSW index rebuild backoff and failure escalation.
- `fn calculate_rebuild_cooldown( consecutive_failures: u32, base_delay: Duration, max_delay: Duration, ) -> Duration` — Helper calculating exponential backoff cooldown given consecutive failures.
- `trait OrphanCleanupIndex` (check_connectivity, rebuild) — Abstraction trait over vector indexes capable of connectivity check and rebuild.

**`background_workers/expiry_workers.rs`**

- `const MAX_EXPIRED_PER_TICK: usize = 100` — Maximum number of expired documents processed in a single expiry cleanup tick.
- `fn start_expiry_cleanup_worker( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` — Starts a background task to periodically clean up expired documents with TTL.
- `fn start_expiry_reaper( collection: Arc<Collection<S>>, interval: Duration, cancel_token: tokio_util::sync::CancellationToken, ) -> tokio::task::JoinHandle<()>` — Deprecated legacy alias for `start_expiry_cleanup_worker`.
- `fn start_decay_cleanup_worker( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationTok` *[feature: background-maintenance]* — Starts a background task for decay-controller-driven importance-score eviction.
- `fn start_thermostat_reaper( collection: Arc<Collection<S, V>>, decay_config: DecayControllerConfig, interval: Duration, cancel_token: tokio_util::sync::CancellationToken,` *[feature: background-maintenance]* — Deprecated legacy alias for `start_decay_cleanup_worker`.

**`background_workers/hyperedge_worker.rs`**

- `const MAX_DEFERRED_HYPEREDGES_PER_TICK: usize = 1_000` — Maximum number of deferred hyperedges processed in a single worker tick.
- `struct DeferredHyperedgeQueue` — Thread-safe FIFO queue for storing hyperedges whose cascade invalidation was deferred to the background worker due to fan-out limits (H5 / AK-6). · Methoden: new, enqueue, drain_up_to
- `fn start_hyperedge_cascade_deferred_worker( collection: Arc<Collection<S>>, queue: Arc<DeferredHyperedgeQueue>, interval: Duration, cancel_token: tokio_util::sync::Cancel` — Starts a background task to process deferred hyperedge tombstones.

**`background_workers/orphan_workers.rs`**

- `const MAX_ORPHANS_PER_TICK: usize = 100` — Maximum number of orphan transactions processed in a single worker tick to avoid starving foreground operations.
- `fn start_orphan_cleanup_worker_with_config( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<I>, interval: Duration, backoff_config: OrphanCleanupBackoffConfig, cancel_token: to` — Starts a background task to periodically clean up orphan transactions and manage HNSW rebuilds with exponential backoff.
- `fn start_orphan_cleanup_worker( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>, interval: Duration, cancel_token: tokio_util::sync::Cancella` — Starts a background task to periodically clean up orphan transactions.
- `fn start_orphan_reaper( buffer: Arc<TxBuffer<T>>, hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>, interval: Duration, cancel_token: tokio_util::sync::CancellationToke` — Deprecated legacy alias for `start_orphan_cleanup_worker`.

**`chunker.rs`**

- `fn estimate_tokens(text: &str) -> usize` — Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln.
- `struct ChunkerConfig` { max_tokens, min_tokens, include_breadcrumbs, split_levels } — Configuration for the Markdown chunker.
- `struct MarkdownChunker` — A deterministic chunker for markdown files. · Methoden: new, with_defaults, chunk
- `fn chunk_text_with_overlap(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<&str>`
- `fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str>`

**`collection/crud/auto_extraction.rs`**

- `struct EntityExtractionConfig` { enabled, max_llm_calls_per_cycle, min_confidence }
- `struct AutoExtractionConfig` { mode, enabled, entity_config } — Konfiguration fuer die automatische Entitaetsextraktion beim Einfuegen von Dokumenten. · Methoden: new, with_mode, is_enabled

**`collection/crud/read.rs`**

- `const DEFAULT_SCAN_LIMIT: usize = 10_000` — Harte Obergrenze für scan()/scan_prefix()-Ergebnisse, falls kein explizites `limit` übergeben wird.
- `const HARD_SCAN_CEILING: usize = 100_000` — Harte Obergrenze für den maximal erlaubten `limit`-Parameter in scan() und scan_prefix().
- `const MAX_SCAN_RESULTS: usize = DEFAULT_SCAN_LIMIT` — Alias for backwards compatibility with earlier MAX_SCAN_RESULTS references.

**`collection/kv_lock.rs`**

- `fn sorted_unique_shards( keys: impl IntoIterator<Item = &'a str>, shard_count: usize, hasher: &ahash::RandomState, ) -> Vec<usize>` — Pure, synchronous function to map string keys to unique shard indices in ascending order.

**`collection/maintenance.rs`**

- `struct PercolationResult` { health, rebonding_triggered, new_edges_added } *[feature: graph-connectivity-health]*

**`collection/mod.rs`**

- `struct CollectionConfig` { backpressure_delay_ms } — Configuration for a collection.
- `struct StoredDocument` { id, embedding, metadata }
- `struct StoredDocumentMeta` { id, metadata } — Leichtgewichtige Metadaten (für doc_key, key_type=1) — KEIN Embedding.
- `fn parse_importance_score(response: &str) -> f32` — Parses an LLM response string into an f32 importance score in `[0.0, 1.0]`.
- `fn compute_default_importance(text_opt: Option<&str>) -> contextra_types::ImportanceScore` — Computes a non-LLM heuristic baseline importance score based on text length and character entropy.
- `fn ensure_importance_metadata( metadata: &mut Option<serde_json::Value>, tx: TxId, text_opt: Option<&str>, )` — Ensures document metadata contains a valid `MemoryImportance` JSON payload.
- `fn extract_effective_importance(metadata: &Option<serde_json::Value>, now_tx: TxId) -> f32` — Extracts the effective importance score of a document at a given transaction ID.
- `struct Collection` { consolidation_guard } — # Concurrency & Lock Hierarchy Lock acquisition within `Collection` follows strict ordering to prevent deadlocks: · Methoden: with_auto_extraction, set_auto_extraction, auto_extraction_config, delete, delete_op, insert_text_only, upsert_text_only, insert_with_ttl, insert_typed, insert, insert_op, insert_many, upsert, upsert_many…

**`collection/query_builder/builder.rs`**

- `const DEFAULT_RERANK_POOL_MULTIPLIER: usize = 10` — Default candidate pool expansion multiplier for Cross-Encoder reranking (10x requested k).
- `const DEFAULT_RERANK_POOL_MAX: usize = 200` — Default upper cap for pre-reranking candidate pool expansion (200 candidates).
- `struct HybridQueryBuilder` — Fluent query builder for unifying vector, text, graph, and hybrid search operations. · Methoden: new, text, embedding, vector, k, weights, fusion_weights, fusion_strategy, filter, metadata_filter, strategy, anchor_entities, anchors, same_community_as…

**`collection/query_builder/scope.rs`**

- `struct ScopeConstraint` { allowed_doc_ids, gamma } — Scoping constraint defining an explicit hard-boundary set of allowed document IDs for predicate-agnostic vector search (ACORN-Hard-Boundary, Spec §5.3, §8.5). · Methoden: new, from_allowed_ids

**`collection/query_builder/strategy.rs`**

- `enum SearchStrategy` ∈ {Rrf, ScoreNormalized, Hops, PersonalizedPageRank, PathRag} — Strategy for hybrid search and graph signal traversal. · Methoden: to_graph_strategy, to_fusion_strategy

**`collection/query_builder/weights.rs`**

- `struct SignalWeights` { vector, text, graph } — Custom weights for vector, text, and graph signals in hybrid search. · Methoden: new

**`collection/search/checkpoint.rs`**

- `struct CheckpointPinGuard` — RAII Guard for pinning snapshot checkpoints during search/scan operations. · Methoden: new, new_at_latest, release
- `fn with_pinned_checkpoint(storage: &S, seq: u64, f: F) -> Result<T>` — Higher-order function wrapping an async block with a pinned checkpoint guard, ensuring `release()` is ALWAYS called even on error paths.
- `fn with_pinned_checkpoint_at_latest(storage: &S, f: F) -> Result<T>` — Higher-order function providing pin-first, read-after semantics with automatic release on completion/error.

**`decay_controller.rs`**

- `struct DecaySignalInputs` { tombstone_ratio, query_load_inverse, w_tombstone, w_query } — Inputs for Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) aus bestehenden System-Metriken.
- `struct DecayControllerConfig` { kappa, base_half_life_tx, eviction_threshold } — Konfiguration für Adaptive Decay Controller (Cache-Eviction mit Time-Weighted Decay) (via MaintenanceConfig).
- `struct AdaptiveDecayController` · Methoden: new, with_defaults, system_temperature, effective_half_life, effective_score, should_evict

**`export.rs`**

- `const SCHEMA_VERSION_V1: &str = "1.0"` — Aktuelle Schema-Version für das JSON Export-Format.
- `struct ExportDocumentV1` { schema_version, exported_at, collections } — Wurzel-Dokument für den Export aller DB-Collections.
- `struct ExportCollectionV1` { name, memories, relations } — Export-Repräsentation einer einzelnen Collection.
- `struct ExportMemoryV1` { id, memory_type, content, embedding, embedding_model, created_at, importance_score, metadata, links } — Export-Repräsentation eines einzelnen Memory-Eintrags.
- `struct ExportRelationV1` { from, to, label } — Export-Repräsentation einer Graph-Beziehung.

**`extraction/open_ie.rs`**

- `fn extract_triples( text: &str, generator: &dyn LlmTextGenerator, config: &EntityExtractionConfig, ) -> Result<Vec<ExtractedTriple>>` — Extrahiert Wissens-Tripel (Subjekt, Prädikat, Objekt) aus dem gegebenen Text mittels LLM.

**`extraction/types.rs`**

- `struct ExtractedTriple` { subject, predicate, object, confidence, source_span } — Ein aus dem Text extrahiertes Wissens-Tripel (Subject, Predicate, Object).
- `struct EntityExtractionConfig` { enabled, max_llm_calls_per_cycle, min_confidence } — Konfiguration für die automatische Entitätsextraktion.

**`filter.rs`**

- `fn extract_memory_type(metadata: &Option<Value>) -> MemoryType` — Extrahiert den MemoryType aus Dokument-Metadaten (Rückwärtskompatibel).
- `enum FilterOp` ∈ {Eq, Ne, Gt, Gte, Lt, Lte, In, NotIn, Exists}
- `enum MetadataFilter` ∈ {Condition, And, Or, Not} · Methoden: matches

**`fusion.rs`**

- `fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, r` — Fuses search result sets using the specified FusionStrategy.
- `fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` — Convenience function for fusing multi-signal search results.

**`import.rs`**

- `struct ImportSummary` { imported_memories, skipped_memories, imported_relations, skipped_relations } — Zusammenfassung eines Import-Vorgangs.

**`lib.rs`**

- `enum no_crypto_stubs::DeletionLayer` ∈ {LsmMemtable, SsTableAllLevels, HnswIndex}
- `enum no_crypto_stubs::DeletionScope` ∈ {Document, Collection, Tenant}
- `struct no_crypto_stubs::LayerCleanupProof` { layer, remaining_count } · Methoden: new_after_verified_empty, verify_and_create
- `enum no_crypto_stubs::ExcludedScope` ∈ {ConsolidatedAndDistilled, LlmParameterMemory}
- `struct no_crypto_stubs::DeletionProof` { signature_version, scope, deleted_keys_hash, deleted_after_tx, timestamp, signature, covered_layers, excluded_scopes, wal_chain_receipt, integrity_warning… } · Methoden: create, tenant_id, verify, export_for_audit
- `struct no_crypto_stubs::TenantIsolatedKvStore` · Methoden: new, purge_tenant_segments, on_rollback, remove_tenant_segment
- `type ConsolidationLauncher = Arc< dyn Fn( Arc<Collection<LsmStorage>>, std::time::Duration, usize, tokio_util::sync::Ca`
- `fn register_consolidation_launcher(_f: F)`
- `struct Document` { id, metadata }
- `struct ContextraStats` { drift_status, calibration_ece, last_calibration_at, active_memory_count, pid_pool_size, index_stats, storage_stats }
- `type DbStats = ContextraStats`
- `struct CommunityDetectionConfig` { auto_trigger_threshold }
- `enum EmbeddingBackend` ∈ {Onnx, None}
- `enum TenantPolicy` ∈ {Optional, Required} — Tenant isolation enforcement policy.
- `struct ContextraConfig` { dimension, max_elements, distance_metric, encryption_passphrase, max_ram_mb, group_commit_window_micros, durability_mode, deletion_proof_active, vector_delete_mode, memtable_size_limit… } · Methoden: with_consolidation_launcher, with_tenant_policy
- `type TenantCollectionMap = ahash::AHashMap< (TenantId, String), Arc<Collection<contextra_store::tenant_codec::TenantS`
- `struct Contextra` · Methoden: collection_for_tenant, collection_with_language_and_tenant, collection, collection_with_language, allocate_tx, list_collections, list_collections_for_tenant, drop_collection, put_kv, get_kv, insert, insert_typed, upsert, insert_many… · impl: SandboxBridge
- `trait SandboxBridge` (db_search, db_insert, db_get) *[feature: sandbox]*

**`temporal_filter.rs`**

- `type FusionResult = crate::SearchResult` — Type alias for search & fusion results in post-retrieval pipelines.
- `struct ValidityWindow` { tx_valid_from, tx_valid_to, business_valid_from, business_valid_to } — Extracted bi-temporal validity window representation for a document or search candidate. · Methoden: is_empty, is_valid_bitemporal
- `fn extract_validity_window(metadata: Option<&Value>) -> ValidityWindow` — Extracts bi-temporal validity window metadata from candidate document metadata.
- `fn apply_temporal_validity_filter( results: Vec<crate::SearchResult>, current_tx: TxId, query_timestamp: Option<u64>, ) -> Vec<crate::SearchResult>` — Applies bi-temporal validity filtering to candidates post-RRF fusion.
- `fn apply_temporal_validity_filter_at( results: Vec<crate::SearchResult>, as_of_timestamp: u64, ) -> Vec<crate::SearchResult>` — Applies historical "as-of" bi-temporal validity filtering to candidates post-RRF fusion.

**`transaction/cleanup.rs`**

- `fn cleanup_orphaned_consolidation_intents( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>` — Aufgerufen beim Öffnen einer Collection / DB.
- `fn recover_pending_intents( storage: &S, next_tx: &std::sync::atomic::AtomicU64, ) -> Result<usize>` — Recovers or reconciles any pending transaction intents found in storage.

**`transaction/compensating_actions.rs`**

- `trait CompensatingAction` (execute) — Trait representing an undo operation to be executed during transaction rollback/compensation.
- `struct CommitLedger` — A ledger of compensating actions collected during 2PC, executed in reverse order (LIFO) on failure. · Methoden: new, push, execute_rollback
- `struct CompensateLsmAction` · Methoden: new · impl: CompensatingAction
- `struct CompensateGraphAction` · Methoden: new · impl: CompensatingAction
- `struct CompensateHnswAction` · Methoden: new · impl: CompensatingAction
- `struct CompensateTextAction` · Methoden: new · impl: CompensatingAction
- `struct RollbackStagedAction` · Methoden: new · impl: CompensatingAction

**`transaction/db_transaction.rs`**

- `struct DbTransaction` { tx_id } — A transaction wrapper that ensures atomic multi-index commits across LSM-Store, HNSW-Index, Text-Index, and Graph-Index. · Methoden: new, record_keys, record_keys_with_old_values, stage_text_insert, stage_text_delete, stage_graph_entity, stage_graph_edge, stage_graph_entity_delete, stage_graph_edge_delete, stage_hyperedge, commit, rollback

**`transaction/intent.rs`**

- `enum CommitIntent` ∈ {Pending, Committed, Aborted, Consolidation, Failed} — Status of a multi-index transaction during the 2-phase commit.

### K.11 `contextra-graph` — 13.025 Zeilen (ohne Tests), 41 Dateien [V-Sig]

CSR-Graph for entity-relation traversal (Signal 3 in 4-Signal Fusion)

**Features:** default = `—`; weitere: `docid-128`, `graph-connectivity-health`, `edge-reinforcement-learning`, `ppr-forward-push`, `k-path-diffusion`, `apprh-diffusion`, `physio-percolation`, `physio-synaptic-edges`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-adapt`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `apprh/diffusion.rs` | 176 | Core diffusion operator for Averaging-based Personalized PageRank for Hypergraphs (APPRH). |
| `apprh/error.rs` | 26 | Error types for Averaging-based Personalized PageRank for Hypergraphs (APPRH). |
| `apprh/gate_monitor.rs` | 260 | — |
| `apprh/mod.rs` | 98 | Averaging-based Personalized PageRank for Hypergraphs (APPRH). |
| `apprh/params.rs` | 113 | Configuration parameters for Averaging-based Personalized PageRank for Hypergraphs (APPRH). |
| `apprh/selector.rs` | 206 | — |
| `apprh/shadow.rs` | 138 | Shadow comparison module comparing Forward-Push PPR against APPRH. |
| `arc_slice.rs` | 158 | Generic zero-copy sub-slice backed by an `Arc<[T]>`. |
| `cascade.rs` | 1060 | — |
| `community.rs` | 630 | Leiden Community Detection algorithm for CsrGraph. |
| `consistency_enforcement.rs` | 453 | Consistency Enforcement / Konflikterkennungs-Register (Feature F-04). |
| `csr.rs` | 24 | CSR-Graph-Implementierung für Entity-Relation-Traversal. |
| `csr/graph_persist.rs` | 269 | Persistenz-Hilfsmethoden für CsrGraph. |
| `csr/graph_write.rs` | 925 | — |
| `csr/inner.rs` | 815 | — |
| `csr/types.rs` | 114 | — |
| `csr/visibility.rs` | 80 | — |
| `edge_reinforcement.rs` | 182 | F-03: Co-Occurrence- und Traversal-basierte Kantenverstärkung (Edge Weight Reinforcement Learning). |
| `edge_reinforcement_buffer.rs` | 202 | — |
| `error.rs` | 111 | Graph mutation error taxonomy for Contextra Graph. |
| `hyperedge.rs` | 825 | Hyperkanten-Kerndatenmodell (`HyperEdge`, `RoleBinding`, `RoleId`, `HyperEdgeId`, `RoleInterner`). |
| `hyperedge_suggest.rs` | 180 | Hyperedge suggestion and LLM validation module (Spec §13, Part 2). |
| `lib.rs` | 127 | Contextra Graph — CSR-Graph for Entity-Relation Traversal & Session DAG. |
| `path_rag/k_path.rs` | 322 | Budgeted k-Path Diffusion for PathRAG Engine. |
| `path_rag/mod.rs` | 459 | PathRAG Engine — Graph-basiertes Retrieval via bidirektionaler Dijkstra. |
| `percolation.rs` | 313 | F-06: Perkolations-Gesundheitsmonitor. |
| `ppr.rs` | 1045 | Personalized PageRank (PPR) power iteration implementation for `CsrGraph`. |
| `ppr/shadow_hook.rs` | 94 | Shadow mode hook for comparing TL-HFD PPR scores against baseline PPR scores (§12). |
| `ppr/snapshot.rs` | 142 | Snapshot-isolated Personalized PageRank (PPR) power iteration support. |
| `ppr_stream.rs` | 152 | Candidate stream for Personalized PageRank (PPR) graph retrieval (DiBud / Spec §21.2 / AK-17). |
| `provenance.rs` | 176 | — |
| `session_dag.rs` | 980 | Session-DAG for Agent State Branching (Contextra Session-DAG Pattern). |
| `tl_hfd/diffusion.rs` | 245 | Core diffusion loop and active region management for TL-HFD (Spec §21.1). |
| `tl_hfd/error.rs` | 22 | Error types for Thresholded Local Hyper-Flow Diffusion (TL-HFD). |
| `tl_hfd/lovasz.rs` | 194 | Lovász extension and subgradient calculations for TL-HFD (Spec §21.1). |
| `tl_hfd/mod.rs` | 54 | Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1). |
| `tl_hfd/params.rs` | 136 | Configuration parameters for Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1). |
| `tl_hfd/shadow.rs` | 153 | Shadow comparison module comparing Forward-Push PPR against TL-HFD (AK-16). |

**`apprh/diffusion.rs`**

- `fn run_apprh_push( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` — Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH).

**`apprh/error.rs`**

- `enum ApprhError` ∈ {InvalidParameter, EmptySeeds, NonFinite} — Error type for APPRH operations and parameter validations.

**`apprh/gate_monitor.rs`**

- `struct ApprhGateMonitorConfig` { window_size, required_consecutive_passes } — Configuration for [`ApprhGateMonitor`]. · Methoden: validate
- `struct ApprhObservationRecord` { max_abs_diff, top_k_overlap, discrepancy, passed } — Historical observation record stored within the sliding window of [`ApprhGateMonitor`].
- `struct ApprhGateMonitorSnapshot` { total_observations, current_streak, last_overlap, max_abs_diff_in_window, min_overlap_in_window, discrepancy_count_in_window, production_ready } — Telemetry snapshot from [`ApprhGateMonitor`].
- `struct ApprhGateMonitor` — Independent, thread-safe monitor evaluating a series of APPRH shadow mode comparisons for production readiness (P1.3). · Methoden: new, with_gate, observe, production_ready, snapshot, reset, config

**`apprh/mod.rs`**

- `fn apprh_local( graph: &G, seeds: &[EntityId], params: &ApprhParams, ) -> Result<AHashMap<EntityId, f32>, ApprhError>` — Computes Averaging-based Personalized PageRank for Hypergraphs (APPRH) over a generic [`PathGraph`].
- `fn forward_push_apprh( graph: &G, seeds: &[EntityId], params: &PprParams, hyperedge_decay_factor: f32, ) -> Result<AHashMap<EntityId, f32>, contextra_types::ContextraErro` — Spec-compliant additive APPRH operator (Spec Y.1.2).

**`apprh/params.rs`**

- `struct ApprhParams` { ppr, hyperedge_decay_factor, max_iterations } — Configuration parameters for Averaging-based Personalized PageRank for Hypergraphs (APPRH). · Methoden: validate

**`apprh/selector.rs`**

- `enum ApprhMode` ∈ {Off, Shadow, Production} — Operating mode for [`ApprhSelector`].
- `struct ApprhSelectorConfig` { sample_interval, max_seeds, shadow_top_k } — Configuration parameters for [`ApprhSelector`].
- `struct ApprhSelector` — Instance-level selector managing APPRH shadow-mode evaluation and production switching (P1.3). · Methoden: new, with_default_monitor, mode, set_mode, promote_to_production, demote_to_shadow, monitor, evaluate_and_run

**`apprh/shadow.rs`**

- `trait ApprhFlipGate` (should_flip) — Formal default-flip gate trait evaluating whether APPRH can replace ForwardPush as default.
- `struct DefaultApprhFlipGate` — Standard default flip gate evaluator for APPRH. · impl: ApprhFlipGate
- `struct ApprhShadowComparison` { max_abs_diff, top_k_overlap, discrepancy, forward_push_results, apprh_results } — Result structure of shadow mode comparison between Forward-Push PPR and APPRH.
- `fn shadow_compare_forward_push_vs_apprh( graph: &G, seeds: &[EntityId], ppr: &PprParams, apprh_params: &ApprhParams, top_k: usize, ) -> Result<ApprhShadowComparison, Appr` — Computes shadow comparison between Forward-Push PPR and APPRH.

**`arc_slice.rs`**

- `struct ArcSlice` — A zero-copy slice view into an underlying `Arc<[T]>`. · Methoden: new, from_vec, try_slice, slice, as_arc, offset, len, is_empty, as_slice · impl: Deref

**`cascade.rs`**

- `const MAX_HYPEREDGE_CASCADE_FANOUT: usize = 1_000` — Maximum number of hyperedges processed synchronously per cascade invalidation run to bound latency spikes.
- `const DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT: usize = MAX_HYPEREDGE_CASCADE_FANOUT` — Default fan-out limit for hyperedge cascade invalidation per document (§6.8).
- `struct CascadeInvalidationReport` { tombstoned_edge_count, affected_node_ids } — Report summarizing the cascade invalidation of graph edges derived from a superseded document.
- `fn cascade_invalidate_edges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<CascadeInvalidationReport>` — Tombstoniert alle Kanten, die von `superseded_doc_id` abgeleitet wurden, als Reaktion auf ein Supersedes-Ereignis.
- `const CASCADE_QUEUE_PREFIX: &str = "__graph:cascade_queue:"` — LSM-Key-Präfix für die persistent cascade queue (§6.6 / §6.8).
- `enum CascadeStatus` ∈ {Pending, InFlight, Completed, Failed} — Status tracking for a cascade invalidation ticket (§6.8).
- `struct DeletionProof` { doc_id, tombstoned_count, wal_seq } — Proof of edge / hyperedge deletion during cascade invalidation (§6.8).
- `struct CascadeTicket` { id, doc_id, status, proof } — Ticket tracking persistent cascade invalidation status (§6.8).
- `trait GraphGarbageCollection` (sweep_orphans) — Garbage collection trait for sweeping orphaned hyperedge references (Anhang B §B.5.1.7).
- `struct HyperedgeCascadeReport` { invalidated, deferred, queued_for_background } — Report summarizing the cascade invalidation of hyperedges derived from a superseded document.
- `type CascadeReport = HyperedgeCascadeReport` — Alias for [`HyperedgeCascadeReport`] (§6.8 specification alignment).
- `fn topological_sort_hyperedge_closure( graph: &CsrGraph, closure_nodes: &[crate::hyperedge::HyperEdgeId], ) -> Vec<crate::hyperedge::HyperEdgeId>` — Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection.
- `fn cascade_invalidate_hyperedges_for_superseded_doc( graph: &CsrGraph, superseded_doc_id: DocId, wal_seq: u64, ) -> Result<HyperedgeCascadeReport>` — Cascade invalidates hyperedges derived from `superseded_doc_id` with fan-out protection and recursive super-edge ancestor invalidation.
- `fn verify_doc_hyperedge_deletion_completeness( graph: &CsrGraph, target_doc_id: DocId, ) -> Result<()>` — Verifies byte-accurately that no active (non-tombstoned) hyperedge in `graph` directly or transitively (via `child_edge_ids`) references `target_doc_id`.

**`community.rs`**

- `struct CommunityDetectionConfig` { max_iterations, seed, hyperedges_included } — Configuration for Leiden Community Detection.
- `struct VirtualHyperedgeNode` { hyperedge_id } · Methoden: new
- `struct StarExpansionIterator` — Stern-Expansion: · Methoden: new, next_with_weight · impl: Iterator
- `struct CommunityAssignment` { entity_id, community_id, hyperedges_included } — Represents the assignment of an entity to a detected community.
- `fn detect_communities( graph: &CsrGraph, config: &CommunityDetectionConfig, ) -> Result<Vec<CommunityAssignment>>` — Detects semantic communities in the given CSR graph using the deterministic Leiden algorithm.

**`consistency_enforcement.rs`**

- `type EdgeId = (EntityId, EntityId)` — Identifikator für eine Kante im CSR-Graphen.
- `struct ConflictPattern` { pattern_hash, contradiction_count, first_detected_tx, last_detected_tx, suppressed } — Eintrag für ein gelerntes Konfliktmuster (Constraint-Violation-Signatur).
- `struct EdgeAssertion` { subject, predicate_hash, object_repr } — Abstrakte Aussage über eine Kante für die semantische Widerspruchsprävention. · Methoden: pattern_hash
- `trait ContradictionDetector` (conflicts) — Trait für modulare Widerspruchserkennungs-Strategien.
- `struct ExactPredicateConflictDetector` — Referenzimplementierung für exakten Prädikats-Konflikt: · impl: ContradictionDetector
- `struct ConsistencyEnforcer` — Consistency-Enforcement-Register zur Verwaltung registrierter Konfliktmuster. · Methoden: new, check_before_insert, detect_contradiction, record_contradiction, is_suppressed, active_patterns, get_pattern, suppression_threshold, suggest_tombstone_candidates

**`csr/graph_write.rs`**

- `struct CsrGraph` { config, inner, write_state, storage, last_tx_id, consistency_enforcer, doc_edge_index, cascade_queue } — Compressed Sparse Row graph for entity-relation traversal. · Methoden: apprh_diffusion, apprh_shadow_compare, persist_entity, persist_edge, delete_edge_persistence, load_from_storage, compact, compact_async, set_communities_batch, get_communities_batch, neighbors, pagerank, get_deleted_node_indices, deleted_view… · impl: GraphGarbageCollection, GraphIndexExt, GraphIndex, PathGraph

**`csr/inner.rs`**

- `struct GraphInner` — Inner state of the CsrGraph to manage contiguous storage. · Methoden: new, get_or_create_index, compact · impl: PathGraph
- `struct MemoryEstimate` { shared_payload_bytes, private_bytes } — Detailed memory estimate separating shared payloads from private structural allocations (§6.3). · Methoden: total_bytes

**`csr/types.rs`**

- `enum EdgeType` ∈ {Default} — Edge type representation for CSR edges.
- `struct Edge` { target, weight, edge_type, cooccurrence_weight, traversal_weight } — Edge structure in CSR graph representation. · Methoden: new
- `struct PersistedEdgePayload` { weight, tx_valid_from, tx_valid_to, business_valid_from, business_valid_to, source_doc_id } — Persisted edge payload format for storage.
- `const SCORE_DECAY: f32 = 0.7` — Score decay factor per hop (0.7^hop).
- `const MAX_TRAVERSAL_HOPS: u8 = 3` — Maximum traversal depth.
- `const MAX_VISITED_NODES: usize = 10_000` — Maximum visited nodes limit during BFS traversal to prevent intermediate hub-node memory explosion.
- `struct CsrGraphConfig` { rebuild_threshold, max_compaction_peak_memory_mb, resource_tracker }

**`csr/visibility.rs`**

- `const WALLCLOCK_TX_HEURISTIC_MIN: u64 = 1_400_000_000_000_000_000` — Untere Schranke für Wall-Clock-abgeleitete TxId-Heuristik.
- `fn is_suspicious_tx_id(tx: TxId) -> bool` — Prüft, ob `tx` aus einem verdächtigen (wall-clock-ähnlichen) Bereich stammt.
- `fn is_edge_visible( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of: TxId, ) -> bool` — Prüft ob eine Kante bezogen auf die Transaktionszeit (MVCC / Systemzeit) zum Zeitpunkt `as_of` sichtbar ist.
- `fn is_edge_visible_business( business_valid_from: Option<i64>, business_valid_to: Option<i64>, business_as_of: i64, ) -> bool` — Prüft ob eine Kante bezogen auf die Businesszeit zum Zeitpunkt `business_as_of` gültig ist.
- `fn is_edge_visible_bitemporal( tx_valid_from: Option<TxId>, tx_valid_to: Option<TxId>, as_of_tx: TxId, business_valid_from: Option<i64>, business_valid_to: Option<i64>, a` — Prüft bi-temporale Sichtbarkeit einer Kante (unabhängige Auswertung von System- und Businesszeit).

**`edge_reinforcement.rs`**

- `struct EdgeReinforcementConfig` { eta, delta, w_max, rho, q, alpha }
- `fn compute_edge_weight(cooccurrence_weight: f32, traversal_weight: f32, alpha: f32) -> f32` — Berechnet das Gesamt-Kantengewicht für eine Kante.
- `fn apply_cooccurrence_reinforcement( edge: &mut Edge, co_activation: f32, config: &EdgeReinforcementConfig, ) -> bool` — Führt Co-Occurrence-Verstärkung durch und prüft Normalisierungs-Invariante.
- `fn apply_traversal_reinforcement( edge: &mut Edge, path_length: usize, config: &EdgeReinforcementConfig, )` — Führt Traversal-Verstärkung durch (Evaporation + Verstärkung).
- `fn apply_weight_normalization(outgoing_edges: &mut [Edge], w_max: f32)` — Gewichtsnormalisierung:

**`edge_reinforcement_buffer.rs`**

- `struct edge_reinforcement_buffer::CooccurrenceSignal` { from, to, co_activation } — Ein gepufferter Co-Occurrence-Update:
- `struct edge_reinforcement_buffer::TraversalSignal` { from, to, path_length } — Ein gepufferter Traversal-Update:
- `struct edge_reinforcement_buffer::EdgeReinforcementBuffer` — Puffert pending Edge-Reinforcement-Updates bis zum nächsten Scheduler-Flush. · Methoden: new, push_cooccurrence, push_traversal, cooccurrence_count, traversal_count, flush_to_graph

**`error.rs`**

- `enum GraphMutationError` ∈ {LockAcquisitionTimeout, RoleBindingInvalid, EpochReclamationPending, InsufficientParticipants, HyperedgeNotFound, DuplicateHyperEdgeId, InvalidWeight, Internal} — Error variants for graph mutation operations.

**`hyperedge.rs`**

- `const HYPEREDGE_PREFIX: &str = "__graph:hyperedge:"` — LSM-Key-Präfix für Hyperkanten.
- `const HYPEREDGE_BY_ENTITY_PREFIX: &str = "__graph:hyperedge_by_entity:"` — LSM-Key-Präfix für Hyperkanten-Index nach Entity.
- `fn star_weight(weight: f32, cardinality: usize) -> f32` — Berechnet das Kantengewicht in der Stern-Expansion nach Konvention K (§6.6 H6).
- `struct HyperEdgeId` · Methoden: new, inner
- `struct RoleId` · Methoden: new, inner
- `struct RoleBinding` { role, entity } — Bindung einer Entity an eine spezifische Rolle innerhalb einer Hyperkante. · Methoden: new
- `struct HyperEdge` { id, predicate, participants, weight, tx_valid_from, tx_valid_to, business_valid_from, business_valid_to, source_doc_id, child_edge_ids } — Eine n-äre Hyperkante zur Verbindung von 2 oder mehr Entitäten mit spezifischen Rollen. · Methoden: new, with_tx_validity, with_business_validity, with_source_doc_id, with_child_edge_ids, validate, serialize, deserialize, view
- `struct HyperEdgeView` { edge, participants } — Zero-Copy read-only view of a [`HyperEdge`] backed by [`ArcSlice<RoleBinding>`]. · Methoden: new, slice_participants · impl: Deref
- `struct RoleInterner` — Concurrent string interner for hyperedge participant roles ([`RoleId`]). · Methoden: new, get_or_intern, resolve, resolve_string, contains_role, contains_id, len, is_empty
- `fn sort_dedup_entities(entities: &[EntityId]) -> Vec<EntityId>` — Sortiert und dedupliziert eine Menge von [`EntityId`]s in kanonischer (aufsteigender) Reihenfolge.
- `struct ConsolidationNodesGuard` — Guard zur Erzwingung kanonischer Lock-Reihenfolge über mehrere Entitäten (H2 / §5.1.2). · Methoden: acquire, try_acquire, entities, with_entities

**`hyperedge_suggest.rs`**

- `const MAX_RELATE_PARTICIPANTS: usize = 64` — Maximum allowed participants in a hyperedge candidate (Combinatorial explosion guard, P5).
- `struct HyperEdgeCandidate` { participants, co_occurrence_count, source_doc_ids } — A candidate hyperedge derived from document co-occurrence.
- `struct ValidatedHyperEdgeCandidate` { candidate, predicate, llm_confidence, accepted } — A candidate hyperedge after LLM semantic validation.
- `enum HyperedgeSuggestError` ∈ {LlmValidationBudgetExceeded} — Errors specific to hyperedge suggestion and validation.
- `fn compute_co_occurrence_candidates( entities_per_document: &[(DocId, Vec<EntityId>)], min_co_occurrence: usize, ) -> Vec<HyperEdgeCandidate>` — Computes hyperedge candidates from document entity co-occurrences.
- `fn validate_candidates_with_llm( candidates: &[HyperEdgeCandidate], entity_labels: &dyn Fn(EntityId) -> Option<String>, generator: &dyn TextGenerator, max_llm_calls_per_c` — Validates hyperedge candidates using an LLM text generator.

**`lib.rs`**

- `trait GraphIndexExt` (remove_entity) — Extension trait for [`contextra_ports::GraphIndex`] providing entity removal functionality.

**`path_rag/k_path.rs`**

- `struct KPathConfig` { k, max_visited_nodes, max_hops } — Configuration parameters for [`KPathDiffusion`].
- `struct KPathResult` { paths, budget_exhausted } — Result returned by [`KPathDiffusion::find_k_paths`].
- `trait KPathDiffusion` (find_k_paths) — Trait for budgeted k-path diffusion search over entity graphs.

**`path_rag/mod.rs`**

- `struct GraphPath` { nodes, edge_weights, confidence, total_flow } — Ein gefundener Pfad zwischen zwei Knoten.
- `trait PathGraph` (neighbors_with_weights, predecessors_with_weights, hyperedges_for_entity, get_hyperedge, hyperedge_participants) — Trait für Graphen die PathRAG konsumieren kann.
- `struct PprParams` { alpha, epsilon, hyperedge_decay } — Configuration parameters for Forward-Push Personalized PageRank (PPR).
- `fn forward_push_ppr( graph: &G, seeds: &[EntityId], params: &PprParams, ) -> AHashMap<EntityId, f32>` — Andersen-Chung-Lang Forward-Push PPR implementation with Hyperedge Expansion (§6.6, H3).
- `const DEFAULT_SUFFICIENCY_THRESHOLD: f64 = 0.1` — Normativer Default-Schwellenwert für das Sufficiency-Gate (ADR-067).
- `struct PathRAGConfig` { max_hops, sufficiency_threshold, hyperedge_expansion_enabled, hyperedge_weight_discount } — Configuration options for [`PathRAGEngine`].
- `struct PathRAGEngine` { config } · Methoden: new, with_defaults, with_config, max_hops, sufficiency_threshold, find_path, find_all_paths, sufficiency_check, to_rrf_signal · impl: KPathDiffusion

**`percolation.rs`**

- `struct PercolationConfig` { critical_threshold, rebonding_similarity, max_new_edges_per_pass } — Konfiguration für den Perkolations-Gesundheitsmonitor.
- `fn compute_percolation_health( active_node_count: usize, active_edge_count: usize, // nach Tombstone-Abzug ) -> Option<f32>` — Berechnet aktuelle Perkolations-Gesundheit φ(t).
- `fn should_trigger_rebonding(health: f32, config: &PercolationConfig) -> bool` — Prüft ob Re-Bonding-Pass ausgelöst werden soll.
- `fn find_rebonding_candidates( graph: &G, embeddings: &HashMap<EntityId, Vec<f32>>, config: &PercolationConfig, ) -> Vec<(EntityId, EntityId, f32)>` — Findet Kandidaten-Paare für Re-Bonding.

**`ppr.rs`**

- `struct DeletedView` — Information view over graph tombstones/deleted node indices. · Methoden: contains, len, is_empty
- `struct PprContext` — Reusable scratch buffers for Personalized PageRank (PPR) power iteration. · Methoden: new
- `fn compute_ppr_with_apprh_selector( inner: &GraphInner, seed_nodes: &[EntityId], config: &PprConfig, deleted_nodes: &DeletedView, ctx: &mut PprContext, selector: &crate::` *[feature: apprh-diffusion]* — Calculates Personalized PageRank using APPRH selector routing and hysteresis gate evaluation (P1.3).

**`ppr/shadow_hook.rs`**

- `fn log_tl_hfd_vs_baseline_discrepancy( baseline: &AHashMap<EntityId, f32>, candidate: &AHashMap<EntityId, f32>, context_id: u64, sink: &dyn ShadowSink<f64>, )` — Protokolliert die mittlere absolute Differenz (MAE) zwischen den Baseline- und TL-HFD-Kandidaten-PPR-Score-Maps über die gegebene Diskrepanz-Senke.

**`ppr_stream.rs`**

- `const DEFAULT_PPR_STREAM_BATCH_SIZE: usize = 16` — Default batch size for candidate retrieval (Spec §21.2).
- `struct PprCandidateStream` — On-demand loading async candidate stream for Personalized PageRank search. · Methoden: new, with_batch_size, with_exclude_seeds, next_batch, next_entity, next_batch_docs, yielded, is_exhausted, max_depth

**`provenance.rs`**

- `struct EdgeProvenance` { edge_id, source_doc_ids, created_at_tx } — Herkunftsnachweis für eine Graph-Kante. · Methoden: new
- `struct DocEdgeIndex` — Rückverfolgung DocId -> betroffene Kanten, für Cascading-Invalidation. · Methoden: new, record, record_provenance, edges_for_doc, remove_doc

**`session_dag.rs`**

- `struct NodesGuard` — Guard, der NUR erhalten werden kann, nachdem der `nodes`-Lock bereits gehalten wird. · Methoden: nodes, edges, edges_write, active_head, active_head_write
- `struct NodesWriteGuard` — Exklusiver Schreib-Guard, der NUR erhalten werden kann, nachdem der `nodes`-Schreiblock bereits gehalten wird. · Methoden: nodes, nodes_mut, edges, edges_write, active_head, active_head_write
- `type NodeIdx = u64` — Index of a node in the Session-DAG.
- `struct AgentStateNode` { step_id, prompt, response, snapshot_tx_id, tool_outputs, compacted } — State of an agent step in the conversation DAG.
- `struct DagEdge` { parent, child, label } — Directed edge in the DAG:
- `const MAX_DAG_STRING_BYTES: usize = 10 * 1024 * 1024` — Maximum allowed byte size for prompt/response strings (10 MB).
- `const MAX_DAG_TRAVERSAL_DEPTH: usize = 10_000` — Maximale Pfadtiefe in einem SessionDag (Schutz vor Zyklen und Tiefenexplosion).
- `struct SessionDagStepOptions` { prompt, response, snapshot_tx_id, tool_outputs, label } — Options for creating or branching a new step in the Session-DAG.
- `struct SessionBranchTree` — Session-DAG for a single agent conversation tree. · Methoden: new, lock_nodes, lock_nodes_write, append_step, branch_from, set_active_head, path_to_head, children_of, active_head, node_count, get_node, save, load

**`tl_hfd/diffusion.rs`**

- `fn run_diffusion( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` — Runs Thresholded Local Hyper-Flow Diffusion over a generic [`PathGraph`].

**`tl_hfd/error.rs`**

- `enum TlHfdError` ∈ {InvalidParameter, NonFinite} — Error type for TL-HFD operations and parameter validations.

**`tl_hfd/lovasz.rs`**

- `fn truncate_participants( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, max_sort_size: usize, ) -> Vec<EntityId>` — Truncates hyperedge participants to at most `max_sort_size` top x-value participants plus the min x-value participant.
- `fn compute_lovasz_extension( participants: &[EntityId], get_x: impl Fn(EntityId) -> f32, ) -> (f32, Option<EntityId>, Option<EntityId>)` — Evaluates Lovász extension $f_e(x) = \max_{u \in e} x_u - \min_{u \in e} x_u$ and subgradient $s_{e, u}(x)$ for participants.

**`tl_hfd/mod.rs`**

- `fn tl_hfd_local( graph: &G, seeds: &[EntityId], params: &TlHfdParams, ) -> Result<AHashMap<EntityId, f32>, TlHfdError>` — Computes Thresholded Local Hyper-Flow Diffusion (TL-HFD) over a generic [`PathGraph`].

**`tl_hfd/params.rs`**

- `struct TlHfdParams` { sigma, delta, gamma, max_iterations, max_top_k_expansion, max_hyperedge_sort_size } — Configuration parameters for Thresholded Local Hyper-Flow Diffusion (TL-HFD). · Methoden: validate

**`tl_hfd/shadow.rs`**

- `const MIN_SHADOW_SAMPLES: u64 = 10_000` — Minimum required sample count for shadow mode default flip evaluation.
- `const MIN_AGREEMENT_THRESHOLD: f32 = 0.85` — Minimum required mean top-k Jaccard agreement threshold for shadow mode default flip evaluation.
- `struct ShadowDiscrepancyReport` { sample_count, mean_topk_jaccard, p99_latency_delta_us, recall_improvement_ratio } — Aggregated discrepancy report over a shadow mode evaluation window.
- `trait DefaultFlipGate` (should_flip) — Formal default-flip gate trait evaluating whether TL-HFD can replace ForwardPush as default.
- `struct TlHfdFlipGate` — Standard default flip gate evaluator for TL-HFD. · impl: DefaultFlipGate
- `struct ShadowComparison` { max_abs_diff, top_k_overlap, discrepancy, forward_push_results, tl_hfd_results } — Result structure of shadow mode comparison between Forward-Push PPR and TL-HFD.
- `fn shadow_compare_forward_push_vs_tl_hfd( graph: &G, seeds: &[EntityId], ppr: &PprParams, tl: &TlHfdParams, top_k: usize, ) -> Result<ShadowComparison, TlHfdError>` — Computes shadow comparison between Forward-Push PPR and TL-HFD.

### K.12 `contextra-infer-candle` — 4.292 Zeilen (ohne Tests), 14 Dateien [V-Sig]

Native Candle GGUF ML inference backend for Contextra

**Features:** default = `—`; weitere: `candle`, `kv-bridge`, `kv-stage-b`, `cuda`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-rank`, `contextra-crypto`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `attention_exporter.rs` | 100 | Attention exporter implementation for Candle inference backend. |
| `embedding.rs` | 459 | — |
| `embedding_provider.rs` | 98 | — |
| `gasp.rs` | 664 | — |
| `gguf_loader.rs` | 95 | — |
| `inference/core.rs` | 979 | — |
| `inference/prefix_reuse.rs` | 91 | — |
| `kv_bridge.rs` | 695 | — |
| `kv_state.rs` | 319 | — |
| `model/quantized_llama.rs` | 560 | — |
| `model_registry.rs` | 143 | — |

**`attention_exporter.rs`**

- `const MAX_TRACKED_REQUESTS: usize = 256` — Maximum number of tracked requests in the attention history ring buffer.
- `struct CandleAttentionExporter` — Sums and exports attention weights across heads and layers of prefill steps. · Methoden: new, record_attention_weights, record_layer_head_scores, tracked_request_count, clear · impl: AttentionExporter

**`embedding.rs`**

- `const DEFAULT_MAX_CONCURRENT_EMBEDDINGS: usize = 8` — Default maximum concurrent embedding operations for Candle vector embedding.
- `trait CandleEmbedInner` (embed, dim) — Inner trait abstracting low-level Candle forward execution for vector embeddings.
- `struct CandleEmbedClient` { device, model, fingerprint, tokenizer, dim, max_concurrent_embeddings, semaphore } — Text embedding client powered by Candle ML backend. · Methoden: new, with_max_concurrent_embeddings, fingerprint, from_dir · impl: EmbeddingProvider
- `struct BertEmbedModel` — Real BERT transformer text embedding model wrapper. · Methoden: load · impl: CandleEmbedInner
- `struct DefaultCandleEmbedModel` { dim } — Default inner Candle embedding mock model for unit testing when weight files are missing. · impl: CandleEmbedInner

**`embedding_provider.rs`**

- `const MAX_CANDLE_EMBED_BATCH_SIZE: usize = 256` — Maximum batch size for Candle GGUF embeddings.

**`gasp.rs`**

- `const DEFAULT_GROUNDING_THRESHOLD: f32 = 0.70` — Standard-Schwellenwert für Grounding-Konfidenz (Default:
- `struct GaspConfig` { threshold, warmup_required, max_observations, fingerprint } — Konfiguration für den `GaspValidator`.
- `struct GaspValidator` — Post-Hoc-Halluzinations-Validator (GASP / TPA-Pattern). · Methoden: new, with_config, set_threshold, record_external_feedback, threshold, config, refresh_config, observation_count, compute_raw_grounding_score · impl: ResponseGroundingValidator, GroundingValidator

**`gguf_loader.rs`**

- `struct GgufMetadata` { architecture, tensor_count, metadata_keys } — Metadata extracted from a GGUF model container.
- `fn parse_gguf_metadata(model_path: &Path) -> Result<GgufMetadata, ContextraError>` — Inspects a GGUF model file and extracts its architecture and metadata header keys.

**`inference/core.rs`**

- `trait CandleModelInner` (generate, generate_stream, kv_layout, generate_stream_with_prefix) — Inner trait abstracting low-level Candle forward/text-generation execution.
- `const DEFAULT_MAX_CONCURRENT_INFERENCES: usize = 4` — Default maximum concurrent inference operations for Candle LLM text generation.
- `struct CandleLlmClient` { device, model, fingerprint, tokenizer, max_concurrent_inferences, semaphore, kv_bridge, prefill_count, prefill_skip_count, prefix_store… } — LLM Text Generator implementation powered by Candle inference engine. · Methoden: new, prefill_count, prefill_skip_count, prefill_skipped_tokens, with_prefix_store, generate_with_prefix_store, with_max_concurrent_inferences, with_kv_bridge, fingerprint, swap_model, from_dir · impl: LlmTextGenerator, LlmTextGeneratorStreaming
- `struct QuantizedLlamaModel` — Real quantized Llama / GGUF model execution wrapper. · Methoden: load · impl: CandleModelInner
- `struct DefaultCandleLlmModel` — Default inner Candle LLM mock model for unit tests when binary weights are absent. · impl: CandleModelInner

**`inference/prefix_reuse.rs`**

- `struct PrefixSeed` { matched_len, state } — Seed state for executing prompt generation with an imported KV cache prefix.
- `struct PrefixRun` { output, prompt_tokens, reused_tokens, exported_prefix } — Execution result of a prefix-assisted generation run.
- `struct KvPrefixContext` { store } — Wrapper context holding a thread-safe KV prefix store reference.
- `fn build_prefix_key( fingerprint: &ModelFingerprint, tokenizer: &tokenizers::Tokenizer, layout: KvLayout, rope: RopeConfig, ) -> Result<PrefixKey>` — Constructs a `PrefixKey` for the given model fingerprint, tokenizer, layout, and RoPE config.
- `fn seed_from_hit(hit: &KvPrefixHit, tokens_len: usize) -> Result<PrefixSeed>` — Reconstructs a `PrefixSeed` from a `KvPrefixHit` and total prompt token count.

**`kv_bridge.rs`**

- `struct KvCacheKey` { chunk_id, fingerprint, rope_offset } — Cache-Lookup-Schlüssel: · Methoden: new
- `struct KvBridgeAdapter` { store, lsm_store, cipher, consultations } — Verbindet Retrieval-Chunks mit dem mandantenisolierten, verschlüsselten KV-Cache. · Methoden: new, with_lsm_fallback, consult_segment, consultation_count, try_get_cached_segment, try_get_cached_segment_async, try_get_cached_segment_async_bytes, store_segment

**`kv_state.rs`**

- `struct LayerKv` { k, v } — Represents the key (`k`) and value (`v`) tensors for a single transformer layer. · Methoden: new
- `struct KvState` { layers, pos } — Explicit KV-Cache state holding layer-wise tensors and current sequence length position. · Methoden: new, is_empty, pos, layer_count, truncate, export_block, import_block, import_block_at

**`model/quantized_llama.rs`**

- `const MAX_SEQ_LEN: usize = 4096`
- `struct QuantizedMatMul`
- `struct LayerWeights` { attention_wq, attention_wk, attention_wv, attention_wo, attention_norm, ffn_norm, n_head, n_kv_head, head_dim, rope_is_neox… } — Transformer layer weights wrapping QMatMul attention projections, RMS norms, and KV cache.
- `struct ModelWeights` { tok_embeddings, layers, norm, output, masks } — Quantized Llama model weights and layer structures with explicit KvState management. · Methoden: from_gguf, export_kv_state, import_kv_state, truncate_kv_cache, clear_kv_cache, forward

**`model_registry.rs`**

- `enum CandleQuantization` ∈ {Q4KM, Q8_0, F16} — Supported Candle quantization formats. · impl: FromStr
- `fn compute_fingerprint( model_path: &Path, quantization: &CandleQuantization, ) -> Result<ModelFingerprint, ContextraError>` — Computes a unique fingerprint for a model file and quantization variant.

### K.13 `contextra-infer-ollama` — 3.029 Zeilen (ohne Tests), 12 Dateien [V-Sig]

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-rank`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `client/config.rs` | 45 | — |
| `client/core.rs` | 984 | — |
| `client/errors.rs` | 49 | — |
| `client/validation.rs` | 63 | — |
| `context_prefixer.rs` | 443 | — |
| `embedding.rs` | 317 | — |
| `importance.rs` | 627 | — |
| `model_info.rs` | 325 | — |

**`client/config.rs`**

- `const DEFAULT_BASE_URL: &str = "http://localhost:11434"` — Standard Ollama base URL
- `const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text"` — Standard embedding model for SME usage (multilingual support for German)
- `const MAX_RETRIES: u32 = 3` — Default maximum retry attempts for transient errors
- `const MAX_BATCH_SIZE: usize = 512` — Maximum allowed batch size for batch operations to prevent request timeouts and memory exhaustion.
- `const MAX_TEXT_BYTES: usize = 10_000_000` — Maximum allowed text length in bytes (10 MB) to prevent OOM or DoS
- `struct OllamaConfig` { base_url, model, request_timeout, connect_timeout, max_retries } — Configuration options for the Ollama HTTP client.

**`client/core.rs`**

- `struct OllamaClient` — HTTP client for interacting with a local Ollama instance. · Methoden: new, with_config, is_available, check_availability, with_defaults, base_url, config, embed_batch, try_embed_batch, list_models, is_model_available, generate_text, generate_text_stream, try_generate_text… · impl: LlmTextGenerator, LlmTextGeneratorStreaming, SegmentSynthesizer

**`client/errors.rs`**

- `fn is_transient_network_error(err: &reqwest::Error) -> bool` — Helper to check if a reqwest error is a transient network error (timeout or connection error).
- `fn classify_reqwest_error(e: reqwest::Error, base_url: &str, context: &str) -> ContextraError` — Classifies a `reqwest::Error` into structured `ContextraError` variants (`Io` for connection refused or timeouts, `Storage` otherwise).
- `fn is_transient_error(e: &ContextraError) -> bool` — Helper to classify transient network errors for retry.

**`client/validation.rs`**

- `fn xml_escape(input: &str) -> String` — Escapes XML special characters in string inputs to prevent tag injection.
- `fn build_rag_prompt(system_context: &str, rag_context: &str, user_query: &str) -> String` — Halluzinationsprävention bei lokalen 7B-Modellen ohne Guard:
- `fn validate_text_length(text: &str, field_name: &str) -> Result<()>` — Validates that input text length does not exceed `MAX_TEXT_BYTES`.
- `fn validate_batch_size(count: usize) -> Result<()>` — Validates that batch size does not exceed `MAX_BATCH_SIZE`.
- `fn validate_model_name(name: &str) -> Result<()>` — Validates that model name does not contain invalid path traversal or whitespace control characters.

**`context_prefixer.rs`**

- `struct ContextPrefixConfig` { model, max_document_chars, max_prefix_tokens } — Konfiguration für Kontext-Präfix-Generierung.
- `struct ContextPrefixEngine` — Generiert LLM-basierte Kontext-Präfixe für Dokument-Chunks. · Methoden: new, generate_prefix, generate_prefix_batch
- `type ContextPrefixer = ContextPrefixEngine` — Compatibility alias for ContextPrefixEngine.
- `fn truncate_chars(s: &str, max_chars: usize) -> String`
- `fn truncate_prefix(s: &str, max_tokens: usize, max_chars: usize) -> String` — Truncates context prefix text while preserving word boundaries up to `max_tokens` (words) and hard character limit `max_chars`.

**`embedding.rs`**

- `struct OllamaEmbedder` — Implementation of `TextEmbeddingEngine` using Ollama's HTTP API. · Methoden: new, with_config, with_defaults, config, with_concurrency, with_expected_dimension, model · impl: EmbeddingProvider

**`importance.rs`**

- `fn score_importance_batch( client: &Arc<OllamaClient>, chunks: &[&str], calibrator: Option<&Arc<Mutex<IsotonicCalibrator>>>, max_concurrent: usize, ) -> Vec<ImportanceAss` — Evaluates importance for multiple text chunks in parallel using tokio::spawn.
- `enum Confidence` ∈ {Parsed, Unparseable} — Confidence or parse status of an LLM importance rating.
- `struct ImportanceAssessment` { score, confidence, model_id, calibrated_confidence } — Result of an LLM importance evaluation containing the score, parse confidence status, model provenance, and optional calibrated confidence. · Methoden: new, with_provenance, value, is_parsed
- `fn score_importance( client: &OllamaClient, chunk_text: &str, ) -> Result<ImportanceAssessment>`
- `fn score_importance_with_calibrator( client: &OllamaClient, chunk_text: &str, calibrator: Option<&Arc<Mutex<IsotonicCalibrator>>>, ) -> Result<ImportanceAssessment>` — Evaluates importance of a text chunk with an optional shared `IsotonicCalibrator`.
- `fn record_importance_outcome( calibrator: &Arc<Mutex<IsotonicCalibrator>>, raw_score: f32, was_useful: bool, )` — Records outcome feedback for an importance score into an `IsotonicCalibrator`.
- `fn parse_importance_score_response(raw_response: &str) -> ImportanceAssessment` — Parses an `ImportanceAssessment` from raw LLM output.

**`model_info.rs`**

- `struct ModelInfo` { modelfile, parameter_size, quantization_level }
- `fn known_dimension(model: &str) -> Option<usize>` — Gibt die Embedding-Dimension für bekannte Modelle zurück.

### K.14 `contextra-infer-onnx` — 1.497 Zeilen (ohne Tests), 5 Dateien [V-Sig]

**Features:** default = `—`; weitere: `reranking`, `onnx`, `candle-backend`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-rank`, `contextra-infer-candle`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `lib.rs` | 857 | — |
| `reranker/config.rs` | 59 | — |
| `reranker/cross_encoder.rs` | 205 | — |

**`lib.rs`**

- `fn default_model_cache_dir(model_name: &str) -> PathBuf` — Returns the default local cache directory for ONNX embedding models (`~/.contextra/models/<model_name>`).
- `fn ensure_onnx_model_download( model_name: &str, cache_dir: Option<&Path>, ) -> Result<PathBuf>` *[feature: onnx]* — Ensures the ONNX embedding model files (`model.onnx` and `tokenizer.json`) exist locally in `cache_dir` (or `~/.contextra/models/<model_name>`).
- `fn ensure_onnx_model_download( _model_name: &str, _cache_dir: Option<&Path>, ) -> contextra_types::Result<PathBuf>`
- `fn create_candle_embedder( client: CandleEmbedClient, ) -> Box<dyn contextra_ports::EmbeddingProvider>` *[feature: candle-backend]* — Creates a trait object `Box<dyn EmbeddingProvider>` wrapping a `CandleEmbedClient`.
- `const MAX_EMBED_BATCH_SIZE: usize = 512` — Conservative default.
- `struct TextEmbedderConfig` { max_sequence_length, pool_size, max_concurrent_embeddings, expected_dim, max_batch_size } *[feature: onnx]* — Configuration settings for the text embedder. · Methoden: validate
- `type OnnxEmbedder = TextEmbedder` *[feature: onnx]*
- `struct TextEmbedder` *[feature: onnx]* — Handles text tokenization and ONNX model inference. · Methoden: from_path, new, load, load_with_config, session_load_count, with_expected_dimension, embed_async · impl: EmbeddingProvider

**`reranker/config.rs`**

- `const MAX_CANDIDATES: usize = 10_000` — Maximale Anzahl von Kandidaten pro Reranking-Aufruf zur Vermeidung unbegrenzter Allokationen.
- `struct RerankConfig` { model_path, tokenizer_path, max_length, batch_size, calibration, calibration_warmup, rerank_deadline_ms, simulate_delay_ms } — Konfiguration für Cross-Encoder Reranking. · Methoden: with_calibration

**`reranker/cross_encoder.rs`**

- `struct CrossEncoderReranker` — Unified public `CrossEncoderReranker` struct independent of active feature flags. · Methoden: record_implicit_feedback, is_calibrated, calibration_observation_count, record_outcome, calibrate, invalidate_calibration, fitted_calibration, passthrough, passthrough_with_config, config, with_calibration, new, rerank · impl: Reranker

### K.15 `contextra-kvcache` — 4.269 Zeilen (ohne Tests), 8 Dateien [V-Sig]

Ring 1 Prefix-Radix tree, KV-Block cache, tenant-isolated memory store and tiering for Contextra

**Features:** default = `—`; weitere: `content-addressed-kv-cache`, `kv-encryption`, `kvcache-attention-eviction`, `kvcache-kivi-quant`, `kivi-quantization`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-crypto`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `attention_score.rs` | 276 | Attention-aware Eviction Module (SnapKV/H2O Foundation). |
| `eviction_worker.rs` | 311 | — |
| `lib.rs` | 37 | Contextra KV-Cache Crate (Ring 1). |
| `prefix_store.rs` | 341 | — |
| `quantize_kivi.rs` | 789 | KIVI 2-Bit Quantization Metadata, Asymmetric Quantization, and Unpacking Infrastructure. |
| `radix.rs` | 849 | — |
| `segment.rs` | 546 | — |
| `store.rs` | 1120 | — |

**`attention_score.rs`**

- `trait AttentionScoreSource` (importance_score, importance_score_for_tenant, register_segment, unregister_segment) — Dyn-compatible source for segment attention importance scores.
- `struct NullAttentionScoreSource` — Default implementation that provides no attention scores (pure LRU behavior). · impl: AttentionScoreSource
- `fn rank_for_eviction( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, ) -> Vec<u64>` — Ranks eviction candidates using a default balanced weighting (50% LRU age, 50% attention score).
- `fn rank_for_eviction_weighted( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>` — Ranks eviction candidates combining normalized LRU access age and attention scores.
- `fn rank_for_eviction_weighted_for_tenant( tenant_id: TenantId, candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>` — Ranks eviction candidates for a specific tenant combining normalized LRU access age and attention scores.

**`eviction_worker.rs`**

- `struct EvictionWorker` — Regelmäßiger Eviction-Pfad (VRAM > 80%-Trigger). · Methoden: spawn, with_attention_exporter, register_segment_request, unregister_segment_request, attention_source, trigger_eviction, notify_block_released, shutdown
- `fn emergency_wipe(store: &TenantIsolatedKvStore)` — SEPARATER Pfad für Notfall-Löschung (z.B Prozess-Shutdown, expliziter Sicherheits-Trigger).

**`prefix_store.rs`**

- `const DEFAULT_BYTE_BUDGET_PER_TENANT: usize = 256 * 1024 * 1024` — Standard-Byte-Budget pro Tenant (256 MiB).
- `struct TenantPrefixKvStore` — Mandantenisolierter KV-Prefix-Store. · Methoden: new, with_byte_budget_per_tenant, with_reuse_policy, lookup, insert, evict · impl: KvPrefixStore

**`quantize_kivi.rs`**

- `struct KiviQuantizeConfig` { key_group_size, quantize_values } — Configuration for KIVI quantization parameters.
- `struct KvTensorView` { keys, values, num_tokens, num_channels } — A view over unquantized raw key and value tensors. · Methoden: new, to_bytes, from_bytes
- `struct KiviBlockMeta` { bits_per_value, scale, zero_point, original_len, num_tokens, num_channels, key_group_size, quantize_values, key_scales, key_zero_points… } — Metadata required to dequantize a 2-bit KIVI block.
- `struct KiviQuantizedBlock` { meta, packed } — A 2-bit quantized byte block with accompanying metadata.
- `fn kivi_quantize(raw: &KvTensorView, config: KiviQuantizeConfig) -> Result<KiviQuantizedBlock>` — Quantizes raw `KvTensorView` into 2-bit `KiviQuantizedBlock`.
- `fn kivi_dequantize(bytes: &[u8], meta: &KiviBlockMeta) -> Result<KvTensorView>` — Dequantizes packed 2-bit bytes back into `KvTensorView`.
- `fn unpack_kivi_block(block: &KiviQuantizedBlock) -> Result<Vec<f32>>` — Dequantizes a 2-bit packed block back into floating point `Vec<f32>`.
- `fn pack_kivi_block(values: &[f32]) -> KiviQuantizedBlock` — Reference implementation for 2-bit linear min/max quantization.
- `fn compress_bytes(input: &[u8]) -> Vec<u8>` — Losslessly compresses byte slice for LSM spill (Quantization -> Compression -> AEAD).
- `fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>>` — Losslessly decompresses byte slice produced by `compress_bytes`.

**`radix.rs`**

- `enum KvReusePolicy` ∈ {Always, CostBased, Never} — Policy zur Steuerung von Prefix-Reuse (§9.2). · Methoden: should_reuse
- `struct KvBlockGuard` { block_id, tenant_id } — RAII-Guard für ausgeliehene/gematchte KV-Blöcke. · Methoden: new, block_id, tenant_id, active_refs
- `struct PrefixMatch` { matched_tokens, matched_len, block_id } — Ergebnis eines Präfix-Matches im Radix-Baum.
- `struct PrefixRadixTree` — Komprimierter Prefix-Radix-Baum für Token-Sequenzen. · Methoden: new, tenant_id, insert, find_longest_prefix, remove, len, is_empty, clear
- `struct KvSegmentRef` { segment_id, tenant_id, matched_tokens } *[feature: content-addressed-kv-cache]* — Referenz auf ein gecachtes KV-Segment. · Methoden: new, matched_len
- `enum KvLookupResult` ∈ {ExactPrefixHit, ContentHashHit, SemanticSimilarityHit, Miss} *[feature: content-addressed-kv-cache]* — Ergebnis eines KV-Cache-Lookups mit `ContentAddressedKvStore`.
- `trait SemanticEmbedder` (embed) *[feature: content-addressed-kv-cache]* — Trait für semantische Embedding-Generierung auf KV-Cache-Keys.
- `struct SemanticCacheConfig` { embedder, similarity_threshold } *[feature: content-addressed-kv-cache]* — Konfiguration für opt-in semantische Vektor-Cache-Lookups. · Methoden: new
- `struct ContentAddressedKvStore` { content_index, position_index } *[feature: content-addressed-kv-cache]* — Content-adressierter KV-Store mit Zwei-Ebenen-Lookup (Position, Content-Hash, Semantic-Search). · Methoden: new, with_reuse_policy, with_semantic_config, position_index, hash_tokens, insert, lookup, remove, len, is_empty

**`segment.rs`**

- `enum KvSegmentContent` ∈ {Raw, KiviQuantized} — Content representation state for a KV segment.
- `const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1` — Aktuelle Version der KV-Segment-Schlüsselableitung.
- `struct ShreddableSegmentKey` — Shred-fähiger Schlüssel für Tier-2 Segmentdateien (Crypto-Shredding). · Methoden: try_new, try_new_random, encrypt, decrypt, shred, is_shredded
- `struct Tier2EncryptedSegment` { tenant_id, segment_id, ciphertext, nonce, key } — Verschlüsselte Tier-2 Segmentdatei (ausgelagerter KV-Block auf Disk). · Methoden: new, read_and_decrypt
- `struct EncryptedSegmentPayload` { layer } *[feature: kv-encryption]* — Encrypted layer representation stored inside a KvSegment when encryption is active.
- `struct KvSegment` { tenant_id, segment_id, key_derivation_version, encrypted, model_fingerprint, rope_offset, active_refs, content } — Ein KV-Cache-Segment. · Methoden: new, new_with_metadata, write_quantized, read_dequantized, new_encrypted, active_refs, acquire_guard, decrypt_data, to_spill_bytes, as_bytes, len, is_empty

**`store.rs`**

- `type SpillHandler = Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>` — Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache.
- `const MAX_PINNED_BYTES_PER_TENANT: u64 = 64 << 20` — Maximum allowed total pinned bytes per tenant for un-expiring pinned segments (`Pin { ttl:
- `struct TenantIsolatedKvStore` — Tenant-isolierter KV-Segment-Store. · Methoden: new, with_shard_count, set_attention_source, attention_source, with_capacity, set_spill_handler, insert_segment_with_directive, insert_segment, release_step, insert_token_sequence, find_prefix_match, acquire_block_guard, on_rollback, remove_segments_for_rollback…

### K.16 `contextra-license` — 153 Zeilen (ohne Tests), 2 Dateien [V-Sig]

License and activation enforcement gate layer for Contextra feature rings

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `lib.rs` | 11 | `contextra-license` License and activation gate module for Contextra feature rings (§14.6, §15). |
| `signed_gate.rs` | 142 | Cryptographic signed license gate implementation (§14.6, §15). |

**`signed_gate.rs`**

- `struct LicensePayload` { tenant_id, allowed_rings, expires_at, feature_flags } — License payload containing tenant binding, granted feature rings, optional expiration timestamp, and additional feature flags.
- `struct SignedLicenseGate` — Cryptographic license gate verifying Ed25519 signatures over serialized [`LicensePayload`]. · Methoden: from_signed_payload, from_signed_payload_with_clock, license_payload, verifying_key, create_test_signed_payload · impl: LicenseGate

### K.17 `contextra-mcp` — 4.842 Zeilen (ohne Tests), 21 Dateien [V-Sig]

**Features:** default = `—`; weitere: `agent-workflows`, `onnx`, `candle`, `ollama`, `kv-bridge`, `test-utils`

**Workspace-Abhängigkeiten:** `contextra`, `contextra-types`, `contextra-ports`, `contextra-license`, `contextra-wire`, `contextra-rank`, `contextra-adapt`, `contextra-crypto`, `contextra-privacy`, `contextra-infer-onnx`, `contextra-infer-ollama`, `contextra-agent`, `contextra-infer-candle`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `config.rs` | 522 | — |
| `egress_guard.rs` | 135 | — |
| `explain.rs` | 268 | — |
| `io.rs` | 83 | — |
| `lib.rs` | 58 | Layer-7-Rand-Crate ohne jegliche unsafe-Toleranz — verarbeitet direkt untrusted stdio-Input, siehe ADR-010. |
| `plugin_status.rs` | 44 | — |
| `prompt_injection/audit.rs` | 68 | — |
| `prompt_injection/guard.rs` | 613 | — |
| `prompt_injection/policy.rs` | 62 | — |
| `protocol.rs` | 88 | — |
| `routing.rs` | 138 | — |
| `sandbox.rs` | 776 | — |
| `server.rs` | 179 | — |
| `validation.rs` | 30 | — |

**`config.rs`**

- `struct EmbeddingConfig` { provider, ollama_url, embed_model, onnx_model_path, candle_model_dir } — Embedding provider configuration settings. · Methoden: from_env, build_provider
- `fn create_embedding_provider( provider_type: &str, ollama_url: &str, embed_model: &str, onnx_model_path: Option<&Path>, candle_model_dir: Option<&Path>, ) -> Result<Arc<d` — Dynamically constructs an `EmbeddingProvider` implementation based on provider identifier.
- `struct LlmConfig` { provider, ollama_url, llm_model, candle_model_dir } — LLM provider configuration settings. · Methoden: from_env, build_generator
- `fn create_llm_text_generator( provider_type: &str, ollama_url: &str, llm_model: &str, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn LlmTextGenerator>, ContextraErr` — Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier.
- `struct RouterConfig` { profiles, profiles_path, calibration_store_path } — Router configuration settings. · Methoden: from_env
- `struct MockLlmGenerator` — Fallback Mock LLM Text Generator. · impl: LlmTextGenerator

**`egress_guard.rs`**

- `const DEFAULT_EGRESS_GUARD_TIMEOUT: Duration = Duration::from_millis(200)` — Standard-Timeout für EgressGuard Vector-Search (200 ms).
- `const DEFAULT_EGRESS_GUARD_THRESHOLD: f32 = 0.85` — Standard-Schwellwert für HNSW Cosine Similarity Match (0.85).
- `const DEFAULT_EGRESS_GUARD_MIN_BYTES: usize = 128` — Minimum Byte-Länge des Payloads, ab der die HNSW-Prüfung durchgeführt wird (128 Bytes).
- `struct EgressGuard` — Layer-4 EgressGuard zur Erkennung und Blockierung von Bulk-Exfiltrationen. · Methoden: new, with_timeout, threshold, min_bytes, timeout, check

**`explain.rs`**

- `struct ExplainRequest` { id, collection } — Request payload for `contextra_explain`.
- `struct ExplainResponse` { id, collection, provenance, explanation, content_provenance } — Response payload for `contextra_explain`.

**`io.rs`**

- `const MAX_RPC_BYTES: usize = 4 * 1024 * 1024` — Maximum allowed single JSON-RPC message size via stdio (4 MB).
- `const MAX_SEARCH_QUERY_BYTES: usize = 64 * 1024` — Maximum allowed search query length in bytes (64 KB).
- `fn read_line_bounded( reader: &mut R, buf: &mut String, max_bytes: usize, ) -> std::io::Result<usize>` — Reads a single line from an async reader into `buf` up to `max_bytes`.

**`plugin_status.rs`**

- `struct PluginStatusResponse` { plugins, feature_ring_active } — Response payload for the `contextra_plugin_status` MCP tool.
- `struct PluginStatusEntry` { name, version, ring, feature_ring_required } — Status entry describing an active plugin.
- `fn handle_plugin_status( registry: &contextra_ports::plugin::PluginRegistry, ) -> Result<PluginStatusResponse, McpError>` — Handler for the `contextra_plugin_status` MCP tool call.

**`prompt_injection/audit.rs`**

- `struct SecurityAuditRecord` { timestamp, event_type, doc_id, collection, pattern_matched, action_taken } — Struktur für Sicherheits-Audit-Logeinträge bei Auslösung des `Escalate`-Modus.
- `struct SecurityAuditLogger` — Separater Audit-Logger für Sicherheitsvorfälle (vollständig isoliert vom Vektor-Index). · Methoden: new, log_event, get_recorded_events

**`prompt_injection/guard.rs`**

- `struct NormalizedPattern` { original, collapsed, no_ws } — Vornormalisiertes Injection-Pattern zur Performance-Optimierung.
- `struct PromptInjectionGuard` { redaction_placeholder } — Signatur- und phrasenbasierter Prompt-Injection-Erkennungsfilter. · Methoden: new, default_patterns, policy, patterns, audit_logger, load_from_file, from_env, is_zero_width, skeletonize_char, normalize_text, collapse_whitespace, strip_whitespace, decode_base64, extract_base64_candidates… · impl: InjectionDetector

**`prompt_injection/policy.rs`**

- `const DEFAULT_REDACTION_PLACEHOLDER: &str = "[REDACTED: potenzielle Prompt-Injection erkannt, Originaltext zur Sic` — Neutraler Standard-Platzhalter für als verdächtig erkannten Text im Strict/Escalate-Modus.
- `enum QuarantinePolicy` ∈ {Strict, FlagOnly, Escalate} — Konfigurierbare Quarantäne-Policy für die Prompt-Injection-Behandlung. · Methoden: from_env
- `struct PromptInjectionConfig` { policy, redaction_placeholder, audit_log_path, custom_patterns } — Konfiguration zur Initialisierung aus einer externen Konfigurationsdatei.
- `fn default_redaction_placeholder() -> String`

**`protocol.rs`**

- `enum McpError` ∈ {ParseError, InvalidRequest, MethodNotFound, InvalidParams, InternalError} — MCP Error representation matching standard JSON-RPC 2.0 error codes. · Methoden: parse_error, invalid_request, method_not_found, invalid_params, internal_error, code
- `fn response_from_error(id: Option<Value>, err: McpError) -> JsonRpcResponse` — Helper function to convert an `McpError` directly into a `JsonRpcResponse`.

**`routing.rs`**

- `struct RoutingHandle` { router, calibrator, pid_controller, _drift_adapter } — Holds strong Arc references to routing and calibration components to maintain live Weak references in `Contextra`.
- `fn setup_routing( db: &Arc<Contextra>, config: &RouterConfig, ) -> Result<Option<Arc<RoutingHandle>>, ContextraError>`
- `fn setup_kv_bridge( _db: &Arc<Contextra>, ) -> Option<Arc<contextra_infer_candle::KvBridgeAdapter>>` *[feature: kv-bridge]* — Conditionally sets up `KvBridgeAdapter` when feature `kv-bridge` is enabled.

**`sandbox.rs`**

- `const MAX_VOLATILE_RESULTS: usize = 1_000` — Maximale Anzahl von volatilen Ergebnissen pro Sandbox-Session.
- `const MAX_VOLATILE_KEY_BYTES: usize = 256` — Maximale Länge eines Volatile-Result-Schlüssels in Bytes.
- `const MAX_VOLATILE_OUTPUT_BYTES: usize = 16 * 1024 * 1024` — Maximale Größe einer volatilen Tool-Ausgabe in Bytes (16 MB).
- `struct ToolDefinition` { name, category, description, input_schema } — Definition eines MCP-Tools mit Name, Kategorie, Beschreibung und JSON-Schema.
- `const TOOL_REGISTRY: &[ToolDefinition] = &[ ToolDefinition { name: "contextra_search", category: ToolCategory::` — Single Source of Truth für MCP-Tool-Registrierung, Discovery (`tools/list`) und -Klassifizierung.
- `enum ToolCategory` ∈ {DatabaseRead, DatabaseWrite, CodeExecution, CloudEgress} — Erlaubte MCP-Tool-Kategorien (Whitelist-Prinzip).
- `struct SandboxPolicy` { allow_db_reads, allow_db_writes, allow_code_execution, allow_cloud_egress, max_execution_ms } — Konfiguration der Sandbox-Policy.
- `struct VolatileToolResult` — Volatiler Tool-Output-Speicher mit Zeroize-Garantie bei Drop. · Methoden: encrypt, decrypt
- `struct McpSandbox` — MCP Sandbox: · Methoden: new, policy, validate_tool_call, classify_method, execute_with_timeout, store_volatile, get_volatile
- `hex::fn encode(bytes: [u8; 32]) -> String`

**`server.rs`**

- `struct McpServer` { db, embedder, sandbox, injection_guard, egress_classifier, routing, plugin_registry, kv_bridge } · Methoden: new, with_write_permission, with_sandbox, with_kv_bridge, with_injection_guard, with_egress_classifier, with_routing, with_plugin_registry, run_stdio, handle_value, handle

**`validation.rs`**

- `fn is_write_allowed_by_env() -> bool` — Checks whether database write access is explicitly enabled via environment variable `CONTEXTRA_MCP_ALLOW_WRITE`.
- `fn validate_collection_name(name: &str) -> Result<(), McpError>` — MCP-Server mit stdio-Transport (JSON-RPC 2.0).

### K.18 `contextra-mvcc` — 2.891 Zeilen (ohne Tests), 5 Dateien [V-Sig]

Multi-Version Concurrency Control (MVCC), sequence log, and transaction buffer for Contextra

**Workspace-Abhängigkeiten:** `contextra-types`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `lib.rs` | 26 | `Contextra` MVCC — Multi-Version Concurrency Control, Sequence Log, Transaction Staging, and SSI Validation. |
| `seq_log.rs` | 607 | Shared versioned sequence log for snapshot-isolated index searches (`_at` family). |
| `snapshot.rs` | 392 | SnapshotRegistry for MVCC-safe reads. |
| `ssi.rs` | 569 | Serializable Snapshot Isolation (SSI) read-set tracking and conflict validation. |
| `tx_buffer.rs` | 1297 | Transactional buffer for staging index operations. |

**`seq_log.rs`**

- `const DEFAULT_MAX_PIN_DURATION: Duration = Duration::from_secs(300)` — Default maximum pin duration (5 minutes) before diagnostic warnings are issued for expired pins.
- `struct SeqLogEntry` { doc_id, insert_seq, delete_seq } — Versioned sequence log entry for snapshot isolation (`_at` family). · Methoden: is_visible
- `enum SeqLogChange` ∈ {Insert, Delete} — Represents a historical sequence log change (insert or delete) for delta replaying. · Methoden: seq, doc_id
- `struct SequenceLog` — Helper structure managing sequence log tracking and visibility filtering for index implementations. · Methoden: new, with_max_pin_duration, set_max_pin_duration, max_pin_duration, pin_snapshot, pin_snapshot_at, unpin_snapshot, expired_pins_at, expired_pins, min_retention_seq_at, min_retention_seq, deletion_seq, record_insert, record_delete…

**`snapshot.rs`**

- `struct SnapshotRegistry` — Registry for active read snapshots. · Methoden: new, register, min_active_seqno, pin, unpin
- `struct SnapshotGuard` — RAII Guard for an active snapshot. · Methoden: seq_no

**`ssi.rs`**

- `const DEFAULT_MAX_TRACKED_COMMIT_KEYS: usize = 1_000_000` — Default maximum tracked committed write keys in [`SequenceLogSsiValidator`] (1,000,000 keys).
- `struct ReadSet` — Tracked read keys, range prefixes, and their snapshot sequence numbers for Serializable Snapshot Isolation (SSI). · Methoden: new, record_read, record_prefix, register_read, get, is_empty, len, clear, min_snapshot_seq, iter, prefixes_iter, keys_map, prefixes_map
- `trait SsiValidator` (validate) — Trait for Serializable Snapshot Isolation (SSI) validation.
- `struct SequenceLogSsiValidator` — Reference implementation of [`SsiValidator`] backed by MVCC [`SequenceLog`] and committed key tracking. · Methoden: new, new_with_bounds, with_sequence_log, prune_through, forget_from, pruned_through, tracked_commit_keys, max_tracked_keys, record_commit_key, record_commit_keys, validate_and_record · impl: SsiValidator

**`tx_buffer.rs`**

- `const STAGING_ENTRY_OVERHEAD_BYTES: usize = 32` — Single canonical constant for overhead calculation per staged entry (32 bytes).
- `const DEFAULT_SHARD_COUNT: usize = 64` — Default number of shards for the transaction buffer.
- `const DEFAULT_MAX_OPS_PER_TX: usize = 10_000`
- `const DEFAULT_MAX_READ_SET_KEYS: usize = 100_000` — Default maximum number of read keys tracked per transaction's [`ReadSet`].
- `const DEFAULT_MAX_TX_STAGED_BYTES: usize = 16 * 1024 * 1024` — Default maximum staged bytes per transaction (16 MiB).
- `const DEFAULT_MAX_TOTAL_STAGED_BYTES: usize = 256 * 1024 * 1024` — Default maximum total staged bytes across all active transactions (256 MiB).
- `struct TxBufferConfig` { tx_timeout, max_active_tx, max_ops_per_tx, max_tx_staged_bytes, max_total_staged_bytes } — Configuration options for `TxBuffer`.
- `enum IndexOp` ∈ {Insert, Delete} — Operation to be executed in an index. · Methoden: doc_id, estimated_bytes
- `trait StagedOpSize` (staged_bytes) — Trait for estimating the byte footprint of a staged payload.
- `struct TxBuffer` — Buffers index operations until commit or rollback. · Methoden: new, new_with_config, new_with_config_ext, staged_bytes, has_tx, begin_at, begin, register_read, try_register_read, record_read, min_read_snapshot, get_read_set, read_set, stage…

### K.19 `contextra-ports` — 3.675 Zeilen (ohne Tests), 20 Dateien [V-Sig]

Canonical dyn-compatible port traits for Contextra subsystems

**Workspace-Abhängigkeiten:** `contextra-types`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `attention.rs` | 17 | Attention exporter port definitions for KV-cache eviction integration. |
| `checkpoint.rs` | 58 | Checkpoint and Snapshot subsystem trait definitions. |
| `clock.rs` | 93 | Clock port trait and system time implementation. |
| `embedding.rs` | 322 | — |
| `graph.rs` | 215 | Graph ports and collection mutation trait definitions (Anhang B §B.5.1.3). |
| `graph_index.rs` | 485 | Graph index trait definition and statistics. |
| `id_gen.rs` | 97 | Unique ID generator port trait and atomic sequential implementation. |
| `kv.rs` | 251 | KV prefix store port trait definitions and types (Spec §9.2). |
| `kv_bridge_port.rs` | 25 | — |
| `lib.rs` | 139 | Core trait definitions for Contextra subsystems. |
| `license.rs` | 112 | License and activation gate port trait definitions (§14.6, §15). |
| `lifecycle.rs` | 125 | Memory lifecycle management, grounding validator traits, and distance calculator contracts. |
| `metrics.rs` | 70 | Metrics reporting port trait definitions. |
| `observability.rs` | 18 | Observability traits and re-exports for memory lifecycle and grounding validation. |
| `plugin.rs` | 435 | Plugin registry and dependency resolution (§12, INV-PLUGIN-DEPENDENCY). |
| `reranker.rs` | 23 | Reranker port trait for cross-encoder post-retrieval ranking. |
| `rng.rs` | 143 | Random number generator port trait and deterministic SplitMix64 implementation. |
| `storage.rs` | 518 | Storage engine subsystem trait definitions and statistics. |
| `text_index.rs` | 215 | Text index and segment synthesizer traits. |
| `vector_index.rs` | 314 | Vector index trait definition and statistics. |

**`attention.rs`**

- `struct RequestId` — Unique identifier for an inference request within a process.
- `trait AttentionExporter` (export_attention_weights) — Port implemented by inference backends (Ring 2) to export attention weights from the latest prefill step to the KV-cache eviction worker (Ring 1).

**`checkpoint.rs`**

- `trait Checkpoint` (take_snapshot, restore) — Abstract contract for generating consistent checkpoints.
- `trait CheckpointCoordinator` (create_named_checkpoint, restore_named_checkpoint, drop_named_checkpoint, list_named_checkpoints) — Unified Checkpoint Coordinator Trait combining named, TxId+seq_no-scoped, persistent checkpoints.
- `trait Snapshot` (seq_no) — Represents a point-in-time view of the database.

**`clock.rs`**

- `trait Clock` (now_unix_nanos, monotonic_nanos) — Port trait for time operations, enabling deterministic testing by decoupling system wall-clock and monotonic time sources.
- `struct SystemClock` — Standard production implementation of [`Clock`] backed by [`SystemTime`] and [`Instant`]. · Methoden: new · impl: Clock

**`embedding.rs`**

- `struct ContextSegment` { chunk_id, text, model_fingerprint, rope_offset } — Representation of a context segment passed to context-aware text generation. · Methoden: new, with_fingerprint, with_rope_offset
- `trait TextEmbeddingEngine` (embed, embed_batch) — Text embedding engine trait.
- `enum EmbeddingError` ∈ {Unavailable, ComputationFailed, InputTooLong} — Error types encountered during embedding operations.
- `trait EmbeddingProvider` (provider_name, embed, embedding_dim, embed_batch) — Abstraction for all embedding providers (OllamaEmbedder, OnnxEmbedder, MockEmbedder).
- `trait TextGenerator` (generate_text) — Abstraction for text generation / LLM engines.
- `trait LlmTextGenerator` (generate, generate_with_context) — Abstract contract for LLM text generation (summarization, importance evaluation, query expansion).
- `trait LlmTextGeneratorStreaming` (generate_stream) — Abstract contract for streaming LLM text generation (token by token / chunk by chunk).
- `struct MockEmbedder` { dim, fixed_output } — Deterministic mock embedding provider for tests. · Methoden: new, with_fixed_output · impl: EmbeddingProvider

**`graph.rs`**

- `struct RoleId` · Methoden: new, inner
- `struct RoleBinding` { role, entity } — Binding of an entity to a specific role within an n-ary hyperedge. · Methoden: new
- `struct HyperEdgeId` · Methoden: new, inner
- `enum GraphMutationError` ∈ {LockAcquisitionTimeout, RoleBindingInvalid, EpochReclamationPending, InsufficientParticipants, HyperedgeNotFound, InvalidWeight, Internal} — Error variants for graph mutation operations.
- `trait GraphCollectionMutation` (relate_n_ary) — Trait defining n-ary hyperedge collection mutations for graph indexing (Anhang B §B.5.1.3).

**`graph_index.rs`**

- `trait GraphIndex` (traverse, neighbors, remove_edge, add_bidirectional, multi_traverse, multi_traverse_at, traverse_at, personalized_page_rank_at, traverse_at_time, traverse_at_bitemporal, personalized_page_rank, add_entity, add_edge, commit…) — Graph-Index Trait — CSR-basierte Entity-Relation-Traversal.
- `struct GraphIndexStats` { num_entities, num_edges, memory_usage_bytes } — Statistics for the GraphIndex layer.
- `trait CommunityResolver` (get_community) — Contract for resolving graph community assignments for entities.

**`id_gen.rs`**

- `trait IdGen` (next_id) — Port trait for unique ID generation.
- `struct SequentialIdGen` — A thread-safe ID generator that produces monotonically increasing 64-bit IDs. · Methoden: new · impl: IdGen

**`kv.rs`**

- `struct KvLayout` { n_layer, n_kv_head, head_dim, dtype } — KV cache layer and dimensions layout configuration.
- `struct RopeConfig` { base, scaling } — Rotary position embedding (RoPE) configuration.
- `struct PrefixKey` { model, tokenizer_hash, layout, rope } — Key identifying a model KV cache prefix structure.
- `struct KvBlock` { block_id, data } — Represents an exported KV block segment.
- `struct KvPrefixHit` { matched_tokens, blocks } — Represents a prefix match hit in the KV store.
- `trait KvPrefixStore` (lookup, insert, evict) — Trait defining the contract for KV prefix caching and block reuse (Spec §9.2).

**`kv_bridge_port.rs`**

- `trait KvBridgeStorage` (get, put, commit) — Abstrahiertes Persistence-Interface für KV-Bridge Tier-2 Spill-over operations.

**`lib.rs`**

- `type BoxFuture = Pin<Box<dyn Future<Output = T> + Send + 'a>>` — Type alias for a pinned, heap-allocated `Future` that is `Send` and dyn-compatible.
- `type BoxStream = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>` — Type alias for a pinned, heap-allocated `Stream` that is `Send` and dyn-compatible.

**`license.rs`**

- `enum VectorDeleteMode` ∈ {SynchronousRepair, BackgroundRepair} — Vector deletion mode for vector index maintenance (§15).
- `enum FeatureRing` ∈ {Fast, Sovereign, Compliance} — Feature-Ring gemäß Spec §15.
- `struct AuthorizedRing` — Token representing an authorized feature ring granting access to engine features (§15). · Methoden: ring
- `enum LicenseError` ∈ {NotActivated, InvalidSignature, Expired} — Fehlerklasse für Lizenz-/Aktivierungsprüfung — crate-lokal, an der Grenze in `ContextraError` konvertierbar (P6-konform, siehe Spec §17).
- `trait LicenseGate` (check_ring, authorize) — Dyn-kompatibler Port (P27) für Lizenzprüfung — Implementierung (Lizenzserver-Anbindung, Ed25519-Signaturprüfung analog zum Löschbeweis-Pfad §14.1) ist NICHT Teil dieses Tickets und folgt in einem eigenen, separat zu beau
- `struct OpenFastGate` — Default open activation gate allowing `Fast` ring features without restriction. · impl: LicenseGate

**`lifecycle.rs`**

- `trait DistanceCalculator` (compute_f32, compute_u8) — Distance calculator trait for vector comparison.
- `struct LifecycleSweepReport` { swept_count, deleted_by_ttl, deleted_by_decay, skipped_pinned } — Report summarizing statistics of a memory lifecycle sweep operation.
- `enum ConsolidationAction` ∈ {Keep, Merge, Supersede, Drop} — Actions planned during memory consolidation.
- `trait MemoryLifecycleManager` (sweep, plan_consolidation) — Trait controlling active Memory Lifecycle management:
- `struct GroundingAssessment` { score, is_grounded, reason } — Result of a post-hoc grounding / attribution validation check.
- `trait GroundingValidator` (validate_grounding) — Abstract contract for post-hoc hallucination / grounding validation.
- `trait ResponseGroundingValidator` (score_grounding) — Abstract contract for grounding validation of LLM-generated responses against raw source strings.
- `trait ContextPreparer` (prepare_context) — Contract for trimming and preparing context windows tailored to token budgets.

**`metrics.rs`**

- `trait MetricsSink` (record_counter, record_gauge, record_histogram) — Metrics sink port trait for recording system counters, gauges, and histograms.
- `struct NoopMetricsSink` — A no-op implementation of [`MetricsSink`] that silently discards all metrics events. · impl: MetricsSink

**`observability.rs`**

- `trait DriftStatusProvider` (overall_drift_status) — Trait for querying Lyapunov drift status from an attached router engine without creating a cyclic dependency (ADR-080).

**`plugin.rs`**

- `struct PluginCapability` { name, version, ring, feature_ring_required } — Capability descriptor for a plugin. · Methoden: new
- `trait PluginManifest` (name, requires, conflicts, feature_ring_required, capability, activate) — Manifest trait implemented by dynamic or static plugins.
- `enum PluginError` ∈ {DependencyCycle, Conflict, LicenseDenied} — Errors during plugin registration, activation, or dependency resolution.
- `struct PluginRegistry` — Registry managing plugin activations, license gating, and dependency resolution. · Methoden: new, is_active, snapshot, current_feature_ring, activate_all

**`reranker.rs`**

- `trait Reranker` (rerank, record_implicit_feedback, rerank_deadline_ms) — Trait for post-retrieval cross-encoder reranking models.

**`rng.rs`**

- `trait Rng` (next_u64, fill_bytes, next_unit_f64) — Port trait for random number generators, enabling deterministic testing by decoupling non-deterministic entropy sources.
- `struct SeededRng` — A fast, deterministic 64-bit pseudorandom number generator using SplitMix64 with atomic state transitions (`AtomicU64`). · Methoden: new · impl: Rng

**`storage.rs`**

- `struct StorageStats` { num_segments, total_size_bytes, memtable_size_bytes } — Statistics for the storage engine.
- `const MAX_SCAN_MERGE_ACCUMULATOR: usize = 100_000` — Harte Obergrenze für die Anzahl distinkter Keys, die eine StorageEngine- Implementierung während eines einzelnen scan()/scan_prefix_bounded()-Aufrufs intern akkumulieren darf, BEVOR limit/cursor angewendet wird.
- `trait StorageRead` (get, get_at_seq, scan_prefix, scan_prefix_bounded, scan_prefix_at, scan, scan_bounded, stats, last_seq_no, last_tx_id) — Synchronous storage read trait — abstracts sync read access to storage (Spec §20.2 / ADR-N02).
- `trait StorageWrite` (put, put_if_absent, put_batch, delete, delete_many, delete_prefix, commit, rollback, rollback_to_tx, flush, pin_checkpoint, unpin_checkpoint) — Asynchronous storage write trait — abstracts async write, commit, and lifecycle operations (Spec §20.2 / ADR-N02).
- `trait StorageEngine` (get, get_at_seq, get_tracked, get_at_seq_tracked, scan_prefix_tracked, supports_ssi_tracking, put, put_if_absent, put_batch, delete, delete_many, delete_prefix, commit, rollback…) — Unified Storage Engine trait — retained for backward compatibility across higher layers.

**`text_index.rs`**

- `trait SegmentSynthesizer` (synthesize_segment, model_id) — Trait-Abstraktion für LLM-Synthesizer zur Segment-Zusammenfassung (REM-Phase).
- `struct TextIndexStats` { num_documents, num_tokens, memory_usage_bytes } — Statistics for a text index.
- `trait TextIndex` (search, search_at, insert, delete, commit, rollback, rollback_to_tx, last_tx_id, len, is_empty, stats) — Text-Index Trait — abstrahiert BM25/Inverted-Index-Operationen.

**`vector_index.rs`**

- `struct VectorIndexStats` { num_vectors, memory_usage_bytes, num_layers, deleted_ratio, rebuild_count } — Statistics for a vector index.
- `trait VectorIndex` (insert, all_doc_ids, insert_batch, search, search_at, search_filtered, delete, commit, rollback, rollback_to_tx, last_tx_id, len, is_empty, stats…) — Vector Index Trait — abstrahiert die HNSW-Vektorsuche.
- `trait HybridSearchProvider` (search_hybrid) — Contract for executing hybrid (vector + text) queries for profile routing.

### K.20 `contextra-privacy` — 2.551 Zeilen (ohne Tests), 10 Dateien [V-Sig]

Cloud Egress Security, DLP & Exfiltration Protection for Contextra (Ring 3)

**Workspace-Abhängigkeiten:** `contextra-ports`, `contextra-types`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `audit_trace.rs` | 78 | — |
| `avv_generator.rs` | 196 | `avv_generator` — Auftragsverarbeitungsvertrag (AVV) Generierung gemäß Art. |
| `bulk_exfiltration_detector.rs` | 301 | — |
| `egress_gateway.rs` | 508 | — |
| `egress_guard.rs` | 304 | Layer-4 EgressGuard — Bulk-Exfiltrations-Erkennung für Cloud Egress Kapselt die k-NN-Ähnlichkeitssuche gegen einen Text-Sucher-Trait / Closure und gar |
| `egress_vault.rs` | 814 | — |
| `error.rs` | 26 | — |
| `guarded_payload.rs` | 76 | Type-State GuardedPayload<S> — Compile-Zeit-Schutz gegen ungesanitisierten Cloud-Egress (§4.14). |
| `lib.rs` | 39 | `contextra-privacy` — Cloud Egress Security, DLP & Exfiltration Protection (Ring 3) |
| `processing_registry.rs` | 209 | `processing_registry` — Maschinell erzeugtes Verzeichnis von Verarbeitungstätigkeiten (Art. |

**`audit_trace.rs`**

- `fn extract_rule_id(classification: &EgressClassification) -> String` — Extrahiert eine stabile, nicht-leere Regel-Kennung aus einer `EgressClassification`.
- `fn compute_audit_trace(payload: &str, rule_id: &str, nanos: u64) -> [u8; 32]` — Berechnet einen deterministischen 32-Byte-BLAKE3-Hash-Trace für eine Klassifikationsentscheidung.
- `trait EgressClassifierTrace` (classify_with_trace) — Erweiterter Trait für Egress-Klassifikatoren mit kryptografischem Audit-Trace.

**`avv_generator.rs`**

- `struct AvvContext` { controller_name, processor_name, contract_date, deletion_proof_mechanism, egress_gateway_active, tenant_isolation_model } — Kontextdaten für die Erstellung eines Auftragsverarbeitungsvertrags (AVV).
- `fn generate_avv_draft(ctx: &AvvContext) -> String` — Generiert einen vollständigen AVV-Entwurf im Markdown-Format.

**`bulk_exfiltration_detector.rs`**

- `struct SessionId` — Session identifier for grouping egress rate-limiting buckets.
- `enum BulkExfiltrationOutcome` ∈ {Allow, Block} — Outcome of evaluating outbound bulk payload volume against session limits.
- `struct BulkExfiltrationDetector` { max_bytes_per_window, window } — Volume and time-window based rate limiter for cloud egress requests (Layer 4). · Methoden: new, record_and_check

**`egress_gateway.rs`**

- `type DefaultEgressClassifier = EgressVault`
- `const MAX_SEARCH_QUERY_BYTES: usize = 64 * 1024` — Maximum allowed search query length in bytes (64 KB).
- `trait EgressGuardCheck` (check) — Trait contract for Layer 4 bulk-exfiltration checks via `EgressGuard`.
- `trait InjectionDetector` (detect) — Trait contract for checking prompt injection in cloud responses without depending on Ring 4 crates.
- `struct CloudResponseRehydrator` — Inbound Layer 5 Re-Hydrator replacing surrogate tokens with original vault entities. · Methoden: new, rehydrate
- `fn pii_vault_forces_crypto_shred(is_pii_match: bool, is_memory_only: bool) -> bool` — Evaluates whether PII vault data forces CryptoShred mode for an affected document (INV-COLLECTION-PROFILE-3).
- `fn resolve_effective_kv_delete_mode( is_pii_match: bool, durability_mode: D, preset_kv_delete_mode: K, crypto_shred_mode: K, ) -> K` — Read-only check for PII Vault coupling (Spec B.1.8 / INV-COLLECTION-PROFILE-3):
- `struct CloudQueryRequest` { query, collection, max_results }
- `struct CloudQueryResponse` { status, query, abstracted, results, abstraction_notice }
- `fn handle_cloud_query( request: CloudQueryRequest, classifier: &dyn EgressClassifier, ) -> Result<CloudQueryResponse, EgressError>`
- `fn handle_cloud_query_scoped( request: contextra_types::TenantScoped<CloudQueryRequest>, expected_tenant_id: &contextra_types::TenantId, classifier: &dyn EgressClassifier` — Handles a cloud query request bound to a tenant scope, enforcing matching `TenantId` before execution.
- `fn handle_cloud_query_scoped_with_guard( request: contextra_types::TenantScoped<CloudQueryRequest>, expected_tenant_id: &contextra_types::TenantId, classifier: &dyn Egres` — Extended cloud query handler enforcing tenant scope matching alongside DLP classifier and Layer 4 guard.
- `fn handle_cloud_query_scoped_with_bulk_detector( request: contextra_types::TenantScoped<CloudQueryRequest>, expected_tenant_id: &contextra_types::TenantId, classifier: &d` — Extended cloud query handler enforcing tenant scope matching, DLP classifier, Layer 4 guard, and bulk volume detector.
- `fn handle_cloud_query_with_guard( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, ) -> Result<CloudQueryRespon`
- `fn check_bulk_exfiltration( detector: &BulkExfiltrationDetector, session: SessionId, payload: &str, ) -> Result<(), EgressError>` — Evaluates a query payload against the `BulkExfiltrationDetector` for a specific `SessionId`.
- `fn handle_cloud_query_with_bulk_detector( request: CloudQueryRequest, classifier: &dyn EgressClassifier, egress_guard: Option<&dyn EgressGuardCheck>, bulk_detector: Optio` — Extended cloud query handler including additive Layer 4 bulk volume check along with DLP classifier and similarity guard.
- `fn process_cloud_response( raw_response: &str, injection_guard: &dyn InjectionDetector, rehydrator: &CloudResponseRehydrator, ) -> Result<String, EgressError>` — Processes inbound responses from cloud LLMs by checking for prompt injections and rehydrating surrogate tokens back to original values.

**`egress_guard.rs`**

- `struct TextSearchResult` { id, score } — Simplified candidate result from text similarity search.
- `trait TextSearchEngine` (search_text) — Trait for text similarity search without depending on `contextra-db::Collection` (Ring 3 DB).
- `const DEFAULT_EGRESS_GUARD_TIMEOUT: Duration = Duration::from_millis(200)` — Standard-Timeout für EgressGuard Vector-Search (200 ms).
- `const DEFAULT_EGRESS_GUARD_THRESHOLD: f32 = 0.85` — Standard-Schwellwert für HNSW Cosine Similarity Match (0.85).
- `const DEFAULT_EGRESS_GUARD_MIN_BYTES: usize = 128` — Minimum Byte-Länge des Payloads, ab der die HNSW-Prüfung durchgeführt wird (128 Bytes).
- `struct EgressGuard` — Layer-4 EgressGuard zur Erkennung und Blockierung von Bulk-Exfiltrationen. · Methoden: new, with_timeout, threshold, min_bytes, timeout, check_scoped, check · impl: EgressClassifier

**`egress_vault.rs`**

- `trait EntityRecognizer` (recognize) — Trait for recognizing entities (NER) in text without creating a direct dependency on heavy embedding or ML crates (DAG-neutral interface).
- `struct NoOpRecognizer` — A default no-op entity recognizer that detects no entities. · impl: EntityRecognizer
- `struct SurrogateVault` — Session-bound vault storing bidirectional mappings between original PII/entities and generated surrogates. · Methoden: new, random_salt, generate_surrogate, get_entity, len, is_empty
- `type BoxFuture = Pin<Box<dyn Future<Output = T> + Send + 'a>>` — Type alias for boxed dyn futures in `EgressClassifier` trait to ensure dyn compatibility.
- `enum PolicyCategory` ∈ {LocalAccess, CloudEgress} — Policy-Kategorie für Audit-Logging und Zugriffskontrolle.
- `enum BlockReason` ∈ {SensitivePattern, PolicyDenied, EgressPolicyDenied, ClassificationTimeout, InternalError} — Grund für das Blockieren einer Egress-Anfrage. · Methoden: category
- `enum EgressClassification` ∈ {Allow, Block} — Klassifikationsergebnis der Egress-Prüfung.
- `struct CompiledPattern` { name, regex } — Ein vorkompiliertes Regex-Muster für die Egress-Klassifikation. · Methoden: new
- `enum EgressVaultError` ∈ {InvalidPattern, PayloadTooLarge, TenantScopeViolation} — Fehler beim Erstellen oder Laden des EgressVaults.
- `const MAX_CLASSIFY_PAYLOAD_BYTES: usize = 65_536` — Maximale zulässige Payload-Länge in Bytes für Layer-1-Klassifikation.
- `fn classify_layer1( payload: &str, patterns: &[CompiledPattern], timeout: Duration, ) -> EgressClassification` — Führt eine Layer-1-Regex-Klassifikation auf dem Payload mit neuem Task und hartem Timeout aus.
- `fn classify_layer1_arc( payload: &str, patterns: Arc<Vec<CompiledPattern>>, regex_set: Arc<regex::RegexSet>, timeout: Duration, ) -> EgressClassification` — Optimierter Ausführungspfad für `classify_layer1` unter Wiederverwendung von vorkompilierten `Arc`-Mengen.
- `struct EgressVault` — Cloud Egress Vault zur Abhandlung und Bündelung von Layer-1-Klassifikationen. · Methoden: default_patterns, try_default, new, with_timeout, with_surrogate_vault, patterns, policy_category, timeout, surrogate_vault, classify_scoped, sanitize_and_vault_scoped, sanitize_and_vault · impl: EgressClassifierTrace, EgressClassifier
- `trait EgressClassifier` (classify) — Schnittstelle für Downstream-Crates (`contextra-mcp`, `contextra-router`).

**`error.rs`**

- `enum EgressError` ∈ {InvalidParams, PolicyViolation, InternalError} · Methoden: invalid_params, policy_violation, internal_error

**`guarded_payload.rs`**

- `struct Unsanitized` — Marker-Typ:
- `struct Sanitized` — Marker-Typ:
- `struct GuardedPayload` — Type-State-Wrapper für Cloud-Egress-Payloads. · Methoden: session_id, new, from_sanitized, into_inner

**`processing_registry.rs`**

- `enum ProcessorRole` ∈ {Controller, Processor} — Rolle bezüglich der Verarbeitungstätigkeit gemäß DSGVO. · Methoden: as_str
- `type TenantId = String` — Identifikator eines Mandanten im System.
- `struct ProcessingActivityRecord` { activity_name, controller_or_processor_role, purpose, data_categories, data_subject_categories, recipients, third_country_transfers, retention_period, technical_and_organizational_measures, tenant_id } — Ein Datensatz einer Verarbeitungstätigkeit gemäß Art.
- `struct ProcessingRegistryExport` { records } — Strukturierte Export-Repräsentation des Verzeichnisses von Verarbeitungstätigkeiten.
- `fn generate_registry(records: &[ProcessingActivityRecord]) -> ProcessingRegistryExport` — Erzeugt aus einer Reihe von Datensätzen eine strukturierte Export-Repräsentation.
- `fn render_markdown(export: &ProcessingRegistryExport) -> String` — Erzeugt eine druckfertige, deutschsprachige Markdown-Tabelle aus dem Export.

### K.21 `contextra-py` — 1.904 Zeilen (ohne Tests), 15 Dateien [V-Sig]

Python bindings for Contextra using PyO3

**Features:** default = `—`; weitere: `docid-128`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-router`, `contextra-rank`, `contextra-adapt`, `contextra-core`, `contextra-store`, `contextra-db`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `bindings/collection.rs` | 90 | — |
| `bindings/common.rs` | 369 | — |
| `bindings/db.rs` | 189 | — |
| `bindings/db_stats.rs` | 41 | — |
| `bindings/document.rs` | 18 | — |
| `bindings/functions.rs` | 118 | — |
| `bindings/hyperedge.rs` | 154 | — |
| `bindings/runtime_state.rs` | 78 | — |
| `bindings/search_result.rs` | 20 | — |
| `bindings/storage_stats.rs` | 24 | — |
| `bindings/vector_index_stats.rs` | 24 | — |
| `kv_links.rs` | 210 | — |

**`bindings/collection.rs`**

- `struct PyCollection` · Methoden: new, is_poisoned, _trigger_panic_for_test, stats, len, is_empty, relate_n_ary, put_kv, put_kv_if_absent, get_kv, relate_bidirectional, link_memories, get_links

**`bindings/common.rs`**

- `const MIN_WORKER_THREADS: usize = 1` — Minimum allowed worker threads for Tokio multi-thread runtime.
- `const MAX_WORKER_THREADS: usize = 256` — Maximum allowed worker threads for Tokio multi-thread runtime.
- `const MAX_ID_LENGTH: usize = 1024` — Maximum allowed length for document string IDs (1024 characters).
- `const MAX_LABEL_LENGTH: usize = 256` — Maximum allowed length for relationship labels (256 characters).
- `const MAX_BATCH_SIZE: usize = 10_000` — Maximum batch size for batch insertion/upsertion (10,000 items).
- `fn dict_to_json(d: &pyo3::Bound<'_, pyo3::types::PyDict>) -> PyResult<serde_json::Value>` — Converts a Python dict to a serde_json::Value.
- `fn opt_dict_to_json( metadata: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>, ) -> PyResult<Option<serde_json::Value>>` — Converts an optional Python dict to an optional serde_json::Value.
- `fn validate_label(label: &str) -> PyResult<()>` — Validates that a relationship label is non-empty, contains no null bytes, and does not exceed maximum length.
- `fn validate_id(id: &str) -> PyResult<()>` — Validates that a string ID is non-empty, contains no null bytes, and does not exceed maximum length.
- `fn validate_collection_name(name: &str) -> PyResult<()>` — Validates that a collection name is non-empty, contains no null bytes, and does not exceed maximum length.
- `fn validate_db_path(path: &str) -> PyResult<()>` — Validates that a database storage path is non-empty and contains no null bytes.
- `fn validate_query_text(text: &str) -> PyResult<()>` — Validates that a search query text is non-empty, contains no null bytes, and does not exceed maximum length.
- `fn validate_batch_size(size: usize) -> PyResult<()>` — Validates batch size against maximum resource allocation limits.
- `fn validate_vector(vector: &[f32]) -> PyResult<()>` — Validates that a vector slice is non-empty and contains no NaN or infinite values.
- `fn validate_id_obj(id_obj: &pyo3::Bound<'_, pyo3::types::PyAny>) -> PyResult<String>` — Validates a document ID provided as a string or numeric value.
- `fn check_subinterpreter_guard(py: Python<'_>) -> PyResult<()>` — Explicitly checks whether the module is being imported inside a CPython sub-interpreter.
- `fn run_blocking_ffi(py: Python<'_>, poisoned: &AtomicBool, f: F) -> PyResult<R>` — Safely executes a blocking closure across FFI boundaries with thread state release and panic containment to guarantee no Rust panic propagates across FFI boundaries into Python.
- `fn json_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<PyObject>` — Converts a serde_json::Value to a Python object.
- `fn doc_to_py(py: Python<'_>, d: contextra_db::Document) -> PyResult<PyDocument>` — Converts a contextra_db::Document to a PyDocument.
- `fn results_to_py( py: Python<'_>, results: Vec<contextra_db::SearchResult>, ) -> PyResult<Vec<PySearchResult>>` — Converts a Vec of SearchResult to Vec of PySearchResult.
- `fn contextra_err(e: contextra_core::ContextraError) -> PyErr` — Maps a ContextraError into a structured Python PyErr with `kind`, `message`, and `details` attributes.

**`bindings/db.rs`**

- `struct PyContextra` · Methoden: new, worker_threads, is_poisoned, _trigger_panic_for_test, collection, list_collections, drop_collection, flush, stats, len, is_empty

**`bindings/db_stats.rs`**

- `struct PyDbStats` { drift_status, calibration_ece, last_calibration_at, active_memory_count, pid_pool_size, index_stats, storage_stats } — Overall database statistics and system observability.

**`bindings/document.rs`**

- `struct PyDocument` { id, metadata } — A document retrieved from Contextra.

**`bindings/functions.rs`**

- `fn open( py: Python<'_>, path: &str, dimension: usize, max_elements: Option<usize>, encryption_passphrase: Option<String>, distance_metric: Option<String>, ) -> PyResult<` — Opens or creates a Contextra database at the given path.
- `fn _trigger_panic_for_test(py: Python<'_>, message: Option<String>) -> PyResult<()>` — Internal Python binding function for testing FFI panic isolation.

**`bindings/hyperedge.rs`**

- `fn validate_hyperedge_args( predicate: &str, participants: &[(&str, &str)], source_doc_id: Option<&str>, ) -> PyResult<()>` — Validates arguments for creating an n-ary hyperedge.

**`bindings/runtime_state.rs`**

- `fn parse_worker_threads_env() -> usize` — Parses and clamps `CONTEXTRA_WORKER_THREADS` environment variable to `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]`.
- `struct PyRuntimeState` { runtime, worker_threads } — Holds the per-interpreter/per-module Tokio runtime state and worker thread configuration.
- `fn get_runtime(py: Python<'_>) -> PyResult<Arc<Runtime>>` — Retrieves or initializes the per-interpreter Tokio runtime attached to the `_contextra` module state.

**`bindings/search_result.rs`**

- `struct PySearchResult` { id, score, metadata } — A single search result from Contextra.

**`bindings/storage_stats.rs`**

- `struct PyStorageStats` { num_segments, total_size_bytes, memtable_size_bytes } — Statistics for the storage engine.

**`bindings/vector_index_stats.rs`**

- `struct PyVectorIndexStats` { num_vectors, memory_usage_bytes, num_layers } — Statistics for a vector index.

**`kv_links.rs`**

- `fn parse_link_relation(s: &str) -> PyResult<LinkRelation>` — Parses a string into a `LinkRelation` (case-insensitive, allocation-free).

### K.22 `contextra-rank` — 3.363 Zeilen (ohne Tests), 20 Dateien [V-Sig]

4-Signal Fusion, Isotonic & Platt Calibration, and Drift Detection for Contextra Cognitive OS

**Features:** default = `—`; weitere: `dibud`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `calibration/conformal.rs` | 218 | Conformal Calibration and Adaptive Conformal Prediction with Propensity Weighting. |
| `calibration/isotonic.rs` | 253 | Isotonic Calibration via PAVA (Pool-Adjacent Violators Algorithm). |
| `calibration/mod.rs` | 10 | Score calibration submodules (Conformal + Isotonic + Platt scaling). |
| `calibration/platt.rs` | 152 | Platt-Scaling (Logistic Calibration): |
| `dibud/driver.rs` | 63 | Synchronous and std-only asynchronous drivers for DiBud fusion state machine. |
| `dibud/mod.rs` | 10 | DiBud (Dynamic Budgeted RRF) fusion state machine, drivers, and types. |
| `dibud/state.rs` | 297 | State machine for DiBud (Dynamic Budgeted RRF) fusion. |
| `dibud/types.rs` | 119 | Types for DiBud (Dynamic Budgeted RRF) fusion. |
| `drift.rs` | 123 | Score distribution and ranking drift detection for Contextra search signals. |
| `explain.rs` | 421 | Provenance explanation module converting machine-readable search provenance into structured explanation objects and German human-readable text. |
| `fusion/global.rs` | 212 | Global Fusion Strategy for Corpus-Wide Topic Retrieval (§B.3.4). |
| `fusion/mod.rs` | 24 | Hybrid Search Signal Fusion implementations (Reciprocal Rank Fusion & Score Normalization). |
| `fusion/normalized.rs` | 284 | — |
| `fusion/provenance.rs` | 296 | — |
| `fusion/resonance.rs` | 81 | — |
| `fusion/rrf.rs` | 413 | — |
| `fusion/signal.rs` | 136 | — |
| `fusion/topk.rs` | 86 | — |
| `fusion/types.rs` | 132 | — |
| `lib.rs` | 33 | `contextra-rank`: |

**`calibration/conformal.rs`**

- `enum ConformalError` ∈ {InvalidAlpha, InvalidPropensityWeight, InvalidNonConformityScore, InvalidThreshold, InvalidLearningRate} — Error type for conformal calibration and threshold updates.
- `trait ConformalCalibrator` (update, threshold) — Trait for deterministic, online conformal calibration and threshold estimation.
- `struct AdaptiveConformalCalibrator` — Adaptive Conformal Calibrator for quantile-based online threshold estimation under covariate shift. · Methoden: new, with_params, with_bounds, alpha, learning_rate, observation_count, invalidate_on_config_change · impl: ConformalCalibrator

**`calibration/isotonic.rs`**

- `struct IsotonicCalibrator` — Isotonic Calibrator using Pool-Adjacent Violators Algorithm. · Methoden: new, with_defaults, record_outcome, observation_count, is_calibrated, last_calibration_at, cached_ece, calibrated_probability, force_rebuild, invalidate_on_config_change, expected_calibration_error

**`calibration/platt.rs`**

- `struct PlattScaler` — Platt Scaler for parametric logit/score calibration. · Methoden: new, identity, is_identity, is_fitted, params, predict, apply, transform, invalidate_on_config_change, fit

**`dibud/driver.rs`**

- `fn fuse_exact_prefix( mut state: DiBudFusionState, vector: &mut I1, text: &mut I2, graph: &mut I3, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> ` — Synchronously executes DiBud fusion using three channel iterators.
- `fn fuse_exact_prefix_async( mut state: DiBudFusionState, mut poll: P, edge_reinforcement: impl Fn(DocId) -> f32, budget: &FusionBudget, ) -> Result<DiBudOutcome, Contextr` — Asynchronously executes DiBud fusion using a channel poll closure returning a std `Future`.

**`dibud/state.rs`**

- `enum DiBudStep` ∈ {Poll, Done} — Represents the step action requested by `DiBudFusionState::next_request`.
- `struct DiBudFusionState` — Core state machine tracking channel depths, observed document scores, and prefix certification. · Methoden: new, channel_depths, exhausted, accesses, provenance_of, feed, next_request

**`dibud/types.rs`**

- `enum BudgetedChannel` ∈ {Vector, Text, Graph} — Identifies the channels managed by DiBud fusion budget. · Methoden: index, from_index
- `struct FusionBudget` { max_total_accesses, min_certified_results, edge_reinforcement_weight, channel_weights, edge_reinforcement_upper_bound } — Dynamic budget configuration for DiBud fusion. · Methoden: validate
- `struct DiBudOutcome` { ranked, certified_len, accesses, budget_exhausted } — Final outcome of a DiBud fusion execution.

**`drift.rs`**

- `enum DriftStatus` ∈ {Stable, DriftDetected, InsufficientData} — Drift status summary representation. · Methoden: as_str
- `struct DriftDetector` — Drift detector monitoring score shifts over sliding windows. · Methoden: new, with_defaults, set_baseline, observe, analyze, overall_drift_status

**`explain.rs`**

- `enum SignalKind` ∈ {Vector, Bm25, Graph, Rerank} — Search signal kind used during multi-signal fusion. · Methoden: name_de, from_name, as_str
- `struct ExplanationEntry` { signal, raw_score, rank, weight, contribution_share } — Breakdown entry representing the contribution of a single search signal.
- `struct RetrievalExplanation` { entries, dominant_signal, coherence_bonus, source_collection } — Structured explanation detailing why a document was retrieved with its given rank. · Methoden: to_human_readable_de
- `fn explain(record: &ProvenanceRecord) -> RetrievalExplanation` — Generates a structured `RetrievalExplanation` from a `ProvenanceRecord`.

**`fusion/global.rs`**

- `struct GlobalFusionConfig` { max_community_nodes, min_community_size, k_rrf } — Configuration for `GlobalFusionStrategy`.
- `struct GlobalFusionStrategy` — Community-aware fusion strategy for corpus-wide topic retrieval (LeanRAG-backed). · Methoden: new, config, fuse

**`fusion/normalized.rs`**

- `fn score_normalized_fusion_with_options( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: ` — Score-normalized fusion (CombSUM with MinMax normalization).
- `fn weights_to_signal_factors( weights: Option<&contextra_types::FusionWeights>, ) -> (f32, f32, f32)` — Converts optional `FusionWeights` into (vector, text, graph) weight tuple.

**`fusion/provenance.rs`**

- `struct ProvenanceBuilder` — Fluent builder for constructing `ProvenanceRecord` instances. · Methoden: new, vector, bm25, graph, rerank_score, source_collection, index_type, expected_total, build, explain_human_readable

**`fusion/resonance.rs`**

- `struct ResonanceConfig` { beta, gamma } — Configuration for Resonance Coherence Bonus.
- `fn fuse_signals( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<FusedScore>` — Convenience API to fuse multi-signal search results.
- `fn apply_resonance_bonus( results: Vec<SearchResult>, valid_signal_count: usize, config: &ResonanceConfig, ) -> Vec<SearchResult>` — Applies Resonance Coherence Bonus to fused search results.

**`fusion/rrf.rs`**

- `fn reciprocal_rank_fusion( result_sets: Vec<Vec<SearchResult>>, max_results: usize, ) -> Vec<SearchResult>` — Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF).
- `fn weighted_reciprocal_rank_fusion( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, ) -> Vec<SearchResult>` — Weighted Reciprocal Rank Fusion with default signal metadata priority (`VectorFirst`).
- `fn weighted_reciprocal_rank_fusion_with_priority( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, ) -> Vec<Search` — Weighted Reciprocal Rank Fusion with explicit metadata merge priority.
- `fn weighted_reciprocal_rank_fusion_with_options( mut result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_prov` — Weighted Reciprocal Rank Fusion with options.
- `fn fuse_search_results_with_strategy( result_sets: Vec<(String, Vec<SearchResult>, f32)>, max_results: usize, priority: MetadataMergePriority, include_provenance: bool, r` — Fuses search result sets using the specified `FusionStrategy`.

**`fusion/signal.rs`**

- `enum SignalKind` ∈ {Vector, Text, Graph, EdgeReinforcement} — Identifies the kind of search signal used during fusion.
- `enum MetadataMergePriority` ∈ {VectorFirst, TextFirst, GraphFirst, Custom} — Configures signal priority order for metadata merging during Reciprocal Rank Fusion. · Methoden: signal_rank

**`fusion/topk.rs`**

- `struct BoundedTopK` — Size-bounded min-heap for Top-K candidate selection during Reciprocal Rank Fusion. · Methoden: new, capacity, len, is_empty, push, into_sorted_vec

**`fusion/types.rs`**

- `struct ProvenanceRecord` { vector_distance, bm25_score, graph_score, rerank_score, signal_ranks, source_collection, index_type, signal_contributions, coherence_bonus } — Provenance record for fused search result. · Methoden: explain_human_readable, explain_human_readable_with_params, synthesized_from
- `struct SignalContribution` { raw_score, rank, rrf_contribution } — Signal contribution details.
- `struct SearchResult` { id, score, metadata, matched_signals, provenance } — Individual search result candidate for fusion.
- `type FusedScore = SearchResult` — Type alias for fused score output item.

### K.23 `contextra-router` — 2.502 Zeilen (ohne Tests), 14 Dateien [V-Sig]

**Features:** default = `—`; weitere: `bandit-routing`, `cloud-egress-guard`, `egress-sherman-morrison`, `flow-corrected-thompson`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-wire`, `contextra-ports`, `contextra-adapt`, `contextra-privacy`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `arm_registry.rs` | 115 | — |
| `dispatch.rs` | 255 | — |
| `fc_ts_dispatch.rs` | 39 | FC-TS-Profilauswahl für contextra-router (§21.3, AK-18). |
| `outcome.rs` | 142 | — |
| `ports_local.rs` | 25 | Deprecated: |
| `profile.rs` | 490 | — |
| `router/mod.rs` | 328 | — |
| `routing_strategy.rs` | 48 | Routing-Strategie-Enum (§4.14, §13.1). |
| `serde_helpers.rs` | 23 | — |
| `transport.rs` | 56 | MCP-Transport-Abstraktion (§4.14). |

**`arm_registry.rs`**

- `struct ArmRegistry` — Verlustfreie, deterministische Abbildung Arm-Index <-> RetrievalStrategy. · Methoden: strategy_for, arm_for
- `enum ArmRegistryError` ∈ {OutOfRange} — Fehlerklasse für ungültige Arm-Indizes — crate-lokal, P6-konform an Grenze konvertierbar.

**`dispatch.rs`**

- `fn dispatch_to_slm(decision: &RoutingDecision) -> Result<String>` — Dispatches the prepared context from a [`RoutingDecision`] to the target SLM's MCP endpoint over stdio JSON-RPC 2.0 (ADR-010 compliant).

**`fc_ts_dispatch.rs`**

- `enum FcTsDispatchError` ∈ {ArmSet, ProfileArmMismatch} — Fehler der FC-TS-Profilauswahl für contextra-router.
- `fn select_profile_fc_ts( profile_names: &[String], arm_set: &FcTsArmSet, context: &[f32], rng: &mut dyn FcTsRng, ) -> Result<(usize, String), FcTsDispatchError>` — Wählt einen Profil-Index über FC-TS-Argmax-Sampling aus `arm_set` für den gegebenen Kontextvektor.
- `fn deterministic_fc_ts_rng(seed: u64) -> impl FcTsRng` — Deterministischer Default-RNG-Konstruktor für reproduzierbare FC-TS-Auswahl bei gegebenem Seed (z.

**`outcome.rs`**

- `struct DecisionId` — Eindeutige ID einer Routing-Entscheidung. · Methoden: from_raw, inner
- `struct DecisionIdGenerator` — Instanzgebundener Generator für monoton steigende `DecisionId`s (Spec §3 P29). · Methoden: new, next
- `enum RoutingOutcome` ∈ {Success, Escalated, Rejected} — Tatsächliches Ergebnis einer getroffenen Routing-Entscheidung. · Methoden: non_conformity_score

**`profile.rs`**

- `enum QuantizationLevel` ∈ {F16, Q8_0, Q4_K_M, Unknown} — Quantization level of the underlying model execution path.
- `struct SlmProfile` { name, mcp_endpoint, domain_communities, token_budget, min_relevance_score, resource_cost_estimate, fingerprint, transport, bandit_state } — Represents a Small Language Model (SLM) target and its domain expertise parameters. · Methoden: new, with_fingerprint, with_resource_cost_estimate, estimated_cost, validate, try_new
- `struct ConformalCalibrator` { alpha, gamma, quantile_threshold, window_errors, window_total } — Conformal-inspirierte Kalibrierung (Coverage-Garantie erst mit record_outcome()). · Methoden: new, update, empirical_error_rate, reset_window
- `struct ProfileCalibrationState` { times_selected, cumulative_confidence, calibrated_min_score, original_min_score, conformal, last_calibrated_fingerprint } — Laufzeit-Kalibrierungsstatistik für ein SLM-Profil. · Methoden: new, check_and_invalidate_fingerprint, is_calibrated, average_confidence, recalibrate_conformal, reset
- `serde_sorted_u64_set::fn serialize(set: &HashSet<u64>, s: S) -> Result<S::Ok, S::Error>`
- `serde_sorted_u64_set::fn deserialize(d: D) -> Result<HashSet<u64>, D::Error>`

**`router/mod.rs`**

- `struct ConfidenceMetrics` { score_lower, score_upper, calibrated, quantile_threshold, non_conformity_score, selection_margin } — Calibrated confidence metrics for a routing decision.
- `struct RoutingDecision` { profile, context, confidence, decision_id, drift_status } — Result of a routing operation containing the selected profile, prepared context, confidence, and decision ID.
- `struct RouterState` { profiles, calibration, lyapunov_watchers } — Inner state for `RouterEngine` holding active profiles, calibration states, and Lyapunov drift watchers.
- `type DefaultRouterEngine = RouterEngine` — Router engine that routes queries to optimal SLM backends based on community assignment and search scores.
- `struct RouterEngine` · Methoden: route, new, with_initial_decision_id, with_routing_strategy, try_new, update_profiles, try_update_profiles, profiles, bandit_decision_propensity, calibration_stats, reset_calibration, drift_status, overall_drift_status, set_lyapunov_baseline… · impl: LocalDriftStatusProvider

**`routing_strategy.rs`**

- `enum RoutingStrategy` ∈ {Cascade, ContextualBandit, FlowCorrectedThompson} — Auswahl-Strategie für SLM-Profil-Routing.

**`serde_helpers.rs`**

- `sorted_u64_set::fn serialize(set: &HashSet<u64>, s: S) -> Result<S::Ok, S::Error>`
- `sorted_u64_set::fn deserialize(d: D) -> Result<HashSet<u64>, D::Error>`

**`transport.rs`**

- `enum Transport` ∈ {StdioMcp, HttpCloud} — Transport-Kanal für SLM-Routing-Entscheidungen. · Methoden: is_cloud

### K.24 `contextra-sandbox` — 1.952 Zeilen (ohne Tests), 7 Dateien [V-Sig]

WASM Execution Boundary for Contextra MCP CodeExecution Permission

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `approval.rs` | 487 | — |
| `capabilities.rs` | 149 | — |
| `error.rs` | 38 | WASM-Sandbox-Fehlertypen (§4.18). |
| `executor.rs` | 633 | — |
| `lib.rs` | 27 | Contextra WASM Execution Boundary (§4.18). |
| `output.rs` | 30 | WasmOutput — Ausgabe einer WASM-Execution mit ZeroizeOnDrop (§4.18, P9). |
| `wasi.rs` | 588 | — |

**`approval.rs`**

- `enum ApprovalRisk` ∈ {Low, Elevated, High} — Risk level classification for a requested WASM execution configuration.
- `fn classify_risk(caps: &WasmCapabilities) -> ApprovalRisk` — Derives the execution risk level deterministically from `WasmCapabilities`.
- `enum ApprovalStatus` ∈ {Pending, Approved, Rejected, Expired} — Status of a human approval decision.
- `enum ApprovalTransitionError` ∈ {Expired, InvalidStateTransition} — Errors occurring during approval state transitions.
- `struct ApprovalRequest` { request_id, risk, requested_capabilities_summary, status, created_at_unix_ms, ttl_ms } — Human approval request state machine for a WASM execution run. · Methoden: new, is_expired, approve, reject, expire, requires_approval

**`capabilities.rs`**

- `struct WasmCapabilities` { allow_stdout, allow_stderr, max_memory_pages, max_fuel, allow_filesystem, allow_network, allow_clock, allow_cloud_egress, max_wall_clock_ms, max_module_size_bytes… } — Whitelist für WASM-Guest-Capabilities. · Methoden: pure_merge_operator
- `struct MergeOperatorCapabilities` — Dedicated pure capabilities preset for I/O-free WASM Merge Operators (§4.18). · Methoden: pure

**`error.rs`**

- `enum SandboxError` ∈ {Timeout, MemoryExceeded, FuelExhausted, CapabilityViolation, WasmTrap, InvalidModule, OutputLimitExceeded, InputTooLarge, ProcessExit, Runtime} — Fehlertypen der WASM-Ausführungsgrenze.

**`executor.rs`**

- `struct CapabilityViolationError` { capability } — Custom error type returned by host functions on capability violation (INV-SBX-3). · impl: Error
- `struct WasmExecutor` — WASM-Ausführungsgrenze für die `CodeExecution`-Permission. · Methoden: new, execute

**`output.rs`**

- `struct WasmOutput` { stdout, stderr, fuel_consumed } — Ausgabe einer WASM-Execution. · Methoden: new

**`wasi.rs`**

- `struct ProcessExitError` { code } — Error thrown on WASI `proc_exit`. · impl: Error
- `struct OutputLimitExceededError` { stream, limit } — Error thrown when stdout or stderr exceeds `max_output_bytes`. · impl: Error

### K.25 `contextra-simd` — 1.399 Zeilen (ohne Tests), 7 Dateien [V-Sig]

Ring 0 SIMD distance kernels and runtime dispatch for Contextra (Unsafe Island)

**Workspace-Abhängigkeiten:** `contextra-core`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `dispatch.rs` | 235 | — |
| `kernels/avx2.rs` | 384 | — |
| `kernels/avx512.rs` | 368 | — |
| `kernels/neon.rs` | 109 | — |
| `kernels/scalar.rs` | 213 | — |
| `lib.rs` | 80 | — |

**`dispatch.rs`**

- `fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>`
- `fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>`
- `fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError>`
- `fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>`
- `fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>`
- `fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError>`
- `fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError>`
- `fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError>`
- `fn cosine_similarity_parts_u8( a: &[u8], b: &[u8], ) -> Result<CosineSimilarityPartsU8, ContextraError>`

**`kernels/avx2.rs`**

- `fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32`
- `fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32`
- `fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32`
- `fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn hsum256_ps_avx(v: __m256) -> f32`
- `fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32`
- `fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32`
- `fn cosine_similarity_parts_u8_avx2(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8`
- `fn hsum256_epi32_avx2(v: __m256i) -> i32`

**`kernels/avx512.rs`**

- `fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32`
- `fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32`
- `fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32`
- `fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn hsum512_ps_avx(v: __m512) -> f32`
- `fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32`
- `fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32`
- `fn cosine_similarity_parts_u8_avx512(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8`
- `fn hsum512_epi32_avx512(v: __m512i) -> i32`

**`kernels/neon.rs`**

- `fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32`
- `fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32`
- `fn dot_product_neon(a: &[f32], b: &[f32]) -> f32`

**`kernels/scalar.rs`**

- `struct CosineSimilarityPartsU8` { dot, norm_a_sq, norm_b_sq }
- `struct CosineSimilarityPartsF32U8` { dot, norm_a_sq, norm_b_sq }
- `fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32`
- `fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32`
- `fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32`
- `fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32`
- `fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32`
- `fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32`
- `fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8`
- `fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32`
- `fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32`
- `fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8`
- `fn normalize_inplace(v: &mut [f32])`

**`lib.rs`**

- `fn validate_vector(vec: &[f32]) -> contextra_core::Result<()>` — Validates that a vector contains no NaN or Infinite values.
- `fn compute_distance( a: &[f32], b: &[f32], metric: DistanceMetric, ) -> contextra_core::Result<f32>`
- `fn compute_distance_trusted( a: &[f32], b: &[f32], metric: DistanceMetric, ) -> contextra_core::Result<f32>`
- `fn compute_distance_f32_bytes_trusted( a: &[f32], b_bytes: &[u8], metric: DistanceMetric, ) -> contextra_core::Result<f32>`

### K.26 `contextra-store` — 15.118 Zeilen (ohne Tests), 50 Dateien [V-Sig]

LSM-Tree storage engine for Contextra

**Features:** default = `wal-integrity`; weitere: `encryption-at-rest`, `deletion-proof`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`, `docid-128`, `legacy-wal-key`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-core`, `contextra-mvcc`, `contextra-crypto`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `compaction.rs` | 21 | Background compaction engine for the LSM-Tree. |
| `compaction/adaptive.rs` | 325 | — |
| `compaction/config.rs` | 86 | — |
| `compaction/engine.rs` | 776 | — |
| `compaction/merge_operator.rs` | 17 | Merge Operator trait abstraction for two-way SSTable value merging (§4.12). |
| `compaction/retention.rs` | 81 | — |
| `kv/delete_mode.rs` | 47 | — |
| `kv/segment.rs` | 170 | — |
| `lib.rs` | 58 | `Contextra` Store — LSM-Tree based storage engine. |
| `lsm/config.rs` | 115 | — |
| `lsm/engine.rs` | 229 | — |
| `lsm/guard.rs` | 14 | — |
| `lsm/mod.rs` | 121 | LSM-Tree (Log-Structured Merge-Tree) storage engine. |
| `lsm/observer.rs` | 487 | — |
| `lsm/validate.rs` | 44 | — |
| `manifest/core.rs` | 387 | — |
| `manifest/entry.rs` | 347 | — |
| `manifest/mod.rs` | 23 | SSTable Manifest for append-only tracking of active SSTable sets. |
| `memtable.rs` | 1011 | — |
| `sstable.rs` | 21 | SSTable (Sorted String Table) implementation. |
| `sstable/block_cache.rs` | 363 | — |
| `sstable/block_search.rs` | 200 | — |
| `sstable/bloom.rs` | 140 | — |
| `sstable/builder.rs` | 469 | — |
| `sstable/reader.rs` | 978 | — |
| `sstable/stream.rs` | 114 | — |
| `system_pressure.rs` | 224 | INTEGRATION GUIDE: |
| `tenant_codec.rs` | 876 | — |
| `wal/encode.rs` | 441 | — |
| `wal/flusher.rs` | 579 | — |
| `wal/hmac.rs` | 494 | — |
| `wal/mod.rs` | 789 | Write-Ahead Log (WAL) for durability and crash recovery with HMAC chaining. |
| `wal/replay.rs` | 759 | — |

**`compaction.rs`**

- `struct TtlMetadata` — Removed TTL feature stub per Decision Option B.

**`compaction/adaptive.rs`**

- `struct WorkloadMetrics` — Operational workload metrics captured deterministically without wall-clock time (P28 compliance). · Methoden: new, record_read, record_write, snapshot
- `struct WorkloadMetricsSnapshot` { read_count, write_count, last_seq_no } — Point-in-time snapshot of operational workload metrics.
- `enum CompactionStrategy` ∈ {WriteOptimizedSTCS, ReadOptimizedAggressive, Balanced} — Compaction strategy variants selected by adaptive planner.
- `struct AdaptiveCompactionPlan` { strategy, candidates, is_full_compaction } — Resulting plan proposed by adaptive compaction planner.
- `trait AdaptiveCompactionPlanner` (plan_compaction) — Trait defining adaptive compaction strategy selection.
- `struct CostBasedAdaptivePlanner` — Reference implementation of cost-based adaptive compaction (EcoTune / ArceKV inspired). · Methoden: new, validate_tombstone_safety · impl: AdaptiveCompactionPlanner

**`compaction/config.rs`**

- `struct CompactionConfig` { min_sstables_per_tier, size_ratio, check_interval, yield_threshold, max_memory_bytes, max_peak_memory_bytes, max_backpressure_wait, max_io_bytes_per_second, enable_adaptive_compaction, adaptive_read_ratio_threshold } — Configuration for the compaction engine.

**`compaction/engine.rs`**

- `struct CompactionEngine` · Methoden: new, with_pressure_rx, with_adaptive_planner, record_read_op, record_write_op, maybe_compact, maybe_compact_with_cancel, merge_sstables, merge_sstables_with_cancel, run_loop

**`compaction/merge_operator.rs`**

- `trait MergeOperator` (merge) — Trait for merging two raw byte values (`existing_val` from older tier, `new_val` from newer tier) for the same key across different SSTable tiers.

**`compaction/retention.rs`**

- `fn retain_key_versions(entries: Vec<T>, get_seq: F, floor_seq: u64) -> Vec<T>` — Filters a sequence of entries for a SINGLE key (sorted by raw sequence DESCENDING) according to MVCC retention rules given a `floor` sequence number.

**`kv/delete_mode.rs`**

- `enum KvDeleteMode` ∈ {TombstoneOnly, CryptoShred} — Mode specifying KV segment deletion behavior.

**`kv/segment.rs`**

- `struct KvSegmentConfig` { delete_mode } — Configuration for KV segment storage and shredding behavior.
- `struct KvSegmentPayload` { group_id, payload, nonce, is_encrypted } — Representation of a written KV segment (encrypted or raw payload with group_id and nonce).
- `struct KvSegmentManager` — Manager for KV segment encryption, decryption, and crypto-shredding. · Methoden: new, delete_mode, write_segment, read_segment, delete_segment, generate_deletion_proof

**`lsm/config.rs`**

- `enum DurabilityMode` ∈ {Full, WalNoHmac, MemoryOnly} · Methoden: validate_against_features
- `enum DurabilityConfigError` ∈ {IncompatibleCombination}
- `struct LsmConfig` { path, memtable_size_limit, max_ram_mb, tx_timeout, compaction, encryption_passphrase, group_commit_window_micros, block_cache_shards, durability_mode, ssi_max_tracked_keys } — Configuration for the LSM storage engine.

**`lsm/engine.rs`**

- `enum StorageHealth` ∈ {Healthy, FlushFailing} — Health status of the storage engine.
- `struct LsmStorage` { snapshot_registry, ssi_validator } — LSM-Tree based storage engine. · Methoden: cleanup_intent_locks_where, clear_intent_locks_for_tx, clear_intent_locks_above_tx, open, pressure_receiver, shutdown, wait_shutdown, close, spawn_tracked, simulate_wal_append_failure_for_test, simulate_wal_append_failure_for_tx_for_test, restore_wal_file_handle_for_test, budget_tracking_drift_bytes, register_observer… · impl: StorageEngine, StorageEngineHandle

**`lsm/guard.rs`**

- `struct CommitGuard` — Proof that `commit_mutex` is currently held by the calling task.
- `struct LsmState`

**`lsm/mod.rs`**

- `const MAX_KEY_SIZE: usize = 65_535` — Maximum key size allowed for LSM operations (65,535 bytes).
- `const MAX_VALUE_SIZE: usize = 134_217_728` — Maximum value size allowed for LSM operations (128MB).
- `const MAX_BATCH_SIZE: usize = 10_000` — Maximum batch size for `delete_many` operations (10,000 items).
- `const MAX_INTERNAL_MERGE_ENTRIES_FACTOR: usize = 8` — Maximum factor for internal merge set size relative to limit in bounded scans.
- `const MAX_GROUP_COMMIT_BATCH_SIZE: usize = 1_000` — Maximum batch size for group commits (1,000 transactions).
- `const MIN_ENTRIES_FOR_SSTABLE_REBUILD: usize = 8` — Minimum surviving entry threshold required to rebuild a new SSTable during rollback.

**`lsm/observer.rs`**

- `const DEFAULT_MAX_OBSERVER_LATENCY: Duration = Duration::from_millis(1)` — Default maximum latency allowed for WAL observer callback execution (1 millisecond).
- `enum WriteOrigin` ∈ {UserWrite} — Origin tag indicating where a committed batch originated from.
- `struct WalEntryRef` { entry } — Lightweight view reference into a `WalEntry`. · Methoden: new, op, seq_no, checksum, prev_hmac, tx_id
- `struct CommitContext` { tx_id, origin, durable } — Context describing a committed transaction passed to observers.
- `struct CommittedBatch` { entries, origin } — Batch of committed WAL entries passed to observers.
- `trait WalObserver` (requires_durability, on_commit) — Synchronous, deterministic observer interface for committed WAL batches.
- `struct ObserverRegistry` — Configuration and handle for managing WAL observers with bounded delivery and circuit breaker. · Methoden: is_empty, new, register_observer, deregister_observer, set_max_observer_latency, max_observer_latency, set_clock, dropped_count, dropped_count_for, is_circuit_breaker_open, is_any_circuit_breaker_open, clear_circuit_breaker, notify_with_context, notify
- `struct AsyncObserverAdapter` — Non-blocking, bounded async adapter wrapper for `WalObserver` implementations. · Methoden: new · impl: WalObserver

**`lsm/validate.rs`**

- `fn validate_key(key: &[u8]) -> Result<()>`
- `fn derive_doc_id(key: &[u8]) -> DocId`
- `fn derive_doc_id(key: &[u8]) -> DocId` *[feature: docid-128]*
- `fn validate_value(value: &[u8]) -> Result<()>`

**`manifest/core.rs`**

- `const MANIFEST_HEADER_MAGIC: &[u8; 4] = b"MFMN"`
- `const CURRENT_MANIFEST_VERSION: u8 = 1`
- `struct Manifest` — Append-only Manifest file handle for recording active SSTable set transitions. · Methoden: open, append, append_batch, load, reconstruct_valid_sstables, reconstruct_dead_sstables, extract_high_water_mark, path, rollover, maybe_rollover

**`manifest/entry.rs`**

- `enum ManifestEntry` ∈ {Add, Remove, RollbackComplete, Replace, WalCheckpoint} — An entry in the SSTable manifest log. · Methoden: to_bytes, from_bytes

**`manifest/mod.rs`**

- `const MAX_MANIFEST_ENTRY_SIZE: u32 = 1024 * 1024` — Maximum allowed payload size for a single manifest entry (1 MB).
- `const DEFAULT_ROLLOVER_THRESHOLD_BYTES: u64 = 64 * 1024` — Default file size threshold (64 KB) to trigger MANIFEST rollover.

**`memtable.rs`**

- `fn is_tombstone(seq: u64) -> bool` — Helper function to check if a sequence number has TOMBSTONE_BIT set.
- `fn is_valid_range_bounds(start: Bound<&[u8]>, end: Bound<&[u8]>) -> bool` — Validates whether start and end key range bounds define a non-empty, valid interval (`start <= end`).
- `const RESERVED_PREFIXES: &[&[u8]] = &[ b"__col:", b"__col_idx:", b"__meta:", b"__rel:", b"__idx:", b"__gra` — List of known reserved system key prefixes.
- `struct MemTable` · Methoden: new, shard_for, put, tx_range, rollback, get, get_at_seq, size, is_empty, iter, scan_prefix_into, scan_prefix_into_matching, scan_range_into, scan_range_into_matching…

**`sstable/block_cache.rs`**

- `const BLOCK_CACHE_SHARDS: usize = 64` — Sharded block cache:
- `struct SieveCacheBackend` — Trait abstraction for block cache backend implementations. · impl: BlockCacheBackend
- `trait BlockCacheBackend` (new, get, insert, len, is_empty, contains)
- `struct LruBlockCacheBackend` — Standard strict-LRU block cache backend using `parking_lot::RwLock<LruCache>` with byte-based capacity. · impl: BlockCacheBackend
- `struct QuickCacheBlockCacheBackend` *[feature: block-cache-v2]* — Lock-optimized block cache backend using `quick_cache::sync::Cache` (S3-FIFO / Clock-based eviction) with byte-based capacity. · impl: BlockCacheBackend
- `type BlockCacheShard = LruBlockCacheBackend`
- `type BlockCacheShard = QuickCacheBlockCacheBackend` *[feature: block-cache-v2]*
- `struct BlockCache` · Methoden: new, new_with_shards, shard_idx, get, insert, len, is_empty, contains
- `const SSTABLE_MAGIC_MFSX: u32 = 0x5853_464D` — Binary search for a key index inside a SSTable data block.
- `const SSTABLE_MAGIC_LEGACY: u32 = 0x4D46_5354`
- `fn create_block_cache(capacity_mb: usize) -> Arc<BlockCache>` — Creates a new block cache instance with default shard count.
- `fn create_block_cache_with_shards(capacity_mb: usize, num_shards: usize) -> Arc<BlockCache>` — Creates a new block cache instance with configurable shard count.

**`sstable/block_search.rs`**

- `fn get_entry_at_index( block_data: &[u8], offsets_start: usize, idx: usize, is_v3: bool, ) -> Result<(usize, usize)>` — Entry offset accessor supporting both 16-bit (legacy v0..v2) and 32-bit (v3+) block formats.
- `fn get_entry_key_at_index( block_data: &[u8], offsets_start: usize, idx: usize, is_v3: bool, ) -> Result<&[u8]>`
- `fn binary_search_first_index_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<Option<usize>>`
- `fn binary_search_index_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<std::result::Result<usize, usize>>`
- `fn binary_search_entry_in_block( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<Option<(usize, usize)>>`
- `fn block_binary_search( block_data: &[u8], offsets_start: usize, num_offsets: usize, key: &[u8], is_v3: bool, ) -> Result<Option<usize>>`

**`sstable/bloom.rs`**

- `struct BloomFilter` — SPECCED: Speichereffizienter Bloom-Filter für SSTable-Pre-Checks. · Methoden: new, insert, may_contain, hash_pair, to_bytes, from_bytes

**`sstable/builder.rs`**

- `struct BlockBuilder` — A builder for SSTable data blocks (Format v4 default, supports v3). · Methoden: new, new_with_version, add, current_size, is_empty, build
- `struct SstableMetadata` { first_key, last_key, file_size, min_tx_id, max_tx_id, min_seq, max_seq } — Metadata for an SSTable.
- `struct SstableBuilder` — A builder for creating new SSTables. · Methoden: create, create_with_key_manager, set_format_version, add, finish

**`sstable/reader.rs`**

- `struct SstableReader` { format_version } — A reader for existing SSTables. · Methoden: first_key, last_key, min_tx_id, max_tx_id, file_path, open, open_with_key_manager, get_at, get, lookup_metrics, metadata, stream, iter, scan_prefix…

**`sstable/stream.rs`**

- `struct SstableStream` · Methoden: next, next_entry

**`system_pressure.rs`**

- `const WAL_QUEUE_CRITICAL_THRESHOLD: usize = 500`
- `const WAL_QUEUE_ELEVATED_THRESHOLD: usize = 100`
- `const BLOCKING_UTIL_CRITICAL: f32 = 0.85`
- `const BLOCKING_UTIL_ELEVATED: f32 = 0.60`
- `enum PressureLevel` ∈ {Normal, Elevated, Critical}
- `struct SystemPressure` { wal_queue_depth, blocking_thread_utilization, embedding_queue_depth, pressure_level }
- `struct SystemPressureMonitor` { pressure_rx } · Methoden: new, compute_pressure, run

**`tenant_codec.rs`**

- `struct TenantKeyCodec` — Encodes LSM storage keys with tenant isolation prefixes. · Methoden: new, encode_chunk_key, encode_graph_key, scan_prefix, collection_prefix, decode_tenant_id
- `trait StorageEngineHandle` (get_handle, get_at_seq_handle, get_tracked_handle, get_at_seq_tracked_handle, scan_prefix_tracked_handle, supports_ssi_tracking_handle, put_handle, put_if_absent_handle, put_batch_handle, delete_handle, delete_many_handle, delete_prefix_handle, commit_handle, rollback_handle…) — Internal trait abstraction enabling `TenantScopedStorage` to wrap both `S:
- `struct TenantScopedStorage` — Tenant-aware `StorageEngine` wrapper implementing strict key-level isolation (INV-TENANT-2). · Methoden: new, tenant_id · impl: StorageEngine

**`wal/encode.rs`**

- `enum WalOp` ∈ {Put, Delete, TxEnd} · Methoden: tx_id
- `const WAL_V2_HEADER: [u8; 4] = *b"MFW2"` — Magic header for V2 batch-encrypted WAL files (`b"MFW2"`).
- `const WAL_V3_HEADER: [u8; 4] = *b"MFW3"` — Magic header for V3 WAL files (`b"MFW3"`).
- `enum WalVersion` ∈ {V1, V2, V3}
- `struct PreparedBatch` — A batch of prepared WAL entries bound to a specific HMAC chain. · Methoden: is_empty, len, entries, into_inner, extend
- `struct WalEntry` { op, seq_no, checksum, prev_hmac } — A single entry in the Write-Ahead Log. · Methoden: tx_id, try_new, compute_checksum_v3, compute_checksum_v2, compute_checksum, to_bytes, from_bytes

**`wal/flusher.rs`**

- `struct WalFlusherConfig` { batch_window_micros, queue_capacity } — Configuration for the WAL background flusher actor.

**`wal/hmac.rs`**

- `enum LegacyKeyStatus` ∈ {Standard, LegacyActive} — Status indicating whether a WAL segment or key configuration uses the obfuscated legacy key. · Methoden: is_legacy, is_standard
- `fn migrate_legacy_key(legacy: &[u8], new_key_material: &[u8]) -> Result<[u8; 32]>` *[feature: legacy-wal-key]* — Migrates a legacy integrity key to a new 32-byte key material state.
- `fn migrate_legacy_key(_legacy: &[u8], _new_key_material: &[u8]) -> Result<[u8; 32]>`

**`wal/mod.rs`**

- `fs::fn read(path: P) -> std::io::Result<Vec<u8>>`
- `fs::fn write( path: P, contents: C, ) -> std::io::Result<()>`
- `fs::fn remove_file(path: P) -> std::io::Result<()>`
- `fs::fn rename(from: P, to: Q) -> std::io::Result<()>`
- `fs::fn copy(from: P, to: Q) -> std::io::Result<u64>`
- `fs::fn hard_link(from: P, to: Q) -> std::io::Result<()>`
- `fs::fn try_exists(path: P) -> std::io::Result<bool>`
- `fs::fn set_permissions( path: P, perm: std::fs::Permissions, ) -> std::io::Result<()>`
- `fs::fn metadata(path: P) -> std::io::Result<LoomMetadata>`
- `struct fs::LoomMetadata` · Methoden: len, permissions
- `struct fs::OpenOptions` · Methoden: new, read, write, create, create_new, append, mode, open
- `struct fs::LoomFile` { pos, buf } · Methoden: new, open, write_all, flush, sync_all, set_len, seek, metadata · impl: AsyncRead, AsyncWrite, AsyncSeek
- `type fs::File = LoomFile`
- `struct KeyManager` · Methoden: try_new, derive_file_key, encrypt_auto_nonce, decrypt_auto_nonce, integrity_key
- `const MAX_WAL_SIZE: u64 = 128 * 1024 * 1024` — Maximum WAL size before triggering a flush (128MB).
- `const MAX_WAL_ENTRY_SIZE: u32 = 64 * 1024 * 1024` — Maximum size for a single WAL entry payload (64MB).
- `const DEFAULT_WAL_QUEUE_CAPACITY: usize = 1_024` — Default capacity for the bounded WAL flusher command channel.
- `static FAIL_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)`
- `static DELAY_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)`
- `static DELAY_APPEND_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)`
- `static FAIL_TRUNCATE_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false)` *[feature: fault-injection]*
- `static FAIL_APPEND_AFTER_PARTIAL_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0)` *[feature: fault-injection]*
- `static FAIL_APPEND_PARTIAL_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false)` *[feature: fault-injection]*
- `struct WalConfig` { key_manager, min_wal_version, flusher_config } · Methoden: with_legacy_fallback
- `struct Wal` { truncate_lock } · Methoden: prepare_batch, integrity_key_for_test, legacy_integrity_key_for_test, append_batch, enqueue_append_batch_locked, append_batch_locked, try_append_batch, try_enqueue_append_batch_locked, try_append_batch_locked, scan_entries_with_callback, truncate, rotate_and_seal, find_tx_offset, last_hmac_snapshot…

**`wal/replay.rs`**

- `type WalSeq = u64`
- `trait ReplayProgressSink` (on_entry_replayed)
- `struct NoopReplayProgressSink` · impl: ReplayProgressSink

### K.27 `contextra-sys` — 620 Zeilen (ohne Tests), 6 Dateien [V-Sig]

Low-level unsafe system abstractions and FFI island for Contextra (Ring 0)

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `acl_win32.rs` | 325 | — |
| `mlock.rs` | 50 | Low-level memory locking abstraction for Unix/Windows target architectures. |
| `mmap.rs` | 40 | — |
| `posix.rs` | 94 | POSIX system call abstractions for test and low-level I/O operations. |
| `vault.rs` | 95 | — |

**`acl_win32.rs`**

- `fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>`
- `fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>`
- `fn verify_file_acl_owner_only(path: &Path) -> std::io::Result<()>`
- `fn set_restrictive_file_acl(path: &Path) -> std::io::Result<()>`

**`mlock.rs`**

- `fn mem_lock(ptr: *const u8, len: usize) -> bool` — Locks a memory region in RAM to prevent swapping (best-effort).
- `fn mem_unlock(addr: usize, len: usize)` — Unlocks a previously locked memory region.
- `fn mem_lock(_ptr: *const u8, _len: usize) -> bool`
- `fn mem_unlock(_addr: usize, _len: usize)`

**`mmap.rs`**

- `fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap>` — Map a file into memory read-only using `memmap2`.

**`posix.rs`**

- `fn reopen_and_dup2(path: &Path, target_fd: i32, flags: i32) -> std::io::Result<()>` — Opens `path` with `flags` (e.g O_RDONLY or O_RDWR\|O_APPEND), then replaces `target_fd` with the new file descriptor via `dup2`.
- `fn reopen_and_dup2(_path: &Path, _target_fd: i32, _flags: i32) -> std::io::Result<()>`

**`vault.rs`**

- `struct LockedRegions` — Encapsulates locked memory regions (addresses and lengths) for sensitive vault buffers. · Methoden: new, lock_slice, unlock_all, len, is_empty

### K.28 `contextra-testkit` — 788 Zeilen (ohne Tests), 5 Dateien [V-Sig]

Deterministic test utilities, ManualClock, InMemoryStorageEngine, and FaultVfs for Contextra

**Workspace-Abhängigkeiten:** `contextra-ports`, `contextra-types`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `fault_vfs.rs` | 190 | — |
| `in_memory_store.rs` | 251 | — |
| `manual_clock.rs` | 110 | — |
| `reference_model.rs` | 220 | — |

**`fault_vfs.rs`**

- `struct FaultConfig` { fail_writes_after, fail_syncs_after, fail_reads_after, fail_all } — Configuration for fault injection.
- `struct FaultVfs` — Simulated in-memory file system with deterministic fault injection capabilities. · Methoden: new, set_config, trigger_crash, reset_counters, write_file, read_file, sync, write_count, read_count, sync_count

**`in_memory_store.rs`**

- `struct InMemoryStorageEngine` — An in-memory, thread-safe implementation of `StorageEngine` for tests. · Methoden: new · impl: StorageEngine

**`manual_clock.rs`**

- `struct ManualClock` — A thread-safe, manually advanceable clock for deterministic tests. · Methoden: new, advance, set_nanos, now_nanos, now_secs_f64, now_system_time · impl: Clock

**`reference_model.rs`**

- `enum RefOp` ∈ {Put, Delete} — Operation for batch execution on the reference model.
- `struct ReferenceModel` — Deterministic, std-only MVCC reference model for Contextra store verification. · Methoden: new, put, delete, rollback, commit, commit_batch, snapshot_seq, get_at, scan_prefix_at, scan_range_at, get_latest, scan_prefix_latest, snapshot_map_at

### K.29 `contextra-text` — 4.347 Zeilen (ohne Tests), 19 Dateien [V-Sig]

Contextra — Text processing and BM25 search for Hybrid Search

**Features:** default = `—`; weitere: `bm25f`, `docid-128`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-ports`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `bm25.rs` | 660 | — |
| `domain/legal_de.rs` | 74 | — |
| `domain/medical_de.rs` | 80 | — |
| `domain/mod.rs` | 78 | Domänenspezifische Vokabularpakete für die erweiterte Morphologie. |
| `inverted/index_struct.rs` | 666 | — |
| `inverted/morph_index.rs` | 82 | — |
| `inverted/types.rs` | 49 | — |
| `lib.rs` | 267 | — |
| `morphology/compound_splitter.rs` | 266 | — |
| `morphology/metrics.rs` | 23 | — |
| `morphology/passthrough.rs` | 24 | — |
| `morphology/stopwords.rs` | 52 | — |
| `morphology/trie.rs` | 57 | — |
| `posting_list.rs` | 506 | — |
| `stream.rs` | 147 | — |
| `tokenizer.rs` | 437 | — |
| `wand.rs` | 829 | — |

**`bm25.rs`**

- `const BM25_K1: f32 = 1.5` — Default k1 parameter for BM25 term frequency saturation scaling.
- `const BM25_B: f32 = 0.75` — Default b parameter for BM25 document length normalization penalty tuning.
- `struct BM25` { k1, b } — BM25 scoring model parameters. · Methoden: new, score_term
- `fn score_term(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` — Calculates the BM25 score for a single term in a document using default `BM25_K1` and `BM25_B`.
- `fn score_term_with_params( tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32, k1: f32, b: f32, ) -> f32` — Calculates the BM25 score for a single term in a document using custom `k1` and `b` parameters.
- `type FieldId = u32` — Field identifier for field-weighted BM25F scoring.
- `struct FieldWeight` { field_id, weight, b } — Field weighting configuration for BM25F scoring. · Methoden: new
- `struct BM25F` { k1, field_weights } — BM25F field-weighted scoring model configuration (§11.3). · Methoden: new, score_term
- `fn score_term_bm25f( field_term_frequencies: &[(FieldId, u32, u32, f32)], field_weights: &[(FieldId, f32, f32)], k1: f32, df: u32, n: u32, ) -> f32` — Calculates the field-weighted BM25F score for a single term across document fields.

**`domain/legal_de.rs`**

- `struct LegalDomainVocabulary` — German legal domain vocabulary (*Deutsches Juristisches Fachvokabular*). · impl: DomainVocabulary
- `const LEGAL_COMPOUND_STEMS: &[&str] = &[ "Akte", "Antragsteller", "Anwalt", "Anwaltschaft", "Auflagenerteilu` — Domain-specific compound stems for German legal language.
- `const LEGAL_PROTECTED_TERMS: &[&str] = &[ "Akte", "Eid", "Frist", "Gut", "Jura", "Klage", "Notar", "Recht", "` — Legal domain terms that must NOT be treated as stopwords.

**`domain/medical_de.rs`**

- `struct MedicalDomainVocabulary` — German medical domain vocabulary (*Deutsches Medizinisches Fachvokabular*). · impl: DomainVocabulary
- `const MEDICAL_COMPOUND_STEMS: &[&str] = &[ "Allergie", "Anamnese", "Arzt", "Attest", "Aufklärung", "Befund", "` — Domain-specific compound stems for German medical language.
- `const MEDICAL_PROTECTED_TERMS: &[&str] = &[ "Arzt", "Dosis", "EKG", "Labor", "OP", "Reha", "Virus", "Wunde", ]` — Medical domain terms that must NOT be treated as stopwords.

**`domain/mod.rs`**

- `trait DomainVocabulary` (compound_stems, protected_terms, domain_id) — Trait für domänenspezifische Vokabularpakete.

**`inverted/index_struct.rs`**

- `struct InvertedIndex` — An inverted index tied to a specific collection namespace. · Methoden: new_with_language, new, with_tokenizer, load_stats, upsert_document, resolve_tombstones, delete_document, search_bm25, search_bm25_at · impl: TextIndex

**`inverted/morph_index.rs`**

- `struct BM25MorphIndex` — An inverted index with morphological optimization. · Methoden: new, tokenizer · impl: TextIndex

**`inverted/types.rs`**

- `enum Language` ∈ {English, German, Custom} — Supported tokenizer languages for BM25 text indexing. · Methoden: from_iso
- `struct TextIndexMetadata` { total_docs, total_tokens, avg_doc_len_x1000 } — Consolidated metadata for the text index.

**`lib.rs`**

- `struct Bm25Scorer` — Evaluates keyword weights and applies standard BM25 logic. · Methoden: new · impl: TextIndex

**`morphology/compound_splitter.rs`**

- `trait MorphologicalTokenizer` (decompose, language) — Trait for morphological tokenization.
- `struct GermanCompoundSplitter` — German compound word splitter (*Komposita-Zerleger*). · Methoden: new, with_min_length, with_dictionary, min_component_len · impl: MorphologicalTokenizer

**`morphology/metrics.rs`**

- `struct TokenReductionMetrics` { original_tokens, decomposed_tokens } — Metrics for measuring token reduction effectiveness. · Methoden: expansion_ratio

**`morphology/passthrough.rs`**

- `struct PassthroughTokenizer` — Passthrough tokenizer for languages without compound words. · Methoden: new · impl: MorphologicalTokenizer

**`morphology/stopwords.rs`**

- `fn get_german_stopwords() -> &'static HashSet<String>` — Returns the static German stopword list.
- `fn is_german_stopword(word: &str) -> bool` — Checks if a normalized lowercased word is a German stopword.
- `fn normalize_umlauts(input: &str) -> String` — Normalisiert deutsche Umlaute für robusten Suchabgleich.

**`morphology/trie.rs`**

- `struct TrieNode` { is_terminal, children } — A prefix trie node for fast lookup and prefix matching of German dictionary words.
- `struct Trie` — A prefix trie data structure for dictionary lookup. · Methoden: new, insert, contains, starts_with

**`posting_list.rs`**

- `const POSTING_LIST_V2_MAGIC: &[u8; 4] = b"PL\x02\x00"`
- `const BLOCK_SIZE: usize = 64` — Block size for Block-Max WAND decomposition.
- `type DocIdRaw = u64`
- `type DocIdRaw = u128` *[feature: docid-128]*
- `struct PostingBlockInfo` { max_doc_id, max_tf, min_doc_len } — Block-Max metadata for a chunk of postings (typically 64 postings).
- `struct Posting` { doc_id, tf, doc_len } — Compact representation of a single posting in a posting list. · Methoden: new, doc_id
- `struct PostingList` — A contiguous, sorted sequence of postings for a specific term. · Methoden: new, empty, as_slice, blocks, len, is_empty, upsert, remove, encode_compact, decode_compact
- `struct ResidentPostingIndex` — In-Memory resident index mapping terms to their posting lists. · Methoden: new, get, insert_list, upsert_posting, remove_posting_from_terms, remove_terms, clear

**`stream.rs`**

- `const DEFAULT_STREAM_BATCH_SIZE: usize = 16` — Default batch size for candidate retrieval.
- `struct Bm25CandidateStream` — On-demand loading async pull-stream for BM25 search candidates. · Methoden: new, with_batch_size, next_batch, next_doc, yielded, is_exhausted, max_depth

**`tokenizer.rs`**

- `trait Tokenizer` (tokenize) — Tokenizer trait for different language-specific or morphological strategies.
- `struct DefaultTokenizer` — Default tokenizer using Unicode word boundaries and generic stopwords. · impl: Tokenizer
- `struct GermanMorphTokenizer` — German tokenizer with morphological compound splitting. · Methoden: new · impl: Tokenizer
- `fn tokenize(text: &str) -> Vec<String>` — Tokenizes text into lowercase words using Unicode word boundaries and filters stopwords.

**`wand.rs`**

- `struct BoundedTopK` · Methoden: new, push, min_threshold, into_sorted_vec
- `struct WandSearchResult` { results, evaluated_docs_count } — Result from Block-Max WAND search containing top-k items and execution metrics.
- `fn block_max_wand_search( storage: &S, terms: &[String], resident_index: &crate::posting_list::ResidentPostingIndex, prefix_helper: impl Fn(&str) -> Vec<u8>, tombstone_ke` — Executes Block-Max WAND traversal over resident posting lists.

### K.30 `contextra-types` — 5.610 Zeilen (ohne Tests), 17 Dateien [V-Sig]

Canonical domain types, IDs, budgets, filters, and error types for Contextra

**Features:** default = `—`; weitere: `docid-128`, `auto-extraction-opt-out`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `error.rs` | 1116 | Error types for `Contextra`. |
| `error_dto.rs` | 580 | Serializable DTO representation of `ContextraError` for IPC and FFI boundaries (ADR-028). |
| `lib.rs` | 30 | `Contextra` Types — Canonical domain models, IDs, budgets, filters, and error types. |
| `model_fingerprint.rs` | 54 | — |
| `retrieval_strategy.rs` | 87 | Semantische Retrieval-Strategie (Spec §9.5). |
| `schema.rs` | 103 | Schema Versioning structures and utilities for Contextra SSTable/WAL Manifest transitions (ADR-082). |
| `tenant_scope.rs` | 64 | Type-level tenant scope binding and enforcement primitives. |
| `tombstone.rs` | 132 | Trait and reference implementation for tombstone semantics checking in Contextra sequence logs and indexes. |
| `types.rs` | 19 | Core domain, budget, filter, importance, and search query data types. |
| `types/budget.rs` | 692 | Resource budget management for Contextra. |
| `types/domain/document.rs` | 220 | — |
| `types/domain/ids.rs` | 465 | — |
| `types/domain/misc.rs` | 569 | — |
| `types/domain/mod.rs` | 28 | Domain types for Contextra. |
| `types/filter.rs` | 418 | Metadata filter expression DSL and evaluation engine. |
| `types/importance.rs` | 311 | Memory Importance scoring and Recency-Decay functionality. |
| `types/saos.rs` | 722 | SAOS (Synthesized Agent Operating System) types and query abstractions. |

**`error.rs`**

- `enum HnswDeletionError` ∈ {NodeNotFound, DisconnectedComponent, VerificationFailed} — Errors that can occur during HNSW graph deletion and repair operations.
- `struct StepId` · Methoden: new, inner
- `enum CacheDirective` ∈ {Pin, NeverCache, Auto, ReleaseAfterStep} — Declarative agent directive controlling segment caching, pinning, and eviction lifecycle.
- `type Result = std::result::Result<T, ContextraError>` — Convenience alias for `Result<T, ContextraError>`.
- `enum ContextraError` ∈ {Internal, InvalidInput, NotFound, PolicyViolation, Storage, Io, WalCorruption, ChecksumMismatch, Transaction, TransactionTimeout, Conflict, InvalidSequenceNumber, Index, EmbeddingDimensionMismatch…} — Unified error type for all `Contextra` operations across the entire workspace. · Methoden: limit_exceeded, model_load, snapshot_unsupported_for_signal, orphaned_vector_reference, capability_unsupported, invalid_input, wal_corruption, checksum_mismatch, kv_quantization, durability_config, pin_budget_exceeded, cross_device_link, wal_truncation_detected, is_occ_conflict

**`error_dto.rs`**

- `struct ContextraErrorDto` { kind, message, details } — Serializable data transfer object representing a [`ContextraError`]. · Methoden: new, with_details · impl: Error

**`model_fingerprint.rs`**

- `struct ModelFingerprint` { hash, model_id, quantization } — Uniquely identifies a model weight file and its quantization tier. · Methoden: new

**`retrieval_strategy.rs`**

- `enum RetrievalStrategy` ∈ {Vector, Text, Graph, Hybrid, Global} — Semantische Retrieval-Strategie für Signal-Fusion (§8/§9).

**`schema.rs`**

- `enum DocIdWidth` ∈ {Bit64, Bit128} — Indicates the DocId bit-width expected or produced by a schema version. · Methoden: current, bytes_len
- `enum ManifestSchemaVersion` ∈ {V1, V2} · Methoden: doc_id_width, as_u8, from_u8, is_compatible_with_current_build

**`tenant_scope.rs`**

- `struct TenantScoped` — Ein Wrapper-Typ, der einen Wert untrennbar an eine `TenantId` bindet. · Methoden: new, tenant_id, into_inner_checked, map
- `enum TenantScopeViolation` ∈ {Mismatch} — Errors raised when attempting an illegal or mismatched tenant scope unpacking operation.

**`tombstone.rs`**

- `trait TombstoneSemanticsCheck` (is_tombstone, make_tombstone) — Trait defining tombstone semantics for index sequence numbers and key-value records.
- `struct SeqBitTombstone` — Default reference implementation of [`TombstoneSemanticsCheck`]. · impl: TombstoneSemanticsCheck

**`types/budget.rs`**

- `enum BudgetStrategy` ∈ {Conservative, Aggressive, Exact} — Strategie für Token-Budget-Management.
- `struct TokenBudget` { limit, strategy, reserved } — Token budget configuration for LLM context management. · Methoden: new, for_model, with_reserved, with_strategy, effective_limit, available, consume, reserve, try_reserve, refund, consumed
- `struct Reservation` — RAII reservation structure for guaranteed atomicity & drop-refund against double-spend. · Methoden: settle
- `struct ResourceBudget` { memory_limit } — Resource budget for memory management.
- `struct ResourceTracker` — Tracks resource usage against a budget. · Methoden: new, consume_memory, release_memory, memory_used, budget, has_memory_capacity

**`types/domain/document.rs`**

- `enum DistanceMetric` ∈ {Cosine, Euclidean, DotProduct} — Distance metric for vector comparison. · Methoden: compute, compute_u8
- `struct Embedding` { data } — Vector embedding representation. · Methoden: new, dim, as_slice, l2_norm, normalize
- `struct RerankResult` { original_index, score } — Result from cross-encoder post-retrieval reranking.
- `struct ScoredDocument` { doc_id, score } — A scored search result. · Methoden: new

**`types/domain/ids.rs`**

- `struct TenantId` — LAYER-BEGRÜNDUNG: · Methoden: new, try_new, inner, as_u64, is_system · impl: TryFrom
- `struct CollectionId` — Internal collection identifier. · Methoden: new, try_new, inner, as_u64
- `struct DocId` — Internal document identifier. · Methoden: new, inner, as_u64, from_key
- `struct DocId` *[feature: docid-128]* — Internal document identifier (128-bit variant).
- `struct EntityId` — Internal entity identifier for graph nodes. · Methoden: new, inner, as_u64, as_bytes, from_doc_id, from_key
- `struct TxId` — Transaction identifier. · Methoden: new, inner, as_u64, internal, try_from_internal_offset, is_valid_origin

**`types/domain/misc.rs`**

- `hex::fn serialize(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>`
- `hex::fn deserialize(deserializer: D) -> Result<[u8; 32], D::Error>`
- `struct RerankResult` { original_index, score } — Result of a cross-encoder reranking operation for a candidate.
- `struct WorkflowState` { tx, graph_hash } — Defines a frozen workflow state acting as a savepoint.
- `const TOMBSTONE_BIT: u64 = 1 << 63` — Bit mask for identifying tombstones in sequence numbers.
- `const EXPIRY_METADATA_KEY: &str = "__expires_at_seq"` — Reserved metadata key for sequence-based document TTL expiration.
- `const MAX_SEARCH_K: usize = 1_000` — Maximum number of search results that any hybrid/vector/text search may return.
- `enum LinkRelation` ∈ {Elaborates, Contradicts, Supersedes, References} — Relationship types between Zettelkasten memory chunks (A-MEM).
- `struct MemoryLink` { target, relation, created_at_tx } — A directional link to another memory chunk in the Zettelkasten.
- `struct Entity` { id, name, entity_type, attributes } — Represents a canonical node in the knowledge graph. · Methoden: new, try_new
- `struct Edge` { from, to, label, weight, tx_valid_from, tx_valid_to, business_valid_from, business_valid_to, source_doc_id } — Graph directed edge representation with explicit bitemporal axis separation. · Methoden: new, with_source_doc_id, try_new, with_weight, with_tx_validity, with_business_validity, with_validity
- `enum MemoryType` ∈ {Episodic, Semantic, Procedural, Working} — Klassifiziert den kognitiven Gedächtnistyp einer gespeicherten Einheit. · Methoden: default_decay, default_ttl_tx, as_metadata_key
- `struct TlHfdParams` { sigma, delta, gamma, max_iterations, max_top_k_expansion, max_hyperedge_sort_size } — Configuration parameters for Thresholded Local Hyper-Flow Diffusion (TL-HFD, Spec §21.1).
- `enum PprAlgorithm` ∈ {Auto, DensePowerIteration, ForwardPush, ShadowMode, TlHfd, ShadowModeTlHfd} — Selection of algorithm strategy for Personalized PageRank (PPR).
- `struct PprConfig` { damping_factor, max_iterations, convergence_epsilon, algorithm, warn_on_non_convergence } — Configuration parameters for Personalized PageRank (PPR).
- `struct ConfigFingerprint` { model_id, quantization, prompt_template_hash, temperature_bits, threshold_bits } — Konfigurations-Fingerabdruck für P8-Kalibrierungs-Integrität. · Methoden: new, with_threshold, threshold, temperature
- `enum AutoExtractionMode` ∈ {Enabled, Disabled} — Mode for automatic OpenIE entity extraction during document ingestion. · Methoden: is_enabled

**`types/filter.rs`**

- `enum FilterExpr` ∈ {Eq, Ne, Gt, Gte, Lt, Lte, In, NotIn, Exists, And, Or, Not} — Metadata filter expressions for pre/post filtering. · Methoden: evaluate

**`types/importance.rs`**

- `struct ImportanceScore` — Scores the importance of a memory item on a normalized scale of `0.0` to `1.0`. · Methoden: new, value
- `enum DecayFunction` ∈ {None, Exponential, StepFloor, WallClockExponential} — Recency-decay mathematical function for episodic relevance. · Methoden: decay_factor, decay_factor_wallclock
- `struct MemoryImportance` { base_score, decay, created_at_tx } — Tracks the importance score and decay parameters of a document. · Methoden: new, effective_score

**`types/saos.rs`**

- `enum FusionStrategy` ∈ {Rrf, ScoreNormalized} — Strategy used for search result signal fusion.
- `enum GraphTraversalStrategy` ∈ {Hops, PersonalizedPageRank, PathRag} — Strategy used for graph retrieval in hybrid search queries.
- `struct FusionWeights` — Normalized fusion weights for hybrid search. · Methoden: new, vector, text, graph, metadata
- `struct ContextChunk` { doc_id, content, relevance, token_count, metadata, contextual_prefix, links } — A chunk of context for LLM budget allocation. · Methoden: combined_text_owned, combined_token_count, has_context_prefix
- `struct ContextWindow` { chunks, total_tokens, truncated } — An aggregated context window constrained by a token budget.
- `struct RerankResult` { original_index, score } — Result of a cross-encoder reranking operation for a candidate document.
- `struct ScoredEntry` { id, final_score, metadata } — Evaluated result for hybrid/4-signal search.
- `struct HybridQuery` { text_query, vector_query, graph_start_node, graph_strategy, fusion_weights, fusion_strategy, filter, same_community_as, memory_type_filter, include_superseded… } — A unified query traversing multiple index signals. · Methoden: builder
- `struct HybridQueryBuilder` — Builder for HybridQuery to improve DX. · Methoden: new, with_text_query, with_vector_query, with_graph_start_node, with_graph_strategy, with_fusion_weights, with_fusion_strategy, with_filter, with_same_community_as, with_memory_type_filter, with_include_superseded, with_include_provenance, with_rerank_pool_multiplier, with_rerank_pool_max…

### K.31 `contextra-vector` — 10.583 Zeilen (ohne Tests), 37 Dateien [V-Sig]

HNSW vector index with SIMD distance computation for Contextra

**Features:** default = `—`; weitere: `docid-128`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`, `partial-index-rebuild`

**Workspace-Abhängigkeiten:** `contextra-types`, `contextra-core`, `contextra-crypto`, `contextra-simd`, `contextra-sys`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `acorn/gamma_augmentation.rs` | 54 | — |
| `acorn/mod.rs` | 72 | — |
| `acorn/naive_reference.rs` | 90 | — |
| `candidate_stream.rs` | 170 | — |
| `compute_pool.rs` | 166 | — |
| `diskann/config.rs` | 79 | — |
| `diskann/types.rs` | 142 | — |
| `hnsw/adaptive_ef.rs` | 230 | — |
| `hnsw/arena.rs` | 370 | — |
| `hnsw/batch.rs` | 90 | — |
| `hnsw/config.rs` | 184 | — |
| `hnsw/deletion.rs` | 354 | — |
| `hnsw/sq8_bias.rs` | 109 | — |
| `hnsw/types.rs` | 727 | — |
| `partial_rebuild.rs` | 364 | — |
| `persistence/header.rs` | 438 | — |
| `persistence/mmap.rs` | 219 | — |
| `persistence/node.rs` | 83 | — |
| `quantize/mod.rs` | 471 | — |
| `quantize_rabitq.rs` | 439 | — |

**`acorn/gamma_augmentation.rs`**

- `fn compute_gamma_edge_budget(base_degree: usize, predicate_selectivity: f32) -> usize` — Computes the augmented edge budget factor $\gamma$ for graph expansion in ACORN search.

**`acorn/mod.rs`**

- `trait FilteredIndex` (search_knn_acorn) — Prädikatsagnostischer Suchpfad (ACORN-Muster, Patel et al.
- `enum AcornError` ∈ {DimensionMismatch, InvalidK, InvalidGamma, Internal} — Errors occurring during ACORN search or graph traversal operations. · impl: Error

**`acorn/naive_reference.rs`**

- `struct NaiveReferenceIndex` — Naive reference index performing brute-force search over all stored points. · Methoden: new, from_vectors, insert · impl: FilteredIndex

**`candidate_stream.rs`**

- `const DEFAULT_VECTOR_STREAM_BATCH_SIZE: usize = 16` — Default batch size for vector candidate retrieval.
- `struct VectorCandidateStream` — On-demand loading async pull-stream for vector search candidates. · Methoden: new, with_batch_size, next_batch, next_doc, yielded, fetched_depth, is_exhausted, max_depth

**`compute_pool.rs`**

- `struct TaskHandle` — Handle to await the outcome of a task submitted to [`ComputePool`]. · Methoden: join
- `struct ComputePool` — Bounded compute thread pool for background index rebuilds and heavy CPU tasks. · Methoden: new, max_workers, active_jobs, execute, spawn

**`diskann/config.rs`**

- `enum DiskAnnFallbackPolicy` ∈ {UseHnswOnFailure, FailFast} — Fallback behavior policy when DiskANN loading, integrity checks, or reads fail.
- `struct DiskAnnConfig` { index_path, dimension, max_degree, beam_width, sector_size, memory_budget, distance_metric, quantize, fallback_policy, pending_flush_threshold… } — Configuration for DiskANN index.

**`diskann/types.rs`**

- `struct DiskAnnIndex` — DiskANN out-of-core vector index. · Methoden: recover_pending_delta, recover_pending_delta_sync, build_to_path_sync, build, build_sync, trigger_background_persist_delta, persist_delta, persist_delta_sync, load, load_sync, search_predicate_augmented, search_internal, try_new, len_sync · impl: VectorIndex

**`hnsw/adaptive_ef.rs`**

- `struct AdaptiveEfPolicy` { min_ef, max_ef, growth_factor, convergence_window, stability_threshold } — Configuration policy for adaptive ef_search (Ada-ef). · Methoden: new, validate
- `struct AdaptiveEfStats` { final_ef, rounds, converged } — Execution statistics for an adaptive search run.
- `struct AdaptiveEfStateMachine` — Deterministic state machine managing Ada-ef expansion rounds. · Methoden: new, current_ef, round_count, is_terminated, stats, step

**`hnsw/arena.rs`**

- `const ARENA_ALIGNMENT_BYTES: usize = 64` — Slot alignment in bytes (64-byte alignment = 16 u32 elements for SIMD cache-line compatibility).
- `const ARENA_ALIGNMENT_U32: usize = ARENA_ALIGNMENT_BYTES / std::mem::size_of::<u32>()` — Alignment in u32 elements (64 bytes / 4 bytes = 16 u32 elements).
- `type BacklinkKey = (usize, usize)` — Composite key (neighbor_ram_idx, layer) for backlink lookups.
- `struct BacklinkTable` { map } — Thread-safe $O(1)$ backlink connection table using `AHashMap`. · Methoden: new, get, insert, clear
- `struct HnswArena` { arena, offsets, capacities, count_offsets, counts, free_list, total_allocated } — Contiguous 64-byte aligned arena allocator for HNSW node connections with free-list slot reuse. · Methoden: new, layer_offset, align_capacity, allocate_node, get_ram_node_connections, update_backlink, try_relink_pruned_neighbors, free_node

**`hnsw/batch.rs`**

- `struct PreparedInsert` { doc_id, vector_data, new_layer, new_idx, final_connections, neighbor_backlinks, should_update_entry_point, should_update_ram_entry_point }
- `struct NeighborBacklink` { neighbor_ram_idx, layer, updated_connections } — Back-link connection update for a neighbor node at a specific layer.
- `struct BatchContext` { running_max_layer, has_entry_point, has_ram_entry_point, backlink_map } — Batch tracking context for running state across multi-operation transaction commits. · Methoden: new

**`hnsw/config.rs`**

- `struct HnswConfig` { dimension, max_elements, m, ef_construction, ef_search, distance_metric, rebuild_threshold, quantize, quantizer_recalibration_sample_size, quantizer_drift_threshold… } · Methoden: validate
- `struct HnswConfigBuilder` — Builder for HnswConfig with resource limit enforcements to prevent OOM. · Methoden: new, max_elements, m, ef_construction, ef_search, distance_metric, quantize, quantizer_recalibration_sample_size, quantizer_drift_threshold, rebuild_threshold, partial_rebuild_config, build

**`hnsw/deletion.rs`**

- `struct DeletionStats` { doc_id, repaired_edges, orphaned_replacements, verified_no_ghost_pointers } — Statistics returned after a ghost-free node deletion and neighborhood repair.
- `trait GhostFreeVectorIndex` (remove_with_graph_repair) — Trait providing ghost-free vector index deletion with synchronous neighborhood graph repair.

**`hnsw/sq8_bias.rs`**

- `struct Sq8Bias` { mean_bias, variance_bias, sample_count } — Measured quantization bias and variance statistics for SQ8 distance calculations. · Methoden: new, calibrate

**`hnsw/types.rs`**

- `const SENTINEL_NO_ENTRY_POINT: u32 = u32::MAX` — Sentinel-Wert für ungültigen / nicht gesetzten Entry-Point in HNSW.
- `const HNSW_REBUILD_DELETION_RATIO: f64 = 0.10` — Standard-Löschanteil (0.10 = 10 % gelöschte Knoten), ab dem ein Rebuild getriggert wird.
- `enum VectorData` ∈ {F32, U8}
- `struct HnswNode` — A node in the HNSW graph.
- `struct Candidate` { index, distance } — Search candidate.
- `struct RebuildGuard`
- `struct SnapshotPinGuard`
- `struct HnswIndex` — The HNSW (Hierarchical Navigable Small World) vector index. · Methoden: search_adaptive, search_adaptive_filtered, search_adaptive_with_stats, try_new, new, quantizer, sq8_bias, deleted_ratio, rebuild_count, visited_dead_nodes, connectivity_score, check_connectivity, is_rebuild_required, rebuild… · impl: FilteredIndex, GhostFreeVectorIndex, VectorIndex
- `struct HnswHotCore` { nodes, doc_to_node, doc_to_node, entry_point, ram_entry_point, max_layer, ml, deleted_count, write_mutex, last_tx_id… } · Methoden: get_entry_point, set_entry_point, get_ram_entry_point, set_ram_entry_point, get_ram_node_connections
- `struct HnswColdCore` { config, validation_error, tx_buffer, quantizer, sq8_bias, mmap_index, seq_log, rebuild_count, visited_dead_nodes, deleted_nodes… }
- `struct HnswIndexCore` { hot, cold } — The core implementation of the HNSW index. · Methoden: compute_insert, compute_insert_with_context, apply_insert, connectivity_score, check_connectivity, rebuild_status, wait_for_rebuild, wait_for_rebuild_with_timeout, deleted_ratio, rebuild_count, visited_dead_nodes, is_rebuild_required, rebuild, rebuild_sync…
- `enum RebuildStatus` ∈ {Idle, Running, Pending}

**`partial_rebuild.rs`**

- `struct PartialRebuildConfig` { critical_ratio, traversal_window, min_global_ratio } — Konfiguration für den Partial-Rebuild-Trigger.
- `struct TraversalTracker` — Ringpuffer für besuchte HNSW-Knoten-IDs während ef_search-Traversierungen. · Methoden: new, record_traversal, find_oversaturated_regions, hot_path_nodes
- `fn should_trigger_partial_rebuild( tracker: &TraversalTracker, tombstone_map: &HashMap<u64, bool>, global_tombstone_ratio: f32, config: &PartialRebuildConfig, ) -> Option` *[feature: partial-index-rebuild]* — Entscheidet ob ein lokaler Partial-Rebuild ausgelöst werden soll.

**`persistence/header.rs`**

- `const HNSW_MAGIC: u32 = 0x484E5357` — Magic number for HNSW files (0x484E5357 = "HNSW").
- `const HNSW_VERSION: u16 = 2` — Current file format version.
- `struct HnswHeader` — The header of an HNSW persistent file. · Methoden: new, new_v2, new_v2_with_bias, magic, version, m, metric, quantized, q_min, q_max, nodes_offset, connections_offset, set_connections_offset, node_count…

**`persistence/mmap.rs`**

- `struct MmapIndex` { mmap, header } — A reader for memory-mapped HNSW indices. · Methoden: open, open_async, get_node_record, get_vector, get_connections

**`persistence/node.rs`**

- `struct NodeRecord` { doc_id, doc_id, max_layer, vector_offset, connections_offset } — Represents a node's metadata in the flat file. · Methoden: from_bytes, to_bytes

**`quantize/mod.rs`**

- `const DEFAULT_P_LOW: f32 = 0.005` — Default low percentile for quantization scaling (0.5%).
- `const DEFAULT_P_HIGH: f32 = 0.995` — Default high percentile for quantization scaling (99.5%).
- `struct ScalarQuantizer` — An 8-bit Scalar Quantizer (SQ8) with per-dimension scaling. · Methoden: try_train, try_train_with_percentiles, train_with_percentiles, train, mins, maxes, scales, inv_scales, dimension, check_drift, drift_ratio, is_rebuild_required, quantize, dequantize…

**`quantize_rabitq.rs`**

- `struct RaBitQQuantizer` — RaBitQ Quantizer with random orthogonal rotation and scalar metadata. · Methoden: try_train, dimension, code_length, quantize, asymmetric_distance

### K.32 `contextra-wire` — 1.729 Zeilen (ohne Tests), 4 Dateien [V-Sig]

Ring 0 Unsafe Island: Auto-generated FlatBuffers IPC code and zero-copy adapters for Contextra

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `adapter.rs` | 43 | — |
| `contextra_generated.rs` | 1505 | — |
| `jsonrpc.rs` | 152 | — |

**`adapter.rs`**

- `struct WireBuffer` — Zero-copy byte buffer slice wrapper for FlatBuffers messages. · Methoden: new, as_slice, into_inner · impl: AsRef

**`contextra_generated.rs`**

- `enum contextra::ipc::EmbeddingOffset` ∈ {}
- `struct contextra::ipc::Embedding` { _tab } · Methoden: init_from_table, create, data, quantized_u8, metric · impl: Follow, Verifiable
- `struct contextra::ipc::EmbeddingArgs` { data, quantized_u8, metric }
- `struct contextra::ipc::EmbeddingBuilder` · Methoden: add_data, add_quantized_u8, add_metric, new, finish
- `enum contextra::ipc::ScoredDocumentOffset` ∈ {}
- `struct contextra::ipc::ScoredDocument` { _tab } · Methoden: init_from_table, create, id, score, metadata, embedding · impl: Follow, Verifiable
- `struct contextra::ipc::ScoredDocumentArgs` { id, score, metadata, embedding }
- `struct contextra::ipc::ScoredDocumentBuilder` · Methoden: add_id, add_score, add_metadata, add_embedding, new, finish
- `enum contextra::ipc::SearchResponseOffset` ∈ {}
- `struct contextra::ipc::SearchResponse` { _tab } · Methoden: init_from_table, create, results, total_hits, processing_time_ms · impl: Follow, Verifiable
- `struct contextra::ipc::SearchResponseArgs` { results, total_hits, processing_time_ms }
- `struct contextra::ipc::SearchResponseBuilder` · Methoden: add_results, add_total_hits, add_processing_time_ms, new, finish
- `enum contextra::ipc::VectorIndexUpdateOffset` ∈ {}
- `struct contextra::ipc::VectorIndexUpdate` { _tab } · Methoden: init_from_table, create, id, embedding, metadata · impl: Follow, Verifiable
- `struct contextra::ipc::VectorIndexUpdateArgs` { id, embedding, metadata }
- `struct contextra::ipc::VectorIndexUpdateBuilder` · Methoden: add_id, add_embedding, add_metadata, new, finish
- `enum contextra::ipc::RoleIdOffset` ∈ {}
- `struct contextra::ipc::RoleId` { _tab } · Methoden: init_from_table, create, id · impl: Follow, Verifiable
- `struct contextra::ipc::RoleIdArgs` { id }
- `struct contextra::ipc::RoleIdBuilder` · Methoden: add_id, new, finish
- `enum contextra::ipc::HyperEdgeIdOffset` ∈ {}
- `struct contextra::ipc::HyperEdgeId` { _tab } · Methoden: init_from_table, create, id · impl: Follow, Verifiable
- `struct contextra::ipc::HyperEdgeIdArgs` { id }
- `struct contextra::ipc::HyperEdgeIdBuilder` · Methoden: add_id, new, finish
- `enum contextra::ipc::RoleBindingOffset` ∈ {}
- `struct contextra::ipc::RoleBinding` { _tab } · Methoden: init_from_table, create, role, entity · impl: Follow, Verifiable
- `struct contextra::ipc::RoleBindingArgs` { role, entity }
- `struct contextra::ipc::RoleBindingBuilder` · Methoden: add_role, add_entity, new, finish
- `enum contextra::ipc::HyperEdgeOffset` ∈ {}
- `struct contextra::ipc::HyperEdge` { _tab } · Methoden: init_from_table, create, id, predicate, participants, weight, tx_valid_from, tx_valid_to, business_valid_from, business_valid_to, source_doc_id, child_edge_ids · impl: Follow, Verifiable
- `struct contextra::ipc::HyperEdgeArgs` { id, predicate, participants, weight, tx_valid_from, tx_valid_to, business_valid_from, business_valid_to, source_doc_id, child_edge_ids }
- `struct contextra::ipc::HyperEdgeBuilder` · Methoden: add_id, add_predicate, add_participants, add_weight, add_tx_valid_from, add_tx_valid_to, add_business_valid_from, add_business_valid_to, add_source_doc_id, add_child_edge_ids, new, finish
- `contextra::ipc::fn root_as_search_response( buf: &[u8], ) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>` — Verifies that a buffer of bytes contains a `SearchResponse` and returns it.
- `contextra::ipc::fn size_prefixed_root_as_search_response( buf: &[u8], ) -> Result<SearchResponse, flatbuffers::InvalidFlatbuffer>` — Verifies that a buffer of bytes contains a size prefixed `SearchResponse` and returns it.
- `contextra::ipc::fn root_as_search_response_with_opts( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>` — Verifies, with the given options, that a buffer of bytes contains a `SearchResponse` and returns it.
- `contextra::ipc::fn size_prefixed_root_as_search_response_with_opts( opts: &'o flatbuffers::VerifierOptions, buf: &'b [u8], ) -> Result<SearchResponse<'b>, flatbuffers::InvalidFlatbuffer>` — Verifies, with the given verifier options, that a buffer of bytes contains a size prefixed `SearchResponse` and returns it.
- `contextra::ipc::fn root_as_search_response_unchecked(buf: &[u8]) -> SearchResponse` — Assumes, without verification, that a buffer of bytes contains a SearchResponse and returns it.
- `contextra::ipc::fn size_prefixed_root_as_search_response_unchecked( buf: &[u8], ) -> SearchResponse` — Assumes, without verification, that a buffer of bytes contains a size prefixed SearchResponse and returns it.
- `contextra::ipc::fn finish_search_response_buffer( fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>, root: flatbuffers::WIPOffset<SearchResponse<'a>>, )`
- `contextra::ipc::fn finish_size_prefixed_search_response_buffer( fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>, root: flatbuffers::WIPOffset<SearchResponse<'a>>, )`

**`jsonrpc.rs`**

- `struct JsonRpcRequest` { jsonrpc, id, method, params } — Eingehende JSON-RPC 2.0 Nachricht (Request oder Notification).
- `struct JsonRpcResponse` { jsonrpc, id, result, error } — Ausgehende JSON-RPC 2.0 Nachricht. · Methoden: ok, err
- `struct JsonRpcError` { code, message, data } — JSON-RPC 2.0 Fehlerobjekt.

### K.33 `contextra` — 795 Zeilen (ohne Tests), 5 Dateien [V-Sig]

Contextra — Embedded hybrid-search for AI agents (Facade)

**Features:** default = `fast, candle`; weitere: `docid-128`, `sovereign`, `compliance`, `audit-export`, `avv-generator`, `onnx`, `ollama`, `router`

**Workspace-Abhängigkeiten:** `contextra-core`, `contextra-types`, `contextra-ports`, `contextra-license`, `contextra-store`, `contextra-db`, `contextra-crypto`, `contextra-privacy`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-router`, `contextra-rank`

| Datei | Zeilen | Zweck (Modul-Doc) |
|---|---|---|
| `agent_memory.rs` | 214 | — |
| `builder.rs` | 280 | — |
| `collection_profile.rs` | 122 | — |
| `lib.rs` | 90 | — |
| `performance_profile.rs` | 89 | — |

**`agent_memory.rs`**

- `struct MemoryId` — Opaque identifier for a memory stored via `AgentMemory`. · Methoden: new, as_str · impl: AsRef
- `struct RelationId` — Opaque identifier for an n-ary relation (hyperedge) stored via `AgentMemory::relate_n_ary`.
- `struct Memory` { id, score, metadata, matched_signals, provenance } — Representation of a recalled memory record.
- `struct AgentMemory` — High-level facade for agent-oriented memory management (`remember`, `recall`, `forget`, `relate`). · Methoden: new, new_arc, engine, remember, recall, forget, relate, relate_n_ary

**`builder.rs`**

- `struct ContextraBuilder` — A builder for configuring and instantiating `Contextra`. · Methoden: new, with_storage_path, with_max_elements, with_distance_metric, with_encryption_passphrase, with_embedding_backend, with_consolidation, with_embedder, with_license_gate, with_signed_license, with_performance_profile, with_config, build

**`collection_profile.rs`**

- `const DEFAULT_AUTO_EXTRACTION_MODE: AutoExtractionMode = AutoExtractionMode::Enabled` — Default auto extraction mode for standard deployment tiers.
- `const ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT: AutoExtractionMode = AutoExtractionMode::Enabled` — DECISION-PENDING (Spec D.6):
- `struct LsmTuning` { memtable_size_limit, max_ram_mb, group_commit_window_micros, block_cache_shards }
- `struct CollectionProfile` { performance, kv_delete_mode, lsm_tuning, auto_extraction } · Methoden: validate, validate_with_license
- `enum CollectionProfileError` ∈ {Performance, KvDeleteMismatch}
- `enum DeploymentTier` ∈ {EdgeMinimal, PowerUserLocal, EnterpriseShared, EnterpriseRegulated} · Methoden: resolve

**`lib.rs`**

- `fn builder(dimension: usize) -> ContextraBuilder` — Creates a new `ContextraBuilder` for configuring and instantiating `Contextra`.
- `fn open(path: impl AsRef<std::path::Path>) -> Result<Contextra, ContextraError>` — Opens or creates a `Contextra` instance at the given storage path using default configuration and `OpenFastGate`.
- `fn open_with_config( path: impl AsRef<std::path::Path>, config: ContextraConfig, ) -> Result<Contextra, ContextraError>` — Opens or creates a `Contextra` instance at the given storage path with an explicit configuration and `OpenFastGate`.

**`performance_profile.rs`**

- `enum PerformanceProfile` ∈ {Compliance, Balanced, BareMetal} · Methoden: resolve
- `struct ResolvedProfileConfig` { durability_mode, vector_delete_mode, deletion_proof_active, feature_ring } · Methoden: enforce_license, validate
- `enum PerformanceProfileError` ∈ {Durability, DurabilityVectorMismatch, LicenseRequired}

## K.34 MCP-Werkzeuge (aus `TOOL_REGISTRY`, [V-Sig])

| Tool | Kategorie | Pflichtparameter | Zweck |
|---|---|---|---|
| `contextra_search` | DatabaseRead | query | Hybrid semantic search (vector + BM25 + graph) over stored documents. |
| `contextra_insert` | DatabaseWrite | id, text | Store a document (auto-embedding, auto-chunking using MarkdownChunker, ~512 tokens). |
| `contextra_get` | DatabaseRead | id | Retrieve a document by ID. |
| `contextra_forget` | DatabaseWrite | collection, confirm | Delete a document or an entire collection with GDPR DeletionProof export. |
| `contextra_collections` | DatabaseRead | — | List all collections. |
| `contextra_consolidate` | DatabaseWrite | — | Manual, synchronous trigger for an immediate memory consolidation pass (structural consolidation pass and optional synthesis) on the specified collection. Automatic backg |
| `contextra_cloud_query` | CloudEgress | query | Executes an external cloud query under egress classification check and automatic abstraction. |
| `contextra_relate` | DatabaseWrite | from, to, label | Create a directed or bidirectional binary relationship between two documents. Write permissions must be enabled. |
| `contextra_relate_n_ary` | DatabaseWrite | doc_id, role | Create an n-ary hyperedge graph relationship connecting multiple participants with assigned roles. Core Spec §6 feature. Write permissions must be enabled. |
| `contextra_explain` | DatabaseRead | id | Provide a human-readable retrieval explanation and provenance breakdown for a document by ID. |
| `contextra_plugin_status` | DatabaseRead | — | Get status of all active plugins with version, ring, and required feature ring. |
| `contextra_upsert` | DatabaseWrite | id | Insert or update a document idempotently by key. |
| `contextra_delete` | DatabaseWrite | id | Delete a document with HNSW neighborhood graph repair and issue a cryptographic DeletionProof. |
| `contextra_create_collection` | DatabaseWrite | collection | Create a new collection with specified DeploymentTier. |
| `contextra_drop_collection` | DatabaseWrite | collection, confirm | Delete an entire collection and issue a collection-wide cryptographic DeletionProof. |

Kategorien: `DatabaseRead` (immer erlaubt), `DatabaseWrite` (Freigabe nötig, `CONTEXTRA_MCP_ALLOW_WRITE`), `CodeExecution` (gesperrt), `CloudEgress` (gesperrt, Opt-in). Transport ausschließlich stdio (ADR-010).


## K.35 Umgebungsvariablen (Produktivcode und `xtask`, [V-Sig])

Gefunden über `env::var`/`var_os`/`env!` außerhalb von Tests. Laufzeitrelevant sind die `CONTEXTRA_*`-Variablen der Crates `mcp`, `checkpoint`, `py`; `xtask`/CI/Agentenbetrieb sind `CONTEXTRA_CI*`, `CONTEXTRA_CLAIM_CRATE`, `CONTEXTRA_MAX_ACTIVE_CLAIMS`, `CONTEXTRA_PR_BODY`, `CONTEXTRA_SESSION_HASH`, `JULES_*`, `PR_*`, `GITHUB_*`. Offener Punkt B-11 (zwei Variablen für den Löschbeweis-Schlüssel, `CONTEXTRA_DELETION_PROOF_KEY` und `CONTEXTRA_PROOF_KEY`) besteht weiter; beide werden in `mcp/src/server_tools.rs` und `tools_crud.rs` gelesen. Provider-Variablen sind teils ohne Präfix (`EMBEDDING_PROVIDER`, `LLM_PROVIDER`) neben `CONTEXTRA_EMBEDDING_PROVIDER`/`CONTEXTRA_LLM_PROVIDER` vorhanden.

| Variable | Fundstelle(n) |
|---|---|
| `CARGO_MANIFEST_DIR` | `xtask/src/check_ring_capabilities_consistency.rs`, `xtask/src/feature_matrix.rs`, `xtask/src/generate_markers.rs`… |
| `CARGO_WORKSPACE_DIR` | `xtask/src/main.rs` |
| `CONTEXTRA_CANDLE_MODEL_DIR` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_CI` | `xtask/src/check_duplicate_intent.rs`, `xtask/src/jules_preflight.rs` |
| `CONTEXTRA_CI_BASE_REF` | `xtask/src/check_commit_messages.rs`, `xtask/src/check_phantom_files.rs` |
| `CONTEXTRA_CLAIM_CRATE` | `xtask/src/jules_preflight.rs` |
| `CONTEXTRA_DELETION_PROOF_KEY` | `contextra-mcp/src/server_tools.rs`, `contextra-mcp/src/tools_crud.rs` |
| `CONTEXTRA_EMBEDDING_PROVIDER` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_EMBED_MODEL` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_LLM_MODEL` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_LLM_PROVIDER` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_MAX_ACTIVE_CLAIMS` | `xtask/src/claim.rs` |
| `CONTEXTRA_MCP_ALLOW_WRITE` | `contextra-mcp/src/validation.rs` |
| `CONTEXTRA_MCP_IDLE_TIMEOUT_SECS` | `contextra-mcp/src/server.rs` |
| `CONTEXTRA_MCP_INJECTION_CONFIG` | `contextra-mcp/src/prompt_injection/guard.rs` |
| `CONTEXTRA_MCP_PATTERNS_FILE` | `contextra-mcp/src/prompt_injection/guard.rs` |
| `CONTEXTRA_MCP_QUARANTINE_POLICY` | `contextra-mcp/src/prompt_injection/policy.rs` |
| `CONTEXTRA_MCP_SECURITY_LOG` | `contextra-mcp/src/prompt_injection/guard.rs` |
| `CONTEXTRA_OLLAMA_URL` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_ONNX_MODEL_PATH` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_ORPHAN_PATH` | `contextra-checkpoint/src/orphan.rs` |
| `CONTEXTRA_ORPHAN_PIN_PATH` | `contextra-checkpoint/src/orphan.rs` |
| `CONTEXTRA_PROOF_KEY` | `contextra-mcp/src/server_tools.rs`, `contextra-mcp/src/tools_crud.rs` |
| `CONTEXTRA_PR_BODY` | `xtask/src/check_duplicate_intent.rs`, `xtask/src/check_phantom_files.rs` |
| `CONTEXTRA_ROUTER_CALIBRATION_PATH` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_ROUTER_PROFILES_JSON` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_ROUTER_PROFILES_PATH` | `contextra-mcp/src/config.rs` |
| `CONTEXTRA_SESSION_HASH` | `xtask/src/claim.rs`, `xtask/src/context_pack.rs`, `xtask/src/jules_preflight.rs` |
| `CONTEXTRA_WORKER_THREADS` | `contextra-py/src/bindings/runtime_state.rs` |
| `EMBEDDING_PROVIDER` | `contextra-mcp/src/config.rs` |
| `GITHUB_ACTIONS` | `xtask/src/check_duplicate_intent.rs` |
| `GITHUB_STEP_SUMMARY` | `xtask/src/harness/verdict.rs` |
| `GITHUB_TOKEN` | `xtask/src/check_duplicate_intent.rs`, `xtask/src/claim.rs`, `xtask/src/jules_preflight.rs` |
| `HOME` | `contextra-infer-onnx/src/lib.rs`, `xtask/src/check_flatbuffers_drift.rs` |
| `JULES_API_KEY` | `xtask/src/harness/jules_dispatch.rs`, `xtask/src/harness/plan_lint.rs` |
| `JULES_SESSION_ID` | `xtask/src/claim.rs`, `xtask/src/context_pack.rs`, `xtask/src/jules_preflight.rs` |
| `LLM_PROVIDER` | `contextra-mcp/src/config.rs` |
| `OUT_DIR` | `xtask/src/harness/mod.rs` |
| `PR_BODY` | `xtask/src/harness/claims_in_pr.rs`, `xtask/src/harness/diff_budget.rs`, `xtask/src/harness/scope_guard.rs`… |
| `PR_LABELS` | `xtask/src/harness/protected_paths.rs`, `xtask/src/harness/task_card.rs` |
| `USERPROFILE` | `contextra-infer-onnx/src/lib.rs` |

---

# TEIL L — Verhaltensspezifikation bisher unbeschriebener Subsysteme (neu in v15)

Alle Abschnitte: Ablauf im Code gelesen am HEAD `b564e830` **[V]**, sofern nicht **[D]**. Nichts ausgeführt.

## L.1 Crypto-Shredding für KV-Segmente (`contextra-crypto/src/kv_shredding.rs`)

- `DEFAULT_SHRED_KEY_GROUP_SIZE = 64`: 64 Records teilen einen Sub-Key (HKDF-Kosten werden über `group_id` amortisiert).
- `derive_subkey(master_key: &KeyManager, group_id: u64) -> Result<SubKey>`: HKDF-SHA256 (`Hkdf::from_prk` über die Master-Key-Bytes), `info = b"contextra-kv-shred-v1:" ‖ group_id.to_le_bytes()`, Ausgabe 32 Byte. <!-- crate-ref-ignore -->
- `SubKey([u8;32])`: `ZeroizeOnDrop`; `Debug` gibt `***REDACTED***` aus; `Clone` kopiert die Bytes.
- `KeyRegistry` (`RwLock<HashMap<u64, SubKey>>`, thread-sicher, nur im RAM): `get_or_derive(master, group_id)` liest unter Read-Lock, leitet bei Fehlen unter Write-Lock (Double-Check) ab und speichert. `revoke_subkey(group_id) -> bool` entfernt den Eintrag in **O(1)**; danach ist `is_key_active(group_id)` falsch. `encrypt_with_group` verschlüsselt mit dem Sub-Key der Gruppe (AES-256-GCM-SIV).
- **Semantik der Löschung:** Nach `revoke_subkey` leitet ein erneutes `get_or_derive` aus dem Master-Key denselben Schlüssel wieder ab. Crypto-Shredding ist deshalb nur wirksam, wenn zusätzlich der Master-Key bzw. die Gruppenzuordnung vernichtet wird; die Registry allein ist kein unwiderruflicher Löschbeweis. **[V, Folgerung aus dem Code]** Anwender: `store/src/kv/segment.rs` (`revoke_subkey`), Layer `KvCacheSegments` im `DeletionProof` (B.6). INV-KV-DELETE-1 (G.3) gilt nur für `CryptoShred`-Segmente.

## L.2 Manifest und Schutz vor SSTable-Wiederauferstehung (`contextra-store/src/manifest/core.rs`, `lsm/recovery.rs`)

- Datei: Magic `MFMN`, `CURRENT_MANIFEST_VERSION = 1`, append-only; `append` und `append_batch` schreiben, flushen und `fsync`en (Batch atomar).
- `ManifestEntry`-Varianten: `Add`, `Remove{path}`, `Replace{removed, …}`, `RollbackComplete`, `WalCheckpoint` (trägt den WAL-High-Water-Mark-HMAC).
- `Manifest::load(path)`: fehlende Datei → leere Liste. **Tail-Truncation** (unvollständiger Frame am Dateiende, etwa nach Stromausfall) wird toleriert: es werden die bis dahin gültigen Einträge geliefert. **Korruption mitten in der Datei oder CRC-Fehler auf vollständigem Frame → `Err`.**
- `reconstruct_valid_sstables(entries) -> Vec<(PathBuf, rank)>`: geordnete Liste der gültigen SSTables; kleinerer Rang = älter (Shadowing-Reihenfolge). Der Rang ist seit S-01 deprecated (Commit #3938, kontinuierliche Compaction).
- `reconstruct_dead_sstables(entries) -> HashSet<PathBuf>`: Dateinamen (nur `file_name`), die durch `Remove` oder `Replace.removed` als tot belegt sind **und** nicht in der gültigen Menge liegen. `Add`, `RollbackComplete`, `WalCheckpoint` erzeugen keine toten Einträge.
- `extract_high_water_mark(entries) -> Option<[u8;32]>`: jüngster WAL-HWM-HMAC (Schutz gegen WAL-Kürzung, INV-WAL-TRUNCATION-1).
- **Recovery:** `lsm/recovery.rs:196-208` berechnet beim Öffnen `valid_set`, `dead_set` und `hwm`; in `:380` wird ein auf der Platte gefundenes SSTable-Pfadschlüssel gegen `dead_set` geprüft. Laut `docs/FEATURES_SPECIFICATION.md` werden übrig gebliebene `.sst`-Dateien toter Tabellen beim Start gelöscht; die Löschung selbst habe ich nicht im Ablauf gelesen **[D]**. Zweck: verhindert, dass nach einem Absturz mitten in der Compaction bereits ersetzte (und ggf. per Löschbeweis gelöschte) Daten wieder sichtbar werden.

## L.3 Mandantenisolierter KV-Prefix-Store (`contextra-kvcache/src/prefix_store.rs`)

`TenantPrefixKvStore` implementiert den Port `contextra_ports::KvPrefixStore`. Schlüssel: `(TenantId, PrefixKey)`; Partitionen je Mandant sind strukturell getrennt. Bausteine: `PrefixRadixTree`, `KvReusePolicy` (B.8).
- `new()`: Budget `DEFAULT_BYTE_BUDGET_PER_TENANT = 256 MiB`. `with_byte_budget_per_tenant(bytes)`, `with_reuse_policy(policy)` (Builder).
- `lookup(tenant, &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>`: längster übereinstimmender Präfix **in Blöcken**.
- `insert(...)`: legt KV-Blöcke für ein Token-Präfix an; bei Überschreitung des Mandantenbudgets **LRU-Eviction** innerhalb desselben Mandanten.
- `evict(tenant, &PrefixKey) -> Result<u64>`: entfernt Einträge, liefert die Zahl entfernter Blöcke.
- Ein Mandant kann Einträge eines anderen weder finden noch verdrängen (INV-TENANT, G.3).

## L.4 Eviction im KV-Cache (`contextra-kvcache/src/eviction_worker.rs`, `attention_score.rs`)

- **`EvictionWorker`**: Hot-Path-Eviction (Auslöser z. B. VRAM > 80 %). Läuft auf einem **dedizierten OS-Thread** mit `std::sync::mpsc`-Kanal (`EvictionCommand`); synchrone `Zeroize`-Arbeit blockiert damit nie den Tokio-Executor. `spawn(Arc<TenantIsolatedKvStore>)`, `with_attention_exporter(Arc<dyn AttentionExporter>)`.
- **`emergency_wipe()`**: synchron, blockierend, vollständig abgeschlossen vor Rückkehr (Prozess-Shutdown, Sicherheitsalarm). Bewusst anderer Pfad als der Worker.
- **`AttentionScoreSource`** (Ring 1): `importance_score(segment_id) -> Option<f32>` (höher = länger behalten; `None` = unbekannt), `importance_score_for_tenant`, `register_segment`/`unregister_segment` für `(tenant, segment) → request_id`. `NullAttentionScoreSource` = reines LRU. Ein Adapter übersetzt den request-zentrierten Port `AttentionExporter` (Ring 2) in diese segment-zentrierte Schnittstelle.
- **Ranking:** Standardgewichtung **50 % LRU-Alter, 50 % Attention-Score**; das Ergebnis ist eine Liste von Segment-IDs, erstes Element wird zuerst evictiert. Feature `kvcache-attention-eviction`.

## L.5 Egress-Kette (`contextra-privacy`)

Schichten (Nummern wie im Code): **Layer 1** Regex-Klassifikation (`egress_vault.rs`) · **Layer 4** `EgressGuard` (HNSW-Ähnlichkeit, B.7) und `BulkExfiltrationDetector` (pro `SessionId`) · **Layer 5** Inbound-Rehydration (`egress_gateway.rs`). Invariante **APM-EGRESS-BYPASS:** Jede Anfrage MUSS `EgressClassifier::classify()` durchlaufen.
- **Layer 1:** `classify_layer1` läuft in eigener Task mit hartem Timeout; Nutzlast ≤ `MAX_CLASSIFY_PAYLOAD_BYTES = 65_536`. **Fail-closed:** Timeout, Panic oder Fehler liefern stets `Block(…)` (`BlockReason`: `SensitivePattern`, `PolicyDenied`, `EgressPolicyDenied`, `ClassificationTimeout`, `InternalError`; je Grund `category()` ∈ `PolicyCategory{LocalAccess, CloudEgress}`). `EgressVault::DEFAULT_TIMEOUT = 100 ms`. **Standardmuster** (`default_patterns()`): `sk-`, `AKIA`, `api_key`, `password` und ein E-Mail-Regex. Das sind Teilstring-Muster; sie erkennen keine allgemeinen PII-Formen (Namen, Adressen, Nummern).
- **Erkennung von Entitäten:** Trait `EntityRecognizer` (byte ranges + Kategorie, DAG-neutral, ohne ML-Abhängigkeit); Standard `NoOpRecognizer` erkennt nichts.
- **Surrogate:** `SurrogateVault::new(session_salt: [u8;16])` / `random_salt()` (OsRng). `generate_surrogate(text)`: Blake3 über `text ‖ salt`, Ergebnis `[USER_ENTITY_xxxx]` mit **vier Hex-Zeichen** (16 Bit). Bei vielen Entitäten pro Sitzung sind Kollisionen möglich; bei Kollision überschreibt das Mapping den früheren Eintrag. **[V, Folgerung]** Mapping und Salt werden bei `Drop` genullt.
- **Rehydration:** `CloudResponseRehydrator::rehydrate(text)` ersetzt Tokens nach dem Muster `[USER_ENTITY_[0-9a-fA-F]{4}]` durch die Originale; unbekannte Tokens bleiben unverändert; panikfrei auf beliebigen UTF-8-Grenzen.
- **Cloud-Anfrage:** `handle_cloud_query[_scoped][_with_guard|_with_bulk_detector]` (Varianten für Mandantenbindung, Layer-4-Guard und Bulk-Detektor), Anfrage ≤ `MAX_SEARCH_QUERY_BYTES = 64 KiB`; Antwort `CloudQueryResponse{status, query, abstracted, results, abstraction_notice}`. `process_cloud_response(raw, injection_guard, rehydrator)`: erst Prompt-Injection-Prüfung (Trait `InjectionDetector`; Treffer → Fehler „Inbound security policy violation“), dann Rehydration.
- **PII-Kopplung (INV-COLLECTION-PROFILE-3):** `pii_vault_forces_crypto_shred(is_pii, is_memory_only) = is_pii && !is_memory_only`. `resolve_effective_kv_delete_mode` erzwingt dann `KvDeleteMode::CryptoShred` für dieses Dokument, unabhängig vom Collection-Preset.

## L.6 WASI-Host der Sandbox (`contextra-sandbox/src/wasi.rs`)

Handgeschriebene **WASI-preview1**-Hostfunktionen auf einem Wasmtime-`Linker`, ohne `wasmtime-wasi`. Implementiert: `fd_read` (nur fd 0 = stdin), `fd_write` (fd 1 = stdout, fd 2 = stderr), `proc_exit`, `clock_time_get`, `random_get`, `args_sizes_get` u. a. Errno-Konstanten: `SUCCESS 0`, `ACCES 2`, `BADF 8`, `INVAL 28`, `NOSYS 52`, `NOTSUP 58`. Invarianten: kein `unsafe`, strikte Grenzprüfung des Linearspeichers, bei Überschreiten von `max_output_bytes` Fehler `OutputLimitExceededError{stream, limit}`; `proc_exit` wird als `ProcessExitError{code}` propagiert.
Standardgrenzen (`capabilities.rs`): `max_module_size_bytes` 10 MiB, `max_output_bytes` 1 MiB, `max_stdin_bytes` 1 MiB. `clock_time_get` ist nur bei gesetztem `allow_clock` erlaubt (sonst `ERRNO_ACCES`) und akzeptiert nur `clock_id == 1`; `random_get` prüft Offset und Länge mit `checked_add` (negative Werte → `ERRNO_INVAL`). Wasmtime 25 mit `async` und `cranelift`, ohne Default-Features.

## L.7 MCP-Werkzeugregister und Bestätigungspflicht (`contextra-mcp/src/sandbox.rs`, `server_tools.rs`)

- `TOOL_REGISTRY: &[ToolDefinition{name, category, description, input_schema}]` ist die einzige Quelle für `tools/list`, Schemas und Klassifikation. Die 15 Einträge mit Kategorie und Pflichtparametern stehen in **K.34**.
- `McpSandbox::classify_method(name)`: Suche im Register; **nicht gefunden → `CodeExecution`** (fail-closed, aber falsche Kategorie; B-02). Default-Policy: Reads erlaubt; Writes, Code und Cloud gesperrt (B.7).
- **Bestätigung:** `contextra_forget` und `contextra_drop_collection` lehnen Aufrufe ohne `confirm: true` mit `invalid_params` ab („confirm parameter must be explicitly set to true“). Nicht-Bool-Werte zählen als `false`.
- `contextra_delete` löscht ein Dokument mit HNSW-Nachbarschaftsreparatur und stellt einen `DeletionProof` aus; `contextra_create_collection` akzeptiert `deployment_tier ∈ {EdgeMinimal, PowerUserLocal (Default), EnterpriseShared, EnterpriseRegulated}`; `contextra_relate_n_ary` verlangt 2 bis 64 Teilnehmer mit `{doc_id, role}`.
- Die Beschreibungen von `contextra_get` und `contextra_explain` tragen den Hinweis, dass Rückgaben aus nicht vertrauenswürdigen Dokumenten stammen und im Client-Prompt zu isolieren sind.

## L.8 Bi-temporaler Filter und Hard-Scope (`contextra-engine`)

- **`ValidityWindow`** {`tx_valid_from`, `tx_valid_to`: `Option<TxId>`; `business_valid_from`, `business_valid_to`: `Option<i64>`}. `is_empty()` ⇒ Kandidat gilt immer (**Fail-Open** für Altdaten ohne Metadaten). `is_valid_bitemporal(as_of_tx, as_of_business: Option<u64>)`: **UND** über Systemzeit (TxId/MVCC) und Geschäftszeit; Intervalle `[valid_from, valid_until)` (Start inklusiv, Ende exklusiv; ADR-033/038). `extract_validity_window(metadata)` liest Zahlen oder Zahlenstrings aus den Metadaten. Der Filter wirkt **nach** der Fusion.
- **`ScopeConstraint{allowed_doc_ids: BTreeSet<DocId>, gamma: u32}`** (Default `gamma = 2`; `BTreeSet` für deterministische Iteration). Builder-Methode `.scope(ScopeConstraint)`, intern `hard_scope`. Erzwingt harte Dokumentgrenzen über ACORN-Traversierung (`search_knn_acorn`). Ein reguläres `.execute()` mit gesetztem Scope schlägt mit `CapabilityUnsupported` fehl (`builder_exec.rs:53`); der Scope-Pfad hat eine eigene Ausführung (`builder_exec.rs:201`).
- **`TenantPolicy`** {`Optional` (Default), `Required`}: bei `Required` ist der Zugriff über `collection(...)` verboten; Aufrufer müssen `collection_for_tenant(...)` nutzen (E-01).

## L.9 Widerspruchsregister und Hyperkanten-Vorschläge (`contextra-graph`)

- **`ConsistencyEnforcer`** (Feature F-04, orthogonal zu F-02): liefert Kandidaten, **löscht nie selbst**; Tombstones setzt der Aufrufer. `EdgeAssertion{subject: NodeIdx, predicate_hash: [u8;32], object_repr: Vec<u8>}`, `pattern_hash()` = Blake3 über alle drei. `ConflictPattern{pattern_hash, contradiction_count, first_detected_tx, last_detected_tx, suppressed}`. Trait `ContradictionDetector`; Referenz `ExactPredicateConflictDetector`: Widerspruch, wenn Subjekt und Prädikat gleich, Objekt verschieden. `DEFAULT_SUPPRESSION_THRESHOLD = 3`; `check_before_insert(&mut self, &EdgeAssertion) -> Option<ConflictPattern>`; ab Erreichen der Schwelle ist das Muster `suppressed`.
- **Hyperkanten-Vorschläge** (`hyperedge_suggest.rs`): `compute_co_occurrence_candidates` ist deterministisch (kein `Rng`, P28): je Dokument Entitäten deduplizieren und sortieren; die exakte Kombination wird über alle Dokumente gezählt; Kombinationen unter `min_co_occurrence` oder mit mehr als `MAX_RELATE_PARTICIPANTS = 64` Teilnehmern werden **verworfen, nicht gekürzt**; Ausgabe aufsteigend nach `participants`. `HyperEdgeCandidate{participants, co_occurrence_count, source_doc_ids}`. Danach validiert ein LLM höchstens `max_llm_calls_per_cycle` Kandidaten (`ValidatedHyperEdgeCandidate{candidate, predicate, llm_confidence ∈ [0,1], accepted}`); Überschreitung → `HyperedgeSuggestError`. Übernommen wird über `relate_n_ary`.

## L.10 Kontextuelles Chunk-Präfix (`contextra-infer-ollama/src/context_prefixer.rs`)

`ContextPrefixEngine` (Alias `ContextPrefixer`) erzeugt vor BM25/Embedding je Chunk ein 1–2-Satz-Kurzpräfix mit einem kleinen Ollama-Modell (Empfehlung `llama3.2:3b` oder `gemma2:2b`). `ContextPrefixConfig`: `max_document_chars` (Default 8000 ≈ 2000 Token), `max_prefix_tokens` (Default 80). `generate_prefix(document, chunk)`: leeres Dokument oder Chunk → `InvalidInput`; Ollama nicht erreichbar → `Storage`/`Io`. Invarianten: XML-Escaping vor dem Prompt-Bau (Prompt-Injection-Isolation); `truncate_chars` wahrt Unicode-Codepoints und Wortgrenzen. `generate_prefix_batch` arbeitet **sequenziell**. Die Doc-Kommentar-Zahlen „49 % / 67 % weniger Retrieval-Fehler“ sind Literaturwerte, **nicht** in Contextra gemessen (vgl. G1-8).

## L.11 Export-Format v1 (`contextra-engine/src/export.rs`, `docs/EXPORT_FORMAT.md`)

`SCHEMA_VERSION_V1 = "1.0"`. `ExportDocumentV1{schema_version (MUSS "1.0"), exported_at (ISO-8601, optional), collections}` → `ExportCollectionV1{name, memories, relations}` → `ExportMemoryV1{id, memory_type ∈ episodic|semantic|procedural|working, Embedding, …}` und `ExportRelationV1`. Nur lesender Zugriff; Ziel: vollständige Memories und Relationen. Der Import liegt in `engine/src/import.rs`.

## L.12 Agent: Budget und Dead-Letter-Queue (`contextra-agent`)

- **`reserve_tokens(budget, amount) -> Reservation`** (RAII): wird die Reservierung ohne `settle()` fallen gelassen, werden die Token automatisch erstattet. Keine Locks über `await`; panikfreie `Drop`-Handler; atomare Erstattung.
- **`DeadLetterQueue`** (Storage = LSM, Präfix **`dlq:`**): `push`, `drain`, `list`, `remove(letter)` über den Schlüssel `(session_id, node_id, step_index)`, `allocate_tx`. `is_already_committed(letter)`: prüft vor dem Replay, ob die `TxId` des Eintrags für denselben Schlüssel schon committet ist; `true` ⇒ Replay ist ein No-Op (idempotent).

## L.13 System-Schnittstellen (`contextra-sys`, `contextra-simd`, `contextra-wire`)

- **`contextra-sys`** (Unsafe-Insel): `mem_lock(ptr, len) -> bool` / `mem_unlock(addr, len)` (`mlock` unter Linux; auf anderen Plattformen No-Op, `mem_lock` liefert dort `true`, obwohl nichts gesperrt wird), `mmap_readonly(&File) -> io::Result<Mmap>` (memmap2), `reopen_and_dup2(path, target_fd, flags)` (POSIX), Win32: `set_restrictive_file_acl(path)` und `verify_file_acl_owner_only(path)` (Datei nur für den Eigentümer), `vault.rs`: `LockedRegions` merkt gesperrte Adressbereiche (`lock_slice(slice, attempt_mlock)`, `unlock_all`, `len`, `is_empty`); der `VolatileContextVault` (INV-VAULT-1/2/3) nutzt `mlock` laut B.9.
- **`contextra-simd`**: Distanzkernel (Dot, Cosine, Euklid/L2) mit Laufzeit-Dispatch in `dispatch.rs` (AVX2, AVX-512, NEON, Scalar; B.3.1).
- **`contextra-wire`**: FlatBuffers-Bindings (`contextra_generated.rs`, automatisch erzeugt aus `schemas/contextra.fbs`; Drift-Check `xtask check_flatbuffers_drift`) und `jsonrpc.rs`.

## L.14 Python-Bindings (`contextra-py`)

PyO3-Klassen: `PyContextra` (`db.rs`), `PyCollection`, `PyDocument`, `PySearchResult`, `PyDbStats`, `PyStorageStats`, `PyVectorIndexStats`, `PyRuntimeState`; Funktion `open(...)` (`functions.rs`), Testhaken `_trigger_panic_for_test`. `CONTEXTRA_WORKER_THREADS` steuert den Runtime-Pool. Fehler werden über `ContextraErrorDto` übersetzt (I.0). Das Crate liegt außerhalb der `default-members` (0.5) und ist laut Teil 0 kein Kernbestandteil.

## L.15 Zusätzliche Hintergrundarbeiter der Engine (`contextra-engine/src/background_workers/`)

Dateien: `expiry_workers.rs`, `orphan_workers.rs`, `hyperedge_worker.rs`, `config.rs` (`OrphanCleanupBackoffConfig`), `mod.rs`. Verhalten und Grenzen (Expiry ≤ 100/Tick, Orphan ≤ 100/Tick mit Backoff 5 s → 300 s, Hyperkanten ≤ 1.000/Tick) stehen in B.4 und B.3.3; Feature `background-maintenance`.

## L.16 Noch nicht per Ablauf gelesen (Stand v15)

Diese Bereiche sind in Teil K nur als **[V-Sig]** erfasst: Innenleben der Kognitions-Pipeline (`cognition/*_phase.rs`), `flow_thompson`, `offpolicy`, `lyapunov`, `pid*` in `adapt`, `persistence` im Vector-Crate, `morphology`/`tokenizer` im Text-Crate, `quantize_kivi`/`quantize_rabitq`, `kv_state` und KV-Bridge in `infer-candle`, `model_info` in `infer-ollama`, `plugin_status`, `processing_registry`, `tenant_codec`, `system_pressure`, `seq_log`, `edge_reinforcement_buffer`, `arc_slice`, `homeostat`, `decay_controller`, `kv_cipher`, `wal_crypto`, `server_dispatch`, `tools_crud`, `in_memory_store`/`fault_vfs`/`reference_model` im Testkit, `template` im AVV-Generator. Sie sind die Kandidaten für die nächste Leserunde (Reihenfolge nach Risiko: `wal_crypto`, `kv_cipher`, `tenant_codec`, `seq_log`, `persistence`, `server_dispatch`).


---

# TEIL M — Abnahme von v15

v15 gilt als vollständig im Sinne der Sichtbarkeit, wenn (1) jedes `pub`-Item der 33 Crates in Teil K steht (Skript: `tree-sitter-rust`, Ausschluss von Tests und `cfg(test)`), (2) jede Feature-Flag-Zeile aus den `Cargo.toml` in K.1–K.33 steht, (3) K.34 mit `TOOL_REGISTRY` und K.35 mit den Env-Zugriffen übereinstimmen. **Offene Arbeit:** Verhaltensbeschreibung für die in L.16 genannten Bereiche; Nachprüfung der Teil-K-Doc-Kommentare auf Veralterung; Ausführung von Build und Tests (bisher nie Teil der Spec-Erstellung).
