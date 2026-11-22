# contextra-text

Volltextsuche und Inverted-Index (BM25 Engine).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, §10, `capabilities.toml`).

## Rolle in einem Satz
`contextra-text` stellt eine Inverted-Index Volltextsuchmaschine mit BM25 Scoring, Block-Max-WAND und Text-Analyzer-Schnittstellen bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Volltextsuche)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
