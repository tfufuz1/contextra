---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "19"
---
## 19. Rückverfolgbarkeitsmatrix

| Bereich | Reifegrad | Verweis |
|---|---|---|
| Key-granulare `kv_locks` statt collection-weitem Mutex | 🟢 | §5.1, §5.2a |
| HNSW-SIMD-Hotpath (unaligned Distanzkernel, `AHashSet`-Vorallokation, `try_write()`-Pruning) | 🟢 | §7.4 |
| SQ8-Perzentil-Clipping | 🟢 | §7.4 |
| BM25 residenter Index + Block-Max WAND | 🟢 | §7.3 |
| BM25F feldgewichtete Bewertung | 🟢 | §7.3 |
| Block-Cache: klassisches LRU | 🟢 (Default) | §5.4 |
| Block-Cache: SIEVE/S3-FIFO | 🟡 | §5.4 |
| Sherman-Morrison-Bandit + CI-Latency-Gate | 🟡 (Opt-in) / Gate 🟢 | §8 |
| Leiden statt Label-Propagation (binärer Pfad) | 🟢 | §7.5 |
| RCU-Snapshot-Swap für `CsrGraph::compact()` | 🟢 | §6.3 |
| Native DiskANN-Tombstones | 🟢 | §7.4 |
| DocId-128-Bit-Migration (Rollout) | 🟡 | §6.1 |
| Score-normalisierte Fusion mit RRF-Fallback | 🟡 (Opt-in) | §7.1 |
| Forward-Push-PPR | 🟢 | §7.2 |
| `ProvenanceBuilder`-Struktur | 🟢 | §7.6 |
| FlatBuffers-CI-Drift-Gate | 🟢 | §12 |
| Cloud-Egress Fünf-Schichten (Surrogat, Bulk, Rehydration) | 🟢 | §10.4 |
| KV-Cache-Bridge LSM-Fallback-Spill | 🟢 | §9.2 |
| Bandit-Drift-Alpha-Eskalation gedeckelt | 🟢 | §8.3 |
| Loom-Test sichtbar grün in CI | 🔴 | §15.3 |
| HNSW-Dateiformat v2 (Arena) | 🔴 | §7.4 |
| RaBitQ/PQ-Quantisierung jenseits SQ8 | 🔴 | §7.4 |
| ADR-Formalrevision (Leiden statt LPA) | 🔴 (Dokumentation) | §18 |
| **N-äre Hyperkanten (gesamt: H1–H6, `relate_n_ary`)** | **🔴** | §6 |
| WAL-Replay-Panic-Fix | ⚠️ Opus 0.1 | §5.3, §17 |
| Bandit-Dimensionsprüfung | ⚠️ Opus 0.2 | §8.2, §17 |
| Drift-Bandit-Kopplung verdrahten | ⚠️ Opus 0.3 | §8.2, §17 |
| Egress-Klassifizierung korrigieren | ⚠️ Opus 0.4 | §10.4, §17 |
| Intent-Recovery differenzieren | ⚠️ Opus 0.5 | §5.3, §17 |
| HNSW-Nachbarlisten-Allokation | ⚠️ Opus 1.1 | §7.4, §17 |
| HNSW-Backlink O(1) | ⚠️ Opus 1.2 | §7.4, §17 |
| Distanzpfad Lock/Allokation | ⚠️ Opus 1.3 | §7.4, §17 |
| SSTable Zero-Copy-Slice | ⚠️ Opus 1.4 | §5.5, §17 |
| AES-Schlüsselplan wiederverwenden | ⚠️ Opus 1.5 | §9.3, §17 |
| MemTable Range-Sharding | ⚠️ Opus 1.6 | §5, §17 |
| Block-Cache byte-basiert | ⚠️ Opus 1.7 | §5.4, §17 |
| `build_provenance` Struct | ⚠️ Opus 1.8 | §7.6, §17 |
| Text-Posting-Format | ⚠️ Opus 1.9 | §7.3, §17 |
| Top-k-Selektion | ⚠️ Opus 1.10 | §7.1, §17 |
| Graph inkrementelle Kompaktierung | ⚠️ Opus 2.1 | §6.3, §17 |
| CSR-Sentinel statt Option | ⚠️ Opus 2.2 | §6.3, §17 |
| Checkpoint-Index-Merge | ⚠️ Opus 2.3 | §17 |
| Manifest-Batch-Fsync | ⚠️ Opus 2.4 | §5, §17 |
| `contextra-py` in Root-Workspace | ⚠️ Opus 2.5 | §9.4, §17 |
| Feature-Powerset CI | ⚠️ Opus 3.1 | §15.4, §17 |
| Panic-Inventar-Gate | ⚠️ Opus 3.2 | §15.4, §17 |

---

<a id="20-migration-v2"></a>
