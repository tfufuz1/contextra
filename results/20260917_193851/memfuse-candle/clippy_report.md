# Diagnose-Report: `memfuse-candle` (clippy)

Fehler: **2**  |  Warnungen: **0**

## `crates/memfuse-calibration/src/isotonic.rs`  (1 Diagnose(n))

- **ERROR [clippy::question_mark]** @ `crates/memfuse-calibration/src/isotonic.rs:201:21`: this `match` expression can be replaced with `?`
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#question_mark
  - _Hinweis:_ `-D clippy::question-mark` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::question_mark)]`
  - _Hinweis:_ try instead

## `crates/memfuse-crypto/src/egress_vault.rs`  (1 Diagnose(n))

- **ERROR [clippy::unnecessary_sort_by]** @ `crates/memfuse-crypto/src/egress_vault.rs:372:13`: consider using `sort_by_key`
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_sort_by
  - _Hinweis:_ `-D clippy::unnecessary-sort-by` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
  - _Hinweis:_ try

