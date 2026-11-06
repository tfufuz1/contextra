# contextra-avv-generator

Auftragsverarbeitungsvertrag (AVV) Template Generator.

Normativitätsklasse: **Erweiterung `[E]`** (Ring 4, `experimental`; §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-avv-generator` ist eine autarke Template-Rendering-Engine zur automatisierten Generierung strukturierter AVV-Dokumentenvorlagen (Art. 28 DSGVO) basierend auf aktiven System- und Sicherheitskonfigurationen.

## Modulare Architektur & Entkopplung
- **Reine Rendering-Engine:** Das Crate führt keine eigene Kryptografie-, Löschketten-, Saga- oder physische Löschlogik aus. Physische Datenbereinigung und Nachweiserstellung verbleiben strikt in den zuständigen Kern- bzw. Löschmodulen (`contextra-erasure`, `contextra-crypto`).
- **Rechtsprüfungsvorbehalt:** Sämtliche generierten Dokumente enthalten verbindlich einen Disclaimer (`> **Hinweis:** Entwurf — ersetzt keine Rechtsberatung.`).
- **Speichersicherheit & Zero-Panic:** Die Crate erzwingt `#![forbid(unsafe_code)]` und verzichtet im gesamten Produktionscode auf `.unwrap()`, `.expect()` und Panics.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 4 (Erweiterung)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `experimental`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-crypto`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
