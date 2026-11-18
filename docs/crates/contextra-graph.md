# contextra-graph — Crate-Spezifikation (GraphRAG-Engine)

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-graph`  
**Referenz:** `docs/SPEC.md` §2, §3, §5, §6, §7

---

## 1. Zweck & Systemrolle

`contextra-graph` implementiert den semantischen Wissensgraphen für GraphRAG in Contextra:
- **Entitäten & Relationen:** Speicherung von typisierten Entitäten (Knoten), gerichteten/gewichteten Beziehungen (Kanten) und n-stelligen Hyperedges.
- **Kontextuelle Pfadsuche:** Traversierung von Wissenspfaden zur Anreicherung von Retrieval-Kontexten für LLMs.
- **Kapselung von Personendaten:** Strikte Bindung von personenbezogenen Knoten an `SubjectRef` und Anonymisierung der Graph-Struktur.
- **Physische Löschung:** Kaskadiertes Entfernen verwaister Kanten und Entitäten im Rahmen der DSGVO-Löschkette.

---

## 2. Architektur & Datenmodell

```
┌─────────────────────────────────────────────────────────────┐
│                       contextra-graph                       │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                    PropertyGraph                    │   │
│   │                                                     │   │
│   │     (Entity: Vertrag) ────[regelt]───► (Entity: PII)│   │
│   │            │                                  │     │   │
│   │            │                                  │     │   │
│   │            ▼                                  ▼     │   │
│   │       [Klasse K]                         [Klasse P] │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ AdjacencyIndex     │    │ HyperedgeTable    │   │   │
│   │   │ (Forward / Reverse)│    │ (N-ary Relations) │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ GraphTraversal     │    │ ErasureLayer Impl │   │   │
│   │   │ (BFS / DFS / A*)   │    │ (Entity-Purge)    │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Datenklassifikation & PII-Kapselung (SPEC §3)

1. **Keine personenbezogenen Klartext-Knoten-IDs:** 
   - Ein Knoten, der eine natürliche Person repräsentiert, MUSS als ID eine `SubjectRef` tragen (128-Bit-Zufall).
   - Attribute mit Personenbezug (Name, Adresse) dürfen nicht im Graphen im Klartext abgelegt werden, sondern verweisen verschlüsselt auf Klasse P im Store.
2. **Klasse K vs. Klasse P Trennung:**
   - Kanten zwischen Unternehmenseinheiten (K) und Personen (P) referenzieren Personen über `PersonalRef`.
   - Wird der Betroffene gelöscht, wird die Kante zur toten Referenz und in Phase 4f bereinigt.

---

## 4. DSGVO-Löschkette: ErasureLayer-Implementierung

`contextra-graph` implementiert `contextra_ports::erasure::ErasureLayer` (Priorität 4f — Graph-Bereinigung):

```rust
impl ErasureLayer for PropertyGraph {
    fn id(&self) -> LayerId {
        LayerId(*b"CTX_GRAPH_RAG_01")
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }

    fn resolve(&self, target: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;
    fn on_gate(&self, scope: &LayerScope) -> BoxFuture<'_, Result<(), String>>;
    fn purge(&self, scope: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>;
    fn witness(&self, scope: &LayerScope, sc: &Scanner<Armed>) -> BoxFuture<'_, Result<LayerWitness, String>>;
}
```

### 4.1 Die 4 Phasen in `contextra-graph`

#### 1. `resolve(target)`
- Identifiziert alle Entitäten-IDs und Kanten-IDs, die zu dem `EraseTarget` gehören.
- Ermittelt Hyperedges, an denen die gelöschten Entitäten beteiligt sind.

#### 2. `on_gate(scope)`
- Markiert die Entitäten-IDs in der InMemory-Gate-Map (`gated_entities`).
- Ab diesem Moment brechen Pfad-Traversierungen an diesen Knoten sofort ab.

#### 3. `purge(scope, cx)` — Physische Graph-Bereinigung
Unter exklusivem Halt der `PurgeBarrier`:
- **Knoten-Löschung:** Entfernen der Knoten aus der Knotentabelle.
- **Kanten-Bereinigung:** Alle einfallenden und ausgehenden Kanten der gelöschten Entitäten werden aus den Adjazenzlisten physisch gelöscht.
- **Hyperedge-Reparatur:** Werden durch das Löschen einer Entität Hyperedges ungültig oder unter die Mindestanzahl an Teilnehmern reduziert, werden diese Hyperedges aufgelöst.
- **Persistierung:** Schreiben der bereinigten Graph-Tabellen auf Platte mit anschließendem `fsync` (`contextra-durable-fs`).

#### 4. `witness(scope, sc)` — Verifikation & Messung
- Scannt Adjazenzlisten und Knotentabellen auf verbleibende Verweise auf die Ziel-IDs.
- Bestätigt `residue_matches == 0`.

---

## 5. Synchroner Lesepfad: ReadGate-Integration

```rust
pub fn traverse_neighbors(&self, from_entity: &EntityId) -> Result<Vec<EntityEdge>, ContextraError> {
    // 1. Prüfe Startknoten gegen ReadGate
    if self.read_gate.check_entity(from_entity) == GateDecision::Erased {
        return Ok(Vec::new()); // Behandle als nicht-existent (Fail-Closed)
    }

    // 2. Iteriere über Nachbarn
    let mut visible_edges = Vec::new();
    for edge in self.adjacency.get_edges(from_entity) {
        if self.read_gate.check_entity(&edge.target) == GateDecision::Open {
            visible_edges.push(edge);
        }
    }
    Ok(visible_edges)
}
```

---

## 6. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig speichersicher.
2. **Keine Zyklen in Traversierungen:** Traversierungsalgorithmen implementieren strikte Tiefen- und Schlingen-Begrenzungen.
3. **Keine Datenlecks:** Verwaiste Kanten dürfen niemals zu toten Entitäten führen.
