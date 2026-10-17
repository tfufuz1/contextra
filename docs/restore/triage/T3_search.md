# Triage-Manifest T3_search (vector, text, rank, kvcache, graph)

- **Stand (`origin/main`):** `a8daea88a5f80ec6995d9e1d5da160516f93d4e2`
- **Anzahl Commits:** 86
- **Pfadbereich:** `crates/contextra-vector/**`, `crates/contextra-text/**`, `crates/contextra-rank/**`, `crates/contextra-kvcache/**`, `crates/contextra-graph/**`

### Zähler je Klasse
- **DEL-R0:** 5
- **DEL-R1:** 4
- **DEL-R6:** 14
- **PORT-FIX:** 18
- **PORT-WIRING:** 15
- **PORT-REQUIRED:** 1
- **SKIP-DOC:** 6
- **SKIP-BOT:** 9
- **SKIP-SUPERSEDED:** 9
- **SKIP-INFRA:** 5

---

## Commits

| sha | Datum | Titel | Dateien | prod/test/doku-Zeilen | Klasse | Begründung | Abhängigkeit | in main schon vorhanden | Konfliktrisiko |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 94601244 | 2026-10-15 | [Entkopplung contextra-vector] Decouple tests and benches from contextra-core | `acorn_vs_post_filter.rs`<br>`audit_benchmarks.rs`<br>`distance_nan_impact_bench.rs`<br>*(+75 weitere)* | +0/-0 / +152/-2786 / +0/-0 | SKIP-SUPERSEDED | Core-Cut / Entkopplungs-Commit nach eb10652f entfaellt; Netto-Zustand zählt. | keine | ja | hoch |
| 3dece684 | 2026-10-15 | [Entkopplung contextra-vector] Decouple contextra-vector from contextra-core and remove experimental features | `Cargo.toml`<br>`mod.rs`<br>`naive_reference.rs`<br>*(+39 weitere)* | +106/-4884 / +0/-0 / +0/-0 | SKIP-SUPERSEDED | Core-Cut / Entkopplungs-Commit nach eb10652f entfaellt; Netto-Zustand zählt. | keine | ja | hoch |
| c2732639 | 2026-10-15 | refactor(rank): remove dibud and conformal calibration | `AGENTS.md`<br>`Cargo.toml`<br>`conformal.rs`<br>*(+16 weitere)* | +11/-756 / +0/-920 / +2/-9 | SKIP-SUPERSEDED | Refactoring zur Entfernung von dibud/conformal (später im Donor vollzogen). | keine | ja | hoch |
| b5f62239 | 2026-10-15 | [W1-Jxx] Parkplatz-Kanten aus Kern-Crates entfernen | `Cargo.toml`<br>`calibration_deep_tests.rs`<br>`calibration_stress_and_fault_tests.rs` | +0/-1 / +2/-134 / +0/-0 | SKIP-SUPERSEDED | Parkplatz-Kanten-Bereinigung vor Core-Cut, im Donor-Endstand enthalten. | keine | ja | niedrig |
| 51e1e765 | 2026-10-15 | feat(ports): redefine Erasure ports per §5.4 and §5.5 (J-07) | `types.rs` | +2/-2 / +0/-0 / +0/-0 | DEL-R0 | Redefinition der Erasure ports (§5.4/§5.5, J-07) bereits durch R0 in contextra-ports abgedeckt. | contextra-ports | ja | niedrig |
| 769808a9 | 2026-10-14 | feat(vector): clarify DiskANN load_sync TODO for F-03 and F-04 safety | `persistence.rs` | +3/-6 / +0/-0 / +0/-0 | SKIP-SUPERSEDED | DiskANN-Kommentar/Fix auf DiskANN-Code, der in eb10652f gelöscht wurde. | keine | ja | hoch |
| c9642adc | 2026-10-14 | fix(deps): remove illegal Ring-DAG dependency edges per Spec §4.3 (F-31) | `Cargo.toml` | +0/-1 / +0/-0 / +0/-0 | PORT-FIX | Entfernung illegaler Ring-DAG-Abhängigkeit in Cargo.toml per Spec §4.3 (F-31). | contextra-graph/Cargo.toml | nein | niedrig |
| 426dfead | 2026-10-14 | Shell-Commit | `AGENTS.md`<br>`AGENTS.md`<br>`AGENTS.md`<br>*(+1 weitere)* | +0/-0 / +0/-0 / +37/-18 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | hoch |
| a51c4b97 | 2026-10-14 | feat(crypto,engine,kvcache): wire deletion, key shredding, and proof verification | `wiring.rs` | +5/-5 / +0/-0 / +0/-0 | DEL-R1 | Verdrahtung von Deletion, Key Shredding und Proof Verification in kvcache für Löschnachweis (R1). | contextra-crypto | ja | niedrig |
| c19a81f2 | 2026-10-13 | feat(vector): implement compiler-resistant HNSW zeroization | `arena.rs`<br>`core_rebuild.rs`<br>`hnsw_zeroize_no_residue.rs` | +39/-13 / +90/-0 / +0/-0 | DEL-R6 | Compiler-resistente HNSW Zeroization in contextra-vector für Löschnachweis (R6). | contextra-vector | nein | hoch |
| f2d4b245 | 2026-10-13 | fix(contextra-text): Wire domain vocabularies, zero-panic tokenizer, BM25 params, and facade wiring | `bm25.rs`<br>`legal_de.rs`<br>`medical_de.rs`<br>*(+8 weitere)* | +322/-26 / +30/-1 / +0/-0 | PORT-WIRING | Verdrahtung der Domain Vocabularies, Zero-Panic Tokenizer und BM25 Parameter in contextra-text. | contextra-text/src/wiring.rs | ja | hoch |
| 42cf7835 | 2026-10-13 | feat(rank,infer-onnx): wire RerankWiring and RerankStage | `drift.rs`<br>`mod.rs`<br>`rerank.rs`<br>*(+3 weitere)* | +133/-5 / +208/-0 / +0/-0 | PORT-WIRING | Verdrahtung von RerankWiring und RerankStage in contextra-rank. | contextra-rank/src/fusion/rerank.rs | nein | niedrig |
| 7efd126a | 2026-10-13 | feat(graph): wire contextra-graph pipeline and enforce AGT-GRAPH-001 fail-closed TxId checks | `selector.rs`<br>`graph_index.rs`<br>`basic_tests.rs`<br>*(+7 weitere)* | +235/-25 / +273/-12 / +0/-0 | PORT-WIRING | Verdrahtung der contextra-graph Pipeline und Durchsetzung der fail-closed TxId-Prüfung (AGT-GRAPH-001). | contextra-graph/src/wiring.rs | nein | niedrig |
| f0131b9e | 2026-10-13 | kvcache: KdfHeader migration, KvShreddingWiring and test API completion | `lib.rs`<br>`prefix_store.rs`<br>`quantize_kivi.rs`<br>*(+8 weitere)* | +420/-10 / +138/-7 / +0/-0 | DEL-R1 | KdfHeader Migration, KvShreddingWiring und Test-API Vervollständigung in kvcache für Löschnachweis (R1). | contextra-kvcache/src/wiring.rs | ja | hoch |
| 35c6bee9 | 2026-10-13 | feat(types): implement TxId::try_new, TenantId::system, and new_unchecked | `cache_scope_scan.rs`<br>`vector_index_impl.rs`<br>`witness.rs`<br>*(+2 weitere)* | +16/-11 / +20/-6 / +0/-0 | DEL-R0 | TxId::try_new und TenantId::system bereits in contextra-types durch R0 abgedeckt. | contextra-types | ja | niedrig |
| 4b7709e6 | 2026-10-13 | Shell-Commit | `residue_scan.rs`<br>`tokenizer.rs`<br>`pipeline_identity_regression.rs`<br>*(+4 weitere)* | +3/-6 / +79/-27 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | mittel |
| 561dcec9 | 2026-10-13 | Shell-Commit | `scratchpad_invalidation.rs` | +1/-71 / +0/-0 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | niedrig |
| e69edff4 | 2026-10-13 | feat(vector): implement HNSW residue witness issuance and deletion hardening | `core_rebuild.rs`<br>`types.rs`<br>`vector_index_impl.rs`<br>*(+3 weitere)* | +234/-0 / +220/-0 / +0/-0 | DEL-R6 | HNSW Residue Witness Issuance und Deletion Hardening in contextra-vector für Löschnachweis (R6). | contextra-vector | nein | hoch |
| 3d1bd1a3 | 2026-10-13 | feat(store): Cache-Scan und SecureBuf (Spezifikation C4.7) | `scratchpad_invalidation.rs`<br>`cache_scope_scan.rs` | +71/-1 / +113/-0 / +0/-0 | DEL-R0 | Cache-Scan und SecureBuf (Spec C4.7) store-Anteil bereits durch R0 wiederhergestellt. | contextra-store | ja | niedrig |
| edeb252d | 2026-10-13 | Shell-Commit | `scratchpad_invalidation.rs`<br>`cache_scope_scan.rs` | +71/-1 / +112/-0 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | niedrig |
| b713f897 | 2026-10-12 | feat(vector,text): implement residue scanners and deletion repair | `inverted.rs`<br>`index_struct.rs`<br>`residue_scan.rs`<br>*(+10 weitere)* | +417/-1 / +681/-0 / +0/-0 | DEL-R6 | Residue Scanners und Deletion Repair in vector (R6) und text (R7). | contextra-vector/src/hnsw/residue_scan.rs | nein | hoch |
| a8a8e2db | 2026-10-12 | Decouple proof dependencies from vector rebuild and facade profile | `core_rebuild.rs` | +3/-2 / +0/-0 / +0/-0 | PORT-FIX | Entkopplung der Proof-Abhängigkeiten von Vector Rebuild und Facade Profile. | contextra-vector/src/hnsw/core_rebuild.rs | nein | hoch |
| bec5536a | 2026-10-12 | refactor(graph): rename DeletionProof to CascadeTombstone | `cascade.rs`<br>`lib.rs` | +42/-5 / +0/-0 / +0/-0 | DEL-R6 | Umbennung von DeletionProof zu CascadeTombstone in graph für Löschnachweis (R6). | contextra-graph/src/cascade.rs | nein | niedrig |
| a3218ad1 | 2026-10-12 | docs(audit): add contextra-text determinism audit and regression tests | `pipeline_identity_regression.rs` | +0/-0 / +104/-1 / +0/-0 | SKIP-DOC | Dokumentation und Audit-Regressions-Tests für contextra-text Determinismus. | docs/audit | ja | niedrig |
| f196a0e0 | 2026-10-11 | fix(vector): guard test modules in diskann behind docid-128 feature | `build.rs`<br>`persistence.rs` | +2/-6 / +0/-0 / +0/-0 | PORT-FIX | Absicherung der DiskANN Test-Module hinter docid-128 Feature (DiskANN später entfernt). | contextra-vector | ja | hoch |
| 2d5c5b0d | 2026-10-11 | fix(doctrine): resolve panic, safety comment, and lint scoping violations in sys, wire, and text crates | `tokenizer.rs` | +14/-11 / +0/-0 / +0/-0 | PORT-FIX | Behebung von Safety-Kommentar und Lint Scoping Violations in text. | contextra-text/src/tokenizer.rs | ja | niedrig |
| c518ad30 | 2026-10-11 | fix(vector): exhaustive match and error propagation in HNSW delete path | `vector_index_impl.rs`<br>`hnsw_delete_exhaustive_match.rs` | +12/-5 / +138/-0 / +0/-0 | DEL-R6 | Exhaustive Match und Error Propagation im HNSW Delete-Pfad (R6). | contextra-vector/src/hnsw/deletion.rs | nein | niedrig |
| 8cedf9fb | 2026-10-11 | Shell-Commit | `tokenizer.rs`<br>`build.rs`<br>`persistence.rs`<br>*(+2 weitere)* | +19/-15 / +0/-0 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | hoch |
| 525ecd80 | 2026-10-11 | contextra-db: add plan TODOs for index-store transaction sync | `build.rs`<br>`persistence.rs`<br>`vector_index_impl.rs`<br>*(+1 weitere)* | +5/-16 / +0/-0 / +0/-0 | SKIP-DOC | Plan-TODOs für Index-Store Transaction Sync in contextra-db. | contextra-db | ja | hoch |
| af63a482 | 2026-10-11 | contextra-rank: sanitize DriftDetector threshold inputs and fix CI gates | `rrf.rs` | +2/-0 / +0/-0 / +0/-0 | PORT-FIX | Sanitizing DriftDetector threshold inputs und Absicherung der State/Non-finite Handhabung in rank. | contextra-rank/src/drift.rs | nein | hoch |
| bd0a2525 | 2026-10-11 | contextra-rank: harden DriftDetector state and non-finite handling | `rrf.rs` | +3/-1 / +0/-0 / +0/-0 | PORT-FIX | Sanitizing DriftDetector threshold inputs und Absicherung der State/Non-finite Handhabung in rank. | contextra-rank/src/drift.rs | nein | hoch |
| 8f9e27ab | 2026-10-11 | contextra-rank: add plan markers for 4-signal RRF fusion & calibration | `drift.rs`<br>`lib.rs` | +10/-0 / +0/-0 / +0/-0 | SKIP-DOC | Plan-Marker für 4-Signal RRF Fusion & Calibration in rank. | contextra-rank/src/fusion/rrf.rs | ja | niedrig |
| f9c2645f | 2026-10-11 | contextra-rank: harden DriftDetector state and non-finite handling | `drift.rs` | +123/-5 / +0/-0 / +0/-0 | PORT-FIX | Sanitizing DriftDetector threshold inputs und Absicherung der State/Non-finite Handhabung in rank. | contextra-rank/src/drift.rs | nein | niedrig |
| 298de109 | 2026-10-11 | contextra-rank: sanitize DriftDetector threshold inputs and add tests | `drift.rs` | +97/-2 / +0/-0 / +0/-0 | PORT-FIX | Sanitizing DriftDetector threshold inputs und Absicherung der State/Non-finite Handhabung in rank. | contextra-rank/src/drift.rs | nein | niedrig |
| a2b063dc | 2026-10-11 | Shell-Commit | `build.rs`<br>`persistence.rs`<br>`vector_index_impl.rs`<br>*(+1 weitere)* | +16/-5 / +0/-0 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | hoch |
| 2a1af3f7 | 2026-10-11 | fix: repair workspace compilation after crypto changes | `hnsw_slot_reuse_ghost_edges.rs` | +0/-0 / +7/-3 / +0/-0 | PORT-FIX | Reparatur der Workspace-Kompilierung nach Crypto-Änderungen in vector/rank/kvcache. | contextra-vector/src/hnsw/vector_index_impl.rs | ja | niedrig |
| 3a14af69 | 2026-10-11 | fix(workspace): resolve workspace compilation after crypto repair | `deletion.rs`<br>`types.rs` | +1/-5 / +0/-0 / +0/-0 | PORT-FIX | Reparatur der Workspace-Kompilierung nach Crypto-Änderungen in vector/rank/kvcache. | contextra-vector/src/hnsw/vector_index_impl.rs | ja | niedrig |
| 240f4f7c | 2026-10-11 | Shell-Commit | `segment.rs`<br>`rrf.rs`<br>`arena.rs`<br>*(+8 weitere)* | +230/-176 / +422/-0 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | hoch |
| 7e1d65ba | 2026-10-10 | fix(mcp): gate mock provider branch behind test-utils feature (#4494) | `segment.rs`<br>`rrf.rs`<br>`arena.rs`<br>*(+8 weitere)* | +176/-230 / +0/-422 / +0/-0 | SKIP-INFRA | Gating des Mock Providers hinter test-utils Feature in mcp. | contextra-mcp | ja | hoch |
| 66f78ca3 | 2026-10-09 | test(vector): verify HNSW slot reuse ghost edges across deletion paths (W4-06) (#4477) | `hnsw_slot_reuse_ghost_edges.rs` | +0/-0 / +207/-150 / +0/-0 | DEL-R6 | HNSW Slot Reuse Ghost Edges Verification & Cleanup (W4-06) für Löschnachweis (R6). | contextra-vector/src/hnsw/connections.rs | nein | niedrig |
| cea972df | 2026-10-09 | fix(vector): clear stale inbound edges on HNSW slot reuse (W4-06) (#4476) | `Cargo.toml`<br>`Cargo.toml`<br>`Cargo.toml`<br>*(+4 weitere)* | +53/-4 / +163/-156 / +0/-0 | DEL-R6 | Löschung veralteter Inbound Edges bei HNSW Slot Reuse (W4-06) für Löschnachweis (R6). | contextra-vector/src/hnsw/connections.rs | nein | niedrig |
| 050c6234 | 2026-10-09 | test(hnsw): verify slot reuse ghost edges and fix arena backlink cleanup (#4475) | `hnsw_slot_reuse_ghost_edges.rs` | +0/-0 / +220/-0 / +0/-0 | DEL-R6 | HNSW Slot Reuse Ghost Edges Verification & Cleanup (W4-06) für Löschnachweis (R6). | contextra-vector/src/hnsw/connections.rs | nein | niedrig |
| 182db152 | 2026-10-09 | test(vector): fix Tokio runtime builder enable_all panic in loom_quantizer_race_test (#4467) | `loom_quantizer_race_test.rs` | +0/-0 / +7/-6 / +0/-0 | PORT-FIX | Fix Tokio Runtime Builder enable_all Panic in loom_quantizer_race_test. | contextra-vector/tests/loom_quantizer_race_test.rs | nein | niedrig |
| ef39f9f0 | 2026-10-08 | supply-chain(tokio): replace full feature with per-crate minimal features (#4464) | `Cargo.toml`<br>`Cargo.toml`<br>`Cargo.toml`<br>*(+1 weitere)* | +4/-4 / +0/-0 / +0/-0 | PORT-REQUIRED | Umstellung der Cargo.toml Tokio-Features auf minimale Per-Crate-Features. | crates/contextra-graph/Cargo.toml | nein | niedrig |
| 330c9847 | 2026-10-08 | P06: Harden persistent key shredding and DeletionProof durability (#4461) | `segment.rs` | +3/-3 / +0/-0 / +0/-0 | DEL-R1 | Harden persistent key shredding and DeletionProof durability (P06) in kvcache. | contextra-kvcache/src/segment.rs | nein | niedrig |
| c17f3225 | 2026-10-08 | fix(store): ensure memtable sorted order for out-of-order seq_nos and fix read-only wal rewrite (#4460) | `deletion.rs` | +4/-0 / +0/-0 / +0/-0 | DEL-R6 | Fix sorted order for out-of-order seq_nos and read-only WAL rewrite in store/vector. | contextra-vector/src/hnsw/deletion.rs | nein | niedrig |
| 72f14e96 | 2026-10-08 | HNSW: In-memory backlink overlay for mmap-resident node repairs (#4458) | `batch.rs`<br>`connections.rs`<br>`core_insert.rs`<br>*(+4 weitere)* | +168/-170 / +137/-0 / +0/-0 | DEL-R6 | HNSW In-memory Backlink Overlay für mmap-resident Node Repairs. | contextra-vector/src/hnsw/connections.rs | nein | niedrig |
| 5f5251e9 | 2026-10-08 | fix(crypto): durable RevocationLog in setup_kv_bridge and fail-closed lock handling (#4454) | `rrf.rs` | +6/-3 / +0/-0 / +0/-0 | DEL-R1 | Durable RevocationLog und Fail-Closed Lock Handling in crypto/rank. | contextra-rank/src/fusion/rrf.rs | nein | hoch |
| 3a4c5192 | 2026-10-08 | HNSW commit(): Op-Normalisierung, Fehleratomaritaet, RAM-Vektor-Loeschung beim Freigeben (#4438) | `core_rebuild.rs`<br>`vector_index_impl.rs`<br>`hnsw_commit_semantics.rs` | +289/-149 / +210/-0 / +0/-0 | DEL-R6 | HNSW commit(): Op-Normalisierung, Fehleratomarität, RAM-Vektor-Löschung beim Freigeben. | contextra-vector/src/hnsw/core_rebuild.rs | nein | hoch |
| 67c7ca58 | 2026-10-07 | feat(store): integrate mandatory SnapshotFloor in CompactionEngine and flush (#4418) | `rrf.rs` | +3/-3 / +0/-0 / +0/-0 | DEL-R0 | SnapshotFloor in CompactionEngine Integration (store/rank) durch R0 abgedeckt. | contextra-store | nein | hoch |
| 4fb1e45b | 2026-10-07 | fix(vector,crypto,wire): propagate dropped IO errors and refactor quantizer training to try_train (#4413) | `tests.rs`<br>`batch.rs`<br>`core_rebuild.rs`<br>*(+9 weitere)* | +157/-63 / +29/-8 / +0/-0 | PORT-FIX | Fehlerfortpflanzung verworfener I/O-Fehler und Refactoring von quantizer.train zu try_train. | contextra-vector/src/quantize/mod.rs | nein | hoch |
| 87eb6287 | 2026-10-07 | test(vector): verify HNSW determinism, lock contention, and NaN policy (#4410) | `deletion.rs` | +4/-0 / +0/-0 / +0/-0 | DEL-R6 | Verifikation der HNSW Determinismus-, Lock Contention- und NaN-Policy für Löschnachweis (R6). | contextra-vector/src/hnsw/deletion.rs | nein | niedrig |
| 1ff44716 | 2026-10-07 | fix(vector): secure mmap against length mismatch and file locks (#4408) | `mmap.rs` | +3/-6 / +0/-0 / +0/-0 | PORT-FIX | Absicherung von Mmap gegen Längen-Mismatch und File Locks in vector. | contextra-vector/src/persistence/mmap.rs | nein | niedrig |
| bb1e833e | 2026-10-07 | feat(vector): migrate DiskANN to contextra-durable-fs (#4407) | `build.rs`<br>`config.rs`<br>`durable_fs.rs`<br>*(+3 weitere)* | +78/-76 / +0/-0 / +0/-0 | SKIP-SUPERSEDED | DiskANN Migration auf contextra-durable-fs / atomic_replace (DiskANN in eb10652f gelöscht). | contextra-vector/src/diskann | ja | hoch |
| 00b7d22a | 2026-10-07 | fix(vector): use atomic_replace and durable_remove for DiskANN file persistence (#4405) | `build.rs`<br>`persistence.rs`<br>`tests.rs`<br>*(+2 weitere)* | +203/-109 / +24/-12 / +0/-0 | SKIP-SUPERSEDED | DiskANN Migration auf contextra-durable-fs / atomic_replace (DiskANN in eb10652f gelöscht). | contextra-vector/src/diskann | ja | hoch |
| 1f279512 | 2026-10-07 | fix(vector): enforce GhostScan completeness and durable persistence (#4404) | `batch.rs`<br>`core_insert.rs`<br>`deletion.rs`<br>*(+7 weitere)* | +378/-66 / +15/-7 / +0/-0 | DEL-R6 | GhostScan Vollständigkeit und durable Persistence für HNSW (R6). | contextra-vector/src/hnsw/durable_fs.rs | nein | niedrig |
| 4b246d3f | 2026-10-07 | feat(durable-fs): add contextra-durable-fs crate and mvcc floor/lease contracts (#4402) | `Cargo.toml` | +1/-0 / +0/-0 / +0/-0 | DEL-R0 | Hinzufügen des contextra-durable-fs Crate durch R0 abgedeckt. | contextra-durable-fs | ja | niedrig |
| e0077c62 | 2026-10-07 | fix(vector): secure mmap against length mismatch and locks (#4399) | `mmap.rs`<br>`tests.rs` | +141/-2 / +0/-0 / +0/-0 | PORT-FIX | Absicherung von Mmap gegen Längen-Mismatch und File Locks in vector. | contextra-vector/src/persistence/mmap.rs | nein | niedrig |
| 7deb6e6d | 2026-10-06 | fix(vector): make DiskANN pending WAL persistence crash-idempotent (#4397) | `persistence.rs`<br>`p03d_diskann_pending_wal_crash_idempotent.rs` | +50/-14 / +94/-0 / +0/-0 | SKIP-SUPERSEDED | DiskANN Migration auf contextra-durable-fs / atomic_replace (DiskANN in eb10652f gelöscht). | contextra-vector/src/diskann | ja | hoch |
| 00518b9e | 2026-10-06 | fix(vector): prevent silent fallback on ScalarQuantizer training failure (#4389) | `mod.rs`<br>`p03c_scalar_quantizer_invalid_training_not_silent.rs` | +87/-32 / +69/-0 / +0/-0 | PORT-FIX | Verhinderung stummer Fallbacks bei ScalarQuantizer Trainingsfehlern. | contextra-vector/src/quantize/mod.rs | nein | hoch |
| e3fdea29 | 2026-10-06 | fix(vector): propagate arena allocation error and enforce atomic insert (#4388) | `arena.rs`<br>`core_insert.rs`<br>`p03a_hnsw_arena_alloc_failure_atomic_insert.rs` | +179/-103 / +50/-0 / +0/-0 | PORT-FIX | Fortpflanzung von Arena Allocation Errors und Durchsetzung atomarer Inserts in HNSW. | contextra-vector/src/hnsw/arena.rs | nein | niedrig |
| dd18772f | 2026-10-06 | Fix HNSW Ghost Pointer Verification Scan Completeness (#4387) | `deletion.rs`<br>`p03b_hnsw_ghost_pointer_verification_complete.rs` | +269/-51 / +123/-0 / +0/-0 | DEL-R6 | Fix HNSW Ghost Pointer Verification Scan Completeness (R6). | contextra-vector/src/hnsw/deletion.rs | nein | niedrig |
| 52e6ea7e | 2026-10-06 | Shell-Commit | `persistence.rs`<br>`arena.rs`<br>`core_insert.rs`<br>*(+2 weitere)* | +39/-0 / +0/-0 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | hoch |
| aa50a53b | 2026-10-06 | Add Mission P05 security and correctness review report (#4382) | `filtered_selectivity_recall.rs` | +0/-0 / +1/-1 / +0/-0 | SKIP-DOC | Dokumentations- und Review-Berichte (P02/P05). | docs/reports | ja | hoch |
| 7e5ef3e6 | 2026-10-06 | docs: add LSM-Tree correctness review report (Mission P02) (#4380) | `filtered_selectivity_recall.rs` | +0/-0 / +1/-1 / +0/-0 | SKIP-DOC | Dokumentations- und Review-Berichte (P02/P05). | docs/reports | ja | hoch |
| 56e5769a | 2026-10-06 | docs: standardize and audit all AGENTS.md files across multi-crate workspace | `AGENTS.md`<br>`AGENTS.md`<br>`AGENTS.md`<br>*(+2 weitere)* | +0/-0 / +0/-0 / +28/-8 | SKIP-DOC | Standardisierung aller AGENTS.md Dateien im Workspace. | AGENTS.md | ja | hoch |
| 7240937b | 2026-10-06 | fix(kvcache): remove unused variable keys_to_use | `scratchpad_invalidation.rs` | +0/-6 / +0/-0 / +0/-0 | PORT-FIX | Entfernung der unbenutzten Variable keys_to_use in kvcache. | contextra-kvcache/src/scratchpad_invalidation.rs | ja | niedrig |
| ce650578 | 2026-10-06 | feat(xtask): implement orphan-symbols scanner (AUFTRAG X01) (#4374) | `graph_write.rs`<br>`provenance.rs`<br>`j14_contextra_graph_graph_schreibpfad_csr_ko_tests.rs`<br>*(+31 weitere)* | +63/-28 / +139/-91 / +0/-0 | SKIP-INFRA | Xtask Orphan-Symbols Scanner / Unwired-Ports Harness Implementierung. | xtask | ja | hoch |
| f5cfd215 | 2026-10-06 | feat(xtask): implement unwired-ports harness command (#4367) | `scratchpad_invalidation.rs` | +5/-1 / +0/-0 / +0/-0 | SKIP-INFRA | Xtask Orphan-Symbols Scanner / Unwired-Ports Harness Implementierung. | xtask | ja | niedrig |
| 1233eb88 | 2026-10-06 | fix(agent): J02-CLOSURE wire contextra-agent symbols and add integration tests (#4366) | `scratchpad_invalidation.rs` | +2/-0 / +0/-0 / +0/-0 | PORT-WIRING | J02-CLOSURE Verdrahtung von contextra-agent Symbolen. | contextra-agent | ja | niedrig |
| c3ce9460 | 2026-10-06 | feat(graph): add integration tests for J15 graph nebenmodule symbols (#4362) | `j15closure-contextra-graph_symbol_verification.rs` | +0/-0 / +139/-0 / +0/-0 | PORT-WIRING | J15-CLOSURE Integrationstests für Graph Nebenmodule Symbole. | contextra-graph/tests | nein | niedrig |
| 62694d1a | 2026-10-06 | wire contextra-text symbols extend_vocabulary, new_with_all_domains, remove_terms (#4358) | `compound_splitter.rs`<br>`posting_list.rs`<br>`j31closure-contextra-text_closure_verification_test.rs` | +18/-28 / +76/-0 / +0/-0 | PORT-WIRING | Verdrahtung der contextra-text Symbole extend_vocabulary, new_with_all_domains, remove_terms. | contextra-text/src/morphology/compound_splitter.rs | ja | mittel |
| f07adb3e | 2026-10-06 | feat(xtask): implement stub-impls harness command (#4356) | `scratchpad_invalidation.rs` | +1/-7 / +0/-0 / +0/-0 | SKIP-INFRA | Xtask Orphan-Symbols Scanner / Unwired-Ports Harness Implementierung. | xtask | ja | niedrig |
| 8e71bac8 | 2026-10-06 | feat(rank): J12-CLOSURE wire 4 contextra-rank symbols and add tests (#4337) | `conformal.rs`<br>`platt.rs`<br>`state.rs`<br>*(+5 weitere)* | +48/-6 / +108/-0 / +0/-0 | PORT-WIRING | J12-CLOSURE Verdrahtung von 4 contextra-rank Symbolen. | contextra-rank/src/calibration/platt.rs | ja | mittel |
| c36fe154 | 2026-10-06 | fix(graph): J14-CLOSURE wire CSR graph write and consistency symbols (#4336) | `graph_write.rs`<br>`hyperedge.rs`<br>`j14_contextra_graph_graph_schreibpfad_csr_ko_tests.rs`<br>*(+1 weitere)* | +28/-7 / +208/-0 / +0/-0 | PORT-WIRING | J14-CLOSURE Verdrahtung der CSR Graph Schreib- und Konsistenz-Symbole in graph. | contextra-graph/src/csr/graph_write.rs | ja | hoch |
| d99d4279 | 2026-10-06 | refactor(kvcache): verify production callers and add closure tests for J06 symbols (#4331) | `eviction_worker.rs`<br>`prefix_store.rs`<br>`scratchpad_invalidation.rs`<br>*(+2 weitere)* | +20/-9 / +184/-0 / +0/-0 | PORT-WIRING | J06-CLOSURE Verifikation und Verdrahtung der kvcache Symbole in Produktionspfaden. | contextra-kvcache/src/store.rs | ja | hoch |
| 43086624 | 2026-10-05 | feat(vector): J32-CLOSURE contextra-vector symbol cleanup and free_node wiring (#4318) | `naive_reference.rs`<br>`build.rs`<br>`core_insert.rs`<br>*(+6 weitere)* | +36/-11 / +162/-0 / +0/-0 | PORT-WIRING | J32-CLOSURE contextra-vector Symbol-Bereinigung und free_node Verdrahtung. | contextra-vector/src/hnsw/core_insert.rs | ja | hoch |
| adf205ab | 2026-10-05 | bench(store): establish durability parity for competitive KV benchmarks (#4310) | `core_rebuild.rs` | +1/-0 / +0/-0 / +0/-0 | SKIP-INFRA | Benchmark Durability Parity Anpassung. | contextra-store | ja | hoch |
| 83462a48 | 2026-10-05 | fix(contextra-graph): wire CSR graph write and consistency symbols (#4304) | `graph_write.rs`<br>`edge_reinforcement_buffer.rs`<br>`hyperedge.rs`<br>*(+1 weitere)* | +207/-23 / +173/-0 / +0/-0 | PORT-WIRING | Verdrahtung der CSR Graph Schreibpfad und Konsistenz-Symbole in graph. | contextra-graph/src/csr/graph_write.rs | ja | hoch |
| 41110097 | 2026-10-05 | fix(contextra-graph): wire graph side modules APPRH, PPR, SessionDAG, ArcSlice, TL-HFD (#4303) | `mod.rs`<br>`cost.rs`<br>`shadow_hook.rs`<br>*(+7 weitere)* | +44/-0 / +287/-0 / +0/-0 | PORT-WIRING | Verdrahtung der Graph Nebenmodule APPRH, PPR, SessionDAG, ArcSlice, TL-HFD in graph. | contextra-graph/src/apprh/mod.rs | ja | niedrig |
| 9ed880b5 | 2026-10-05 | fix(contextra-text): wire compound splitter and posting list symbols (#4299) | `compound_splitter.rs`<br>`posting_list.rs`<br>`j31-contextra-text_wiring_test.rs` | +4/-23 / +211/-0 / +0/-0 | PORT-WIRING | Verdrahtung der Compound Splitter und Posting List Symbole in text. | contextra-text/src/morphology/compound_splitter.rs | ja | mittel |
| beb3a0bb | 2026-10-05 | fix(contextra-kvcache): wire uncalled symbols into production paths and add integration tests (#4280) | `segment.rs`<br>`store.rs`<br>`j06_contextra_kvcache_wiring.rs` | +111/-4 / +205/-0 / +0/-0 | PORT-WIRING | Verdrahtung nicht aufgerufener kvcache Symbole in Produktionspfade. | contextra-kvcache/src/store.rs | ja | hoch |
| 538dd199 | 2026-10-05 | fix(contextra-rank): wire uncalled symbols into production paths (#4278) | `conformal.rs`<br>`platt.rs`<br>`state.rs`<br>*(+4 weitere)* | +22/-15 / +172/-0 / +0/-0 | PORT-WIRING | Verdrahtung nicht aufgerufener rank Symbole in Produktionspfade. | contextra-rank/src/calibration/platt.rs | ja | mittel |
| e2f354ed | 2026-10-05 | Fix data-integrity bug in ResidentPostingIndex::remove_terms (#4266) | `posting_list.rs` | +31/-3 / +0/-0 / +0/-0 | PORT-FIX | Fix Data-Integrity Bug in ResidentPostingIndex::remove_terms in text. | contextra-text/src/posting_list.rs | ja | niedrig |
| cd529156 | 2026-10-05 | fix(vector): replace deprecated check_drift call in diskann/types.rs (#4263) | `types.rs` | +1/-1 / +0/-0 / +0/-0 | SKIP-SUPERSEDED | Fix check_drift Aufruf in diskann/types.rs (DiskANN in eb10652f gelöscht). | contextra-vector/src/diskann/types.rs | ja | hoch |
| e31ed5c4 | 2026-10-05 | Shell-Commit | `core_rebuild.rs` | +0/-1 / +0/-0 / +0/-0 | SKIP-BOT | Shell-Commit ohne funktionale Quelltext-Änderungen. | keine | nein | hoch |

---

## Anhang: Dateien (Klassifizierung A bis H)

| Pfad | Fall A bis H | Konflikt in merge-tree | beteiligte Commits |
| --- | --- | --- | --- |
| `crates/contextra-graph/AGENTS.md` | D | ja | 426dfead, 56e5769a |
| `crates/contextra-graph/Cargo.toml` | D | nein | c9642adc, cea972df, ef39f9f0 |
| `crates/contextra-graph/src/apprh/mod.rs` | B | nein | 41110097 |
| `crates/contextra-graph/src/apprh/selector.rs` | B | nein | 7efd126a |
| `crates/contextra-graph/src/cascade.rs` | B | nein | bec5536a |
| `crates/contextra-graph/src/csr/graph_index.rs` | B | nein | 7efd126a |
| `crates/contextra-graph/src/csr/graph_write.rs` | D | ja | ce650578, c36fe154, 83462a48 |
| `crates/contextra-graph/src/csr/tests/basic_tests.rs` | B | nein | 7efd126a |
| `crates/contextra-graph/src/edge_reinforcement_buffer.rs` | B | nein | 83462a48 |
| `crates/contextra-graph/src/hyperedge.rs` | B | nein | c36fe154, 83462a48 |
| `crates/contextra-graph/src/hyperedge_suggest.rs` | B | nein | 7efd126a |
| `crates/contextra-graph/src/lib.rs` | B | nein | 7efd126a, bec5536a |
| `crates/contextra-graph/src/ppr.rs` | B | nein | 7efd126a |
| `crates/contextra-graph/src/ppr/cost.rs` | B | nein | 41110097 |
| `crates/contextra-graph/src/ppr/shadow_hook.rs` | B | nein | 41110097 |
| `crates/contextra-graph/src/provenance.rs` | D | ja | ce650578 |
| `crates/contextra-graph/src/tl_hfd/mod.rs` | B | nein | 41110097 |
| `crates/contextra-graph/src/wiring.rs` | H | nein | 7efd126a |
| `crates/contextra-graph/tests/graph_wiring_test.rs` | H | nein | 7efd126a |
| `crates/contextra-graph/tests/hyperedge_suggest.rs` | B | nein | 7efd126a |
| `crates/contextra-graph/tests/hyperedge_suggest_llm_test.rs` | B | nein | 7efd126a |
| `crates/contextra-graph/tests/j14_contextra_graph_graph_schreibpfad_csr_ko_tests.rs` | H | nein | ce650578, c36fe154, 83462a48 |
| `crates/contextra-graph/tests/j14closure-contextra-graph_tests.rs` | H | nein | ce650578, c36fe154 |
| `crates/contextra-graph/tests/j15-contextra-graph-graph-nebenmodule-appr-p_apprh_tests.rs` | H | nein | 41110097 |
| `crates/contextra-graph/tests/j15-contextra-graph-graph-nebenmodule-appr-p_arc_slice_tests.rs` | H | nein | ce650578, 41110097 |
| `crates/contextra-graph/tests/j15-contextra-graph-graph-nebenmodule-appr-p_ppr_cost_and_hook_tests.rs` | H | nein | ce650578, 41110097 |
| `crates/contextra-graph/tests/j15-contextra-graph-graph-nebenmodule-appr-p_ppr_stream_tests.rs` | H | nein | 41110097 |
| `crates/contextra-graph/tests/j15-contextra-graph-graph-nebenmodule-appr-p_session_dag_tests.rs` | H | nein | ce650578, 41110097 |
| `crates/contextra-graph/tests/j15-contextra-graph-graph-nebenmodule-appr-p_tl_hfd_tests.rs` | H | nein | 41110097 |
| `crates/contextra-graph/tests/j15closure-contextra-graph_symbol_verification.rs` | H | nein | ce650578, c3ce9460 |
| `crates/contextra-graph/tests/provenance_delete_cascade.rs` | G | nein | - |
| `crates/contextra-kvcache/AGENTS.md` | D | ja | 426dfead, 56e5769a |
| `crates/contextra-kvcache/Cargo.toml` | E | nein | cea972df, ef39f9f0 |
| `crates/contextra-kvcache/src/eviction_worker.rs` | B | nein | ce650578, d99d4279 |
| `crates/contextra-kvcache/src/lib.rs` | B | nein | f0131b9e |
| `crates/contextra-kvcache/src/prefix_store.rs` | B | nein | f0131b9e, d99d4279 |
| `crates/contextra-kvcache/src/quantize_kivi.rs` | B | nein | f0131b9e |
| `crates/contextra-kvcache/src/scratchpad_invalidation.rs` | B | nein | f0131b9e, 561dcec9, edeb252d, 7240937b, f5cfd215, 1233eb88, f07adb3e, d99d4279 |
| `crates/contextra-kvcache/src/segment.rs` | B | nein | f0131b9e, 240f4f7c, 7e1d65ba, 330c9847, beb3a0bb |
| `crates/contextra-kvcache/src/store.rs` | D | ja | f0131b9e, ce650578, d99d4279, beb3a0bb |
| `crates/contextra-kvcache/src/wiring.rs` | H | nein | a51c4b97, f0131b9e |
| `crates/contextra-kvcache/tests/cache_scope_scan.rs` | H | nein | 35c6bee9, 3d1bd1a3 |
| `crates/contextra-kvcache/tests/e2e_lifecycle_with_engine.rs` | G | nein | - |
| `crates/contextra-kvcache/tests/j06_contextra_kvcache_wiring.rs` | H | nein | f0131b9e, ce650578, beb3a0bb |
| `crates/contextra-kvcache/tests/j06closure-contextra-kvcache_wiring.rs` | H | nein | f0131b9e, ce650578, d99d4279 |
| `crates/contextra-kvcache/tests/kivi_aead_order_enforced.rs` | B | nein | f0131b9e |
| `crates/contextra-kvcache/tests/kv_lifecycle_hooks_tenant_isolation.rs` | D | ja | ce650578 |
| `crates/contextra-kvcache/tests/two_tenant_isolation_and_segment_validation.rs` | H | nein | f0131b9e |
| `crates/contextra-rank/AGENTS.md` | D | ja | c2732639, 426dfead, 56e5769a |
| `crates/contextra-rank/Cargo.toml` | B | nein | c2732639, b5f62239, cea972df, ef39f9f0 |
| `crates/contextra-rank/src/calibration/conformal.rs` | F | nein | c2732639, 8e71bac8, 538dd199 |
| `crates/contextra-rank/src/calibration/isotonic.rs` | D | nein | c2732639 |
| `crates/contextra-rank/src/calibration/mod.rs` | B | nein | c2732639 |
| `crates/contextra-rank/src/calibration/platt.rs` | B | nein | 8e71bac8, 538dd199 |
| `crates/contextra-rank/src/dibud/driver.rs` | F | nein | c2732639 |
| `crates/contextra-rank/src/dibud/mod.rs` | F | nein | c2732639 |
| `crates/contextra-rank/src/dibud/state.rs` | F | nein | c2732639, ce650578, 8e71bac8, 538dd199 |
| `crates/contextra-rank/src/dibud/types.rs` | F | nein | c2732639, 8e71bac8 |
| `crates/contextra-rank/src/drift.rs` | B | nein | 42cf7835, 8f9e27ab, f9c2645f |
| `crates/contextra-rank/src/fusion/mod.rs` | B | nein | 42cf7835 |
| `crates/contextra-rank/src/fusion/rerank.rs` | H | nein | 42cf7835 |
| `crates/contextra-rank/src/fusion/rrf.rs` | D | ja | af63a482, bd0a2525, 240f4f7c, 7e1d65ba, 5f5251e9, 67c7ca58 |
| `crates/contextra-rank/src/lib.rs` | B | nein | c2732639, 42cf7835, 8f9e27ab |
| `crates/contextra-rank/tests/calibration_deep_tests.rs` | B | nein | b5f62239 |
| `crates/contextra-rank/tests/calibration_stress_and_fault_tests.rs` | B | nein | b5f62239 |
| `crates/contextra-rank/tests/calibration_zero_panic_test.rs` | H | nein | 42cf7835 |
| `crates/contextra-rank/tests/conformal_tests.rs` | F | nein | c2732639 |
| `crates/contextra-rank/tests/dibud_equivalence.rs` | F | nein | c2732639 |
| `crates/contextra-rank/tests/dibud_requires_streaming_channels.rs` | F | nein | c2732639 |
| `crates/contextra-rank/tests/isotonic_ece_rebuild.rs` | D | ja | ce650578 |
| `crates/contextra-rank/tests/j12_contextra_rank_platt_fitted.rs` | H | nein | 538dd199 |
| `crates/contextra-rank/tests/j12closure-contextra-rank_platt_fitted.rs` | H | nein | ce650578, 8e71bac8 |
| `crates/contextra-rank/tests/rerank_stage_test.rs` | H | nein | 42cf7835 |
| `crates/contextra-rank/tests/rrf_two_phase_scaling.rs` | D | ja | ce650578 |
| `crates/contextra-text/AGENTS.md` | D | ja | 56e5769a |
| `crates/contextra-text/src/bm25.rs` | B | nein | f2d4b245 |
| `crates/contextra-text/src/domain/legal_de.rs` | B | nein | f2d4b245 |
| `crates/contextra-text/src/domain/medical_de.rs` | B | nein | f2d4b245 |
| `crates/contextra-text/src/inverted.rs` | C | nein | b713f897 |
| `crates/contextra-text/src/inverted/index_struct.rs` | C | nein | b713f897 |
| `crates/contextra-text/src/inverted/residue_scan.rs` | C | nein | 4b7709e6, b713f897 |
| `crates/contextra-text/src/lib.rs` | D | ja | f2d4b245 |
| `crates/contextra-text/src/morphology/compound_splitter.rs` | B | nein | f2d4b245, 62694d1a, 9ed880b5 |
| `crates/contextra-text/src/morphology/passthrough.rs` | B | nein | f2d4b245 |
| `crates/contextra-text/src/morphology/tests.rs` | B | nein | f2d4b245 |
| `crates/contextra-text/src/posting_list.rs` | D | nein | ce650578, 62694d1a, 9ed880b5, e2f354ed |
| `crates/contextra-text/src/tokenizer.rs` | B | nein | f2d4b245, 4b7709e6, 2d5c5b0d, 8cedf9fb |
| `crates/contextra-text/src/wiring.rs` | C | nein | f2d4b245 |
| `crates/contextra-text/tests/domain_compound_integration.rs` | B | nein | f2d4b245 |
| `crates/contextra-text/tests/domain_vocab_quality.rs` | B | nein | f2d4b245 |
| `crates/contextra-text/tests/j31-contextra-text_wiring_test.rs` | C | nein | ce650578, 9ed880b5 |
| `crates/contextra-text/tests/j31closure-contextra-text_closure_verification_test.rs` | C | nein | ce650578, 62694d1a |
| `crates/contextra-text/tests/pipeline_identity_regression.rs` | B | nein | 4b7709e6, a3218ad1 |
| `crates/contextra-text/tests/text_residue_injection.rs` | C | nein | 4b7709e6, b713f897 |
| `crates/contextra-text/tests/text_residue_scan.rs` | C | nein | 4b7709e6, b713f897 |
| `crates/contextra-vector/AGENTS.md` | D | ja | 426dfead, 56e5769a |
| `crates/contextra-vector/Cargo.toml` | B | nein | 3dece684, cea972df, ef39f9f0, 4b246d3f |
| `crates/contextra-vector/benches/acorn_vs_post_filter.rs` | B | nein | 94601244 |
| `crates/contextra-vector/benches/audit_benchmarks.rs` | B | nein | 94601244 |
| `crates/contextra-vector/benches/distance_nan_impact_bench.rs` | B | nein | 94601244 |
| `crates/contextra-vector/benches/filtered_search_selectivity_bench.rs` | F | nein | 94601244 |
| `crates/contextra-vector/benches/flush_threshold_amplification.rs` | F | nein | 94601244 |
| `crates/contextra-vector/benches/hnsw_bench.rs` | B | nein | 94601244 |
| `crates/contextra-vector/benches/hnsw_delete_search_bench.rs` | B | nein | 94601244 |
| `crates/contextra-vector/benches/overfetch_selectivity.rs` | B | nein | 94601244 |
| `crates/contextra-vector/benches/partial_rebuild_recall_bench.rs` | B | nein | 94601244 |
| `crates/contextra-vector/benches/sq8_bench.rs` | B | nein | 94601244 |
| `crates/contextra-vector/fuzz/Cargo.toml` | B | nein | 94601244 |
| `crates/contextra-vector/fuzz/fuzz_targets/fuzz_hnsw_persistence.rs` | B | nein | 94601244 |
| `crates/contextra-vector/fuzz/fuzz_targets/hnsw_insert_search.rs` | B | nein | 94601244 |
| `crates/contextra-vector/src/acorn/mod.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/acorn/naive_reference.rs` | B | nein | 3dece684, 43086624 |
| `crates/contextra-vector/src/candidate_stream.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/diskann/build.rs` | F | nein | 3dece684, f196a0e0, 8cedf9fb, 525ecd80, a2b063dc, bb1e833e, 00b7d22a, 43086624 |
| `crates/contextra-vector/src/diskann/config.rs` | F | nein | 3dece684, bb1e833e |
| `crates/contextra-vector/src/diskann/filtered.rs` | F | nein | 3dece684 |
| `crates/contextra-vector/src/diskann/format.rs` | F | nein | 3dece684 |
| `crates/contextra-vector/src/diskann/mod.rs` | F | nein | 3dece684, bb1e833e |
| `crates/contextra-vector/src/diskann/persistence.rs` | D | ja | 3dece684, 769808a9, f196a0e0, 8cedf9fb, 525ecd80, a2b063dc, bb1e833e, 00b7d22a, 7deb6e6d, 52e6ea7e, ce650578 |
| `crates/contextra-vector/src/diskann/predicate_augmented.rs` | F | nein | 3dece684 |
| `crates/contextra-vector/src/diskann/search.rs` | F | nein | 3dece684 |
| `crates/contextra-vector/src/diskann/tests.rs` | F | nein | 3dece684, 4fb1e45b, bb1e833e, 00b7d22a |
| `crates/contextra-vector/src/diskann/types.rs` | D | ja | 3dece684, ce650578, cd529156 |
| `crates/contextra-vector/src/diskann/vector_index_impl.rs` | F | nein | 3dece684 |
| `crates/contextra-vector/src/distance.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/hnsw/acorn_filtered.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/hnsw/adaptive_ef.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/hnsw/arena.rs` | B | nein | 3dece684, c19a81f2, b713f897, 240f4f7c, 7e1d65ba, cea972df, e3fdea29, 52e6ea7e |
| `crates/contextra-vector/src/hnsw/batch.rs` | B | nein | 3dece684, 240f4f7c, 7e1d65ba, 72f14e96, 4fb1e45b, 1f279512 |
| `crates/contextra-vector/src/hnsw/config.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/hnsw/connections.rs` | H | nein | 3dece684, 240f4f7c, 7e1d65ba, 72f14e96 |
| `crates/contextra-vector/src/hnsw/core_insert.rs` | B | nein | 3dece684, 240f4f7c, 7e1d65ba, 72f14e96, 1f279512, e3fdea29, 52e6ea7e, ce650578, 43086624 |
| `crates/contextra-vector/src/hnsw/core_rebuild.rs` | D | ja | 3dece684, c19a81f2, e69edff4, b713f897, a8a8e2db, 3a4c5192, 4fb1e45b, 43086624, adf205ab, e31ed5c4 |
| `crates/contextra-vector/src/hnsw/core_search.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/hnsw/deletion.rs` | B | nein | 3dece684, 3a14af69, 240f4f7c, 7e1d65ba, c17f3225, 72f14e96, 4fb1e45b, 87eb6287, 1f279512, dd18772f, 52e6ea7e |
| `crates/contextra-vector/src/hnsw/durable_fs.rs` | H | nein | 3dece684, 4fb1e45b, 1f279512 |
| `crates/contextra-vector/src/hnsw/mod.rs` | B | nein | b713f897, 240f4f7c, 7e1d65ba, 72f14e96, 1f279512 |
| `crates/contextra-vector/src/hnsw/residue_scan.rs` | H | nein | 3dece684, b713f897 |
| `crates/contextra-vector/src/hnsw/sq8_bias.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/hnsw/tests.rs` | B | nein | 3dece684, 1f279512 |
| `crates/contextra-vector/src/hnsw/types.rs` | B | nein | 3dece684, 51e1e765, e69edff4, 3a14af69, 240f4f7c, 7e1d65ba, 72f14e96, 1f279512 |
| `crates/contextra-vector/src/hnsw/vector_index_impl.rs` | B | nein | 3dece684, 35c6bee9, e69edff4, b713f897, c518ad30, 8cedf9fb, 525ecd80, a2b063dc, 3a4c5192, 4fb1e45b, 1f279512 |
| `crates/contextra-vector/src/hnsw/verify.rs` | H | nein | 1f279512 |
| `crates/contextra-vector/src/hnsw/witness.rs` | H | nein | 3dece684, 35c6bee9, e69edff4, b713f897 |
| `crates/contextra-vector/src/lib.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/persistence/header.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/persistence/mmap.rs` | B | nein | 3dece684, 4fb1e45b, 1ff44716, e0077c62 |
| `crates/contextra-vector/src/persistence/node.rs` | B | nein | 3dece684 |
| `crates/contextra-vector/src/persistence/tests.rs` | B | nein | 3dece684, 4fb1e45b, e0077c62 |
| `crates/contextra-vector/src/quantize/mod.rs` | D | ja | 3dece684, 4fb1e45b, 00518b9e, 52e6ea7e, ce650578 |
| `crates/contextra-vector/src/quantize/tests.rs` | B | nein | 3dece684, 8cedf9fb, 525ecd80, a2b063dc |
| `crates/contextra-vector/src/quantize_rabitq.rs` | D | ja | 3dece684, ce650578 |
| `crates/contextra-vector/tests/acorn_hnsw_differential.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/acorn_naive_recall.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/adaptive_ef_recall.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/campaign_hnsw_backoff.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/campaign_j23_vector_index.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/campaign_j24_test.rs` | B | nein | 94601244, 4fb1e45b, 00b7d22a |
| `crates/contextra-vector/tests/campaign_j41_vector.rs` | C | nein | ce650578 |
| `crates/contextra-vector/tests/deleted_nodes_lock_contention.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/deletion_ghost_pointer_proptest.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/determinism_test.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/differential_testing.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/diskann_corruption_fallback_test.rs` | F | nein | 94601244, 00b7d22a |
| `crates/contextra-vector/tests/diskann_docid128_tombstones.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/diskann_fault_injection_test.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/diskann_recall_test.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/diskann_search_filtered.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/distance_determinism.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/docid_128_hnsw.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/docid_high_bits_no_collision.rs` | D | ja | 94601244 |
| `crates/contextra-vector/tests/filtered_selectivity_recall.rs` | D | ja | 94601244, 4fb1e45b, aa50a53b, 7e5ef3e6, ce650578 |
| `crates/contextra-vector/tests/hnsw_commit_semantics.rs` | H | nein | 94601244, 3a4c5192 |
| `crates/contextra-vector/tests/hnsw_concurrent_ops.rs` | H | nein | 94601244, 35c6bee9, e69edff4 |
| `crates/contextra-vector/tests/hnsw_delete_exhaustive_match.rs` | H | nein | 94601244, c518ad30 |
| `crates/contextra-vector/tests/hnsw_edge_cases.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/hnsw_layer_determinism.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/hnsw_mmap_ghost_pointer_repair.rs` | H | nein | 94601244, 240f4f7c, 7e1d65ba, 72f14e96 |
| `crates/contextra-vector/tests/hnsw_rebuild_search_consistency.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/hnsw_residue_injection.rs` | H | nein | 94601244, 4b7709e6, b713f897 |
| `crates/contextra-vector/tests/hnsw_residue_scan.rs` | H | nein | 94601244, 4b7709e6, b713f897 |
| `crates/contextra-vector/tests/hnsw_seed_determinism.rs` | D | nein | 94601244 |
| `crates/contextra-vector/tests/hnsw_slot_reuse_ghost_edges.rs` | H | nein | 94601244, 2a1af3f7, 240f4f7c, 7e1d65ba, 66f78ca3, cea972df, 050c6234 |
| `crates/contextra-vector/tests/hnsw_snapshot_delete_test.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/hnsw_witness_matrix.rs` | H | nein | 94601244, 35c6bee9, e69edff4 |
| `crates/contextra-vector/tests/hnsw_zeroize_no_residue.rs` | H | nein | 94601244, c19a81f2 |
| `crates/contextra-vector/tests/j32closure-contextra-vector_acorn_test.rs` | H | nein | 94601244, ce650578, 43086624 |
| `crates/contextra-vector/tests/j32closure-contextra-vector_candidate_stream_test.rs` | H | nein | 94601244, 43086624 |
| `crates/contextra-vector/tests/j32closure-contextra-vector_free_node_test.rs` | H | nein | 94601244, 43086624 |
| `crates/contextra-vector/tests/j32closure-contextra-vector_persistence_test.rs` | H | nein | ce650578, 43086624 |
| `crates/contextra-vector/tests/k02_snapshot_rebuild_determinism.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/loom_quantizer_race_test.rs` | B | nein | 94601244, cea972df, 182db152 |
| `crates/contextra-vector/tests/mmap_toctou_test.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/nan_validation_policy.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/p03a_hnsw_arena_alloc_failure_atomic_insert.rs` | H | nein | 94601244, e3fdea29 |
| `crates/contextra-vector/tests/p03b_hnsw_ghost_pointer_verification_complete.rs` | H | nein | 94601244, 1f279512, dd18772f |
| `crates/contextra-vector/tests/p03c_scalar_quantizer_invalid_training_not_silent.rs` | H | nein | 94601244, 4fb1e45b, 00518b9e |
| `crates/contextra-vector/tests/partial_rebuild_recall.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/partial_rebuild_recall_regression.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/partial_rebuild_veto_f02_guard.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/poisoning.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/proptest_distance_quantize.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/quantize_persistence_audit.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/quantizer_encapsulation_regression.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/rabitq_calibration_recall.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/rabitq_recall_calibration.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/ram_reduction.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/recalibration.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/recall.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/recall_audit.rs` | F | nein | 94601244 |
| `crates/contextra-vector/tests/recall_vs_brute_force.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/reconstruction_attack_test.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/rollback.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/simd_numerical_audit.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/sq8_bias_calibration.rs` | B | nein | 94601244 |
| `crates/contextra-vector/tests/sq8_rerank_recall.rs` | D | nein | 94601244 |
| `crates/contextra-vector/tests/vector_candidate_stream.rs` | B | nein | 94601244 |

---

## Entscheidung nötig

Keine offenen Fragen der Klasse `DECISION` verblieben. Alle 86 Commits konnten eindeutig klassifiziert werden.

## Abhängigkeiten und Reihenfolge

1. **Ring 0 / Ports Unterbau (DEL-R0):** Commits wie `51e1e765` und `35c6bee9` sind bereits in Ring 0 / Ports vorhanden.
2. **Typen & Traits / Port-Requirements (PORT-REQUIRED):** Commit `ef39f9f0` stellt Cargo.toml Minimal-Features für Tokio bereit.
3. **Composition-Root-Verdrahtung (PORT-WIRING):** Durch die Freigabe `WIRING-FREIGABE: JA` können Module in `vector`, `text`, `rank`, `kvcache` und `graph` direkt verdrahtet werden (`f2d4b245`, `42cf7835`, `7efd126a`, `62694d1a`, `8e71bac8`, `c36fe154`, `d99d4279`, `43086624`, `83462a48`, `41110097`, `9ed880b5`, `beb3a0bb`, `538dd199`).
4. **Löschnachweis (DEL-R1, DEL-R6):** Die Löschnachweis-Anteile in `kvcache` (R1) sowie `vector` und `graph` (R6) werden durch die dedizierten Portierungs-Aufträge R1 und R6 abgedeckt.
5. **Fehlerkorrekturen (PORT-FIX):** Korrekturen wie `e2f354ed` (Posting List integrity fix in `text`), `00518b9e` (ScalarQuantizer training error propagation in `vector`), `e3fdea29` (HNSW Arena allocation failure handling in `vector`), `1ff44716` / `e0077c62` (Mmap length & lock guard in `vector`) und `af63a482` (DriftDetector sanitization in `rank`) basieren auf bestehendem `main`-Code und stellen Portierungs-Kandidaten dar.

## Auffälligkeiten

- **Entfernung von DiskANN und RaBitQ in `contextra-vector`:** Im Commit `3dece684` (späterer Stand vor Core-Cut) wurden `DiskANN` (`src/diskann/*`) und `RaBitQ` (`src/quantize_rabitq.rs`) entfernt. Alle vorigen Commits (`bb1e833e`, `00b7d22a`, `7deb6e6d`, `f196a0e0`, `cd529156`), die DiskANN bearbeiteten, sind durch diese Löschung im Netto-Zustand des Spenders `eb10652f` gegenstandslos und als `SKIP-SUPERSEDED` eingestuft.
- **Entfernung von Dibud und Conformal Calibration in `contextra-rank`:** Im Commit `c2732639` wurden Dibud (`src/dibud/*`) und Conformal Calibration (`src/calibration/conformal.rs`) entfernt. Entsprechende Frühere Commits wurden ebenfalls als `SKIP-SUPERSEDED` eingestuft.
- **Null-Aktionen / Shell-Commits:** Insgesamt 9 Commits (`426dfead`, `4b7709e6`, `561dcec9`, `edeb252d`, `8cedf9fb`, `a2b063dc`, `240f4f7c`, `52e6ea7e`, `e31ed5c4`) enthalten keine Änderungen an Produktions- oder Test-Quelltext im Pfadbereich.

## Löschungen respektiert

Folgende in `main` nach dem Reset gelöschte Test- und Hilfsdateien verbleiben gelöscht gem. Maintainer-Vorgabe (Backup 8412f086):
- `crates/contextra-graph/tests/provenance_delete_cascade.rs`
- `crates/contextra-kvcache/tests/e2e_lifecycle_with_engine.rs`
- `crates/contextra-kvcache/tests/kv_lifecycle_hooks_tenant_isolation.rs`
- `crates/contextra-rank/tests/isotonic_ece_rebuild.rs`
- `crates/contextra-rank/tests/rrf_two_phase_scaling.rs`
- `crates/contextra-vector/tests/docid_high_bits_no_collision.rs`
