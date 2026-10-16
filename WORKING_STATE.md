# Contextra — Globaler Arbeitsstand (WORKING_STATE.md)

> **Stand:** 2026-10-16 | **Format:** v2 (Automatisierte Generierung via `cargo xtask sync-docs`)

## 1. Systemstatus & Architektur
Contextra ist eine hochperformante, einbettbare Vektor- und Graph-Engine für Rust.

## 2. Session-Kontinuität
- **Standard-Session-ID:** `1539947784225388476` (automatische Kontinuität, falls kein Override per Flag)
- **Offene contextra_tags (DEBT/ANCHOR/AI-TAG):** 7 (siehe Tabelle unten)
- **Aktive Task-Claims (.jules/claims.toml):** 0

## 3. Crate-Inventar & Ringe
| Crate | Pfad | Layer | LOC | Status | Ring | Reife | Beschreibung |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `contextra-types` | `crates/contextra-types` | 0 | 5.5k | 🟢 stable | `Ring 0` | `stable` | Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen) |
| `contextra-ports` | `crates/contextra-ports` | 1 | 4.3k | 🟢 stable | `Ring 0` | `stable` | Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung |
| `contextra-sys` | `crates/contextra-sys` | 0 | 2.5k | 🟢 stable | `Ring 0` | `stable` | Systemnahe Bindings, Low-Level I/O und Memory-Mapping |
| `contextra-wire` | `crates/contextra-wire` | 0 | 1.5k | 🟢 stable | `Ring 0` | `stable` | FlatBuffers Wire-Protokolle und Serialisierungs-Formate |
| `contextra-simd` | `crates/contextra-simd` | 1 | 2.6k | 🟢 stable | `Ring 0` | `stable` | SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen |
| `contextra-durable-fs` | `crates/contextra-durable-fs` | 1 | 1.0k | 🟢 stable | `Ring 0` | `stable` | Durable Filesystem Operations und synchrone Disk-Flushes |
| `contextra-crypto` | `crates/contextra-crypto` | 2 | 14.2k | 🟢 stable | `Ring 0` | `stable` | Kryptographische Vaults, Zeroize, Egress-Verschlüsselung und HMAC |
| `contextra-mvcc` | `crates/contextra-mvcc` | 2 | 3.6k | 🟢 stable | `Ring 0` | `stable` | Multi-Version Concurrency Control und Isolation-Mechanismen |
| `contextra-text` | `crates/contextra-text` | 2 | 4.9k | 🟢 stable | `Ring 0` | `stable` | Volltextsuche und Inverted-Index (BM25 Engine) |
| `contextra-rank` | `crates/contextra-rank` | 2 | 3.2k | 🟢 stable | `Ring 0` | `stable` | Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen (RRF, Isotonische/Platt Kalibrierung, Bonus Scorers) |
| `contextra-store` | `crates/contextra-store` | 3 | 27.1k | 🟢 stable | `Ring 1` | `stable` | LSM-Tree Storage Engine, WAL Durability und Compaction Engine |
| `contextra-vector` | `crates/contextra-vector` | 3 | 10.7k | 🟢 stable | `Ring 0` | `stable` | Vektorsuche und Indexierung (HNSW, DiskANN, Quantisierung) |
| `contextra-checkpoint` | `crates/contextra-checkpoint` | 3 | 4.7k | 🟢 stable | `Ring 1` | `stable` | Persistent Checkpoint Engine und Blake3 Manifest-Verifikation |
| `contextra-engine` | `crates/contextra-engine` | 4 | 15.4k | 🟢 stable | `Ring 3` | `stable` | Compute Pool und Asynchrone Storage-Execution Layer (ADR-N02) |
| `contextra-db` | `crates/contextra-db` | 5 | 2.2k | 🟢 stable | `Ring 3` | `stable` | Contextra Main Database Abstraction, Hybrid Search & Collections |
| `contextra` | `crates/contextra` | 6 | 1.4k | 🟢 stable | `Ring 4` | `stable` | Haupt-Library Facade für Endanwender |
| `contextra-mcp` | `crates/contextra-mcp` | 3 | 6.7k | 🟢 stable | `Ring 4` | `stable` | Model Context Protocol Server & Cloud Egress Gateway |
| `contextra-testkit` | `crates/contextra-testkit` | 4 | 3.9k | 🟢 stable | `Tooling` | `stable` | Test Fixtures, Mock Engines und Chaos Matrix Harness |
| `contextra-bench` | `benchmarks/contextra-bench` | 6 | 6.2k | 🟢 stable | `Tooling` | `stable` | Contextra Reproducible Benchmark Harness for Retrieval Accuracy |
| `xtask-heavy` | `xtask-heavy` | 7 | 83 | 🟡 experimental | `Tooling` | `experimental` | Heavy Xtask Integration Benchmark und Long-Running Heavy Verification Tasks |

## 4. Invarianten & Qualitäts-Gates
| Invariante / Gate | Beschreibung | Ziel-Ebene | status |
| :--- | :--- | :--- | :--- |
| **Zero-Panic (P7)** | Kein unwrap/expect in Produktionscode | Ring 0, Ring 1 | 🟢 Aktiv |
| **Ring-Layering** | Strikter gerichteter Azyklischer Graph | Alle Crates | 🟢 Aktiv |
| **Async Purity** | Keine Async-Blockaden in Ring 0 | Ring 0 | 🟢 Aktiv |
| **Determinismus** | Reproduzierbarkeit via Port-Injektion | Ring 0 | 🟢 Aktiv |

## 5. DAG & Topologie
### Layer 0 (Rang 0)
- **`contextra-types`** (Ring: `Ring 0`) -> Abhaengigkeiten: keine
- **`contextra-sys`** (Ring: `Ring 0`) -> Abhaengigkeiten: keine
- **`contextra-wire`** (Ring: `Ring 0`) -> Abhaengigkeiten: keine

### Layer 1 (Rang 1)
- **`contextra-ports`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-types
- **`contextra-simd`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-types
- **`contextra-durable-fs`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-sys

### Layer 2 (Rang 2)
- **`contextra-crypto`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-durable-fs, contextra-ports, contextra-types
- **`contextra-mvcc`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-text`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-rank`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types

### Layer 3 (Rang 3)
- **`contextra-store`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-crypto, contextra-durable-fs, contextra-mvcc, contextra-ports, contextra-sys, contextra-types
- **`contextra-vector`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-crypto, contextra-durable-fs, contextra-mvcc, contextra-ports, contextra-simd, contextra-sys, contextra-types
- **`contextra-checkpoint`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-mvcc, contextra-ports, contextra-types
- **`contextra-mcp`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-crypto, contextra-ports, contextra-types, contextra-wire

### Layer 4 (Rang 4)
- **`contextra-engine`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-checkpoint, contextra-crypto, contextra-mvcc, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector
- **`contextra-testkit`** (Ring: `Tooling`) -> Abhaengigkeiten: contextra-ports, contextra-store, contextra-types

### Layer 5 (Rang 5)
- **`contextra-db`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-checkpoint, contextra-crypto, contextra-engine, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector

### Layer 6 (Rang 6)
- **`contextra`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-db, contextra-engine, contextra-ports, contextra-rank, contextra-store, contextra-types
- **`contextra-bench`** (Ring: `Tooling`) -> Abhaengigkeiten: contextra-crypto, contextra-db, contextra-ports, contextra-store, contextra-text, contextra-types, contextra-vector

### Layer 7 (Rang 7)
- **`xtask-heavy`** (Ring: `Tooling`) -> Abhaengigkeiten: contextra-bench


## 6. KI-Tags & Schulden-Register
| Datei | Zeile | Typ | Schweregrad | Beschreibung | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `xtask/src/check_stale_tags.rs` | 376 | `AI-TAG` | **MINOR** | TODO(audit-C-1): Fix startup flush. (ID: AGT-STORE-bae66245) (TS: 2026-09-10T19:14:58Z) (SESSION: 21a8d3e8) | OPEN |
| `xtask/src/check_stale_tags.rs` | 378 | `AI-TAG` | **MINOR** | Non-audit tag (TS: 2026-09-10T19:14:58Z) | OPEN |
| `xtask/src/check_stale_tags.rs` | 409 | `AI-TAG` | **MINOR** | TODO(audit-1.1): Old stale audit tag (ID: AGT-TEST-00000001) (TS: 2026-01-01T00:00:00Z) (SESSION: 00000001) | OPEN |
| `xtask/src/check_stale_tags.rs` | 410 | `AI-TAG` | **MINOR** | TODO(audit-1.2): Recent audit tag (ID: AGT-TEST-00000002) (TS: 2026-09-10T00:00:00Z) (SESSION: 00000002) | OPEN |
| `crates/contextra-db/tests/split_brain_vector_orphan_test.rs` | 2 | `ANCHOR` | **INFO** | STATUS:IN_PROGRESS (TS:2026-09-09T22:00:00Z) (SESSION: 504d02fc) | IN_PROGRESS |
| `crates/contextra-store/src/compaction/engine.rs` | 718 | `AI-TAG` | **MINOR** | BOUND_UNIFICATION: Standardize boundary check (ID: AGT-STORE-c7e14a2b) (TS: 2026-10-12T17:20:27Z) (SESSION: a832a6e7) | OPEN |
| `benches/migration_benchmarks.rs` | 1 | `ANCHOR` | **INFO** | STATUS:PARTIAL (TS:2026-08-29T00:00:00Z) — Benchmark Suite für LangGraph Migration | PARTIAL |

## 7. Chronik & Modifikations-Historie
| Datum | Komponente | Beschreibung | Tag-ID | Session |
| :--- | :--- | :--- | :--- | :--- |
| TS:2026-10 | `crates/contextra-db/src/multistep.rs` | // STAND: TS:2026-10-15T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-store/tests/block_cache_invalidation.rs` | // STAND: TS:2026-10-12T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-crypto/src/secure_buf.rs` | // STAND: TS:2026-10-12T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-crypto/tests/secure_buf.rs` | // STAND: TS:2026-10-12T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-vector/src/quantize/mod.rs` | // STAND: TS:2026-10-06T12:00:00Z (SESSION: P03-F03) | `-` | `P03-F03` |
| TS:2026-10 | `crates/contextra-crypto/tests/campaign_j25.rs` | // STAND: TS:2026-10-04 (Kampagne J-25) | `-` | `-` |
| TS:2026-09 | `crates/contextra-engine/tests/tenant_wrapper_coverage.rs` | // STAND: TS:2026-09-28 | `-` | `-` |
| TS:2026-09 | `crates/contextra-crypto/tests/deletion_proof_hash_collision.rs` | // STAND: TS:2026-09-27T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-store/tests/wal_recovery_runtime_stall.rs` | // STAND: TS:2026-09-26T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-crypto/tests/legacy_hmac_collision_property.rs` | // STAND: TS:2026-09-26T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | // STAND: TS:2026-09-25T15:10:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-crypto/tests/deletion_proof_prefix_collision_proptest.rs` | // STAND: TS:2026-09-17T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-engine/src/chunker.rs` | // STAND: TS:2026-09-17T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-vector/src/candidate_stream.rs` | // STAND: TS:2026-09-15T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-text/src/stream.rs` | // STAND: TS:2026-09-14T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-crypto/src/kv_segment/store.rs` | // STAND: TS:2026-09-13T00:00:00Z | `-` | `-` |
| TS:2026-09 | `xtask/src/lint_unsafe_slice_bounds.rs` | // STAND: TS:2026-09-12T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-db/tests/consolidation_double_trigger_test.rs` | // STAND: TS:2026-09-12T00:00:00Z | `-` | `-` |
| TS:2026-09 | `crates/contextra-engine/src/import.rs` | // STAND: TS:2026-09-12 | `-` | `-` |
| TS:2026-09 | `crates/contextra-engine/src/export.rs` | // STAND: TS:2026-09-12 | `-` | `-` |
