# Diagnose-Report: `memfuse-graph[edge-reinforcement-learning]` (feature-check)

Fehler: **4**  |  Warnungen: **0**

## `crates/memfuse-store/src/wal/replay.rs`  (4 Diagnose(n))

- **ERROR [E0453]** @ `crates/memfuse-store/src/wal/replay.rs:105:13`: allow(unsafe_code) incompatible with previous forbid
  - _Hinweis:_ `forbid` lint level was set on command line (`-F unsafe_code`)
- **ERROR [unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:118:20`: usage of an `unsafe` block
  - _Hinweis:_ requested on the command line with `-F unsafe-code`
- **ERROR [E0453]** @ `crates/memfuse-store/src/wal/replay.rs:153:13`: allow(unsafe_code) incompatible with previous forbid
  - _Hinweis:_ `forbid` lint level was set on command line (`-F unsafe_code`)
- **ERROR [unsafe_code]** @ `crates/memfuse-store/src/wal/replay.rs:171:24`: usage of an `unsafe` block
