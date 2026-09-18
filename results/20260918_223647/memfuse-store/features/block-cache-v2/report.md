# Diagnose-Report: `memfuse-store[block-cache-v2]` (feature-check)

Fehler: **10**  |  Warnungen: **0**

## `crates/memfuse-store/src/sstable.rs`  (2 Diagnose(n))

- **ERROR [E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - _Hinweis:_ expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - _Hinweis:_ change the output type to match the trait
- **ERROR [E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - _Hinweis:_ expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - _Hinweis:_ change the output type to match the trait

## `crates/memfuse-store/src/wal/replay.rs`  (8 Diagnose(n))

- **ERROR [E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - _Hinweis:_ `forbid` lint level was set on command line (`-F unsafe_code`)
- **ERROR [E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - _Hinweis:_ `forbid` lint level was set on command line (`-F unsafe_code`)
- **ERROR [unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - _Hinweis:_ requested on the command line with `-F unsafe-code`
- **ERROR [unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - _Hinweis:_ requested on the command line with `-F unsafe-code`
- **ERROR [E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - _Hinweis:_ `forbid` lint level was set on command line (`-F unsafe_code`)
- **ERROR [E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - _Hinweis:_ `forbid` lint level was set on command line (`-F unsafe_code`)
- **ERROR [unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block
- **ERROR [unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block
