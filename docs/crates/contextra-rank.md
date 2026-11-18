# contextra-rank — Crate-Spezifikation (Fusion & Reranking)

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-rank`  
**Referenz:** `docs/SPEC.md` §2, §5, §6, §13

---

## 1. Zweck & Systemrolle

`contextra-rank` ist die mathematische Fusions- und Reranking-Komponente für das hybride Retrieval (Hybrid-Search) in Contextra:
- **Score-Fusion:** Reciprocal Rank Fusion (RRF), gewichtete lineare Kombination (Weighted Linear Combination) und Normalisierungsalgorithmen (Min-Max, Z-Score).
- **Reranking-Pipelines:** Mehrstufige Verfeinerung von Zwischenergebnissen unter Berücksichtigung von Relevanz-Scores, semantischer Nähe und Graph-Gewichten.
- **Latenz- & Budgetsteuerung:** Einhaltung harter Zeitschranken (Deadlines) während des Rerankings.
- **Invarianz gegenüber gelöschten Daten (Z-E8):** Mathematische Garantie, dass gelöschte oder gesperrte Dokumente die relative Rangfolge und Relevanzbewertung freigegebener Dokumente nicht verzerren.

---

## 2. Architektur & Fusions-Pipelines

```
┌─────────────────────────────────────────────────────────────┐
│                        contextra-rank                       │
│                                                             │
│   ┌──────────────┐     ┌──────────────┐    ┌────────────┐   │
│   │ Vektor-Hits  │     │ Volltext-Hits│    │ Graph-Hits │   │
│   └──────┬───────┘     └──────┬───────┘    └─────┬──────┘   │
│          │                    │                  │          │
│          ▼                    ▼                  ▼          │
│   ┌─────────────────────────────────────────────────────┐   │
│   │               RankFusionEngine                      │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ Score Normalizer   │    │ Reciprocal Rank   │   │   │
│   │   │ (Min-Max / Sigmoid)│    │ Fusion (RRF)      │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ Tie-Breaker        │    │ Resonance Scorer  │   │   │
│   │   │ (DocId aufsteigend)│    │ (Graph Connectivity)      │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └──────────────────────────┬──────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│                     Final Ranked Results                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Fusions-Algorithmen

### 3.1 Reciprocal Rank Fusion (RRF)
RRF kombiniert Ranglisten aus Vektor-, Volltext- und Graph-Suche ohne Abhängigkeit von absoluten Score-Skalen:

$$RRF(d) = \sum_{m \in M} \frac{w_m}{k + r_m(d)}$$

- $M$: Menge der Suchmodalitäten ($Vektor$, $Text$, $Graph$).
- $r_m(d)$: Rang des Dokuments $d$ in Modalität $m$ (1-basiert). Ist $d$ in $m$ nicht enthalten, gilt $r_m(d) = \infty$.
- $k$: Glättungskonstante (Standard: $k = 60$).
- $w_m$: Gewichtungsfaktor der Modalität $m$ ($\sum w_m = 1.0$).

### 3.2 Tie-Breaking & Deterministische Sortierung (INV-5)
Haben zwei Dokumente $d_1$ und $d_2$ identische Fusions-Scores ($RRF(d_1) == RRF(d_2)$), MUSS der Gleichstand deterministisch aufgelöst werden:
- Sortierung strikt nach `DocId` aufsteigend: `d1.doc_id.cmp(&d2.doc_id)`.
- Es darf keine zufällige oder hash-seed-abhängige Reihenfolge entstehen.

---

## 4. Datenschutz-Garantie: Unabhängigkeit der Rangfolge (Z-E8)

Die SPEC §6 und Zusage Z-E8 verlangen:
> „Rangfolge, Zählwerte und relative Bewertungen dürfen sich durch unzugängliche oder gelöschte Dokumente nicht ändern.“

### 4.1 Technische Durchsetzung
1. **Kandidaten-Vorfilterung:** `contextra-rank` verarbeitet **ausschließlich** Trefferkandidaten, die zuvor erfolgreich das `ReadGate` passiert haben. Ein gesperrtes Dokument gelangt niemals in die Fusionspipeline.
2. **Kandidaten-Nachforderung (Auffüllen auf k):** Werden Kandidaten im Vorfeld durch das `ReadGate` verworfen, fordert die Orchestrierungsebene (`contextra-db`) solange weitere Treffer aus den E1-Schichten an, bis die geforderte Anzahl $k$ an gültigen Kandidaten vorliegt, bevor die Normalisierung und RRF-Berechnung einsetzt.

---

## 5. Schnittstellen

```rust
pub struct RankItem {
    pub doc_id: DocId,
    pub score: f32,
    pub original_rank: usize,
}

pub trait RankFusion: Send + Sync {
    fn fuse(
        &self,
        vector_hits: &[RankItem],
        text_hits: &[RankItem],
        graph_hits: &[RankItem],
        k: usize,
    ) -> Result<Vec<RankItem>, ContextraError>;
}
```

---

## 6. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sichere Implementierung.
2. **Reines Berechnungscrate:** Hält keine eigene Persistenz, keine offenen Dateihandles und speichert keine Klartext-Nutzlasten.
3. **Deterministische Gleitkomma-Behandlung:** NaNs oder unendliche Werte in Eingabe-Scores werden abgefangen und abgewiesen (`Err(ContextraError::InvalidInput)`).
