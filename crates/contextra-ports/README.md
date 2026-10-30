# contextra-ports

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Mehrere Ports (`graph`, `attention`, `plugin`, `license`, `checkpoint`, `scanner`) bedienen nur entfernte Crates (SPEC §6). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, Anhang A, `capabilities.toml`).

## Rolle in einem Satz
`contextra-ports` definiert sämtliche abstrakten Port-Traits und Schnittstellenverträge zur sauberen Entkopplung von Kern und Peripherie ohne eigene Implementierungen.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Port-Fundament, deterministisch)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)
