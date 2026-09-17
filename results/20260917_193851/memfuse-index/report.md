# Diagnose-Report: `memfuse-index` (check)

Fehler: **0**  |  Warnungen: **3**

## `crates/memfuse-index/benches/flush_threshold_amplification.rs`  (3 Diagnose(n))

- **WARNING [unused_imports]** @ `crates/memfuse-index/benches/flush_threshold_amplification.rs:5:5`: unused import: `std::fs`
  - _Hinweis:_ `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default
  - _Hinweis:_ remove the whole `use` item
- **WARNING [unused_imports]** @ `crates/memfuse-index/benches/flush_threshold_amplification.rs:6:5`: unused import: `std::time::Instant`
  - _Hinweis:_ remove the whole `use` item
- **WARNING [dead_code]** @ `crates/memfuse-index/benches/flush_threshold_amplification.rs:17:4`: function `percentile` is never used
  - _Hinweis:_ `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

