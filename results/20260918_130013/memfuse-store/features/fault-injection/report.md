# Diagnose-Report: `memfuse-store[fault-injection]` (feature-check)

Fehler: **2**  |  Warnungen: **1**

## `crates/memfuse-store/tests/group_commit_test.rs`  (3 Diagnose(n))

- **WARNING [unused_imports]** @ `crates/memfuse-store/tests/group_commit_test.rs:5:20`: unused import: `MemFuseError`
  - _Hinweis:_ `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default
  - _Hinweis:_ remove the unused import
- **ERROR [E0308]** @ `crates/memfuse-store/tests/group_commit_test.rs:54:13`: mismatched types
  - _Hinweis:_ expected enum `Option<bytes::Bytes>`    found enum `Option<Vec<u8>>`
- **ERROR [E0308]** @ `crates/memfuse-store/tests/group_commit_test.rs:71:13`: mismatched types
  - _Hinweis:_ expected enum `Option<bytes::Bytes>`    found enum `Option<Vec<u8>>`
