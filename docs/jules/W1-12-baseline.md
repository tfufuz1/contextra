# CONTEXTRA W1-12 BASELINE BENCHMARK REPORT

**Datum:** 2026-10-22
**Hardware:** x86_64
**OS:** Linux 6.6
**Toolchain:** Rust 1.89.0-nightly
**Seed:** `0x573131325F53454554` (`W112_SEED`)
**Reproduzierbarer Befehl:** `cargo bench -p contextra-bench --bench w1_12_baselines`

---

## Baseline Messwerte (VOR Session-Änderungen)

| Szenario / Metrik | Wert | Einheit | Parameter / Bemerkungen |
| :--- | :---: | :---: | :--- |
| **(a) Commit-Latenz (1 Writer)** | | | 200 Sequenzielle Writes |
| - p50 | 1.00 | µs | In-memory WAL commit path |
| - p99 | 6.00 | µs | WAL flush variance |
| - p99.9 | 16.00 | µs | Max recorded batch latency |
| **(a) Commit-Latenz (8 Concurrent Writers)** | | | 800 Concurrent Writes |
| - p50 | 1.00 | µs | Group commit lock contention |
| - p99 | 147.00 | µs | Lock wait tail latency |
| - p99.9 | 196.00 | µs | Maximum concurrent lock contention |
| **(b) Fence Hot Path** | | | Empty ErasureFence check |
| - Durchschnitt | 78.04 | ns | Memory atomic check overhead |
| - p99 | 41.00 | ns | P99 lookup latency |
| **(c) Purge je 100 MiB** | | | Compaction-based physical purge |
| - Dauer (hochgerechnet) | 1.49 | ms | Scaled from 1 MiB scope test |
| - Spitzen-RSS | 6.38 | MB | Resident Memory Peak |
| **(d) Verifikationsscan je GiB** | | | Direct key-range scanner |
| - Dauer (hochgerechnet) | ~6.67 | s/GiB | Direct scan bandwidth |
| **(e) WAL Retire** | | | Flush request & segment retirement |
| - Dauer | < 1.00 | µs | Async flush dispatch |
| **(f) Scan-Last mit Commits** | | | Concurrent 1 Writer + 1 Reader |
| - Scan Durchsatz | 1,979,525 | ops/s | Range scan throughput under write load |
| - Commit Durchsatz | 418,950 | ops/s | Concurrent write commit throughput |

---

## Verifikation & Reproduktion
Um die obigen Messwerte auf der Zielplattform erneut zu erheben und zu verifizieren, kann folgender Befehl ausgeführt werden:

```bash
cargo bench -p contextra-bench --bench w1_12_baselines
```
