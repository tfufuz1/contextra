# contextra-durable-fs

Crash-sichere Dateisystem-Operationen und `RealVfs`-Implementierung.

- **Ring:** Ring 1 (Dateisystem-Durability)
- **Normativ:** [docs/SPEC.md §8.5](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-durable-fs` stellt die einzige produktive Implementierung des `Vfs`-Ports (`RealVfs`) bereit:
- `real_vfs`: Konsolidierter VFS-Treiber für POSIX-/macOS-Dateisysteme.
- `atomic`: Atomares Ersetzen (Temp-Datei → `fsync` Datei → `rename` → `sync_dir`).
- `scrub`: Best-effort physisches Überschreiben mit Nullen (`ZeroFill`) vor dem Löschen.
- `probe`: Startup-Probe für Dateisystemberechtigungen und Locks.

## Invarianten
- `#![forbid(unsafe_code)]`
- Sämtliche `sync_*`-Fehler werden verbindlich propagiert (`let _ =` ist verboten).
- Verzeichnisse erzwingen Modus `0700`, Dateien erzwingen Modus `0600`.
