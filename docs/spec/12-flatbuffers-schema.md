---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "12"
---
## 12. FlatBuffers-Schema (vollständig, `schemas/contextra.fbs`)

> **Namenshinweis (Fassung 4, verifiziert A4.2):** Das reale Schema im Repository benennt die Tabelle schlicht
> `HyperEdge` (Namespace `Contextra.IPC`), nicht `HyperEdgeFb`. Die Bezeichnung `HyperEdgeFb` in diesem
> Abschnitt ist als **logischer** Name für die IPC-Repräsentation zu verstehen (in Abgrenzung zum
> In-Memory-Typ `HyperEdge` aus §6.4); bei der tatsächlichen Schema-Pflege in `schemas/contextra.fbs` gilt der
> im Repo vorhandene Tabellenname. Diese Diskrepanz existierte bereits vor Fassung 4 und wird hier nur
> dokumentiert, nicht aufgelöst — eine Umbenennung im Schema selbst wäre eine eigene, vom H4-Gate erfasste
> Breaking-Change-Entscheidung und liegt außerhalb des Auftrags dieser Fassung.

> **Erweiterung `child_edge_ids` (Fassung 4, §21.4, Voraussetzung für LeanRAG/H5, siehe A4.4.1):** Ohne diese
> rekursive Referenz kann `cascade_invalidate_hyperedges_for_superseded_doc` (§6.6 H5) einen durch LeanRAG
> abstrahierten Super-Knoten nicht bis zu den Original-Tripeln durchqueren — ein Löschauftrag (Art.-17-DSGVO,
> §10) für ein Quelldokument würde dann bei einer bereits konsolidierten Hyperkante silently fehlschlagen.
> **Diese Erweiterung MUSS durch das CI-Drift-Gate (H4, unten) laufen und grün sein, bevor LeanRAG (§21.4)
> produktiv aktiviert wird — unabhängig davon, ob LeanRAG selbst schon gemerged ist.**

```fbs
namespace contextra.ipc;

table RoleBindingFb {
  role: uint32;
  entity: uint64;
}

table HyperEdgeFb {
  id: uint64;
  predicate_tag: uint32;          // EdgeType-Diskriminante
  participants: [RoleBindingFb];  // min. 2, validiert applikationsseitig
  weight: float32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;            // 0 = None (Sentinel, dokumentiert)
  business_valid_from: int64;
  business_valid_to: int64;       // i64::MIN = None (Sentinel)
  source_doc_id: uint64;          // oder uint128-Encoding bei docid-128
  child_edge_ids: [uint64];       // NEU (Fassung 4, §21.4): IDs der subsumierten Original-/Sub-Hyperkanten
                                   // einer LeanRAG-Super-Hyperkante. Leer/fehlend = Blatt-Hyperkante (kein
                                   // LeanRAG-Abstraktionsprodukt). Rekursiv: ein Kind kann selbst wieder
                                   // Kinder tragen, falls mehrstufige Aggregation je entschieden wird
                                   // (aktuell nicht spezifiziert, §21.4 sieht nur eine Aggregationsstufe vor).
}

table EdgeFb {
  target: uint64;
  weight: float32;
  edge_type_tag: uint32;
  tx_valid_from: uint64;
  tx_valid_to: uint64;
  business_valid_from: int64;
  business_valid_to: int64;
  source_doc_id: uint64;
}

root_type HyperEdgeFb;
```

**CI-Drift-Gate (`xtask check-flatbuffers-drift`):** Vergleicht Hash des generierten Codes gegen committeten
Referenz-Hash. Jede Schema-Änderung ohne begleitende Regenerierung schlägt den Merge-Gate-Job fehl. **Dieses
Gate MUSS grün sein, bevor `HyperEdgeFb` gemerged wird (H4), und erneut grün sein, bevor die
`child_edge_ids`-Erweiterung gemerged wird (AK-19, §16.2).**

---

<a id="13-fehler"></a>
