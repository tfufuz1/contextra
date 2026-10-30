# contextra-rank

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Modul `dibud` ist deklariert, der Kopfkommentar verweist auf entfernte Kognition. Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen.

Normativitätsklasse: **Rand `[R]`** (Ring 0, `experimental`; §6.1, §6.9, §12, `capabilities.toml`).

## Rolle in einem Satz
`contextra-rank` bietet Score-Kalibrierung (Platt, Isotonisch, Conformal) und Ergebnis-Anreicherungen zur Verfeinerung von Retrieval-Rankings.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Ranking & Kalibrierung)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental` (abweichend in `capabilities.toml` als `stable` geführt, siehe `docs/status/ABW-L3.md`)
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
