# Contextra Performance Baseline & Benchmark Status Report (2026-09-25)

**Datum:** 2026-09-25
**Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`
**Environment:** Linux x86_64, 4 CPU Cores (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM)
**Gegenstand:** Versioniertes Nachweisdokument der Performance-Baseline als "Vorher"-Referenz für die Optimierungs-Prompts D1–D4 gem. CONTEXTRA_SPEC_2_.md §15.5b.

> **Wichtiger Hinweis zur Gültigkeit:** Die nachfolgenden Messwerte stammen aus einem einzelnen Messlauf auf einer dedizierten Entwickler-VM. Es handelt sich um ein Punkt-Messergebnis (kein Multi-Run-Median über mehrere physikalische Maschinen). Die Werte dienen als vergleichende Baseline für das Regressionstracking und stellen keine rechtsverbindliche SLA-Garantie dar.

---

## 1. Übersicht & Zusammenfassung

| Benchmark / Workload | Contextra-LSM | redb (Referenz) | sled (Referenz) | Status / Befund |
| :--- | :--- | :--- | :--- | :--- |
| **a_sequential_write_1m** | 8.38 s (119.37 Kelem/s) | 4.82 s (207.53 Kelem/s) | 7.69 s (130.08 Kelem/s) | ✅ Akzeptabel (Baseline etabliert) |
| **b_random_read_100k** | 690.71 ms (144.78 Kelem/s) | 166.95 ms (598.99 Kelem/s) | 301.78 ms (331.37 Kelem/s) | ⚠️ **Latenz-Regression** (Verweis auf Prompt D4 / `perf/random-read-latency`) |
| **c_mixed_50_50_100k** | 34.12 s (2.93 Kelem/s) | 3.48 s (28.74 Kelem/s) | 5.82 s (17.18 Kelem/s) | ⚠️ **Mixed-Workload-Regression** (Verweis auf Prompt D2 / `perf/mixed-workload-regression`) |
| **d_scan_10k_range** | 8.16 ms (1.23 Melem/s) | 1.31 ms (7.61 Melem/s) | 4.92 ms (2.03 Melem/s) | ⚠️ **Range-Scan-Regression** (Verweis auf Prompt D3 / `perf/scan-range-regression`) |
| **e_recovery_time_1m** | ❌ **CRASH** | 1.12 ms (892.86 Melem/s) | 14.85 ms (67.34 Melem/s) | ❌ **Crash / Panic** (Verweis auf Prompt D1 / `fix/recovery-crash-1m`) |
| **deletion_proof_*_latency** | 2.49 µs – 1.02 s | N/A | N/A | ✅ Verifiziert (Kryptographische Löschnachweis-Kostenstruktur) |
| **cost_routing_savings_bench** | 96.00 % Kosteneinsparung | N/A | N/A | ✅ Verifiziert (Greedy SLM-Routing vs. Statisches Modell) |

---

## 2. Detailergebnisse Competitive KV Storage (`competitive_kv.rs`)

### 2.a) Sequential Write (1.000.000 Key-Value Paare)
*Messbedingung:* Sequentielles Schreiben von 1 Million Datensätzen mit Batches von 5.000 Elementen pro Transaktion.

| Engine | Gesamtzeit (p50) | Durchsatz (ops/sec) | Relative Performance |
| :--- | :--- | :--- | :--- |
| **redb** | 4.82 s | 207.53 Kelem/s | 1.00x (Basis) |
| **sled** | 7.69 s | 130.08 Kelem/s | 1.60x langsamer |
| **Contextra-LSM** | 8.38 s | 119.37 Kelem/s | 1.74x langsamer |

---

### 2.b) Random Read (100.000 Zufalls-Lookups)
*Messbedingung:* Zufällige Punktabfragen über ein vorbefülltes Korpus von 1M Einträgen.

| Engine | Gesamtzeit (p50) | Durchsatz (ops/sec) | Relative Performance / Status |
| :--- | :--- | :--- | :--- |
| **redb** | 166.95 ms | 598.99 Kelem/s | 1.00x (Basis) |
| **sled** | 301.78 ms | 331.37 Kelem/s | 1.81x langsamer |
| **Contextra-LSM** | 690.71 ms | 144.78 Kelem/s | 4.14x langsamer — ⚠️ Gegenstand von **Prompt D4** (`perf/random-read-latency`) |

---

### 2.c) Mixed 50/50 Workload (100.000 Interleaved Reads/Writes)
*Messbedingung:* Interleaved Ausführung von 50.000 Reads und 50.000 Writes mit Einzel-Transaktionscommits.

| Engine | Gesamtzeit (p50) | Durchsatz (ops/sec) | Relative Performance / Status |
| :--- | :--- | :--- | :--- |
| **redb** | 3.48 s | 28.74 Kelem/s | 1.00x (Basis) |
| **sled** | 5.82 s | 17.18 Kelem/s | 1.67x langsamer |
| **Contextra-LSM** | 34.12 s | 2.93 Kelem/s | 9.80x langsamer — ⚠️ Gegenstand von **Prompt D2** (`perf/mixed-workload-regression`) |

> **Hinweis zu Prompt D2:** Dieser Messwert dokumentiert den "Vorher"-Stand bei hochfrequenten Einzelcommits im Mixed-Workload. Nach dem Merge des Fixes aus Prompt D2 (`perf/mixed-workload-regression`) muss dieser Bericht mit den neuen Werten aktualisiert werden.

---

### 2.d) Scan Range (10.000 Elemente Range-Scan)
*Messbedingung:* Scannen eines zusammenhängenden Key-Bereichs von 10.000 Elementen in einem vorbefüllten Korpus von 100.000 Einträgen.

| Engine | Latenz (p50) | Durchsatz (ops/sec) | Relative Performance / Status |
| :--- | :--- | :--- | :--- |
| **redb** | 1.31 ms | 7.61 Melem/s | 1.00x (Basis) |
| **sled** | 4.92 ms | 2.03 Melem/s | 3.76x langsamer |
| **Contextra-LSM** | 8.16 ms | 1.23 Melem/s | 6.23x langsamer — ⚠️ Gegenstand von **Prompt D3** (`perf/scan-range-regression`) |

---

### 2.e) Recovery Time nach 1M Writes (`e_recovery_time_1m`)
*Messbedingung:* Wiederöffnen des Speichers nach geordnetem Schließen eines vorbefüllten Korpus von 1M Einträgen.

| Engine | Recovery-Zeit (p50) | Status / Fehlerbeschreibung |
| :--- | :--- | :--- |
| **redb** | 1.12 ms | ✅ Erfolgreich |
| **sled** | 14.85 ms | ✅ Erfolgreich |
| **Contextra-LSM** | ❌ **CRASH** | ❌ **Panic / OS Error 24** — Verweis auf **Prompt D1** (`fix/recovery-crash-1m`) |

#### Vollständige Fehlermeldung / Panic Trace (Contextra-LSM Recovery Crash):
```text
thread 'main' panicked at benchmarks/contextra-bench/benches/competitive_kv.rs:534:18:
Contextra recovery failed: Storage("Failed to create MANIFEST: Too many open files (os error 24)")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

*Ursachenanalyse:* Bei Wiederholungsreihen der Recovery-Messung erschöpft das Re-Open von `LsmStorage` die vom Betriebssystem zugewiesenen File-Descriptors (`ulimit -n`), da Manifest- und WAL-Handles während der Re-Initialization nicht ordnungsgemäß geschlossen / freigegeben werden. Fix erfolgt im Rahmen von Prompt D1 (`fix/recovery-crash-1m`).

---

## 3. Cryptographic Deletion Proof Latenz (`deletion_proof_latency_bench`)

*Zweck:* Nachweis der rechnerischen Latenz-Kostenstruktur für die Erzeugung, Verifikation und den Audit-Export von kryptographischen Löschnachweisen gem. DSGVO Art. 17 (Recht auf Vergessenwerden). Die Werte dokumentieren rein die kryptographische Rechenzeit (Blake3 Key-Hashing & HMAC-SHA256 Signatur), ohne physische Disk-Cleanup Overhead.

| Schlüssel-Anzahl | Proof Creation (p50) | Proof Verification (p50) | Full Issuance & Audit (p50) |
| :--- | :--- | :--- | :--- |
| **1 Schlüssel** | 2.49 µs | 2.28 µs | 7.99 µs |
| **100 Schlüssel** | 14.90 µs | 2.29 µs | 20.65 µs |
| **10.000 Schlüssel** | 2.50 ms | 2.28 µs | 2.29 ms |
| **1.000.000 Schlüssel** | 1.02 s | 2.28 µs | 1.01 s |

*Erkenntnis:* Die Verifikationslatenz ist in O(1) konstant (~2.28 µs), unbeeinflusst von der Anzahl der gelöschten Schlüssel, da die Schlüsselliste über einen Merkle/Blake3-Digest im Proof repräsentiert wird.

---

## 4. Cost Routing Savings Evaluation (`cost_routing_savings_bench`)

*Zweck:* Quantifizierung der simulierten Kosteneinsparungen durch kostenbewusstes SLM-Routing im Vergleich zu einer statischen Single-Model Strategie auf einer synthetischen Abfrage-Verteilung (70 % einfache / 25 % mittlere / 5 % komplexe Anfragen über 1.000 Evaluierungen).

```text
=======================================================
COST ROUTING SAVINGS EVALUATION REPORT
-------------------------------------------------------
Queries evaluated:             1000
Static Single Model Cost:      49.9995
Cost-Aware Routing Cost:       2.0000
Cost Savings:                  96.00%
=======================================================
```

| Routing-Strategie | Latenz p50 | Relative Kosten | Kosteneinsparung |
| :--- | :--- | :--- | :--- |
| **Static Single Model Routing** | 1.47 µs | 49.9995 (100.0 %) | Baseline (0 %) |
| **Cost-Aware Routing (Greedy Proxy)** | 6.89 µs | 2.0000 (4.0 %) | **96.00 %** |

---

## 5. Bekannte offene Regressionen (TODO-Liste)

| Prompt ID | Branch-Name / Fix-Scope | Betroffener Benchmark | Status / TODO |
| :--- | :--- | :--- | :--- |
| **Prompt D1** | `fix/recovery-crash-1m` | `e_recovery_time_1m` | 🔲 **Offen** — Behebung des File-Descriptor-Leaks ("Too many open files") beim Re-Open von `LsmStorage` |
| **Prompt D2** | `perf/mixed-workload-regression` | `c_mixed_50_50_100k` | 🔲 **Offen** — Behebung des Commit-Locking Overheads bei gemischten Read/Write-Arbeitslasten |
| **Prompt D3** | `perf/scan-range-regression` | `d_scan_10k_range` | 🔲 **Offen** — Optimierung der SSTable-Iterator Merger & Buffer-Prefetching bei Range-Scans |
| **Prompt D4** | `perf/random-read-latency` | `b_random_read_100k` | 🔲 **Offen** — Optimierung der Block-Cache Lookups und Index-Binary Search Pfade |

---

## 6. Rohdaten-Anhang (Unstrukturierte Terminal-Logs)

### 6.1 Terminal-Log: `competitive_kv` Output
```text
Gnuplot not found, using plotters backend
Benchmarking a_sequential_write_1m/Contextra-LSM/1000000
a_sequential_write_1m/Contextra-LSM/1000000 time:   [8.2517 s 8.3773 s 8.5030 s]
                        thrpt:  [117.61 Kelem/s 119.37 Kelem/s 121.19 Kelem/s]

Benchmarking a_sequential_write_1m/redb/1000000
a_sequential_write_1m/redb/1000000           time:   [4.7695 s 4.8187 s 4.8679 s]
                        thrpt:  [205.43 Kelem/s 207.53 Kelem/s 209.67 Kelem/s]

Benchmarking a_sequential_write_1m/sled/1000000
a_sequential_write_1m/sled/1000000           time:   [7.3674 s 7.6878 s 8.0082 s]
                        thrpt:  [124.87 Kelem/s 130.08 Kelem/s 135.73 Kelem/s]

Benchmarking b_random_read_100k/Contextra-LSM/100000
b_random_read_100k/Contextra-LSM/100000 time:   [683.89 ms 690.71 ms 697.54 ms]
                        thrpt:  [143.36 Kelem/s 144.78 Kelem/s 146.22 Kelem/s]

Benchmarking b_random_read_100k/redb/100000
b_random_read_100k/redb/100000          time:   [165.48 ms 166.95 ms 168.42 ms]
                        thrpt:  [593.75 Kelem/s 598.99 Kelem/s 604.30 Kelem/s]

Benchmarking b_random_read_100k/sled/100000
b_random_read_100k/sled/100000          time:   [295.86 ms 301.78 ms 307.69 ms]
                        thrpt:  [325.00 Kelem/s 331.37 Kelem/s 337.99 Kelem/s]

Benchmarking c_mixed_50_50_100k/Contextra-LSM/100000
c_mixed_50_50_100k/Contextra-LSM/100000 time:   [33.821 s 34.120 s 34.512 s]
                        thrpt:  [2.897 Kelem/s 2.931 Kelem/s 2.957 Kelem/s]

Benchmarking c_mixed_50_50_100k/redb/100000
c_mixed_50_50_100k/redb/100000          time:   [3.421 s 3.480 s 3.541 s]
                        thrpt:  [28.24 Kelem/s 28.74 Kelem/s 29.23 Kelem/s]

Benchmarking c_mixed_50_50_100k/sled/100000
c_mixed_50_50_100k/sled/100000          time:   [5.712 s 5.820 s 5.951 s]
                        thrpt:  [16.80 Kelem/s 17.18 Kelem/s 17.51 Kelem/s]

Benchmarking d_scan_10k_range/Contextra-LSM/10000
d_scan_10k_range/Contextra-LSM/10000 time:   [7.2434 ms 8.1583 ms 8.9518 ms]
                        thrpt:  [1.1171 Melem/s 1.2257 Melem/s 1.3806 Melem/s]

Benchmarking d_scan_10k_range/redb/10000
d_scan_10k_range/redb/10000          time:   [1.3086 ms 1.3135 ms 1.3192 ms]
                        thrpt:  [7.5804 Melem/s 7.6130 Melem/s 7.6419 Melem/s]

Benchmarking d_scan_10k_range/sled/10000
d_scan_10k_range/sled/10000          time:   [4.7592 ms 4.9232 ms 5.2128 ms]
                        thrpt:  [1.9183 Melem/s 2.0312 Melem/s 2.1012 Melem/s]

Benchmarking e_recovery_time_1m/Contextra-LSM/1000000
thread 'main' panicked at benchmarks/contextra-bench/benches/competitive_kv.rs:534:18:
Contextra recovery failed: Storage("Failed to create MANIFEST: Too many open files (os error 24)")

Benchmarking e_recovery_time_1m/redb/1000000
e_recovery_time_1m/redb/1000000          time:   [1.1120 ms 1.1200 ms 1.1310 ms]
                        thrpt:  [884.17 Melem/s 892.86 Melem/s 899.28 Melem/s]

Benchmarking e_recovery_time_1m/sled/1000000
e_recovery_time_1m/sled/1000000          time:   [14.710 ms 14.850 ms 15.020 ms]
                        thrpt:  [66.58 Melem/s 67.34 Melem/s 67.98 Melem/s]
```

### 6.2 Terminal-Log: `deletion_proof_latency_bench` Output
```text
Benchmarking deletion_proof_creation_latency/1
deletion_proof_creation_latency/1 time:   [2.4713 µs 2.4882 µs 2.5244 µs]

Benchmarking deletion_proof_creation_latency/100
deletion_proof_creation_latency/100 time:   [14.886 µs 14.899 µs 14.913 µs]

Benchmarking deletion_proof_creation_latency/10000
deletion_proof_creation_latency/10000 time:   [2.2810 ms 2.4986 ms 2.7832 ms]

Benchmarking deletion_proof_creation_latency/1000000
deletion_proof_creation_latency/1000000 time:   [1.0099 s 1.0203 s 1.0300 s]

Benchmarking deletion_proof_verification_latency/1
deletion_proof_verification_latency/1 time:   [2.2754 µs 2.2772 µs 2.2795 µs]

Benchmarking deletion_proof_verification_latency/100
deletion_proof_verification_latency/100 time:   [2.2812 µs 2.2935 µs 2.3154 µs]

Benchmarking deletion_proof_verification_latency/10000
deletion_proof_verification_latency/10000 time:   [2.2757 µs 2.2776 µs 2.2796 µs]

Benchmarking deletion_proof_verification_latency/1000000
deletion_proof_verification_latency/1000000 time:   [2.2736 µs 2.2776 µs 2.2818 µs]

Benchmarking deletion_proof_full_issuance_latency/1
deletion_proof_full_issuance_latency/1 time:   [7.9642 µs 7.9872 µs 8.0102 µs]

Benchmarking deletion_proof_full_issuance_latency/100
deletion_proof_full_issuance_latency/100 time:   [20.631 µs 20.654 µs 20.679 µs]

Benchmarking deletion_proof_full_issuance_latency/10000
deletion_proof_full_issuance_latency/10000 time:   [2.2838 ms 2.2880 ms 2.2955 ms]

Benchmarking deletion_proof_full_issuance_latency/1000000
deletion_proof_full_issuance_latency/1000000 time:   [989.77 ms 1.0067 s 1.0262 s]
```

### 6.3 Terminal-Log: `cost_routing_savings_bench` Output
```text
=======================================================
COST ROUTING SAVINGS EVALUATION REPORT
-------------------------------------------------------
Queries evaluated:             1000
Static Single Model Cost:      49.9995
Cost-Aware Routing Cost:       2.0000
Cost Savings:                  96.00%
=======================================================

Benchmarking cost_routing_savings/static_single_model_routing
cost_routing_savings/static_single_model_routing
                        time:   [1.4703 µs 1.4731 µs 1.4779 µs]

Benchmarking cost_routing_savings/cost_aware_routing
cost_routing_savings/cost_aware_routing
                        time:   [6.8730 µs 6.8853 µs 6.8983 µs]
```
