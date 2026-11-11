# Changelog

All notable changes to `contextra-simd` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Klasse-A Seam-Vertragsdatei `src/contract.rs` mit `DistanceFn` Trait und `MetricDistanceFn`.
- Standalone-Beispiel `examples/standalone.rs` (17 Zeilen, ohne Default-Features ausführbar).
- Erweiterte Differenztests gegen Skalar-Orakel für Sonderwerte (Nullen, Vorzeichen-Nullen, Denormale, extreme Magnituden) und R3-Dimensionen (1, 4, 17, 128, 768, 1536).
- Dokumentation der Sonderwerte in `docs/decoupling/SPECIAL_VALUES.md`.
- Unsafe-Inventar in `docs/decoupling/UNSAFE_INVENTORY.md` für alle 272 Unsafe-Stellen.
- Public-API Snapshot in `docs/decoupling/public-api.txt`.
- `#![deny(missing_docs)]` Linter für lückenlose öffentliche Dokumentation.

### Changed
- `Cargo.toml` Metadaten auf R3-Reifegrad aktualisiert (`readme`, `repository`, `keywords`, `categories`, `publish = false`).
- Unbenutzte Abhängigkeit `bytemuck` aus `Cargo.toml` und `Cargo.lock` entfernt.
- `capabilities.fragment.toml` auf Schema v3 angeglichen.
