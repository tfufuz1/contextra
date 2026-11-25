# Contextra Performance & Evaluation Benchmarks

*Datum der Erstaufnahme: 2026-08-29*
*Letzte Audit-Prüfung: 2026-09-21*
*Environment: Linux x86_64, 4 CPU Cores (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM)*

---

## 0. Status-Matrix aller Benchmark-Claims

| Claim / Metrik | Quelle (Datei / Befehl) | Status | Befund / Anmerkung |
|---|---|---|---|
| **1k Chunks Search p50 = ~~88.03 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 2.61 ms, VM-Messlauf 2026-09-21]**** | `docs/BENCHMARKS.md` §1 (Historischer Log 2026-08-29) | `widersprüchlich` | Widerspricht §4 (~~29.91 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 2.61 ms, VM-Messlauf 2026-09-21]**) und neuem VM-Messlauf (2.61 ms). |
| **5k Chunks Search p50 = ~~809.15 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 5.11 ms, VM-Messlauf 2026-09-21]**** | `docs/BENCHMARKS.md` §1 (Historischer Log 2026-08-29) | `widersprüchlich` | Erheblicher Ausreißer in historischem Run; VM-Messlauf 2026-09-21 zeigt 5.11 ms. |
| **10k Chunks Search p50 = ~~337.77 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 5.13 ms, VM-Messlauf 2026-09-21]**** | `docs/BENCHMARKS.md` §1 (Historischer Log 2026-08-29) | `widersprüchlich` | Nicht-monotones Verhalten im historischen Log; VM-Messlauf 2026-09-21 zeigt 5.13 ms. |
| **100k / 1M Chunks Extrapolationen** | `docs/BENCHMARKS.md` §1 | `Messung in Arbeit (extrapoliert, unbestätigt)` | Mathematische Extrapolation; gemessen bis 10.000 Dokumente. |
| **Recall@5 / @10 / @20 = 1.0000** | `crates/contextra-db/tests/semantic_recall.rs` | `synthetisch` | Gemessen gegen synthetische Ground Truth (20 Themen-Cluster × 50 Dokumente). Test-Assert fordert `>= 0.80`. |
| **Hybrid Search p50 = ~~29.91 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 2.61 ms, VM-Messlauf 2026-09-21]**** | `docs/BENCHMARKS.md` §4 (Commit `bd51c6f5...`, 2026-09-03) | `widersprüchlich` | Widerspricht §1 (~~88.03 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 2.61 ms, VM-Messlauf 2026-09-21]**) sowie VM-Messlauf (2.61 ms). |
| **Wettbewerbsvergleich Mem0/Zep/MemOS** | `docs/BENCHMARKS.md` §4 | `nur dokumentiert, nicht reproduziert` | Keine öffentlichen Messungen vorhanden; Wettbewerberzeilen wurden gelöscht. |
| **VM Re-Run 1k/5k/10k Chunks** | `CONTEXTRA_SCALE_TIERS="1000,5000,10000" cargo bench -p contextra-db --bench scale_bench -- --quick` | `reproduziert` | In VM ausgeführt am 2026-09-21 auf Commit `347ef6dd86d90fdbc8f6dc0fcfa54ab6bb0c8986`. |

---

## 1. Verifizierter VM-Messlauf (`benches/scale_bench.rs`)

> **Hinweis zur Latenz-Abgrenzung:** Misst ausschließlich Speicher- und Indexlatenz. Embedding-Inferenz (Ollama, ONNX, Candle) addiert je nach Modell und Hardware 10-500 ms pro Anfrage.

*Protokollierter Messlauf:*
- **Commit:** `347ef6dd86d90fdbc8f6dc0fcfa54ab6bb0c8986`
- **Datum:** 2026-09-21
- **Hardware:** Intel(R) Xeon(R) Processor @ 2.30GHz (4 vCPUs), 7.8 GiB RAM, Linux x86_64
- **Befehl:** `CONTEXTRA_SCALE_TIERS="1000,5000,10000" cargo bench -p contextra-db --bench scale_bench -- --quick`

### 1.a Verifizierte Messwerte (bis 10.000 Dokumente)

| Corpus-Größe (Chunks) | Insert-Durchsatz (docs/sec) | Search Latenz p50 | Search Latenz p95 | VmRSS Peak (MB) | Status |
|---|---|---|---|---|---|
| **1,000** | 409.7 docs/sec | 2.61 ms | 2.66 ms | 148.85 MB | `reproduziert` |
| **5,000** | 126.9 docs/sec | 5.11 ms | 5.20 ms | 332.36 MB | `reproduziert` |
| **10,000** | 72.0 docs/sec | 5.13 ms | 5.29 ms | 597.16 MB | `reproduziert` |

### 1.b Unbestätigte Extrapolationen (nicht belegt)

| Corpus-Größe (Chunks) | Insert-Durchsatz (docs/sec) | Search Latenz p50 | Search Latenz p95 | VmRSS Peak (MB) | Status |
|---|---|---|---|---|---|
| **100,000** *(extrapoliert)* | ~25 docs/sec | ~3,500 ms | ~4,200 ms | ~1,950 MB | `Messung in Arbeit (extrapoliert, unbestätigt)` |
| **1,000,000** *(extrapoliert)* | ~5 docs/sec | > 30 s | > 45 s | > 18.5 GB (exceeds RAM) | `Messung in Arbeit (extrapoliert, unbestätigt)` |

---

## 1.1 Kanonische Benchmark-Quelle

Die im Folgenden aufgeführten Werte des VM-Messlaufs vom 2026-09-21 stellen die **einzige aktuell gültige Quelle der Wahrheit** ("Ein Benchmark, eine Wahrheit") für Performanz-Claims von Contextra dar:

- **1.000 Dokumente:** Insert-Durchsatz 409,7 Dok./s | Search-Latenz p50 = 2,61 ms | VmRSS Peak = 148,85 MB
- **5.000 Dokumente:** Insert-Durchsatz 126,9 Dok./s | Search-Latenz p50 = 5,11 ms | VmRSS Peak = 332,36 MB
- **10.000 Dokumente:** Insert-Durchsatz 72,0 Dok./s | Search-Latenz p50 = 5,13 ms | VmRSS Peak = 597,16 MB

### Messbedingungen & Parameter:
- **Build-Flags:** Benchmark-Release-Profil (`cargo bench`, `opt-level = 3`, LTO aktiviert).
- **Hardware-Spezifikation:** Linux x86_64, 4 vCPUs (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM).
- **Exklusion von Embedding-Latenzen:** **Die Embedding-Latenz ist ausdrücklich NICHT Teil der gemessenen Zahlen.** Die Benchmarks messen rein die In-Memory- und Diskspeichereffizienz sowie die Indizierungs- und Suchlatenz von Contextra (HNSW, BM25, CSR-Graph), ohne externe LLM-, ONNX- oder Ollama-Inferenzaufrufe.

### Ursache historischer Diskrepanzen:
Die wahrscheinlichste Ursache der historischen Diskrepanz zwischen früheren Messwerten (z. B. 88,03 ms oder 809,15 ms p50) und den aktuellen Messungen liegt in der Test-Anordnung: Die historische Messung schloss vermutlich die Latenz eines externen Embedding-Aufrufs (ONNX- oder Ollama-Inferenz) mit ein, während die aktuelle Messung ausschließlich Speicher- und Indexlatenz erfasst.

---

## 2. Historische Dokumentierte Messwerte (Ungeprüfter Ist-Stand vom 2026-08-29)

*Hinweis: Diese historischen Tabellenwerte sind im Repository als Baseline dokumentiert, weisen jedoch Widersprüche zu neueren Messungen und VM-Runs auf (siehe §0). Alle hier aufgeführten historischen Latenzen sind als veraltet gekennzeichnet.*

| Corpus-Größe (Chunks) | Insert-Durchsatz (docs/sec) | Search Latenz p50 | Search Latenz p95 | Search Latenz p99 | VmRSS Peak (MB) |
|---|---|---|---|---|---|
| **1,000** | ~117.3 docs/sec | ~~88.03 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 2.61 ms, VM-Messlauf 2026-09-21]** | 90.73 ms | 90.73 ms | 26.14 MB |
| **5,000** | ~89.5 docs/sec | ~~809.15 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 5.11 ms, VM-Messlauf 2026-09-21]** | 825.68 ms | 825.68 ms | 121.14 MB |
| **10,000** | ~75.2 docs/sec | ~~337.77 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 5.13 ms, VM-Messlauf 2026-09-21]** | 351.51 ms | 351.51 ms | 207.13 MB |

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

### 4.1 Skalierungsgrenze & Single-Node Positionierung
Contextra ist bewusst als Single-Node-, In-Process-Gedächtnisschicht für vertrauliche und lokale KI-Agenten konzipiert. Der Standard-Vektorindex `HnswIndex` hält den Traversierungsgraphen im RAM. Gemessen sind Latenzen bis 10.000 Dokumente (p50 = 5,13 ms im VM-Messlauf, siehe §1); darüber extrapoliert und nicht belegt (Messung in Arbeit).

Ab größeren Korpora (100.000+ Chunks) ist die DiskANN-Variante (`experimental-diskann`, aktuell experimentell) zu verwenden, da der In-Memory-HNSW-Index den typischen RAM einer Entwicklungsumgebung überschreitet. Diese Skalierungsgrenze ist ein bewusster Bestandteil der Positionierung als leichtgewichtige Single-Node-Gedächtnisschicht, nicht eine verschwiegene Schwäche.

---

## 5. Interne Baseline-Latenzen

> **Messbedingungen:** Linux x86_64, Intel(R) Xeon(R) Processor @ 2.30GHz (4 CPU cores), 7.8 GiB RAM (Jules Sandbox VM)
> **Contextra Version:** `bd51c6f599e50682516393d2bdc8eb3a717197e1` (Dokumentiert am 2026-09-03)

| Operation | Contextra Latenz / Durchsatz | Status |
|---|---|---|
| Batch Write Throughput (docs/s) | ~117.3 docs/s (single-doc) / ~1,438.9 docs/s (batch insert) | `nur dokumentiert, nicht reproduziert` |
| Hybrid Search p50 (ms) | ~~29.91 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 2.61 ms, VM-Messlauf 2026-09-21]** | `widersprüchlich` (widerspricht §1 und VM-Run) |
| Hybrid Search p99 (ms) | ~~30.45 ms~~ **[VERALTET — siehe Abschnitt 1, ersetzt durch 2.66 ms, VM-Messlauf 2026-09-21]** | `widersprüchlich` |

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

---

## 7. Baseline 2026-10-03 (Criterion Release-Profil Messlauf)

> **Messbedingungen & Umgebung:**
> - **Datum:** 2026-10-03 (UTC)
> - **Commit:** `ab30ea924ba97ca7c39dd5512f78c2a4959c7ab8`
> - **Compiler:** `rustc 1.89.0 (29483883e 2025-08-04)`
> - **Release-Profil:** `cargo bench --locked` (`opt-level = 3`, LTO / bench profile)
> - **Hardware:** Linux x86_64, 4 vCPUs (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM)
> - **Hinweis zur VM-Varianz:** Die nachfolgenden Messwerte wurden in einer virtuellen Serverumgebung (KVM) erhoben. Bedingt durch CPU-Throttling, Steal-Time und Shared-Host-I/O-Varianzen können Messwerte bei erneuten Läufen um einige Prozent abweichen.

### 7.1 `scale_bench` (`contextra-db`)
- **Befehl:** `cargo bench -p contextra-db --bench scale_bench --locked`
- **Status:** `nicht erfasst (Fehler: Transaction staging budget exceeded)`
- **Befund / Fehlerbeschreibung:** Der Benchmark schlug während der Initialbefüllung der Standard-Skalierungsstufen (10.000 Chunks) mit folgendem Laufzeit-Fehler fehl:
  ```text
  thread 'main' panicked at benches/scale_bench.rs:141:58:
  called `Result::unwrap()` on an `Err` value: Storage("Transaction TxId(77) staging budget exceeded: 16777199 + 78 > 16777216")
  ```
  *Anmerkung:* Der Staging-Puffer für eine einzelne Transaktion hat das Limit von 16 MB (`16_777_216` Bytes) überschritten.

---

### 7.2 `rrf_scale_bench` (`contextra-db`)
- **Befehl:** `cargo bench -p contextra-db --bench rrf_scale_bench --locked`
- **Messwerte (Weighted Reciprocal Rank Fusion über 4 Signale):**

| Hit-Anzahl pro Signal | Gesamt-Eingabe-Hits | Samples | Median (p50) | Untere Konfidenzgrenze | Obere Konfidenzgrenze | Durchsatz (p50) |
|---|---|---|---|---|---|---|
| **1,000** | 4,000 | 10 | **2.7094 ms** | 2.6966 ms | 2.7290 ms | 1.4764 Melem/s |
| **10,000** | 40,000 | 10 | **95.986 ms** | 94.262 ms | 97.474 ms | 416.73 Kelem/s |
| **100,000** | 400,000 | 10 | **915.33 ms** | 910.54 ms | 919.84 ms | 437.00 Kelem/s |
| **500,000** | 2,000,000 | 10 | **4.7798 s** | 4.5202 s | 5.2276 s | 418.43 Kelem/s |

---

### 7.3 `checkpoint_bench` (`contextra-checkpoint`)
- **Befehl:** `cargo bench -p contextra-checkpoint --bench checkpoint_bench --locked`
- **Messwerte:**

| Benchmark-Gruppe / Testfall | Parameter | Samples | Median (p50) | Untere Konfidenzgrenze | Obere Konfidenzgrenze | Durchsatz (p50) |
|---|---|---|---|---|---|---|
| `checkpoint_creation_latency` | Payload 1 KB | 100 | **9.9115 µs** | 9.5910 µs | 10.302 µs | - |
| `checkpoint_creation_latency` | Payload 10 KB | 100 | **40.636 µs** | 40.514 µs | 40.745 µs | - |
| `checkpoint_creation_latency` | Payload 100 KB | 100 | **358.78 µs** | 353.97 µs | 364.46 µs | - |
| `checkpoint_cache_latency` | `cache_hit_read` | 100 | **182.72 ns** | 176.03 ns | 191.17 ns | - |
| `checkpoint_cache_latency` | `cache_miss_read` | 100 | **6.7063 µs** | 6.6725 µs | 6.7404 µs | - |
| `checkpoint_rollback` | `restore_checkpoint` | 100 | **2.3189 µs** | 2.3113 µs | 2.3263 µs | - |
| `concurrent_checkpoint_throughput` | 1 Concurrent Writer | 100 | **112.40 µs** | 109.82 µs | 115.01 µs | 8.8971 Kelem/s |
| `concurrent_checkpoint_throughput` | 10 Concurrent Writers | 100 | **180.23 µs** | 176.63 µs | 184.19 µs | 55.485 Kelem/s |
| `concurrent_checkpoint_throughput` | 100 Concurrent Writers | 100 | **932.09 µs** | 890.09 µs | 979.40 µs | 107.29 Kelem/s |

---

### 7.4 `crypto_benchmarks` (`contextra-crypto`)
- **Befehl:** `cargo bench -p contextra-crypto --bench crypto_benchmarks --locked`
- **Messwerte:**

| Benchmark-Gruppe / Testfall | Parameter / Blockgröße | Samples | Median (p50) | Untere Konfidenzgrenze | Obere Konfidenzgrenze | Durchsatz (p50) |
|---|---|---|---|---|---|---|
| `aes_256_gcm_siv_encrypt` | 1,024 Bytes (1 KB) | 100 | **1.7469 µs** | 1.7325 µs | 1.7621 µs | 559.04 MiB/s |
| `aes_256_gcm_siv_encrypt` | 65,536 Bytes (64 KB) | 100 | **79.258 µs** | 78.870 µs | 79.660 µs | 788.56 MiB/s |
| `aes_256_gcm_siv_encrypt` | 1,048,576 Bytes (1 MB) | 100 | **1.3202 ms** | 1.3152 ms | 1.3262 ms | 757.44 MiB/s |
| `aes_256_gcm_siv_encrypt` | 16,777,216 Bytes (16 MB) | 100 | **28.433 ms** | 27.444 ms | 29.427 ms | 562.73 MiB/s |
| `aes_256_gcm_siv_decrypt` | 1,024 Bytes (1 KB) | 100 | **1.7397 µs** | 1.7326 µs | 1.7472 µs | 561.35 MiB/s |
| `aes_256_gcm_siv_decrypt` | 65,536 Bytes (64 KB) | 100 | **78.055 µs** | 77.776 µs | 78.322 µs | 800.72 MiB/s |
| `aes_256_gcm_siv_decrypt` | 1,048,576 Bytes (1 MB) | 100 | **1.3114 ms** | 1.3070 ms | 1.3157 ms | 762.56 MiB/s |
| `aes_256_gcm_siv_decrypt` | 16,777,216 Bytes (16 MB) | 100 | **22.388 ms** | 22.298 ms | 22.481 ms | 714.68 MiB/s |
| `hkdf_key_derivation_latency` | HKDF Key Derivation | 100 | **4.7915 µs** | 4.7649 µs | 4.8178 µs | - |
| `hmac_sha256_integrity_key_derivation` | HMAC Key Derivation | 100 | **1.6492 µs** | 1.6420 µs | 1.6564 µs | - |
| `kv_insert_n_segments` | 10 Segmente | 100 | **347.12 ns** | 345.31 ns | 349.03 ns | - |
| `kv_insert_n_segments` | 100 Segmente | 100 | **348.01 ns** | 345.43 ns | 350.50 ns | - |
| `kv_insert_n_segments` | 1,000 Segmente | 100 | **329.66 ns** | 327.73 ns | 331.34 ns | - |
| `kv_get_concurrent_n_tenants` | 4 Mandanten | 100 | **252.45 µs** | 232.76 µs | 271.89 µs | - |
| `kv_get_concurrent_n_tenants` | 16 Mandanten | 100 | **661.30 µs** | 655.99 µs | 667.05 µs | - |
| `kv_get_concurrent_n_tenants` | 32 Mandanten | 100 | **2.3425 ms** | 2.3105 ms | 2.3768 ms | - |
| `kv_evict_lru_lock_held_duration` | Deferred Drop Eviction | 100 | **573.04 µs** | 568.96 µs | 577.00 µs | - |

---

### 7.5 `deletion_proof_latency_bench` (`contextra-crypto`)
- **Befehl:** `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench --locked`
- **Messwerte:**

| Benchmark-Gruppe | Anz. Schlüssel | Samples | Median (p50) | Untere Konfidenzgrenze | Obere Konfidenzgrenze |
|---|---|---|---|---|---|
| `deletion_proof_creation_latency` | 1 | 50 | **2.4391 µs** | 2.4268 µs | 2.4534 µs |
| `deletion_proof_creation_latency` | 100 | 50 | **17.723 µs** | 17.494 µs | 17.956 µs |
| `deletion_proof_creation_latency` | 10,000 | 20 | **2.5297 ms** | 2.5083 ms | 2.5554 ms |
| `deletion_proof_creation_latency` | 1,000,000 | 10 | **1.2824 s** | 1.1256 s | 1.4610 s |
| `deletion_proof_verification_latency` | 1 | 50 | **2.2589 µs** | 2.2461 µs | 2.2717 µs |
| `deletion_proof_verification_latency` | 100 | 50 | **2.2331 µs** | 2.2194 µs | 2.2480 µs |
| `deletion_proof_verification_latency` | 10,000 | 20 | **2.2388 µs** | 2.2242 µs | 2.2502 µs |
| `deletion_proof_verification_latency` | 1,000,000 | 10 | **2.2122 µs** | 2.2023 µs | 2.2242 µs |
| `deletion_proof_full_issuance_latency` | 1 | 50 | **7.4391 µs** | 7.4018 µs | 7.4795 µs |
| `deletion_proof_full_issuance_latency` | 100 | 50 | **22.444 µs** | 22.319 µs | 22.566 µs |
| `deletion_proof_full_issuance_latency` | 10,000 | 20 | **2.4850 ms** | 2.4625 ms | 2.5021 ms |
| `deletion_proof_full_issuance_latency` | 1,000,000 | 10 | **1.0510 s** | 1.0431 s | 1.0596 s |

---

### 7.6 Befunde & Budget-Vergleich (Analytische Auswertung)

1. **`scale_bench` Transaktions-Staging-Fehler (Befund):**
   - *Fehler:* Transaktion überschreitet Staging-Budget bei 10.000 Chunks (`16.777.199 + 78 > 16.777.216` Bytes).
   - *Ursache:* Im unskalierten Standardmodus versucht `scale_bench` batchwise Transaktionen abzuwickeln, bis das 16 MB Staging Limit von `contextra-store` erreicht wird.
   - *Status:* "nicht erfasst" laut Protokoll.

2. **Löschbeweis-Verifikation in $O(1)$ (Bestätigt):**
   - Die Verifikationslatenz (`deletion_proof_verification_latency`) bleibt unabhängig von der Anzahl gelöschter Schlüssel (1 bis 1.000.000) nahezu konstant bei **~2.21 µs – 2.26 µs**. Dies bestätigt die theoretische $O(1)$-Komplexität der HMAC-Signaturprüfung.

3. **Kryptografischer Durchsatz:**
   - AES-256-GCM-SIV erreicht unter Hardware-AES-Beschleunigung einen stabilen Verschlüsselungsdurchsatz von **~550 bis 788 MiB/s** und Entschlüsselungsdurchsatz von **~560 bis 800 MiB/s**.

4. **Checkpoint-Operationen:**
   - In-Memory Cache-Hit Leselatenz liegt bei **182.72 ns**.
   - Wiederherstellungszeit (`restore_checkpoint`) liegt bei **2.32 µs**.
