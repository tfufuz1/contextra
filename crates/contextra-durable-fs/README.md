# contextra-durable-fs

Crash-sichere Dateisystem-Operationen und VFS-Durability.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-durable-fs` stellt die kanonische Produktion-Implementierung des virtuellen Dateisystem-Ports (`Vfs` aus `contextra-ports`) bereit. Es kapselt plattformspezifische I/O-Aufrufe und garantiert crash-konsistente Schreiboperationen.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Persistenz-Fundament)
- **Status:** 🟢 Autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten:** Nur Low-Level Bindings aus `contextra-sys`

## Kernfunktionen
- **Atomares Schreiben (`write_atomic`):** Sequenz aus temporärer Datei, `sync_all`, atomarem Rename und Directory-Sync.
- **Plattform-Validierung (`probe_platform_durability`):** Automatische Erkennung und Ablehnung ungeeigneter Cloud-Sync- oder Netzwerk-Dateisysteme.
- **Rechte-Erzwingung:** Durchsetzung sicherer POSIX-Dateiberechtigungen (0700 Verzeichnisse, 0600 Dateien).
