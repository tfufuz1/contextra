# Contextra Performance & Evaluation Benchmarks

*Datum der Erstaufnahme: 2026-08-29*
*Letzte Audit-Prüfung: 2026-09-21*
*Environment: Linux x86_64, 4 CPU Cores (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM)*

---

## 0. Status-Matrix aller Benchmark-Claims

| Claim / Metrik | Quelle (Datei / Befehl) | Status | Befund / Anmerkung |
|---|---|---|---|
| **1k Chunks Search p50 = 88.03 ms** | `docs/BENCHMARKS.md` §1 (Historischer Log 2026-08-29) | `widersprüchlich` | Widerspricht §4 (29.91 ms) und neuem VM-Messlauf (2.61 ms). |
| **5k Chunks Search p50 = 809.15 ms** | `docs/BENCHMARKS.md` §1 (Historischer Log 2026-08-29) | `widersprüchlich` | Erheblicher Ausreißer in historischem Run; VM-Messlauf 2026-09-21 zeigt 5.11 ms. |
| **10k Chunks Search p50 = 337.77 ms** | `docs/BENCHMARKS.md` §1 (Historischer Log 2026-08-29) | `widersprüchlich` | Nicht-monotones Verhalten im historischen Log; VM-Messlauf 2026-09-21 zeigt 5.13 ms. |
| **100k / 1M Chunks Extrapolationen** | `docs/BENCHMARKS.md` §1 | `nur dokumentiert, nicht reproduziert` | Mathematische Extrapolation ohne realen Benchmark-Lauf. |
| **Recall@5 / @10 / @20 = 1.0000** | `crates/contextra-db/tests/semantic_recall.rs` | `synthetisch` | Gemessen gegen synthetische Ground Truth (20 Themen-Cluster × 50 Dokumente). Test-Assert fordert `>= 0.80`. |
| **Hybrid Search p50 = 29.91 ms** | `docs/BENCHMARKS.md` §4 (Commit `bd51c6f5...`, 2026-09-03) | `widersprüchlich` | Widerspricht §1 (88.03 ms) sowie VM-Messlauf (2.61 ms). |
| **Wettbewerbsvergleich Mem0/Zep/MemOS** | `docs/BENCHMARKS.md` §4 | `nur dokumentiert, nicht reproduziert` | Keine öffentlichen Messungen vorhanden; Wettbewerberzeilen wurden gelöscht. |
| **VM Re-Run 1k/5k/10k Chunks** | `CONTEXTRA_SCALE_TIERS="1000,5000,10000" cargo bench -p contextra-db --bench scale_bench -- --quick` | `reproduziert` | In VM ausgeführt am 2026-09-21 auf Commit `347ef6dd86d90fdbc8f6dc0fcfa54ab6bb0c8986`. |

---

## 1. Verifizierter VM-Messlauf (`benches/scale_bench.rs`)

*Protokollierter Messlauf:*
- **Commit:** `347ef6dd86d90fdbc8f6dc0fcfa54ab6bb0c8986`
- **Datum:** 2026-09-21
- **Hardware:** Intel(R) Xeon(R) Processor @ 2.30GHz (4 vCPUs), 7.8 GiB RAM, Linux x86_64
- **Befehl:** `CONTEXTRA_SCALE_TIERS="1000,5000,10000" cargo bench -p contextra-db --bench scale_bench -- --quick`

| Corpus-Größe (Chunks) | Insert-Durchsatz (docs/sec) | Search Latenz p50 | Search Latenz p95 | VmRSS Peak (MB) | Status |
|---|---|---|---|---|---|
| **1,000** | 409.7 docs/sec | 2.61 ms | 2.66 ms | 148.85 MB | `reproduziert` |
| **5,000** | 126.9 docs/sec | 5.11 ms | 5.20 ms | 332.36 MB | `reproduziert` |
| **10,000** | 72.0 docs/sec | 5.13 ms | 5.29 ms | 597.16 MB | `reproduziert` |
| **100,000** *(extrapoliert)* | ~25 docs/sec | ~3,500 ms | ~4,200 ms | ~1,950 MB | `nur dokumentiert, nicht reproduziert` |
| **1,000,000** *(extrapoliert)* | ~5 docs/sec | > 30 s | > 45 s | > 18.5 GB (exceeds RAM) | `nur dokumentiert, nicht reproduziert` |

---

## 2. Historische Dokumentierte Messwerte (Ungeprüfter Ist-Stand vom 2026-08-29)

*Hinweis: Diese historischen Tabellenwerte sind im Repository als Baseline dokumentiert, weisen jedoch Widersprüche zu neueren Messungen und VM-Runs auf (siehe §0).*

| Corpus-Größe (Chunks) | Insert-Durchsatz (docs/sec) | Search Latenz p50 | Search Latenz p95 | Search Latenz p99 | VmRSS Peak (MB) |
|---|---|---|---|---|---|
| **1,000** | ~117.3 docs/sec | 88.03 ms | 90.73 ms | 90.73 ms | 26.14 MB |
| **5,000** | ~89.5 docs/sec | 809.15 ms | 825.68 ms | 825.68 ms | 121.14 MB |
| **10,000** | ~75.2 docs/sec | 337.77 ms | 351.51 ms | 351.51 ms | 207.13 MB |

*Rohdaten-Log für historische RSS-Messung:* `benches/results/scale_rss.csv`

---

## 3. Semantische Retrieval-Evaluierung (`Recall@k` - Synthetisches Korpus)

Verifizierte Messung gegen synthetische Ground Truth (`crates/contextra-db/tests/semantic_recall.rs`):
- **Korpus:** 20 Themen-Cluster × 50 Dokumente = 1.000 Dokumente, 100 Test-Queries (5 pro Cluster).
- **Setup:** Synthetische Vektoren (Cluster-Phase + Gauß-Rauschen) und deterministische Schlüsselwörter.

| Metrik | Synthetisches Testergebnis | Test-Schwellenwert (Assert) | Status |
|---|---|---|---|
| **Recall@5** | 1.0000 (100.0 %) | N/A | `synthetisch` |
| **Recall@10** | 1.0000 (100.0 %) | `>= 0.80` | `synthetisch` |
| **Recall@20** | 1.0000 (100.0 %) | N/A | `synthetisch` |

---

## 4. Explizite Messgrenzen & Transparenz

1. **Keine verifizierten Cross-System-Vergleiche**: Öffentliche, reproduzierbare Vergleichsmessungen mit standardisierten Datensätzen gegen externe Wettbewerber (z.B. Mem0, Zep, MemOS, ChromaDB, Qdrant) liegen für das aktuelle Release nicht vor.
2. **Synthetisches Korpus**: Die Testdokumente und -vektoren in `semantic_recall.rs` und `scale_bench.rs` wurden deterministisch synthetisiert. Real-World-Textkorpora (z.B. LoCoMo, LongMemEval, BEIR) weisen abweichende Sparsity-, Rausch- und Cluster-Eigenschaften auf.
3. **Excluded Embedding Latency**: Während der Benchmarks wurden keine Embeddings durch ein lokales ONNX/Ollama-Modell berechnet; gemessen wird rein die Speicher- und Indizierungs-Latenz.
4. **Vollständiger In-Memory HNSW-Graph**: Bei 1M+ Chunks überschreitet der RAM-Bedarf von `HnswIndex` die physische RAM-Grenze typischer Developer-VMs (7.8 GiB).

---

## 5. Interne Baseline-Latenzen

> **Messbedingungen:** Linux x86_64, Intel(R) Xeon(R) Processor @ 2.30GHz (4 CPU cores), 7.8 GiB RAM (Jules Sandbox VM)
> **Contextra Version:** `bd51c6f599e50682516393d2bdc8eb3a717197e1` (Dokumentiert am 2026-09-03)

| Operation | Contextra Latenz / Durchsatz | Status |
|---|---|---|
| Batch Write Throughput (docs/s) | ~117.3 docs/s (single-doc) / ~1,438.9 docs/s (batch insert) | `nur dokumentiert, nicht reproduziert` |
| Hybrid Search p50 (ms) | 29.91 ms | `widersprüchlich` (widerspricht §1 und VM-Run) |
| Hybrid Search p99 (ms) | 30.45 ms | `widersprüchlich` |

---

## 6. Zielmetriken & Neue Benchmark-Kategorien (Opus-Optimierungen)

Mit der Umsetzung der Opus-Optimierungen (Stufe 0–3) werden folgende neue Benchmark-Kategorien und architektonische Zielwerte eingeführt:

### 6.1 Neue Benchmark-Kategorien

- **Bandit-Latenz-Budget (`check-bandit-latency-budget`)**: Messung der LinUCB-Bandit-Updates (Diagonal vs. Sherman-Morrison) auf Cache-Line-aligned SIMD-Vektoren. **Latenzziel**: Update-Overhead < 50 µs pro Query.
- **Block-Cache Hit-Latenz (`SieveCacheBackend`)**: Vergleich der Leselatenz (Hit-Pfad) zwischen dem sperrenden `LruBlockCacheBackend` und dem lock-freien `SieveCacheBackend` unter hochgradig paralleler Thread-Last.
- **WAL-Ring-Puffer-Durchsatz (`wal_ring_buffer`)**: Messung der Transaktionslatenz bei asynchronem Flusher-Task im Vergleich zum Mutex-geschützten synchronen `fsync`.

### 6.2 Architektonische Zielwerte (Allokations- & Zero-Copy-Ziele)

- **HNSW-Allokationsreduktion**: Der Distanz- und Traversierungspfad im Vektorindex muss durch den Arena-Allocator (HNSW v2) so optimiert werden, dass die Anzahl der Heap-Allokationen pro `search_knn`-Query signifikant sinkt (Ziel: Zero-Allocation Traversal).
- **SSTable-Zero-Copy**: Vermeidung des Kopierens kompletter SSTables in den Speicher (Nutzung von Mmap/Zero-Copy-Deserialisierung).
- **AES-Key-Schedule-Wiederverwendung**: Der Key-Schedule (`Aes256GcmSiv`) wird als `OnceLock` initialisiert, was die Overhead-Zeiten pro Verschlüsselungsoperation drastisch reduziert, da der Schedule nicht pro Operation neu berechnet wird.
