---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "18"
---
## 18. Gesamtroadmap

### Stufe 0 — Unmittelbar

1. **Opus-Optimierungen Stufe 0** (§17): WAL-Replay-Panic, Bandit-Dimensionsprüfung, Drift-Kopplung, Egress-Klassifizierung, Intent-Recovery.
2. Loom-Test für Group-Commit sichtbar grün in CI (reine Verifikationslücke).
3. Benchmark-Ausführung des Bandit-Latency-Gates mit produktivem $d$.

### Stufe 1 — Strukturell

4. **Opus-Optimierungen Stufe 1** (§17): HNSW-Hot-Path, SSTable, AES, MemTable, Block-Cache, Provenance, Text-Index, Fusion.
5. **N-äre Hyperkanten** (§6), vollständig spezifiziert, Reihenfolge: H4-Nachweis → Datenmodell (§6.4) → H2 → H1 → H3 → H5 → H6 → Stern-Expansion.
6. HNSW-Dateiformat v2 (Arena + CSR + allokationsfreie Traversierung).

### Stufe 1½ — SOTA-Algorithmen (Fassung 4, §21, §17 SOTA-Roadmap, Priorisierung Teil A4.5)

6a. **TL-HFD** (S.2) im `ShadowMode` neben `forward_push_ppr` (§21.1) — Default-Umstellung erst nach
    Log-Auswertung, siehe Teil A4.7.
6b. **`child_edge_ids`-Schema-Erweiterung** (§12) + rekursive H5-Cascade-Traversierung (§6.6 H5, AK-19) —
    **Pflicht-Vorarbeit** für 6c, unabhängig davon vorziehbar.
6c. **LeanRAG Semantic Aggregation** (S.1) als dritte Stufe der bestehenden Konsolidierungs-Pipeline
    (§21.4) — ersetzt und zieht Punkt 14 aus der bisherigen Stufe 4 (unten) nach vorn; Voraussetzung: 6b
    abgeschlossen, Namensraumbereinigung `ConsolidationConfig`/`SynthesisPhaseResult` (Teil A4.6 Punkt 3)
    erledigt.
6d. **DiBud-Vorarbeit** (S.3, Schritt 1): Streaming-`Iterator<Item = DocId>`-Grenzen in `contextra-vector`
    und `contextra-text` (AK-17) — eigenständig wertvoll unabhängig vom weiteren DiBud-Fortschritt, da sie
    auch anderen Lazy-Loading-Anwendungsfällen nutzt.
6e. **DiBud-Algorithmus** (S.3, Schritt 2): erst nach 6d und nach unabhängigem Benchmark gegen das Paper
    (Teil A4.7).
6f. **Flow-Corrected Thompson Sampling** (S.4) als neue `BanditImplementation`-Variante (§21.3) — niedrigste
    Dringlichkeit dieser Stufe, aber ohne strukturelle Abhängigkeit von 6a–6e und daher parallelisierbar.

### Stufe 2 — Speicher, Struktur und Produktions-Default-Entscheidungen

7. **Opus-Optimierungen Stufe 2** (§17): Graph-Kompaktierung, CSR-Sentinel, Checkpoint, Manifest, contextra-py.
8. Bandit-Default `DiagonalApproximation` → `ShermanMorrison` (⚖️ sobald Gate besteht).
9. Block-Cache-Default LRU → SIEVE (⚖️ sobald entschieden).
10. RaBitQ-/PQ-Evaluierung (nach HNSW v2).

### Stufe 3 — Governance und Produktentscheidungen

11. **Opus-Optimierungen Stufe 3** (§17): Feature-Powerset CI, Panic-Inventar.
12. Formale ADR-Revision der Leiden-Umstellung.
13. Major-Release-Planung: DocId-128-Cutover mit DiskANN-Tier-Vollfreigabe bündeln.

### Stufe 4 — Fernziele

14. ~~Memory Consolidation (`consolidate_via_llm()`)~~ — **vorgezogen nach Stufe 1½, Punkt 6c** (Fassung 4,
    Teil A4.5 Rang 1: höchstes Beleg-Tier, größter Teil der Infrastruktur bereits vorhanden). An dieser Stelle
    verbleibt nur noch eine mögliche **vierte** Pipeline-Stufe (mehrstufige rekursive Aggregation über
    Super-Hyperkanten hinweg), die §21.4 bewusst nicht spezifiziert, da sie ohne produktive Erfahrung mit der
    dritten Stufe reine Spekulation wäre.
15. CausalEdge
16. Passives WAL-Shipping
17. Vollständige `ProvenanceRecord`-API-Exposition
18. `edge-reinforcement-learning`-Vollspezifikation
19. Automatische NLP-Extraktion n-ärer Fakten aus Freitext

---

<a id="19-matrix"></a>
