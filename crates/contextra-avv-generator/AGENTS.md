# AGENTS.md — `contextra-avv-generator`

## 1. Rolle
`contextra-avv-generator` generiert strukturierte AVV-Dokumentenvorlagen (Art. 28 DSGVO) basierend auf aktiven System- und Verschlüsselungskonfigurationen.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Erweiterung)
- **Klasse:** Erweiterung `[E]`
- **Reifegrad:** `experimental`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-crypto`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33)
- **REQ-AVV-001:** Vollständige Deklaration aller Submodule (`assistance`, `audit`, `confidentiality`, `data_subject_rights`, `error`, `instructions`, `template`, `termination`).
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, Template Unit-Tests.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]`).
- Keine unvollständigen rechtlichen Garantieversprechungen ohne Disclaimer.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Konsumiert: `contextra-types` und Re-Exports in `contextra-privacy`.

## 6. Tests & Gates
- **Template Generation Tests:** Validierung der generierten AVV-Passagen.

## 7. Offene Punkte
- **P-01 bis P-14 (§20.13):** Offene juristische Prüfpunkte bezüglich AVV-Textinhalten.
