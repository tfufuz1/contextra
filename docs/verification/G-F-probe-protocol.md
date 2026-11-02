# Evidenzprotokoll-Vorlage G-F: Startprobe-Matrix (`docs/verification/G-F-probe-protocol.md`)

Dieses Protokoll dient als Abnahme- und Nachweisvorlage für die Plattformstufen-Startprobe gemäß SPEC §8.5 D-56 und T-PLAT-1.

---

## 1. Getestete Plattformen und Dateisysteme

| Dateisystem | OS / Architektur | Startprobe-Ergebnis | Klassifikation | Restrisiko | Bemerkung |
|---|---|---|---|---|---|
| **ext4** | Linux x86_64 / aarch64 | `Supported(Tier::Tier1)` | Tier 1 | Keines | Standard-Linux-Dateisystem, atomares Rename + fsync verifiziert |
| **XFS** | Linux x86_64 / aarch64 | `Supported(Tier::Tier1)` | Tier 1 | Keines | High-Performance-LSM-Dateisystem, direkte I/O-Haltbarkeit |
| **btrfs** | Linux x86_64 / aarch64 | `Supported(Tier::Tier2)` | Tier 2 | `FsJournalOrCow` | Copy-on-Write erfordert Berichtsausweis im Löschprotokoll |
| **ZFS** | Linux / FreeBSD | `Supported(Tier::Tier2)` | Tier 2 | `FsJournalOrCow` | CoW-Transaktionsmodell mit Metadaten-Journaling |
| **APFS** | macOS aarch64 / x86_64 | `Supported(Tier::Tier2)` | Tier 2 | `FsJournalOrCow` | `F_FULLFSYNC` erforderlich, Altblöcke können gehalten werden |
| **NFS** | Alle Plattformen | `Refused(UnsupportedFilesystem::Nfs)` | Refused | – | Fehlende synchrone Durability- und Verzeichnisgarantien |
| **SMB / CIFS** | Alle Plattformen | `Refused(UnsupportedFilesystem::Smb)` | Refused | – | Unvollständige atomare Rename-Semantik |
| **FUSE** | Alle Plattformen | `Refused(UnsupportedFilesystem::Fuse)` | Refused | – | Userspace-Indirektion ohne synchrone Flushtreue |
| **overlayfs** | Linux Container | `Refused(UnsupportedFilesystem::Overlayfs)` | Refused | – | Schichtenspeicher ohne Crash-Konsistenz-Zusagen |
| **NTFS** | Windows | `Refused(UnsupportedFilesystem::Windows)` | Refused | – | Windows-Support gemäß SPEC D-84 entfernt |
| **Unbekannt** | Alle | `Refused(UnsupportedFilesystem::Unknown)` | Refused | – | Unbekannte Dateisystemtypen werden strikt abgewiesen |

---

## 2. Gemessene Haltbarkeitseigenschaften (Referenzmessung)

| Eigenschaft | Anforderung (SPEC D-56, L-4) | Status | Gemessener Wert / Verhalten |
|---|---|---|---|
| **Atomares Rename** | Zieldatei darf nie halb geschrieben sichtbar sein | Erfüllt | `probe.rename_atomic == true` (`rename_durable` verifiziert) |
| **Verzeichnis-fsync** | Metadaten-Flush auf übergeordnetem Verzeichnis | Erfüllt | `probe.dir_fsync_ok == true` (`flush_dir` erfolgreich) |
| **Schreibatomarität 64 Byte** | Ausgerichtete 64-Byte Schreibvorgänge stabil | Erfüllt | `probe.write_atomic_64 == true` (Offset 0 und 64 verifiziert) |
| **Schreibatomarität 80-Byte-Slots** | L-4 Slot-Atomarität an festen Grenzen | Erfüllt | `probe.write_atomic_64 == true` (Offsets 0, 80, 160 verifiziert) |
| **fsync_p50 Stichprobe** | Median Latenzmessung über 7 Iterationen | Erfüllt | `probe.fsync_p50` gemessen (Grundlage für D-30 Schwellenwerte) |
| **Freier Festplattenspeicher** | Plattenreserve gemäß SPEC D-65, DF-10 | Erfüllt | `free_space(&Path)` liefert Byte-Kapazität |
