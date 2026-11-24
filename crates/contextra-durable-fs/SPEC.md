# Spezifikation: contextra-durable-fs

> **Revision R12** (Repo-Abgleich, HEAD `60ea034`): Änderungen gemäß `SPEC.md` K-58…K-63 und Anhang B.5. Abschnitte mit Kennzeichnung „R12“ sind normativ.

## 1. Identität & Einordnung
- **Crate:** `contextra-durable-fs`
- **Klasse:** L0 Fundament, Status `experimental` (R12). Keine Kanten zu anderen Crates (K-58).
- **Ring:** Ring 0 (Fundament)
- **Sicherheitsniveau:** `#![forbid(unsafe_code)]`, zero-panic doctrine
- **R12-Abgrenzung:** Diese Crate ist die einzige Stelle, an der `std::fs` im Durability-Pfad erlaubt ist (OS-Grenze). `scripts/check_forbidden_apis.sh` verbietet `std::fs` im Speicherkern (`contextra-storage`) und darf hier nicht gelten.

## 2. Invarianten & Garantien
- **`INV-FSYNC-1` (fsync-Poisoning):**
  Schlägt ein Flush (`sync_file` / `flush`) auf einem Dateihandle ([`DurableHandle`]) fehl, wechselt das Handle dauerhaft in den Status `Poisoned`. Jede nachfolgende Schreiboperation scheitert sofort ohne stillen Retry.
- **`INV-VFS-ATOMIC-1` (Crash-Konsistente Ersetzung):**
  `write_atomic` erstellt eine temporäre Datei im selben Verzeichnis wie das Ziel, schreibt und flusht diese via `sync_file`, führt ein atomares `rename` aus und synchronisiert abschließend das Elternverzeichnis via `sync_dir`.
- **`INV-FSYNC-2` (R12):** Nach einem Schreibvorgang auf einen Journal- oder Atomic-Pfad folgt `sync_file`; nach einem `rename` folgt `sync_dir`. Ein EINTR (`ErrorKind::Interrupted`) wird wiederholt; alle anderen Fehler werden gemeldet und poisonen das Handle.
- **`INV-POISON-1` (Fail-Closed Systemstatus):**
  Einmal vergiftet ([`HealthState::Poisoned`]), kann dieser Zustand während der Prozesslaufzeit nicht wieder verlassen werden.
- **`INV-JOURNAL-CRC-1` (Prüfsummen- und Längenrahmen):**
  Jeder Record im [`AppendOnlyJournal`] besitzt einen 20-Byte-Rahmen mit Magic `CTXJ`, monoton steigender Sequenznummer, Längenfeld und CRC32.

## 3. Schnittstellen
- [`Vfs`]: Lokaler synchroner Trait für Dateisystemzugriffe.
- [`StdVfs`]: Standardimplementierung über `std::fs`.
- [`AppendOnlyJournal`]: Append-only Log mit konfigurierbaren Sync-Politiken und Recovery.
- [`FaultInjectingVfs`]: VFS-Dekorator für Tests und Crash-Punkt-Simulation.
- [`AtomicHealthState`]: Lock-freier Zustandsautomat für Speichersysteme.

## 4. R12-Status

- Ein produktiver `RealFs`-Adapter fehlt noch (SD-70). Bis dahin nutzt der Speicherkern den Port `Fs` aus `storage-ports` mit `FaultInjectingVfs` für Tests.
- `AppendOnlyJournal` ist Vorlage für das neue WAL-Format nur insoweit, als Rahmen und CRC übernommen werden; das Speicherformat wird in `contextra-storage` neu definiert.
