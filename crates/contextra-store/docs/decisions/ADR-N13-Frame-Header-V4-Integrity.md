# ADR-N13: WAL Frame V4 Header Integrity and Corruption Discrimination

## Status
Pending Implementation

## Context
A crash during append can cause torn writes at the WAL tail, which `open-heal` safely truncates. However, corruptions in middle frames (e.g. bitflips in length headers) must not be treated as torn tails, as truncating them would discard valid, subsequent committed transactions.

## Decision
1. Future WAL Frame V4 will introduce enhanced header-level integrity (e.g., CRC-protected length headers or framed chunk preambles).
2. The replay scanner will unambiguously discriminate between clean torn tails at physical EOF and corrupted length headers in middle entries.
3. Backward compatibility with V2 and V3 frames will be retained via transparent rewrite on open.

## Validation
Specification tests are maintained in `crates/contextra-store/tests/wal_spec_pending.rs` and can be run via:
`cargo test -p contextra-store --test wal_spec_pending -- --ignored`
