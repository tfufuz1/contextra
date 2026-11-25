# AGENTS.md — contextra-vector
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.31

## 1. Zweck
Vektorsuch-Engine des Contextra-Systems (Signal 1 der 4-Signal-Fusion). Implementiert Hierarchical Navigable Small World (HNSW) Graphen, 8-Bit Skalar-Quantisierung (SQ8) sowie DiskANN Out-of-Core-Suche (hinter `experimental-diskann`). Bietet SIMD-beschleunigte Distanzberechnungen durch Delegierung an `contextra-simd` und implementiert `VectorIndex` aus `contextra-ports`.

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | Modul-Deklaration, `#![forbid(unsafe_code)]` Crate-Boundary |
| `hnsw/` | `HnswIndexCore`, Graph-Traversal, Heuristic Node Selection (`vector_index_impl.rs`, `core_insert.rs`, `core_search.rs`, `core_rebuild.rs`) |
| `diskann/` | Out-of-Core DiskANN Vektorindex (`filtered.rs`, `predicate_augmented.rs`, `persistence.rs`, `vector_index_impl.rs`) |
| `acorn/` | ACORN Filtered Vector Search Extensions (`gamma_augmentation.rs`, `naive_reference.rs`) |
| `persistence/` | Snapshots, Binärformat Header (`header.rs`), Memory-Mapping (`mmap.rs`), Knoten-Format (`node.rs`) |
| `quantize/` | `ScalarQuantizer` (SQ8), Asymmetrische/Symmetrische Distanz-Approximation |
| `distance.rs` | Safe Abstraktionsschicht für Distanzmetriken (Cosine, Euclidean, DotProduct) via `contextra-simd` |
| `partial_rebuild.rs` | Inkrementelle HNSW-Graph-Härtung und Rebuild-Optionen (VETO-F02) |
| `compute_pool.rs` | Dedizierter Synchroner Threadpool für rechenintensive Vektoroperationen |
| `candidate_stream.rs` | Candidate Streaming Iteratoren für Vektor-Suche |
| `quantize_rabitq.rs` | Experimental RaBitQ Quantisierungs-Prototyp |

## 3. Invarianten

- **VETO-F02 / ADR-097:** Kein partielles HNSW-Rewiring außer Tombstone-Pruning und kontrolliertem Rebuild. (`cargo test -p contextra-vector --test k02_snapshot_rebuild_determinism`)
- **INV-DELETION-2:** Keinerlei Geisterzeiger auf gelöschte Knoten nach Tombstone-Pruning. (`cargo test -p contextra-vector hnsw::deletion`)
- **P24:** Löschkosten sind lokal begrenzt; physisches Pruning erfolgt deferred im Background-Rebuild. (`cargo test -p contextra-vector`)
- **INV-VECTOR-SAFE:** 100% Safe Rust mit `#![forbid(unsafe_code)]` im gesamten Crate-Root; SIMD-Intrinsics sind strikt nach `contextra-simd` ausgelagert. (`cargo xtask check-unsafe-islands`)
- **NaN-Validierung:** Vektorkoordinaten und Queries müssen vor/während Distanzberechnungen auf NaN/Inf validiert werden. (`cargo test -p contextra-vector distance`)

## 4. Verboten / Anti-Patterns

```rust
// ❌ FALSCH — unsafe Block oder allow(unsafe_code) in contextra-vector verwenden:
unsafe { ... }
// ✅ KORREKT — Safe Rust nutzen; SIMD-Operationen an contextra-simd delegieren.

// ❌ FALSCH — Direct tokio::spawn oder async runtime im Hotpath nutzen:
tokio::spawn(async move { ... });
// ✅ KORREKT — Ring 0 ist rein synchron (P26); Threadpool aus compute_pool.rs nutzen.

// ❌ FALSCH — DiskANN im Standard-Build erzwingen:
// DiskANN gehört isoliert hinter `experimental-diskann` Feature-Gate.
```

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- **Ring-0 Sync-Reinheit (P26):** `contextra-vector` ist ein Ring-0-Crate ohne `tokio`-Abhängigkeit. All-In-Memory Traversal läuft synchron.
- Feingranulares In-Memory Locking via `parking_lot::RwLock` / `parking_lot::Mutex`.
- Locks dürfen nie über I/O-Grenzen gehalten werden; Sperrenhierarchie einhalten.

## 6. Verifikation

```bash
cargo test -p contextra-vector --locked
cargo test -p contextra-vector --features experimental-diskann
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-vector
cargo xtask check-ring0-async-purity
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL

- `quantize_rabitq.rs` ist als RaBitQ-Prototyp vorhanden, aber noch nicht in den Produktions-Searchpath integriert.
- `experimental-diskann` ist opt-in und verlangt explizite Feature-Aktivierung.
