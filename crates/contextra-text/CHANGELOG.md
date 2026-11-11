# Changelog

All notable changes to `contextra-text` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Autarkic Class-A decoupling: Reached Maturity Level R3.
- Minimal environment trait contract in `contract.rs` (`Clock`, `Rng`, `DocKey`, `Epoch`, `Error`).
- Minimal stand-alone example in `examples/standalone.rs` utilizing `MemoryTextIndex` (<= 30 lines).
- Full documentation coverage enforced via `#![deny(missing_docs)]`.
- Public API surface snapshot documented in `docs/decoupling/public-api.txt`.

### Changed
- Decoupled core inverted index tests from `contextra-ports` and `contextra-types` (G-L2 reference count = 0).
- Core index uses local `PostingStore` trait abstraction in `src/storage.rs`.
- Compatibility bridge to `StorageEngine` and `TextIndex` cleanly isolated under `src/compat/` behind the `workspace` feature.
- Upgraded Modularity Score to 100/100.
