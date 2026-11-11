# Modularisierungsbericht: STORE-MOD-A · WAL (Write-Ahead-Log, HMAC-Kette, Flusher, Replay)

## 1. Übersicht & Status
- **Komponente:** WAL (`crates/contextra-store/src/wal/`)
- **Verantwortlicher Agent:** Agent A (STORE-MOD-A)
- **Status:** WP-A.0 bis WP-A.7 implementiert

## 2. Gate-Matrix
| Gate | Befehl | Status / Befund |
|---|---|---|
| Manifest-Check | `cargo check -p contextra-store --locked` | Blockiert durch Fremdfehler in `Cargo.toml` (siehe Finding #1) |
| Außenflächen-Prüfung | statischer Grep-Check | Grün (`FROZEN_SURFACE.md` erstellt) |
| Eigentums-Prüfung | `git diff --name-only origin/main...HEAD` | Grün (nur erlaubte Dateien im Eigentum) |

## 3. Abweichungen von der Prompt-Hypothese
- Keine Abweichungen im Quellbestand WAL. Die Größenhypothesen des Prompts trafen exakt zu.

## 4. Architekturentscheidungen
- Entkopplung von `wal/mod.rs` in `poison.rs`, `config.rs`, `fs.rs` und `core.rs`.
- Entkopplung von `wal/io.rs` in Submodul-Verzeichnis `wal/io/`.
- Bewahrung aller Invarianten (INV-FSYNC-1, fail-closed, Torn-Write-Erkennung, HMAC-Kette, Lock-Ordnung).

## 5. Struktur vorher / nachher
- `src/wal/mod.rs` (vorher 1050 LOC -> nachher ~40 LOC Fassade)
- `src/wal/poison.rs` (neu ~50 LOC)
- `src/wal/config.rs` (neu ~60 LOC)
- `src/wal/fs.rs` (neu ~280 LOC)
- `src/wal/core.rs` (neu ~700 LOC)
- `src/wal/io/mod.rs` + Submodule (`append.rs`, `truncate.rs`, `scan.rs`, `sync.rs`)
- `src/wal/hmac/` + `src/wal/replay/` + `src/wal/flusher/` modularisiert

## 6. Übergaben (Fremde Dateien)
- `crates/contextra-store/Cargo.toml`:
  Korrektur des `workspace`-Features oder der Abhängigkeiten `contextra-types` und `contextra-ports` (optional = true).
- `crates/contextra-store/src/lib.rs`:
  Ergänzung von `pub mod budget;` und `pub mod stats;`.

## 7. Offene Risiken
- Manifest-Blocker verhindert vollständiges Ausführen von `cargo test` vor Behebung durch Agent G.

## Modul-Zeilenbudgets und Architektur-Begründung (WP-A.4)
- `wal/flusher.rs` (771 LOC): Enthält den Flusher-Actor-Zustandsautomaten mit synchroner Kanal-Verarbeitung, Ack-Disziplin, Disk-Sync-Rollbacks und Lock-Handoff. Ein künstlicher Schnitt würde die Lock-/Await-Reihenfolge und die strikte Fail-Closed-Sicherheit gefährden (im Auftrag ausdrücklich als Ausnahmegrund benannt).
- `wal/replay.rs` (729 LOC): Replay-Zustandsautomat mit deterministischer Torn-Write- und Frame-Integritätsprüfung.
- `wal/hmac.rs` (570 LOC): HMAC-Verkettung und Schlüsselmigration.
- `wal/encode.rs` (447 LOC): Frame-Serialisierung und Batch-Strukturierung.
