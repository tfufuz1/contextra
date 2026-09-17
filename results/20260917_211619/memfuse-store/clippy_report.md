# Diagnose-Report: `memfuse-store` (clippy)

Fehler: **1**  |  Warnungen: **0**

## `crates/memfuse-store/tests/docid_128_kv_engine.rs`  (1 Diagnose(n))

- **ERROR [clippy::field_reassign_with_default]** @ `crates/memfuse-store/tests/docid_128_kv_engine.rs:9:5`: field assignment outside of initializer for an instance created with Default::default()
  - _Hinweis:_ consider initializing the variable with `memfuse_store::LsmConfig { path: dir.path().to_path_buf(), ..Default::default() }` and removing relevant reassignments
  - _Hinweis:_ for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#field_reassign_with_default
  - _Hinweis:_ `-D clippy::field-reassign-with-default` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(clippy::field_reassign_with_default)]`

