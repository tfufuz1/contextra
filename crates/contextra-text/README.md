# contextra-text

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

Volltextsuche und Inverted-Index (BM25 Engine).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, §10, `capabilities.toml`).

## Rolle in einem Satz
`contextra-text` stellt eine Inverted-Index Volltextsuchmaschine mit BM25 Scoring, Block-Max-WAND und Text-Analyzer-Schnittstellen bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Volltextsuche)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
