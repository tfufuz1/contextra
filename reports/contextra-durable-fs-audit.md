# Durability- und Audit-Bericht: contextra-durable-fs (SPEC §8.5, D-04, D-50, D-55, D-56, D-83, D-97)

- Datum: 2026-11-03
- Bearbeiter: Jules (Dateisystem-Durability-Ingenieur)
- Crate: `contextra-durable-fs` (Ring 1)

## 1. ZWECK-SATZ & MANIFEST (D-97, T-DOC-1, T-ARCH-5)
Wortgleiche Zweck-Deklaration über alle kanonischen Dokumente:
> "Dateisystem-Implementierung `RealVfs` mit dauerhaften Schreib- und Austauschoperationen."

- `crates/contextra-durable-fs/Cargo.toml` (`description`)
- `crates/contextra-durable-fs/README.md` (Zeile 1)
- `crates/contextra-durable-fs/src/lib.rs` (`//!`)

## 2. MODUL-STRUKTUR & ARCHITEKTUR
- `real_vfs`: `RealVfs` und `RealFile` implementieren `contextra_ports::vfs::Vfs` und `VfsFile`.
  - Positionelles Lesen und Schreiben (`read_at`, `write_at`)
  - Sync-Garantien: `sync_data`, `sync_all`, `sync_dir`
  - Fsync-Poisoning (D-55, INV-FSYNC-1): Jeder I/O- oder fsync-Fehler vergiftet das Dateihandle permanent; Folgeoperationen schlagen fehl mit `FatalIo("file handle is poisoned due to previous I/O or sync failure")`.
- `atomic`: Atomare Schreib- und Austauschoperationen (`write_atomic`, `atomic_replace`, `clean_tmp`).
  - Muster: tmp-Datei -> `sync_data` -> `rename` -> übergeordnetes Verzeichnis `sync_dir` (Basis aller Crash-Garantien).
  - Waisen-Bereinigung: `clean_tmp` leert `tmp/` beim Systemstart (DF-08, T-CRASH-6).
- `dir_lock`: Exklusive Verzeichnissperre `LOCK` mit `flock`-Semantik (D-50, T-LOCK-1, `DIR_LOCKED_MSG`).
- `probe`: Startprobe (D-56, T-PLAT-1) mit statfs-Erkennung:
  - Tier 1: Linux ext4 / XFS
  - Tier 2: macOS APFS, btrfs, ZFS (Restrisiko `FsJournalOrCow`)
  - Ablehnung: NFS, SMB, FUSE, overlayfs als `UnsupportedFilesystem`.
  - Atomaritätsprobe für 64-Byte-/80-Byte-Schreibvorgänge an ausgerichteten Offsets.
  - Latenzmessung: Ermittelt `fsync_p50`.
  - Freispeicherprüfung: `free_space` (DF-10, T-OPS-2).
- `scrub`: Sicheres Überschreiben vor `unlink` (`scrub_before_unlink`, D-83) nur bei `rotational == true` und nicht-COW; sonst No-Op mit Ausweis im `ScrubOutcome`.
- Bereinigung: Tote Module `health.rs` und `fault_injector.rs` sowie altes `vfs.rs` sind vollständig eliminiert. `FaultVfs` liegt kanonisch im Dev-Crate `contextra-testkit` (D-84).

## 3. AUDIT-KENNZAHLEN & COMPLIANCE
- Produktionszeilen (`src/`): 1.891 LOC (Budget: 2.000 LOC eingehalten).
- Unsichere Blöcke: 0 (`#![forbid(unsafe_code)]` strikt durchgesetzt).
- Paniktreffer (`unwrap`, `expect`, `panic!`, `unreachable!`): 0 im Produktionscode.
- Registrierte Testsuiten: 8 Testsuiten mit 30 Tests, 100% grün (`cargo test -p contextra-durable-fs --offline`).
- Clippy: 0 Warnungen (`cargo clippy -p contextra-durable-fs --all-targets --offline -- -D warnings`).
- Formatierung: `cargo fmt -p contextra-durable-fs --check` bestanden.
