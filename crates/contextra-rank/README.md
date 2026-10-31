# contextra-rank

Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen.

Normativitätsklasse: **Rand `[R]`** (Ring 0, `experimental`; §6.1, §6.9, §12, `capabilities.toml`).

## Rolle in einem Satz
`contextra-rank` bietet Score-Kalibrierung (Platt, Isotonisch, Conformal) und Ergebnis-Anreicherungen zur Verfeinerung von Retrieval-Rankings.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Ranking & Kalibrierung)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental` (abweichend in `capabilities.toml` als `stable` geführt, siehe `docs/status/ABW-L3.md`)
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
