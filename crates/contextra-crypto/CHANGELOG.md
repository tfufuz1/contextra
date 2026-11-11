# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `src/contract.rs`: Minimal environmental contract traits (`CryptoEntropy`, `DeterministicEntropy`, `KmsContract`, `InMemoryKms`) for autarkic decoupled operation without I/O.
- `examples/standalone.rs`: Self-contained minimal example (<= 30 lines) demonstrating AEAD record seal and open without default features.
- Formal RFC/NIST Known-Answer-Tests covering RFC 8452 (AES-256-GCM-SIV), RFC 5869 (HKDF), RFC 9106 (Argon2id), and RFC 4231 (HMAC-SHA256).
- Golden corpus test vectors with deterministic SHA-256 verification in `tests/golden/`.
- Fuzzing suite with three active targets: `wal_hmac_chain_verify_fuzz`, `aead_seal_open_fuzz`, and `kdf_argon2_hkdf_fuzz`.
- Additive raw 32-byte key derivation methods on `KeyManager` (`derive_kv_key_bytes`, `derive_kv_key_scoped_bytes`, `cipher_for_bytes`, `cipher_for_scoped_bytes`).

### Changed
- Removed deprecated dev-dependency on `contextra-testkit` to achieve pure layer isolation (Gate G-L1 and G-L4 isolation proof).
- Hardened WAL HMAC chain fuzz target to eliminate false production asserts.
- Cleaned legacy deletion proof leftovers from fuzz and benchmark suites.
