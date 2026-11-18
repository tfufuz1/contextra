# contextra-rank — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env` (E0-Fundamente, max. 5)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-rank`  
**Referenzdokumente:** `docs/SPEC.md` §2, §6, §13 (INV-4, INV-5); `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-rank` ist die mathematische Fusions-, Kalibrierungs- und Reranking-Komponente für das hybride Retrieval (Hybrid-Search) in Contextra:
- **Score-Fusion:** Reciprocal Rank Fusion (RRF) und gewichtete Signalkombination über Vektor-, Volltext- und Graph-Treffer.
- **Deterministisches Tie-Breaking (INV-5):** Strikte Auflösung von Score-Gleichständen nach `DocId` aufsteigend. Bitidentische Ergebnisse bei gleichen Eingaben.
- **Score-Kalibrierung:** Isotonische Regression und Platt-Scaling zur Umrechnung heterogener Distanzmetriken in kalibrierte Wahrscheinlichkeiten.
- **Datenschutz-Invarianz (Z-E8):** Mathematische Garantie, dass gelöschte oder gesperrte Dokumente weder Rangfolge noch relative Relevanzbewertung freigegebener Dokumente verzerren.

### Beseitigung von Ebenenverstößen & Altlasten (R1, R3)
1. **Keine E2-Abhängigkeiten:** Die optionalen Abhängigkeiten zu `contextra-types` und `contextra-ports` sowie die Features `workspace`, `types-compat` und `ports-compat` werden vollständig entfernt.
2. **Kanonische Fundament-Einbindung:** `contextra-rank` bindet direkt `contextra-ids` (für `DocId`) und `contextra-env` (für `Clock`) ein.
3. **Leere Default-Features (R3):** `default = []` (alle experimentellen Features wie `exp-dibud`, `exp-resonance` werden opt-in).
4. **Altlasten-Bereinigung (`AP-21`):** In `src/lib.rs` Zeilen 27-28 werden alle Referenzen auf `infer-candle`, `infer-ollama`, `infer-onnx` und `py` entfernt.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-5)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Keine Panics; alle Fehlerpfade liefern `RankError`.
3. **INV-5 Determinismus:** Identische Trefferlisten führen stets zu exakt derselben Reihenfolge.
4. **Z-E8 Vorfilterung:** Die Fusions-Engine akzeptiert ausschließlich Kandidaten, die das `ReadGate` bereits erfolgreich passiert haben.

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
│   │                  RankFusionEngine                   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ Score Normalizer   │    │ Reciprocal Rank   │   │   │
│   │   │ (Min-Max / Sigmoid)│    │ Fusion (RRF)      │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ Deterministic      │    │ Isotonic / Platt  │   │   │
│   │   │ Tie-Breaker (INV-5)│    │ Calibrator        │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └──────────────────────────┬──────────────────────────┘   │
│                              │                              │
│                              ▼                              │
│                     Final Ranked Results                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Algorithmen & Kanonische Typen

### 3.1 `RankItem` & Treffermodell

```rust
use contextra_ids::DocId;

#[derive(Debug, Clone, PartialEq)]
pub struct RankItem {
    pub doc_id: DocId,
    pub score: f32,
    pub original_rank: usize,
}

impl Eq for RankItem {}

impl PartialOrd for RankItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RankItem {
    /// Deterministische Sortierung (INV-5):
    /// 1. Höherer Score gewinnt.
    /// 2. Bei absolutem Score-Gleichstand: Kleinere DocId gewinnt (Tie-Break).
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.score.partial_cmp(&self.score) {
            Some(std::cmp::Ordering::Equal) | None => self.doc_id.cmp(&other.doc_id),
            Some(ord) => ord,
        }
    }
}
```

---

### 3.2 Reciprocal Rank Fusion (RRF)

$$RRF(d) = \sum_{m \in M} \frac{w_m}{k + r_m(d)}$$

```rust
use ahash::AHashMap;
use contextra_ids::DocId;

pub struct RrfConfig {
    pub k: f32, // Glättungsfaktor, Standard: 60.0
    pub vector_weight: f32,
    pub text_weight: f32,
    pub graph_weight: f32,
}

impl Default for RrfConfig {
    fn default() -> Self {
        Self {
            k: 60.0,
            vector_weight: 1.0,
            text_weight: 1.0,
            graph_weight: 0.5,
        }
    }
}

pub fn reciprocal_rank_fusion(
    vector_hits: &[DocId],
    text_hits: &[DocId],
    graph_hits: &[DocId],
    config: &RrfConfig,
    limit: usize,
) -> Vec<RankItem> {
    let mut scores: AHashMap<DocId, f32> = AHashMap::new();

    let mut add_modal_ranks = |hits: &[DocId], weight: f32| {
        for (rank_0, &doc_id) in hits.iter().enumerate() {
            let r = (rank_0 + 1) as f32;
            let contribution = weight / (config.k + r);
            *scores.entry(doc_id).or_insert(0.0) += contribution;
        }
    };

    add_modal_ranks(vector_hits, config.vector_weight);
    add_modal_ranks(text_hits, config.text_weight);
    add_modal_ranks(graph_hits, config.graph_weight);

    let mut items: Vec<RankItem> = scores
        .into_iter()
        .map(|(doc_id, score)| RankItem {
            doc_id,
            score,
            original_rank: 0,
        })
        .collect();

    // Deterministische Sortierung gemäß INV-5
    items.sort();
    items.truncate(limit);

    for (idx, item) in items.iter_mut().enumerate() {
        item.original_rank = idx + 1;
    }

    items
}
```

---

## 4. Fehlerbehandlung (`RankError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum RankError {
    #[error("Score value is invalid (NaN or Infinity)")]
    InvalidScore,

    #[error("Candidate pool empty: no valid items to rank")]
    EmptyCandidatePool,

    #[error("Calibration error: {0}")]
    CalibrationFailed(String),

    #[error("Ranking deadline exceeded (budget exhaustion)")]
    BudgetExceeded,
}
```

---

## 5. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-rank"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Deterministic score fusion (RRF), calibration, and reranking for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
ahash = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
proptest = "1.4"
criterion = { workspace = true }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
dibud = []
resonance = []
```

---

## 6. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports`.
   - `contextra-ids` und `contextra-env` zu `[dependencies]` hinzufügen.
   - Features `workspace`, `types-compat`, `ports-compat` entfernen.
   - Default-Features leeren (`default = []`, R3).
2. **Altlasten-Bereinigung (`AP-21`):**
   - In `src/lib.rs` Zeilen 27-28 Referenzen auf `infer-candle`, `infer-ollama`, `infer-onnx`, `py` entfernen.
3. **INV-5 Determinismus-Tests:**
   - Proptest schreiben, der verifiziert, dass Permutationen von Eingabelisten mit gleichen Scores bitidentische `RankItem`-Listen mit aufsteigender `DocId` liefern.
4. **Verifikation:**
   - `cargo test -p contextra-rank`
