# Diagnose-Report: `memfuse-graph` (clippy)

Fehler: **3**  |  Warnungen: **0**

## `crates/memfuse-crypto/src/egress_vault.rs`  (1 Diagnose(n))

- **ERROR [clippy::unnecessary_sort_by]** @ `crates/memfuse-crypto/src/egress_vault.rs:372:13`: consider using `sort_by_key`
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unnecessary_sort_by
  - _Hinweis:_ `-D clippy::unnecessary-sort-by` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::unnecessary_sort_by)]`
  - _Hinweis:_ try

## `crates/memfuse-graph/src/csr.rs`  (1 Diagnose(n))

- **ERROR [dead_code]** @ `crates/memfuse-graph/src/csr.rs:352:19`: methods `hyperedges_for_entity` and `insert_hyperedge` are never used
  - _Hinweis:_ `-D dead-code` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[expect(dead_code)]` or `#[allow(dead_code)]`

## `crates/memfuse-graph/src/session_dag.rs`  (1 Diagnose(n))

- **ERROR [clippy::too_many_arguments]** @ `crates/memfuse-graph/src/session_dag.rs:180:5`: this function has too many arguments (8/7)
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
  - _Hinweis:_ `-D clippy::too-many-arguments` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::too_many_arguments)]`
