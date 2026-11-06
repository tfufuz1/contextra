# contextra-text

Volltextsuche und Inverted-Index (BM25 Engine).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, §10, `capabilities.toml`).

## Rolle & Zweck
`contextra-text` ist der autarke Volltext-Baustein des Contextra-Ökosystems. Er implementiert eine performante Inverted-Index-Engine mit BM25-Scoring, Block-Max WAND dynamischem Pruning und konfigurierbarer Textanalyse.

Der Crate kann unabhängig von anderen Bausteinen als eigenständige Textsuchmaschine verwendet werden.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Volltextsuche)
- **Status:** 🟡 In Migration (Bereinigung von Löschketten-Relikten gemäß `PLAN.md` Phase 4)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Erlaubte Abhängigkeiten:** Ausschließlich `contextra-types` und `contextra-ports`.

## Deprecated Debt & Entkopplung
- **Löschketten-Relikte:** Verbliebene Deletion-Proof- und DSGVO-Referenzen gelten als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)).
- **Zielarchitektur:** Bereinigungs- und Scanning-Operationen erfolgen rein über generische Ports (`LayerScanner`). Die übergeordnete Nachweisführung liegt in [`contextra-erasure`](../contextra-erasure/).
