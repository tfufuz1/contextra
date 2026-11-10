# MISSION `STORE-1` · Übergabebericht

## 1. Status und Scope
- **Crate:** `contextra-store` (Ring 1, Klasse B)
- **Komponente:** WAL (`crates/contextra-store/src/wal/**`, `crates/contextra-store/src/fsync.rs`)
- **Ziel:** Crash-sichere, fail-closed Durability-Grundlage des LSM-Kerns.

## 2. Erfüllte Invarianten & Architektur
- **INV-FSYNC-1:** Fsync-Fehler vergiften das WAL-Handle irreversibel (`PoisonCause::FsyncFailed`), nachfolgende Appends scheitern sofort, und Recovery-Versuche von Fsync-Poison werden deterministisch abgewiesen.
- **Fail-Closed Durability:** Beschädigungen in der Mitte des WAL lösen hard `ContextraError::Corrupt` aus; unvollständige Schreibabbrüche am Dateiende (Torn Write) werden sauber abgeschnitten.
- **Framing & Integrität:** CRC32C-Prüfsummenvalidierung und HMAC-SHA256 Hash-Chaining (`prev_hmac`).
- **Zero-Panic:** Keine unwrap/expect/panic Aufrufe im Produktionscode des WAL- und Fsync-Subsystems.
- **Testabdeckung:** Regressionstests in `crates/contextra-store/tests/wal_boundary_and_mutation_hardening.rs` und `crates/contextra-store/tests/wal_poison_causes.rs`.

## 3. Offene Punkte & Übergabehinweise für Folge-Prompts
- **STORE-5 (Konvergenz):** Die vollständige Entkopplung der Importkante von `contextra-crypto` in `wal/encode.rs` und `wal/replay.rs` zugunsten lokaler Traits (`RecordSealer`, `RecordAuthenticator`) sowie das Crate-weite Cargo.toml und lib.rs gehören vertragsgemäß STORE-5 im Konvergenzlauf.
- **Multi-Segment-Rotation:** Segmentrotation und globale Preallocation sind als Bausteine für Welle 4 im LSM-Storage-Orchestrator verankert.
