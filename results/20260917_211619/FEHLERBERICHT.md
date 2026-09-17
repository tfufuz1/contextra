# Gesamt-Fehlerbericht (alle Crates, gruppiert nach Datei)

Dieser Bericht ist die direkte Grundlage für die Jules-Prompt-Erstellung:
jede unten stehende Datei-Gruppe ist ein Kandidat für EINEN isolierten Jules-Prompt.
Dateien im selben Crate, die thematisch zusammenhängen (z.B. Typen aus derselben
Modul-Familie), sollten in EINEM Prompt gebündelt werden, um Merge-Konflikte zu
vermeiden — siehe Design-Prinzip 'Datei- & Crate-Isolierung'.


## Crate: `memfuse-agent`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::useless_conversion`×2

- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: consider removing `.into()`


## Crate: `memfuse-bench`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::useless_conversion`×2

- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: consider removing `.into()`


## Crate: `memfuse-calibration`

### `crates/memfuse-calibration/tests/calibration_deep_tests.rs` (2 Fehler)

Fehlercodes: `clippy::manual_range_contains`×2

- **[clippy::manual_range_contains]** @ `crates/memfuse-calibration/tests/calibration_deep_tests.rs:81:13`: manual `RangeInclusive::contains` implementation
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#manual_range_contains
  - Hinweis: `-D clippy::manual-range-contains` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::manual_range_contains)]`
  - Hinweis: use
- **[clippy::manual_range_contains]** @ `crates/memfuse-calibration/tests/calibration_deep_tests.rs:358:22`: manual `RangeInclusive::contains` implementation
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#manual_range_contains
  - Hinweis: use

### `crates/memfuse-calibration/tests/calibration_stress_and_fault_tests.rs` (1 Fehler)

Fehlercodes: `clippy::manual_range_contains`×1

- **[clippy::manual_range_contains]** @ `crates/memfuse-calibration/tests/calibration_stress_and_fault_tests.rs:107:22`: manual `RangeInclusive::contains` implementation
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#manual_range_contains
  - Hinweis: `-D clippy::manual-range-contains` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::manual_range_contains)]`
  - Hinweis: use

### `crates/memfuse-calibration/tests/isotonic_mutation_hardening_test.rs` (1 Fehler)

Fehlercodes: `clippy::manual_range_contains`×1

- **[clippy::manual_range_contains]** @ `crates/memfuse-calibration/tests/isotonic_mutation_hardening_test.rs:234:26`: manual `RangeInclusive::contains` implementation
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#manual_range_contains
  - Hinweis: `-D clippy::manual-range-contains` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::manual_range_contains)]`
  - Hinweis: use


## Crate: `memfuse-candle`

### `crates/memfuse-candle/src/gasp.rs` (1 Fehler)

Fehlercodes: `clippy::manual_range_contains`×1

- **[clippy::manual_range_contains]** @ `crates/memfuse-candle/src/gasp.rs:474:17`: manual `RangeInclusive::contains` implementation
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#manual_range_contains
  - Hinweis: `-D clippy::manual-range-contains` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::manual_range_contains)]`
  - Hinweis: use


## Crate: `memfuse-db`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::useless_conversion`×2

- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: consider removing `.into()`


## Crate: `memfuse-index`

### `crates/memfuse-index/src/hnsw.rs` (4 Fehler)

Fehlercodes: `clippy::useless_conversion`×4

- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: consider removing `.into()`


## Crate: `memfuse-mcp`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::useless_conversion`×2

- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: consider removing `.into()`


## Crate: `memfuse-router`

### `crates/memfuse-index/src/hnsw.rs` (2 Fehler)

Fehlercodes: `clippy::useless_conversion`×2

- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:857:48`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: `-D clippy::useless-conversion` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::useless_conversion)]`
  - Hinweis: consider removing `.into()`
- **[clippy::useless_conversion]** @ `crates/memfuse-index/src/hnsw.rs:1992:38`: useless conversion to the same type: `u64`
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#useless_conversion
  - Hinweis: consider removing `.into()`


## Crate: `memfuse-store`

### `crates/memfuse-store/tests/docid_128_kv_engine.rs` (1 Fehler)

Fehlercodes: `clippy::field_reassign_with_default`×1

- **[clippy::field_reassign_with_default]** @ `crates/memfuse-store/tests/docid_128_kv_engine.rs:9:5`: field assignment outside of initializer for an instance created with Default::default()
  - Hinweis: consider initializing the variable with `memfuse_store::LsmConfig { path: dir.path().to_path_buf(), ..Default::default() }` and removing relevant reassignments
  - Hinweis: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#field_reassign_with_default
  - Hinweis: `-D clippy::field-reassign-with-default` implied by `-D warnings`
  - Hinweis: to override `-D warnings` add `#[allow(clippy::field_reassign_with_default)]`

