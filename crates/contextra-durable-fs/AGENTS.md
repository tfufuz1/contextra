# AGENTS – contextra-durable-fs

Abgeleitet aus [`docs/SPEC.md`](../../docs/SPEC.md) (8.5, 14.1, 14.7, D-04, D-50, D-55, D-56, D-83). Bei Widerspruch gilt die SPEC (D-98). Gemeinsame Regeln: [`AGENTS.md`](../../AGENTS.md).

**Zweck:** Dateisystem-Implementierung `RealVfs` mit dauerhaften Schreib- und Austauschoperationen.

**Ring und Abhängigkeiten:** Ring 1. Erlaubt: `types`, `ports`, `sys`.

## Fachprofil (Arbeitsrolle)

Dateisystem-Durability-Ingenieur (ext4, XFS, btrfs, ZFS, APFS; Datenbank-Storage-Erfahrung). Dieses Profil beschreibt Arbeitsweise und Fachwissen und begründet keine eigene Anforderung.

- POSIX-Durability: `fsync`/`fdatasync`, Verzeichnis-`fsync` nach `rename` und `create`, atomarer Austausch, Torn Writes, Sektor- und Page-Atomarität, `O_DIRECT`-Grenzen, Barriers, `ENOSPC`.
- „fsyncgate“: Nach einem `fsync`-Fehler ist der Seitenzustand unbestimmt. Handle vergiften, Recovery per Replay, nie blind wiederholen.
- Crash-Konsistenz-Prüfung (ALICE/CrashMonkey-Denkweise): Abbruch an jedem Syscall-Punkt, Reordering, Teilpersistenz.
- Plattform-Probe: Verdikt je Tier, rotational- und COW-Erkennung samt Fehlerfall „unbekannt“ (dm, md, LVM).

## Normative Anker

- **`RealVfs` implementiert `ports::Vfs`** (ein Trait, D-04): Datei öffnen, positionelles Lesen und Schreiben, `sync_data`/`sync_all`/`sync_dir`, atomarer Austausch (tmp, `fsync`, `rename`, Verzeichnis-`fsync`), exklusiver Verzeichnis-Lock (D-50, T-LOCK-1).
- **`fsync`-Fehler** werden unverändert nach oben gemeldet und sind für die Datei fatal; kein Wiederholen auf demselben Handle (D-55, I-10, FM-5, T-FSYNC-1).
- **Startprobe (D-56, FM-11, T-PLAT-1):** `statfs`-Typ, atomares `rename`, Verzeichnis-`fsync`, Atomaritätsprobe für 64-Byte-Schreibvorgänge an ausgerichteten Offsets. Unbekannte Dateisysteme, NFS, SMB, FUSE, overlayfs: `Config::UnsupportedFilesystem`. Tier 1: Linux ext4/XFS; Tier 2: macOS APFS (`F_FULLFSYNC`), btrfs, ZFS (Restrisiko `FsJournalOrCow` immer). Die Probe liefert `fsync_p50` (Bezug für D-30).
- **`scrub`** (Überschreiben vor `unlink`) nur bei `rotational = true` und nicht-COW; sonst entfällt es und das Restrisiko wird ausgewiesen (D-83). Überschreiben ist nie ein Löschnachweis auf SSD oder COW.
- **Waisen:** Verzeichnis `tmp/` wird beim Start geleert (5.6, 14.1). Keine Datei- oder Verzeichnisnamen außer Tags, `file_id`, `erasure_id`, festen Namen (D-75, T-RAW-6).
- Kein `unsafe` (AR-8): Syscalls laufen über `contextra-sys`.

## Entscheidungshilfen

- Jede Persistenzoperation braucht einen Eintrag in der Crash-Matrix (15.1, `FaultVfs`): Abbruch nach jedem `write`, `fsync`, `rename`, `unlink`.
- Code, der nicht im Modulbaum liegt, ist ein Defekt: einbinden oder löschen.
- Tests dürfen nicht durch Cargo-Einstellungen (`autotests = false`) stillgelegt sein.

## Verboten

Eigener `Vfs`-Trait, `unsafe`, `mmap`, `std::net`, Netzwerk-Bibliotheken, `scrub` auf Flash oder COW ohne Probe, Wiederholen eines fehlgeschlagenen `fsync`.

## Abnahme

`T-PLAT-1`, `T-LOCK-1`, `T-CRASH-1` bis `T-CRASH-6`, `T-FSYNC-1`; Conformance gegen `FaultVfs` und In-Memory-`Vfs` (`tests/tk_fault_vfs_conformance.rs`, `tests/sy_*`).

## Arbeitspakete

WP-1.4 (einzige `Vfs`-Implementierung, `store`/`vector` ohne `std::fs`), WP-5.2 (Probe, Plattenreserve). Statusquelle: `.agents/jules/tasks/BACKLOG.md`.

## Prüfbefehle

```bash
cargo test -p contextra-durable-fs --locked
cargo test --locked --test sy_durability_tests --test sy_fsclass_tests
cargo test -p contextra-arch-tests --locked
```
