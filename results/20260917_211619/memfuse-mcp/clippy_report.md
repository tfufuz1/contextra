# Diagnose-Report: `memfuse-mcp` (clippy)

Fehler: **2**  |  Warnungen: **0**

## `crates/memfuse-index/src/hnsw.rs`  (2 Diagnose(n))

- **ERROR [clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - _Hinweis:_ `-D clippy::useless-conversion` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - _Hinweis:_ consider removing `.into()`
- **ERROR [clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - _Hinweis:_ consider removing `.into()`

