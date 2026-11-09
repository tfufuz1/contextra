# Spezifikation: contextra-durable-fs

## 1. Identität & Einordnung
- **Crate:** `contextra-durable-fs`
- **Klasse:** Klasse A (vollständig autarker Baustein, B1–B12)
- **Ring:** Ring 0 (Fundament)
- **Sicherheitsniveau:** `#![forbid(unsafe_code)]`, zero-panic doctrine

## 2. Invarianten & Garantien
- **`INV-FSYNC-1` (fsync-Poisoning):**
  Schlägt ein Flush (`sync_file` / `flush`) auf einem Dateihandle ([`DurableHandle`]) fehl, wechselt das Handle dauerhaft in den Status `Poisoned`. Jede nachfolgende Schreiboperation scheitert sofort ohne stillen Retry.
- **`INV-VFS-ATOMIC-1` (Crash-Konsistente Ersetzung):**
  `write_atomic` erstellt eine temporäre Datei im selben Verzeichnis wie das Ziel, schreibt und flusht diese via `sync_file`, führt ein atomares `rename` aus und synchronisiert abschließend das Elternverzeichnis via `sync_dir`.
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
