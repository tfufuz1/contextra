# Changelog

All notable changes to `contextra-adapt` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Compatibility bridge in `compat/` (`clock.rs`, `error.rs`) gated under feature `workspace`.
- Experimental feature flags `exp-bandit-routing`, `exp-flow-thompson`, `exp-rie-greedy` with legacy aliases.
- Standalone minimal example `examples/standalone.rs` (≤ 30 lines).
- Golden conformance suite in `tests/golden/` with bit-identical regression verification.
- Public API snapshot in `docs/decoupling/public-api.txt`.

### Changed
- Crate hardened to Class A building block at maturity level R3.
- Enforced `#![deny(missing_docs)]` across public API items.
- Standardized Cargo metadata and package attributes.
