# contextra-graph — Crate-Spezifikation

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Bindung:** `docs/SPEC.md` §2, §3, §5, §6, §7

---

## 1 Zweck & Domäne

`contextra-graph` implementiert den Wissensgraphen für GraphRAG:
- Speicherung und Traversierung von Entitäten, Beziehungen und Hyperedges
- Zeitbehaftete und gewichtete Kanten
- Pfad- und Nachbarschaftsextraktion zur Kontextanreicherung für LLMs

## 2 Datenklasse & Kapselung von Personendaten

- **Klasse D (Ableitung) & Klasse P (Beziehungen):**
  - Graphknoten und Kanten, die natürliche Personen repräsentieren, MÜSSEN an `SubjectRef` gebunden sein.
  - Niemals Klartext-Namen als primäre Knoten-Keys verwenden.
  - Trennung: Fachliche Wissensbeziehungen (K) referenzieren Personen über `PersonalRef`.

## 3 Löschkette & ErasureLayer-Implementierung

`contextra-graph` implementiert `ErasureLayer` (Phase 4f):

1. **`resolve(target)`:** Findet alle Knoten, Kanten und Adjazenzlisteneinträge, die mit dem `EraseTarget` verknüpft sind.
2. **`on_gate(scope)`:** Sperrt die Knoten im InMemory-Graph-Cache. Traversierungsabfragen ignorieren gesperrte Entitäten ab sofort.
3. **`purge(scope, cx)`:**
   - Physisches Entfernen der Knoten und inzidenten Kanten aus den Graph-Segmenten.
   - Bereinigung von Hyperedges und Re-Balancing der Adjazenzstrukturen.
   - Persistierung und fsync der bereinigten Graph-Tabellen.
4. **`witness(scope, scanner)`:**
   - Scannt die Graph-Struktur nach verbliebenen Knoten-IDs oder verwaisten Kanten.
   - Bestätigt `residue_matches == 0`.

## 4 Lesepfad

- Jede Graph-Traversierung prüft an jedem Schritt gegen `ReadGate`.
- Kanten zu gesperrten Knoten werden unterdrückt (verhält sich wie ein nicht-existenter Knoten).

## 5 Invarianten

- `#![forbid(unsafe_code)]`.
- Zyklenfreie oder deterministisch terminierende Traversierung.
