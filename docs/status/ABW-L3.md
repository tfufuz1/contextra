# Abweichungsbericht Lane L3 — Crates Ring 0 und Werkzeug

**Datum:** 2026-10-25
**Spezifikation:** Spezifikation 1.2 (§6.1, §6.6, §6.7, §6.9, §6.10, §7–§12, §20, Anhang A)
**Quelle der Wahrheit:** Spezifikation 1.2; Ist-Stand belegt durch `capabilities.toml` und `Cargo.toml`.

---

## 1. Übersicht der Ist-vs.-Soll-Abweichungen in `capabilities.toml` (§6.9)

Gemäß **INV-DOC-SINGLE-SOURCE-1** ist `capabilities.toml` die maßgebliche Konfigurationsdatei. Folgende Abweichungen zwischen der Soll-Spezifikation 1.2 (§6.9 Steckbrief) und `capabilities.toml` wurden identifiziert:

| Crate | Eigenschaft | Soll (Spezifikation 1.2 §6.9) | Ist (`capabilities.toml`) | Anmerkung / Befund |
|---|---|---|---|---|
| `contextra-graph` | Reifegrad | `experimental` (`[R]`) | `stable` | Gemäß §6.9 ist `graph` `[R]` (Rand) mit Zielreifegrad `experimental`. |
| `contextra-graph` | `may_depend_on` | `['contextra-types', 'contextra-ports']` | `['contextra-core']` | `capabilities.toml` führt veraltete `contextra-core` auf. |
| `contextra-rank` | Reifegrad | `experimental` (`[R]`) | `stable` | Gemäß §6.9 ist `rank` `[R]` (Rand) mit Zielreifegrad `experimental`. |
| `contextra-crypto` | `may_depend_on` | `['contextra-types', 'contextra-ports']` | `['contextra-core', 'contextra-ports']` | `capabilities.toml` führt veraltete `contextra-core` auf. |
| `contextra-vector` | `may_depend_on` | `['contextra-types', 'contextra-ports', 'contextra-simd']` | `['contextra-core', 'contextra-crypto', 'contextra-simd', 'contextra-sys']` | `capabilities.toml` führt veraltete `contextra-core`, `crypto`, `sys` auf. |
| `contextra-text` | `may_depend_on` | `['contextra-types', 'contextra-ports']` | `['contextra-core']` | `capabilities.toml` führt veraltete `contextra-core` auf. |
| `contextra-simd` | `may_depend_on` | `['contextra-types']` | `['contextra-core']` | `capabilities.toml` führt veraltete `contextra-core` auf. |
| `contextra-audit-export` | Ring | Ring 4 (`[E]`) | Ring 0 | `audit-export` gehört gemäß §6.9 zu Ring 4 (`[E]`). |
| `contextra-audit-export` | `may_depend_on` | `['contextra-types', 'contextra-crypto', 'contextra-ports']` | `['contextra-types']` | Unvollständige `may_depend_on` in `capabilities.toml`. |
| `contextra-avv-generator` | Ring | Ring 4 (`[E]`) | Ring 0 | `avv-generator` gehört gemäß §6.9 zu Ring 4 (`[E]`). |
| `contextra-avv-generator` | `may_depend_on` | `['contextra-types', 'contextra-crypto', 'contextra-ports']` | `['contextra-types']` | Unvollständige `may_depend_on` in `capabilities.toml`. |
| `contextra-testkit` | `may_depend_on` | `['contextra-types', 'contextra-ports', ...]` | `['contextra-core']` | `capabilities.toml` führt veraltete `contextra-core` auf. |

---

## 2. Ist-vs.-Soll-Abweichungen in Dokumentation & Dokumenten-Metadaten

| Crate | Befund |
|---|---|
| `contextra-rank` | `README.md` fehlte vollständig. |
| `contextra-audit-export` | `README.md` fehlte vollständig. |
| `contextra-avv-generator` | `README.md` fehlte vollständig. |
| `contextra-license` | `README.md` fehlte vollständig. |
| `contextra-crypto` | `AGENTS.md` nannte fälschlicherweise `contextra-privacy` im Titel. |
| Diverse | `AGENTS.md` / `README.md` entsprachen in Struktur und Feldbezeichnungen teilweise nicht dem in Lane L3 geforderten Standard (§6.9, Anhang A). |

---

## 3. Offene Punkte & rechtliche Vorbehalte (§20.13, §34)

- **P-01 bis P-14 (§20.13):** Rechtliche Prüfpunkte zur Löschattestierung und DSGVO-Konformität bleiben explizit offen.
- **F-01 bis F-08 (§34):** Grundsatzentscheidungen zur Architektur und Löschmechanik sind getroffen und werden umgesetzt.
- **INV-CLAIMS-1 (§4.3):** Keine Formulierung wie „100 % gelöscht“, „DSGVO-konform“ oder „garantiert“ in Dokumenten oder Doc-Kommentaren. Termini richten sich strictly nach der Claim-Taxonomie (§20.12).
