# contextra-sys

Systemnahe Bindings, Low-Level I/O und Memory-Mapping.

Normativitätsklasse: **Kern `[K]`** (Ring 0 Unsafe-Insel, `stable`; §6.1, §6.7, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-sys` bietet systemnahes Memory-Mapping (mmap), Prozess-Härtung und plattformspezifische I/O-Primitiven über eine sichere Fassade.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Unsafe-Insel 2 von 3, `#![deny(unsafe_code)]` ausgenommen)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)

## Schnittstellen & Modularität
- Vollständig autarker Baustein ohne interne Workspace-Kopplung (Score: 100/100).
- Kapselt POSIX- und Win32-Systemaufrufe unter strikter Einhaltung von `// SAFETY:`-Invarianten.
