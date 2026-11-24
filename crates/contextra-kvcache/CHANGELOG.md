# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Semantic Versioning.

## [Unreleased]

### Added
- Autarker Umwelt- und Indexvertrag in `src/contract.rs` (`Clock`, `Rng`, `KvCacheEngine`) mit I/O-freier Referenzimplementierung `MemoryKvCache`.
- Standalone-Beispiel `examples/standalone.rs` (<= 30 Zeilen).
- Eigenschaftsbasierte Tests in `tests/radix_property_test.rs` fuer `PrefixRadixTree` gegen ein naives Map-Orakel und strikte Mandantenisolation.
- Kryptografischer Beweistest in `tests/zeroize_proof_test.rs` fuer `ShreddableSegmentKey` und Zeroize-Drop-Garantien.
- Lizenzen `LICENSE-MIT` und `LICENSE-APACHE` im Crate hinterlegt.

### Changed
- Vollstaendige Feature- und Nahtstellen-Hygiene durchgefuehrt: `contextra-testkit` aus den Dev-Dependencies entfernt.
- P28-Determinismus-Audit behoben: `HashMap` in `src/eviction_worker.rs` durch deterministische `BTreeMap` ersetzt; Zeitmessungs-Annotationen ergaenzt.
- Compiler-Warnungen und unbenutzte Definitionen vollstaendig bereinigt.
- API-Dokumentation mit `#![deny(missing_docs)]` abgesichert.
- Metadaten in `Cargo.toml` gemaess R3-Baukastenstandard ergaenzt.
