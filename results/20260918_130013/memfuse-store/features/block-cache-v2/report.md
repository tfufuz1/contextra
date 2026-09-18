# Diagnose-Report: `memfuse-store[block-cache-v2]` (feature-check)

Fehler: **2**  |  Warnungen: **0**

## `crates/memfuse-store/src/sstable.rs`  (2 Diagnose(n))

- **ERROR [E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - _Hinweis:_ expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - _Hinweis:_ change the output type to match the trait
- **ERROR [E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - _Hinweis:_ expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - _Hinweis:_ change the output type to match the trait
