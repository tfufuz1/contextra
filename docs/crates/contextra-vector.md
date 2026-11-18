# contextra-vector — Crate-Spezifikation

**Ebene:** E1b (Kern-Closure: `ids`, `env`, `simd`, `durable-fs`)  
**Bindung:** `docs/SPEC.md` §2, §3, §5, §6, §7

---

## 1 Zweck & Domäne

`contextra-vector` ist die Vektor-Index-Engine für dichte semantische Ähnlichkeitssuche:
- Hierarchical Navigable Small World (HNSW) Index im Arbeitsspeicher und auf SSD
- Distanzmaße: Cosine, L2, Dot-Product (beschleunigt über `contextra-simd`)
- Skalare Quantisierung (SQ8, SQ4) für speichereffizienten Betrieb

## 2 Datenklasse & Embedding-Inversion-Schutz

- **Klasse D (Ableitung):** Vektoren und Embeddings sind abgeleitete Daten. Da Vektoren durch Inversionsangriffe Quelltext rekonstruierbar machen können, gelten sie als schutzbedürftig.
- **Kapselung:** Vektor-IDs bilden eine 1:1 Zuordnung zu `DocId` oder `PersonalRef`.

## 3 Löschkette & ErasureLayer-Implementierung

`contextra-vector` implementiert `ErasureLayer` (Priorität 4e — Vektor-Reparatur):

1. **`resolve(target)`:** Findet die internen Vektor-IDs und HNSW-Knoten, die zu den Ziel-Keys gehören.
2. **`on_gate(scope)`:** Markiert die Ziel-Vektoren in einer InMemory-Surrogat-Ausschlussmaske (`RoaringBitmap`). Künftige kNN-Suchen überspringen diese Knoten mit O(1) Overhead.
3. **`purge(scope, cx)`:**
   - **Graph-Reparatur:** Entfernt die Knoten physisch aus den HNSW-Schichten.
   - Verbindet benachbarte Knoten neu, um die Konnektivität des Small-World-Graphen zu erhalten.
   - Überschreibt die Vektordaten-Slices in den persistenten Indexdateien mit Nullen und synchronisiert via `durable-fs`.
4. **`witness(scope, scanner)`:**
   - Führt eine gezielte Vektorabfrage auf die gelöschten Vektor-IDs durch und scannt die Vektordatei.
   - Bestätigt `residue_matches == 0`.

## 4 Lesepfad & ReadGate

- Vektorsuchen filtern Kandidaten vor oder während des HNSW-Graph-Traversal gegen `ReadGate`.
- Gelöschte Vektoren fließen nicht in die Top-k Liste ein.

## 5 Invarianten

- `#![forbid(unsafe_code)]` (SIMD ist in `contextra-simd` gekapselt).
- Deterministische Suchergebnisse bei identischem Graphen und Suchparameter.
