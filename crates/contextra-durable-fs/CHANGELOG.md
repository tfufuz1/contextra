# Changelog

All notable changes to `contextra-durable-fs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Contract Trait `DurableVfs` and `Flushable` in `src/contract.rs`.
- In-memory reference implementation `MemoryVfs` for isolated testing without disk I/O.
- Error bridge in `src/compat/error.rs` with formatted error codes.
- Golden fixtures and test `tests/golden_format_test.rs` ensuring byte-for-byte serialization format stability (`INV-1`).
- Exhaustive crash matrix and recovery tests in `tests/crash_matrix_exhaustive.rs` verifying atomicity across all crash points.
- Standalone runnable example (`examples/standalone.rs`) under 30 lines.
- `#![deny(missing_docs)]` enforcement for public API.

### Changed
- Refined platform durability documentation in `src/platform.rs`.
- Cleaned up obsolete autonomy directives from `AGENTS.md`.
- Cargo package metadata updated with full attributes for R3 release readiness.
