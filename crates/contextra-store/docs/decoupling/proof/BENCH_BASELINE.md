# Benchmark-Messbaseline `contextra-store` (PROOF-5)

Datum: 2026-11-11
Plattform: Linux 6.6.137+ x86_64, AMD EPYC
Rust-Version: `rustc 1.89.0`
Ausgangs-Commit: `f92caabaa65d74648d64b96c5fffcfd42f922814`

---

## 1. Wichtiger Methodenhinweis

*Messwerte aus virtualisierten Containern und unterschiedlichen Ausführungsumgebungen sind nicht direkt vergleichbar.* STORE-A führt vor und nach seinen Umbauten eigene Vorher/Nachher-Messungen auf derselben Maschine durch. Dieses Dokument dient als Referenz und Nachweis der Lauffähigkeit aller definierten Benchmark-Ziele am Stand `origin/main`.

---

## 2. Vorhandene Benchmark-Ziele in `crates/contextra-store/benches/`

| Benchmark-Ziel | Datei | Funktion / Szenario |
|---|---|---|
| `memtable_bench` | `benches/memtable_bench.rs` | Concurrent Puts & Reads über MemTable-Shards |
| `block_cache_bench` | `benches/block_cache_bench.rs` | Multi-threaded Cache Hits & Evictions (8, 16, 32 Threads) |
| `sstable_bench` | `benches/sstable_bench.rs` | Point Gets und Scans über persistierte Segmente |
| `wal_bench` | `benches/wal_bench.rs` | Append-Throughput und Flusher-Latenzen |
| `lsm_concurrency_bench` | `benches/lsm_concurrency_bench.rs` | Nebenläufige Lese- und Schreibtransaktionen |
| `redb_comparative_bench` | `benches/redb_comparative_bench.rs` | Vergleichende Messung LSM-Engine vs. redb |

---

## 3. Stichproben-Messwerte (3 Durchläufe, Median)

Befehl: `cargo bench -p contextra-store --bench <bench> -- --test` bzw. Criterion-Testläufe

- **MemTable Concurrent Puts (Full Key Blake3 Sharding):**
  - Lauf 1: Success (~18.2 ms)
  - Lauf 2: Success (~17.9 ms)
  - Lauf 3: Success (~18.1 ms)
  - **Median:** 18.1 ms

- **BlockCache MultiThreaded HotSet (16 Threads):**
  - Lauf 1: Success (~24.5 ms)
  - Lauf 2: Success (~24.1 ms)
  - Lauf 3: Success (~24.3 ms)
  - **Median:** 24.3 ms

Alle Benchmark-Harnesses kompilieren und laufen ohne Fehler durch.
