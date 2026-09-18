# Diagnose-Report: `memfuse-crypto` (check)

Fehler: **0**  |  Warnungen: **5**

## `crates/memfuse-crypto/src/anti_tamper.rs`  (2 Diagnose(n))

- **WARNING [unsafe_code]** @ `crates/memfuse-crypto/src/anti_tamper.rs:126:9`: usage of an `unsafe` block
  - _Hinweis:_ requested on the command line with `-W unsafe-code`
- **WARNING [unsafe_code]** @ `crates/memfuse-crypto/src/anti_tamper.rs:136:9`: usage of an `unsafe` block

## `crates/memfuse-crypto/src/kv_segment/segment.rs`  (2 Diagnose(n))

- **WARNING [unsafe_code]** @ `crates/memfuse-crypto/src/kv_segment/segment.rs:216:9`: usage of an `unsafe` block
- **WARNING [unsafe_code]** @ `crates/memfuse-crypto/src/kv_segment/segment.rs:226:9`: usage of an `unsafe` block

## `crates/memfuse-crypto/tests/kv_segment_proptests.rs`  (1 Diagnose(n))

- **WARNING [unsafe_code]** @ `crates/memfuse-crypto/tests/kv_segment_proptests.rs:88:9`: usage of an `unsafe` block
  - _Hinweis:_ requested on the command line with `-W unsafe-code`
