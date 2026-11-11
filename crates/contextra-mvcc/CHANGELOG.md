# Changelog

All notable changes to `contextra-mvcc` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `src/contract.rs`: Minimal environment traits re-exporting `Clock`, `SystemClock`, and `MetricsSink` from `contextra-env`.
- `examples/standalone.rs`: Minimal standalone example (≤ 30 lines) demonstrating snapshot registration, staging, and SSI conflict detection without default features.
- Model-based property proof in `tests/ssi_serializability_model.rs` validating serializability against a formal direct serialization graph (DSG) cycle oracle across random transaction histories.
- Table-driven display and conversion test for `MvccError` variants.
- Snapshot of public API in `docs/decoupling/public-api.txt`.

### Changed
- Promoted `#![warn(missing_docs)]` to `#![deny(missing_docs)]` across the entire crate.
- Replaced unallowed dev-dependency on `contextra-testkit` with `contextra-env` (`ManualClock`, local metrics event sink).
- Updated crate metadata and documentation to reflect standalone R3 readiness.
