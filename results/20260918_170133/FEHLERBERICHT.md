# Gesamt-Fehlerbericht (gruppiert nach Crate -> Datei)

Direkte Grundlage für Jules-Prompts: jede Datei-Gruppe = ein Kandidat für einen
isolierten, parallel ausführbaren Prompt.


## Crate: `memfuse-graph`

### `crates/memfuse-graph/src/community.rs` (8 Fehler)

Fehlercodes: `E0425`×6, `E0308`×2

- **[E0425]** @ `crates/memfuse-graph/src/community.rs:313:28`: cannot find value `num_real_nodes` in this scope
  - Hinweis: the leading underscore in `_num_real_nodes` marks it as unused, consider renaming it to `num_real_nodes`
- **[E0425]** @ `crates/memfuse-graph/src/community.rs:313:28`: cannot find value `num_real_nodes` in this scope
  - Hinweis: the leading underscore in `_num_real_nodes` marks it as unused, consider renaming it to `num_real_nodes`
- **[E0425]** @ `crates/memfuse-graph/src/community.rs:319:77`: cannot find value `num_total_nodes` in this scope
  - Hinweis: a local variable with a similar name exists
- **[E0425]** @ `crates/memfuse-graph/src/community.rs:319:77`: cannot find value `num_total_nodes` in this scope
  - Hinweis: a local variable with a similar name exists
- **[E0425]** @ `crates/memfuse-graph/src/community.rs:321:58`: cannot find value `num_total_nodes` in this scope
  - Hinweis: a local variable with a similar name exists
- **[E0425]** @ `crates/memfuse-graph/src/community.rs:321:58`: cannot find value `num_total_nodes` in this scope
  - Hinweis: a local variable with a similar name exists
- **[E0308]** @ `crates/memfuse-graph/src/community.rs:331:13`: `if` and `else` have incompatible types
  - Hinweis: expected unit type `()`        found tuple `(EntityId, u64)`
- **[E0308]** @ `crates/memfuse-graph/src/community.rs:331:13`: `if` and `else` have incompatible types
  - Hinweis: expected unit type `()`        found tuple `(memfuse_core::EntityId, u64)`


## Crate: `memfuse-index`

### `crates/memfuse-index/src/hnsw.rs` (4 Fehler)

Fehlercodes: `clippy::too_many_arguments`×4

- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:1309:1`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
  - Hinweis: `-D clippy::too-many-arguments` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::too_many_arguments)]`
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:1309:1`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
  - Hinweis: `-D clippy::too-many-arguments` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::too_many_arguments)]`
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:2085:5`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:2085:5`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments

