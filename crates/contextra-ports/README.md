# contextra-ports

Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, Anhang A, `capabilities.toml`).

## Rolle & Zweck
`contextra-ports` ist das Schnittstellen-Fundament des modularen Baukastensystems. Es definiert alle abstrakten Port-Traits (VFS, Storage, Vektor-Index, Volltext, Bereinigung/Purger), über die die einzelnen Lego-Bausteine miteinander kommunizieren.

Durch die Verwendung von Ports existieren keine direkten Abhängigkeiten zwischen konkreten Speicher- und Index-Engines.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Port-Fundament, deterministisch)
- **Status:** 🟢 Autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten:** Ausschließlich `contextra-types`.

## Wichtige Ports
- **`vfs::Vfs`:** Abstraktion für Dateisystemoperationen und Persistenz.
- **`storage::StorageEngine`:** Einheitliche Schnittstelle für Key-Value- und LSM-Speicher.
- **`erasure::Purger` / `erasure::LayerScanner`:** Neutrale Bereinigungs- und Scan-Ports für den isolierten Löschbaustein `contextra-erasure`.
- **`vector_index::VectorIndex` & `text_index::TextIndex`:** Abstraktionen für Vektor- und Volltextindizes.
