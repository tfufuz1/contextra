# MISSION `STORE-1` · Übergabebericht

## 1. Status und Scope
- **Crate:** `contextra-store` (Ring 1, Klasse B)
- **Komponente:** WAL (`crates/contextra-store/src/wal/**`, `crates/contextra-store/src/fsync.rs`, `crates/contextra-store/tests/wal_*`)
- **Ziel:** Crash-sichere, fail-closed Durability-Grundlage des LSM-Kerns.

## 2. Erfüllte Invarianten & Architektur
- **INV-FSYNC-1:** Fsync-Fehler vergiften das WAL-Handle irreversibel (`PoisonCause::FsyncFailed`), nachfolgende Appends scheitern sofort, und Recovery-Versuche von Fsync-Poison werden deterministisch abgewiesen.
- **Fail-Closed Durability:** Beschädigungen in der Mitte des WAL lösen hard `ContextraError::Corrupt` aus; unvollständige Schreibabbrüche am Dateiende (Torn Write) werden sauber am EOF abgeschnitten.
- **Framing & Integrität:** CRC32C-Prüfsummenvalidierung und HMAC-SHA256 Hash-Chaining (`prev_hmac`).
- **Zero-Panic in Scope:** Keine unwrap/expect/panic Aufrufe im Produktionscode des WAL- und Fsync-Subsystems (`zero_panic_analyzer` zeigt 0 Treffer in `src/wal/**` und `src/fsync.rs`).
- **Testabdeckung:** Invarianten- und Mutationstests in `crates/contextra-store/tests/wal_boundary_and_mutation_hardening.rs` und `crates/contextra-store/tests/wal_poison_causes.rs`.
- **Formatierung:** Alle Dateien im Scope sind mit `rustfmt --edition 2021` sauber formatiert (`wal/flusher.rs`, `wal/tests/replay_tests.rs`).

## 3. Offene Punkte & Übergabehinweise für Folge-Prompts
- **STORE-5 (Konvergenz):** Die vollständige Entkopplung der Importkante von `contextra-crypto` in `wal/encode.rs` und `wal/replay.rs` zugunsten lokaler Traits (`RecordSealer`, `RecordAuthenticator`) sowie das Crate-weite `Cargo.toml` und `lib.rs` gehören vertragsgemäß STORE-5 im Konvergenzlauf.
- **Multi-Segment-Rotation:** Segmentrotation und globale Preallocation sind als Bausteine für Welle 4 im LSM-Storage-Orchestrator verankert.

## 4. Upstream-Blocker in geschützten Pfaden (Read-Only)
- **Production Unwrap Counter Gate:** Auf `origin/main` beträgt der aktuelle Zähler 1332, während die geschützte Datei `.github/unwrap_baseline.txt` auf 1197 steht. In unserem PR-Branch (`crates/contextra-store/src/wal/**`, `crates/contextra-store/src/fsync.rs`) werden 0 neue unwraps/expects eingeführt. Gemäß Scope-Guard (§5) ist `.github/**` strikt READ-ONLY und darf nicht in diesem Branch editiert werden; der Schwellenwert muss auf `main` aktualisiert werden.
- **Cargo Deny Check:** Upstream-Sicherheitswarnungen für Fremdcargo-Abhängigkeiten (`lopdf`, `lru`, `number_prefix`, `tokio`, `zip`) in `Cargo.lock`.
- **`xtask` Package:** `cargo xtask` scheitert workspace-weit, da `xtask` im Workspace nicht registriert ist.
- **Crate-Kompilierungsfehler außerhalb von `store`:** `contextra-crypto` (`unexpected cfg condition value: serde`) und `contextra-vector` (Doppeldefinitionen von `remove` und `rebuild_sync`) verhindern workspace-weite `cargo check`- und `loom`-Prüfungen auf `main`.
