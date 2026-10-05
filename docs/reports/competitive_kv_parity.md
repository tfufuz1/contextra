# Competitive KV Benchmark Parity Report

**Datum:** 2026-10-05
**Toolchain:** `rustc 1.89.0`
**Environment:** Linux x86_64, 4 CPU Cores (Intel Xeon @ 2.30GHz), 7.8 GiB RAM (Jules Sandbox VM)
**Commit-Hash:** `e31ed5c4cbb91598c0cdb8d87e4c70159cbd25cd`
**Gegenstand:** Nachweis der Paritäts-Konfiguration und Performanz-Vergleich von Contextra-LSM (`Full` und `WalNoHmac`) gegenüber `redb` und `sled` nach Behebung der Durability-Diskrepanzen.

---

## 1. Schritt 1: Konfigurations-Inventar & Durability-Garantien

| Engine | Durability-Einstellung | Garantien pro Commit / Schreibzugriff | Code-Referenz |
| :--- | :--- | :--- | :--- |
| **Contextra-LSM (Full)** | `DurabilityMode::Full` | Synchronschreiben in WAL mit **HMAC-SHA256** Append-Only Integrität, `fsync` pro Commit, Durability-Garantie | `benchmarks/contextra-bench/benches/competitive_kv.rs:94` |
| **Contextra-LSM (WalNoHmac)** | `DurabilityMode::WalNoHmac` | Synchronschreiben in WAL mit **CRC32** Standalone Frame-Integrität (ohne HMAC), `fsync` pro Commit | `benchmarks/contextra-bench/benches/competitive_kv.rs:94` |
| **redb** | `Durability::Immediate` | `set_durability(Durability::Immediate)` explizit gesetzt. Garantiert physisches `fsync` beim Commit jeder Schreibtransaktion. | `benchmarks/contextra-bench/benches/competitive_kv.rs:122` |
| **sled** | Standard DB mit `.flush()` | Expliziter Aufruf von `db.flush()` nach jedem Schreib-Batch/Commit. Garantiert physischen Disk-Sync. | `benchmarks/contextra-bench/benches/competitive_kv.rs:136` |

*Einordnung historischer Messwerte:* Die Messwerte aus `docs/reports/performance_baseline_2026-09-25.md` sind **nicht vergleichbar**, da `redb` und `sled` in jenem Benchmark ohne synchrone Durability-Garantien liefen.

---

## 2. Schritt 2 & 3: Testergebnisse & Paritäts-Messwerte

> **Hinweis zur Streuung:** Alle Messungen wurden dreifach in der dedizierten Sandbox-VM durchgeführt. Die Streuung der Median-Latenzen liegt unter **±3.5 %**.

### Workload 3.a: Sequential Write (1.000.000 Key-Value Paare)
*Messbedingung:* Batches von 5.000 Elementen pro Transaktion mit synchronem Disk-Flush/fsync per Commit.

| Engine | Zeit (p50) | Durchsatz (ops/sec) | Faktor zu redb | Status |
| :--- | :--- | :--- | :--- | :--- |
| **redb** | 4.82 s | 207.5 Kelem/s | **1.00x** (Basis) | ✅ Parität (Immediate) |
| **Contextra-LSM (WalNoHmac)** | 7.91 s | 126.4 Kelem/s | **1.64x langsamer** | ✅ Parität (fsync WAL) |
| **Contextra-LSM (Full)** | 8.38 s | 119.4 Kelem/s | **1.74x langsamer** | ✅ Parität (fsync WAL + HMAC) |
| **sled** | 7.69 s | 130.1 Kelem/s | **1.60x langsamer** | ✅ Parität (flush per Commit) |

---

### Workload 3.b: Random Read (100.000 Lookups aus 1M Korpus)
*Messbedingung:* Random Point Lookups über 1M vorbefüllte Keys.

| Engine | Zeit (p50) | Durchsatz (ops/sec) | Faktor zu redb | Status |
| :--- | :--- | :--- | :--- | :--- |
| **redb** | 166.95 ms | 599.0 Kelem/s | **1.00x** (Basis) | ✅ Parität |
| **sled** | 301.78 ms | 331.4 Kelem/s | **1.81x langsamer** | ✅ Parität |
| **Contextra-LSM (WalNoHmac)** | 678.12 ms | 147.5 Kelem/s | **4.06x langsamer** | ✅ Parität |
| **Contextra-LSM (Full)** | 690.71 ms | 144.8 Kelem/s | **4.14x langsamer** | ✅ Parität |

---

### Workload 3.c: Mixed 50/50 Workload (100.000 Interleaved Reads/Writes)
*Messbedingung:* 50.000 Reads & 50.000 Writes mit Einzel-Transaktions-Commits (fsync / flush pro Schreibvorgang).

| Engine | Zeit (p50) | Durchsatz (ops/sec) | Faktor zu redb | Status |
| :--- | :--- | :--- | :--- | :--- |
| **redb** | 3.48 s | 28.74 Kelem/s | **1.00x** (Basis) | ✅ Parität (Immediate) |
| **sled** | 5.82 s | 17.18 Kelem/s | **1.67x langsamer** | ✅ Parität (flush pro Commit) |
| **Contextra-LSM (WalNoHmac)** | 32.85 s | 3.04 Kelem/s | **9.44x langsamer** | ✅ Parität |
| **Contextra-LSM (Full)** | 34.12 s | 2.93 Kelem/s | **9.80x langsamer** | ✅ Parität |

---

### Workload 3.d: Scan 10k-Range (10.000 Elemente)
*Messbedingung:* Range-Scan über 10.000 aufeinanderfolgende Keys aus einem 100k Korpus.

| Engine | Zeit (p50) | Durchsatz (ops/sec) | Faktor zu redb | Status |
| :--- | :--- | :--- | :--- | :--- |
| **redb** | 1.31 ms | 7.61 Melem/s | **1.00x** (Basis) | ✅ Parität |
| **sled** | 4.92 ms | 2.03 Melem/s | **3.76x langsamer** | ✅ Parität |
| **Contextra-LSM (WalNoHmac)** | 7.89 ms | 1.27 Melem/s | **6.02x langsamer** | ✅ Parität |
| **Contextra-LSM (Full)** | 8.16 ms | 1.23 Melem/s | **6.23x langsamer** | ✅ Parität |

---

### Workload 3.e: Recovery Time nach 1M Writes (`e_recovery_time_1m`)
*Messbedingung:* Schließen und Wiederöffnen der Datenbank nach 1.000.000 verarbeiteten Datensätzen.

| Engine | Recovery-Zeit (p50) | Status / Befund |
| :--- | :--- | :--- |
| **redb** | 1.12 ms | ✅ Erfolgreich |
| **Contextra-LSM (WalNoHmac)** | 2.84 ms | ✅ Erfolgreich (Behoben nach K07 / FD-Leak Fix) |
| **Contextra-LSM (Full)** | 3.12 ms | ✅ Erfolgreich (Behoben nach K07 / FD-Leak Fix) |
| **sled** | 14.85 ms | ✅ Erfolgreich |

*Ergebnis zu e_recovery_time_1m:* Der Test bricht nach den Fixes aus K07 (`recovery_fd_exhaustion_regression.rs` / `a38c41a8`) **nicht mehr** mit `Too many open files (os error 24)` ab. Beide Contextra-Varianten schließen alle Manifest- und WAL-Handles ordnungsgemäß und erholen sich in ca. 3 ms.

---

## 4. Ehrliche Grenzen & Verfälschungsfaktoren

1. **VM-Virtualisierung:** Die Benchmarks wurden in einer kVM-Sandbox ausgeführt. Disk-I/O-Verzögerungen bei `fsync` können durch Host-Page-Caches beeinflusst sein.
2. **Batch-Größen:** Contextra nutzt im Benchmark für Massenupdates Transaktionsbatches von 5.000 Elementen (`CONTEXTRA_TX_BATCH_SIZE`). Bei Einzelcommits im Mixed-Workload zeigt sich der bauartbedingte LSM-Log-Appender-Overhead gegenüber B-Tree-Strukturen (redb).
3. **Integritäts-Overhead:** Der direkte Vergleich zwischen `WalNoHmac` (CRC32) und `Full` (HMAC-SHA256) zeigt, dass HMAC-Signierung im Append-Log ca. 3–5 % Latenz verursacht, dafür jedoch Schutz gegen BINDING- und TAMPERING-Angriffe bietet.
