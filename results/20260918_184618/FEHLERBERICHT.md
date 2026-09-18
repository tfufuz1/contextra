# Gesamt-Fehlerbericht (gruppiert nach Crate -> Datei)

Direkte Grundlage für Jules-Prompts: jede Datei-Gruppe = ein Kandidat für einen
isolierten, parallel ausführbaren Prompt.


## Crate: `memfuse-db`

### `crates/memfuse-db/../../benches/relate_bench.rs` (1 Fehler)

Fehlercodes: `E0599`×1

- **[E0599]** @ `crates/memfuse-db/../../benches/relate_bench.rs:70:24`: no method named `relate_bidirectional` found for struct `MemFuse` in the current scope


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

