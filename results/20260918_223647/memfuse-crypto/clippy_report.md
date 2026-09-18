# Diagnose-Report: `memfuse-crypto` (clippy)

Fehler: **5**  |  Warnungen: **0**

## `crates/memfuse-crypto/src/anti_tamper.rs`  (2 Diagnose(n))

- **ERROR [unsafe_code]** @ `crates/memfuse-crypto/src/anti_tamper.rs:126:9`: usage of an `unsafe` block
  - _Hinweis:_ `-D unsafe-code` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(unsafe_code)]`
- **ERROR [unsafe_code]** @ `crates/memfuse-crypto/src/anti_tamper.rs:136:9`: usage of an `unsafe` block

## `crates/memfuse-crypto/src/kv_segment/segment.rs`  (2 Diagnose(n))

- **ERROR [unsafe_code]** @ `crates/memfuse-crypto/src/kv_segment/segment.rs:216:9`: usage of an `unsafe` block
- **ERROR [unsafe_code]** @ `crates/memfuse-crypto/src/kv_segment/segment.rs:226:9`: usage of an `unsafe` block

## `crates/memfuse-crypto/tests/kv_segment_proptests.rs`  (1 Diagnose(n))

- **ERROR [unsafe_code]** @ `crates/memfuse-crypto/tests/kv_segment_proptests.rs:88:9`: usage of an `unsafe` block
  - _Hinweis:_ `-D unsafe-code` implied by `-D warnings`
  - _Hinweis:_ to override `-D warnings` add `#[allow(unsafe_code)]`
