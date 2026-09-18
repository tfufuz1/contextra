# Gesamt-Fehlerbericht (gruppiert nach Crate -> Datei)

Direkte Grundlage für Jules-Prompts: jede Datei-Gruppe = ein Kandidat für einen
isolierten, parallel ausführbaren Prompt.


## Crate: `memfuse-agent`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::too_many_arguments`×2

- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:1309:1`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
  - Hinweis: `-D clippy::too-many-arguments` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::too_many_arguments)]`
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:2085:5`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments


## Crate: `memfuse-bench`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::too_many_arguments`×2

- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:1309:1`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
  - Hinweis: `-D clippy::too-many-arguments` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::too_many_arguments)]`
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:2085:5`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments


## Crate: `memfuse-core[docid-128]`

### `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs` (4 Fehler)

Fehlercodes: `E0308`×4

- **[E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types
- **[E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types
- **[E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types
- **[E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types

### `/rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/core/src/macros/mod.rs` (2 Fehler)

Fehlercodes: `E0308`×1, `E0277`×1

- **[E0308]** @ `/rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/core/src/macros/mod.rs:46:35`: mismatched types
- **[E0277]** @ `/rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/core/src/macros/mod.rs:46:32`: can't compare `u64` with `u128`
  - Hinweis: the trait `PartialEq<u128>` is not implemented for `u64`
  - Hinweis: `u64` implements trait `PartialEq<Rhs>`

### `crates/memfuse-core/src/types/domain.rs` (1 Fehler)

Fehlercodes: `E0308`×1

- **[E0308]** @ `crates/memfuse-core/src/types/domain.rs:1821:34`: mismatched types
  - Hinweis: associated function defined here
  - Hinweis: you can convert a `u64` to a `u128`


## Crate: `memfuse-mcp`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::too_many_arguments`×2

- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:1309:1`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments
  - Hinweis: `-D clippy::too-many-arguments` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::too_many_arguments)]`
- **[clippy::too_many_arguments]** @ `crates/memfuse-index/src/hnsw.rs:2085:5`: this function has too many arguments (8/7)
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_arguments


## Crate: `memfuse-store[block-cache-v2]`

### `crates/memfuse-store/src/sstable.rs` (2 Fehler)

Fehlercodes: `E0053`×2

- **[E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - Hinweis: expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - Hinweis: change the output type to match the trait
- **[E0053]** @ `crates/memfuse-store/src/sstable.rs:182:59`: method `weight` has an incompatible type for trait
  - Hinweis: expected signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u64`    found signature `fn(&BlockWeighter, &(_, _), &bytes::Bytes) -> u32`
  - Hinweis: change the output type to match the trait


## Crate: `memfuse-store[fault-injection]`

### `crates/memfuse-store/tests/group_commit_test.rs` (2 Fehler)

Fehlercodes: `E0308`×2

- **[E0308]** @ `crates/memfuse-store/tests/group_commit_test.rs:54:13`: mismatched types
  - Hinweis: expected enum `Option<bytes::Bytes>`    found enum `Option<Vec<u8>>`
- **[E0308]** @ `crates/memfuse-store/tests/group_commit_test.rs:71:13`: mismatched types
  - Hinweis: expected enum `Option<bytes::Bytes>`    found enum `Option<Vec<u8>>`
