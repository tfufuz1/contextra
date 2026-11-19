> **Löschpfad v3.0:** Die löschrelevanten Abschnitte dieser Spezifikation sind mit Stand 19.11.2026 durch `docs/SPEC.md` v3.0 ersetzt bzw. präzisiert (§5, §6, §7, §9, §12, §13, §14, §16 der SPEC). Widerspricht ein Abschnitt unten der SPEC, gilt die SPEC. Abschnitte, die so markiert sind, sind überholt. Weitere Verweise: §23 SPEC (Ersetzungstabelle).

# contextra-graph — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env`, `durable-fs` (E0-Fundamente, max. 5)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-graph`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3, §5.3 (Schritt 4f), §6, §7; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-graph` implementiert die hochskalierbare GraphRAG-Engine (CSR — Compressed Sparse Row) für Contextra:
- **Wissensgraph & CSR-Speicher:** Knoten, typisierte Kanten und n-stellige Hyperedges zur semantischen Modellierung komplexer Unternehmens- und Fachbeziehungen.
- **Path-RAG & Personalisierter PageRank (PPR):** Graph-Traversierung und Relevanz-Diffusion zur Kontextanreicherung für LLM-Prompts.
- **PII-Schutz (SPEC §3):** Personenbezogene Entitäten werden zwingend über `SubjectRef` adressiert; Kanten zu Unternehmenseinheiten nutzen `PersonalRef`.
- **Graph-Bereinigung (DSGVO Phase 4f):** Physisches Entfernen von Entitäten, inzidenten Kanten und Reparatur betroffener Hyperedges.

### Beseitigung von Ebenenverstößen & Altlasten (R1, R3)
1. **Keine E2-Abhängigkeiten:** Die optionalen Abhängigkeiten zu `contextra-types` und `contextra-ports` sowie die Features `workspace`, `types-compat` und `ports-compat` werden vollständig entfernt.
2. **Leere Default-Features (R3):** `default = []` (alle Funktionserweiterungen wie `session-dag`, `entity-extraction`, `hyperedge-suggest` werden opt-in).
3. **Altlasten-Bereinigung (`AP-21`):** In `src/hyperedge.rs` (Z. 310–313) und `src/percolation.rs` (Z. 19–22) werden alle Verweise auf das entfernte `contextra-cognition` gelöscht.

### Architekturgarantien (R1–R6, INV-1, INV-3, INV-4, INV-5)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Keine Panics; alle Operationen liefern `GraphError`.
3. **Determinismus (INV-5):** Graph-Traversierungen liefern bei identischem Stand und Seeds stets dieselbe Knotenreihenfolge.
4. **Keine PII im Klartext:** Entitätsknoten speichern keine Klartext-PII im Graphen.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────┐
│                       contextra-graph                       │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                    PropertyGraph                    │   │
│   │                                                     │   │
│   │     (Entity: Vertrag) ────[regelt]───► (Entity: PII)│   │
│   │            │                                  │     │   │
│   │            ▼                                  ▼     │   │
│   │       [Klasse K]                         [Klasse P] │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ CsrAdjacencyIndex  │    │ HyperedgeTable    │   │   │
│   │   │ (Forward / Reverse)│    │ (N-ary Relations) │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ GraphTraversal     │    │ GraphPurger       │   │   │
│   │   │ (PPR / Path-RAG)   │    │ (DSGVO Phase 4f)  │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. DSGVO-Löschkette: Graph-Bereinigung (Phase 4f)

```rust
use contextra_ids::{EntityId, SubjectRef};
use std::path::Path;

pub struct GraphPurgeOutcome {
    pub entities_deleted: u64,
    pub edges_removed: u64,
    pub hyperedges_repaired: u64,
}

pub struct GraphPurger;

impl GraphPurger {
    /// Führt die physische Bereinigung des CSR-Graphen aus (Phase 4f).
    ///
    /// 1. Entfernt Knoten aus der Knotentabelle.
    /// 2. Bereinigt alle inzidenten ein- und ausgehenden Kanten in den CSR-Adjazenzlisten.
    /// 3. Löst unterbesetzte Hyperedges auf.
    /// 4. Schreibt die CSR-Strukturen durabel auf Platte (contextra-durable-fs).
    pub fn purge_entities(
        graph_dir: &Path,
        entities: &[EntityId],
    ) -> Result<GraphPurgeOutcome, GraphError> {
        // CSR In-Memory Mutationen, gefolgt von atomarem File-Sync
        Ok(GraphPurgeOutcome {
            entities_deleted: entities.len() as u64,
            edges_removed: 0,
            hyperedges_repaired: 0,
        })
    }
}
```

---

## 4. Fehlerbehandlung (`GraphError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Entity not found: {0}")]
    EntityNotFound(u64),

    #[error("Hyperedge invalid: requires at least {expected} nodes, got {actual}")]
    HyperedgeUnderflow { expected: usize, actual: usize },

    #[error("Graph storage corrupted: {0}")]
    CorruptedStorage(String),

    #[error("Traversal deadline exceeded")]
    Timeout,

    #[error("I/O error during graph persistence: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## 5. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-graph"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "CSR-Graph for entity-relation traversal, Hyperedges, and GraphRAG algorithms"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-durable-fs = { workspace = true }
bytes = { workspace = true }
parking_lot = { workspace = true }
ahash = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"
criterion = { workspace = true }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
session-dag = []
entity-extraction = []
hyperedge-suggest = []
```

---

## 6. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports`.
   - Features `workspace`, `types-compat`, `ports-compat` entfernen.
   - `default = []` setzen (R3).
2. **Altlasten-Bereinigung (`AP-21`):**
   - In `src/hyperedge.rs` und `src/percolation.rs` Bezüge zu `contextra-cognition` entfernen.
3. **Phase 4f Graph-Purge:**
   - Kaskadierte Kanten- und Hyperedge-Bereinigung absichern.
4. **Verifikation:**
   - `cargo test -p contextra-graph`
