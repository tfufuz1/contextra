# Changelog

All notable changes to `contextra-privacy` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Minimal environment contract traits in `src/contract.rs` (`Clock`, `EgressPolicy`, `TextSearchEngine`, `EntityRecognizer`).
- `PrivacyError` as primary crate error type with `thiserror` and `#[non_exhaustive]`, maintaining `EgressError` and `Error` as type-safe compatibility aliases.
- Table-driven unit test `test_privacy_error_display_and_conversion` verifying error string display formatting.
- `ZeroizeOnDrop` implementation for `GuardedPayload` ensuring payload and session identifier memory zeroization upon drop.
- Known-Answer-Test (KAT) suite and corpus fixtures in `tests/corpus/` (`iban.json`, `email.json`, `phone.json`, `credit_card.json`) executed by `tests/known_answer_tests.rs`.
- Integration tests in `tests/zeroize_and_policy_fail_closed.rs` verifying fail-closed policy evaluation and drop lifecycle.
- Minimal standalone example in `examples/standalone.rs` (<= 30 lines) executable with `--no-default-features`.
- Public API snapshot in `docs/decoupling/public-api.txt`.

### Changed
- Enforced `#![deny(missing_docs)]` across all public API items.
- Modernized and harmonized crate documentation in `README.md` including explicit architectural boundaries against `contextra-vault`, `contextra-prompt-guard`, and `contextra-mcp`.
- Neutralized all legal and compliance assertions (INV-7).
