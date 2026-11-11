# Changelog

Alle nennenswerten Änderungen an `contextra-checkpoint` werden in dieser Datei dokumentiert.
Das Format basiert auf [Keep a Changelog](https://keepachangelog.com/de/1.0.0/).

## [Unreleased]

### Added
- `contract.rs`: Minimale Umwelt-Schnittstellen und Verträge (`SnapshotSource`, `CheckpointVfs`, `FsSeam`, `WorkflowState`).
- `contract::InMemorySnapshotSource`: Vollständige, I/O-freie Referenzimplementierung für Tests und Standalone-Verwendung.
- `examples/standalone.rs`: Kompaktes Ausführungsbeispiel (< 30 Zeilen) ohne Workspace-Abhängigkeiten.
- `tests/crash_restore_proofs.rs`: Systematische Crash-, Fault-Injection- und Property-Roundtrip-Beweise.
- `#![deny(missing_docs)]` im Crate-Root zur Gewährleistung lückenloser API-Dokumentation.

### Changed
- `CheckpointError::Io`: Display-Präfix an `ContextraError` angeglichen (`"I/O error: {0}"`).
- `src/store.rs`: Direkte Referenzen auf `contextra_ports` und `contextra_types` durch lokale bzw. Basis-Typen ersetzt.
- `src/fs_seam.rs` und `src/vfs.rs`: Neutrale Dokumentation ohne direkte Port-Verweise im Kern.

### Removed
- `src/source.rs`: Veraltete, redundante Nahtdatei entfernt (kanonische Trait-Definition in `snapshot_source.rs` bzw. `contract.rs`).
- Veralteter Autonomie-Direktivenblock aus `AGENTS.md` entfernt.
