# Changelog

Alle nennenswerten Änderungen an `contextra-sys` werden in dieser Datei dokumentiert.

Das Format basiert auf [Keep a Changelog](https://keepachangelog.com/de/1.1.0/).

## [Unreleased]

### Added
- `src/contract.rs`: Minimale Umwelt-Traits (`MemoryLocker`) und I/O-freie Referenzimplementierung `NoopMemoryLocker` sowie native `OsMemoryLocker`.
- `examples/standalone.rs`: Kompaktes Standalone-Beispiel (≤ 30 Zeilen) zur autarken Verwendung von `mmap_readonly`.
- `tests/error_paths.rs`: Umfassende Fehlerpfad- und Lebenszyklus-Tests für mmap-, durability-, mlock- und Vault-Bereinigung.
- `docs/decoupling/UNSAFE_INVENTORY.md`: Detailliertes, auditierbares Inventar aller 22 `unsafe`-Stellen.
- `docs/decoupling/public-api.txt`: Snapshot der öffentlichen API gemäß Baukasten-Spezifikation.
- Strikte Lint-Vorgabe `#![deny(missing_docs)]` in `src/lib.rs`.

### Changed
- `capabilities.fragment.toml`: Aktualisiert auf Schema v3 (`tier = "stable"`, leere interne Abhängigkeiten).
- `Cargo.toml`: Ergänzt um vollständige Metadaten (`repository`, `keywords`, `categories`, `readme`, `include`, `publish = false`).
- `README.md`: Aktualisiert mit Plattformmatrix, Fehlermodell, Entnahme-Rezept und tatsächlichen Abhängigkeiten.
