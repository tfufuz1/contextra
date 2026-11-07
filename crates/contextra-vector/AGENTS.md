# AGENTS.md — `contextra-vector`

> **Ring 1 (`capabilities.toml`: Ring 0 ≠)** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: HNSW-Vektorindex, SQ8-Quantisierung, gefilterte Suche (ACORN). Implementiert `VectorIndex`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, verifiziert; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-vector --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `mvcc`, `ports`, `simd`, `sys`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types`, `mvcc`, `simd`, `sys` |
| Features | `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `partial-index-rebuild`, `graph`, `docid-128` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Kante `crypto` vollständig gelöst (`DiskAnnHmac` nutzt autarkes BLAKE3).
- Direkte Datei-I/O in Migration auf `Vfs`.
- Löschbegriffe neutralisiert: Bereinigung nur über Wartungsprimitive.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `hnsw/` (`core_insert`, `core_search`, `core_rebuild`, `arena`, `batch`, `adaptive_ef`, `acorn_filtered`, `sq8_bias`, `deletion`, `purge`, `scanner`, `vector_index_impl`) | HNSW; **`purge.rs`/`scanner.rs` = Port-Implementierungen** (Wartungsprimitive, deprecated) |
| `quantize/`, `quantize_rabitq.rs` | SQ8, RaBitQ (experimentell) |
| `persistence/` (`header`, `node`, `mmap`) | Dateiformat, Persistenz |
| `diskann/` | DiskANN out-of-core Graphindex (experimentell) |
| `acorn/`, `candidate_stream.rs`, `distance.rs`, `compute_pool.rs`, `partial_rebuild.rs` | gefilterte Suche, Kandidatenstrom, Distanz, Compute-Pool |

## 4. Verträge & Invarianten

1. **VETO-F02:** Löschen = Tombstone + Slot-Nullung + **vollständiger** Rebuild. Kein Rewiring. Slots werden erst nach Rebuild wiederverwendet (INV-HNSW-SLOT-1). **Achtung:** `partial_rebuild.rs`/Feature `partial-index-rebuild` steht in Spannung dazu; nur für Wartung, nie als Löschstrategie.
2. **Tenant-Trennung (INV-TENANT-IDX-1):** Index physisch je (Tenant, Collection[, Modell-Hash]); keine Kante über Tenants.
3. **Determinismus:** HNSW-Level folgen aus `hash(doc_id, seed)`, nie aus Laufzeit-RNG (INV-INDEX-APPLY-1); ein Applier in WAL-Reihenfolge.
4. **Snapshot-Sicht:** `born_seq`/`died_seq` je Eintrag (INV-INDEX-SNAPSHOT-1).
5. **Filter vor Distanz** (ACL-Bitset), exaktes Nachranking bei SQ8.
6. Eingaben mit `NaN`/`±Inf` werden abgewiesen (`simd::validate_vector`).
7. Neu verboten: DSGVO-/Proof-Begriffe; Bereinigung nur über Wartungsprimitive.

## 5. Ports

**Implementiert:** `VectorIndex`, `Purger` (deprecated), `LayerScanner` (deprecated). **Konsumiert:** `Clock`, `Rng`, `Vfs` (Soll).

## 6. Nebenläufigkeit & Performance

- Suche liest Snapshot lock-arm; Schreiben über einen Applier. Sperren-Contention bei Tombstones: `tests/deleted_nodes_lock_contention.rs`.
- Rechenlast über `compute_pool` (begrenzt), nie auf dem Executor.
- Komplexität: Suche ≈ `O(log n · ef · d)`; per `benches/` (hnsw, sq8, distance) und Recall-Tests belegen; Baselines nicht verschlechtern.

## 7. Anti-Patterns

- Nachbarn beim Löschen umverdrahten.
- `thread_rng` im Aufbau.
- Index-Dateien mit `std::fs` öffnen (wo Vfs vorgegeben).
- `crypto`-Typen in Signaturen.

## 8. Tests & Messung

```bash
cargo test -p contextra-vector --locked
cargo clippy -p contextra-vector --all-targets -- -D warnings
cargo bench -p contextra-vector
cargo test -p contextra-vector --locked --features docid-128
(cd crates/contextra-vector && cargo +nightly fuzz run hnsw_insert_search -- -max_total_time=60)
```
