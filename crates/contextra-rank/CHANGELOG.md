# Changelog

All notable changes to `contextra-rank` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- `contract.rs` defining minimal environmental abstraction (`Clock` trait) for core standalone independence.
- Standalone runnable example in `examples/standalone.rs` executing multi-signal RRF under `--no-default-features`.
- Comprehensive deterministic tie-breaking proof test suite in `tests/fusion_tie_break_proof.rs`.
- Legacy reference test harness for `DocId` parsing and hashing in `tests/legacy_reference/doc_id_reference.rs`.

### Changed
- Strengthened Clippy checks, removing `unwrap()` and `panic!` calls across test harnesses.
- Updated crate metadata and documentation to fulfill R3 release-readiness requirements.
