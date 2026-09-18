# Diagnose-Report: `memfuse-router` (check)

Fehler: **0**  |  Warnungen: **2**

## `crates/memfuse-db/src/memory_consolidation.rs`  (1 Diagnose(n))

- **WARNING [deprecated]** @ `crates/memfuse-db/src/memory_consolidation.rs:388:68`: use of deprecated method `memfuse_core::DocId::as_u64`: Nutze inner()
  - _Hinweis:_ `#[warn(deprecated)]` on by default

## `crates/memfuse-router/src/tests.rs`  (1 Diagnose(n))

- **WARNING [unused_mut]** @ `crates/memfuse-router/src/tests.rs:2547:13`: variable does not need to be mutable
  - _Hinweis:_ `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default
  - _Hinweis:_ remove this `mut`
