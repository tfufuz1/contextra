# contextra-rank — Crate-Spezifikation

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Bindung:** `docs/SPEC.md` §2, §5, §6, §13

---

## 1 Zweck & Domäne

`contextra-rank` ist die Fusions- und Reranking-Komponente für Hybrid-Search:
- Reciprocal Rank Fusion (RRF) und gewichtete Score-Fusion
- Normalisierung und Kalibrierung von Vektor- und Volltext-Scores
- Reranking-Pipelines mit Zeitbudget-Steuerung (Deadlines)

## 2 Neutralität gegenüber gelöschten Daten (Z-E8)

- **Unabhängigkeit der Rangfolge (Z-E8):** Die relative Rangfolge und Score-Zusammensetzung freigegebener Dokumente darf nicht durch gesperrte Dokumente verzerrt werden.
- **Kandidaten-Vorfilterung:** Reranker und Fusion erhalten ausschließlich Kandidaten, die das `ReadGate` passiert haben.
- **Auffüllen auf k:** Werden Kandidaten durch das `ReadGate` abgewiesen, werden aus den vorgeschalteten Suchkanälen weitere Kandidaten nachgezogen, bis das angeforderte Top-k erreicht ist.

## 3 Invarianten

- Deterministische Sortierung: Bei identischem Fusions-Score erfolgt der Tie-Break zwingend nach `DocId` aufsteigend (INV-5).
- Reines Berechnungscrate: Hält keine eigene Persistenz und keine personenbezogenen Daten.
- `#![forbid(unsafe_code)]`.
