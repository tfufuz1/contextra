# ADR-Vorschlag: Graphindex `acta-graph` (Entwurf, nicht Teil der SPEC)

**Status:** Vorschlag. Die SPEC fuehrt keinen Graphindex (3.3 nennt 30 Produktions-Crates; Prinzip 3.1 Nr. 1: Anforderung vor Mechanismus; Default-deny nach 0.4). Aufnahme erfordert diesen ADR (naechste freie Nummer in Anhang F: ADR-34) sowie Aenderungen an SPEC 3.3, 5.5, 9.4 und `rings.toml`. Die Spezifikation wird hier **nicht** veraendert.

**Entscheidung des Auftraggebers:** Der Graphanteil (CSR-Traversierung, Personalized PageRank, Pfad-RAG, Entitaetsextraktion) ist ein wichtiger RAG-Baustein und wird extrahiert: `archive/acta-graph/` (Kern) und `archive/_reserve/graph-ext/` (Erweiterungen).

## Einordnung
- Crate `acta-graph`, **L3**, Budget 4 000 LOC, interne Abhaengigkeiten: `base, crypto, gate, vault, seg` (wie `acta-text`/`acta-vec`). Keine Querkante zu `text`/`vec`.
- Abrufpipeline (`acta-rag`, 9.4): Graph liefert zusaetzliche Kandidatenlisten (K und P getrennt) in die RRF-Fusion. `acta-rag` haengt dann zusaetzlich von `acta-graph` ab (Level 4 > 3, erlaubt).
- Graph ist **Ableitung D** (5.5): `DK`-Segmente je Mandant und `IndexGen` mit `KOrd`-Lineage; `DP`-Segmente je `ClusterId`; Takedown/Loeschung nur per Ersatzsegment ueber `ErasureLayer` (`acta-gate`). `purger.rs` der Altquelle ist die Vorlage.

## Muss-Anforderungen vor Aufnahme
1. **Zusagen-Konformitaet:** keine Sonderwege. Z7 (Determinismus): PPR, Gewichte und Entscheidungen in Festkomma Q32.32 und ganzzahlig; Gleichstand fest (Ordinal aufsteigend). Die Altquelle rechnet in `f32`/`f64` und muss umgebaut werden.
2. **Zugriff (Z16):** Traversierung nur innerhalb der Kandidatenmenge (`K: V ∩ A_K`; `P: A_P ∩ Live`). Nachbarschaften unzugaenglicher Knoten duerfen weder Treffer noch Trefferzahlen, Fehlerbilder oder Laufzeitklassen beeinflussen (9.4 Seitenkanaele).
3. **Personenbezug:** Entitaetsextraktion (`entity_extraction.rs`) aus P-Dokumenten erzeugt abgeleitete Personendaten. Diese gehoeren in DP-Segmente, werden nicht gesichert (`backup_derived_dp = false`, 5.5) und nach Wiederherstellung neu aufgebaut. Kein Graph aus P in DK.
4. **Plattenformat:** von Hand kodiert, versiegelt (Umschlag v1), ohne `serde`/`bincode` (3.8, G-NOSERDE-DISK); keine Alt-Formate.
5. **Kern synchron** (3.1 Nr. 7, G-NOTOKIO), `unsafe` nur in `acta-sys`, keine `parking_lot`/`arc-swap`/`scc`/`ahash` ohne Whitelist-Eintrag (3.8): Lesezugriff ueber unveraenderliche Schnappschuesse (3.6).
6. **Hub-Regeln:** `compat/` der Altquelle (Brueckenmodul) wird nicht uebernommen, sondern in direkte Trait-Implementierungen aufgeloest (HR-6); `TODO(acta-hr3)`-Importe dem besitzenden Crate zuordnen (HR-2/HR-3).
7. **Tests:** Zusagenmatrix 16.3 um Graph-Tests erweitern (Positivkontrolle nach 16.4): Purge/Takedown, Schnappschussisolation (`ppr_snapshot_isolation`), Referenzvalidierung (`ppr_reference_validation`), CSR-Invarianten (`proptest_csr_invariants`).

## Nicht Teil des Kerns (Reserve)
`archive/_reserve/graph-ext/`: Hyperkanten, Community (Leiden), Kaskaden, Diffusion (APPRH, TL-HFD), Perkolation, `session_dag`, Kantenverstaerkung. `session_dag` und `edge_reinforcement` sind Lern- bzw. Agentenlogik (Nicht-Ziel 1.4) und duerfen nur ohne Ableitung aus P-Nutzungsdaten uebernommen werden. Jeder Baustein einzeln: Messbericht + ADR.

## Umsetzungsschritte nach Annahme
1. SPEC: 3.3-Tabelle (+ `acta-graph`), 5.5 (+ Graph-Segmente), 9.4 (+ Graph-Listen in der Fusion, `k_graph`, `k_graph_p` in `retrieval_cfg`), Anhang A, Anhang F (ADR-34).
2. `archive/acta-graph/_skeleton/*` nach `crates/acta-graph/` kopieren; `rings.toml.fragment` in `rings.toml`, Workspace-`Cargo.toml` und `deny.toml` (Whitelist) einarbeiten; `crates/acta-graph/CLAUDE.md` pruefen.
3. Ueberfuehrung in der Reihenfolge `csr` → `types/storage` → `ppr` (Festkomma) → `purger` (ErasureLayer) → `provider/provenance` → `path_rag` → `entity_extraction`; je Schritt `cargo xtask gates` gruen, dann naechster.

## Preis und Revisionstrigger
- **Preis:** 31. Produktions-Crate, +4 000 LOC Eigencode, zusaetzliche Angriffs- und Loeschflaeche (Graph ist weitere Ableitung).
- **Revisionstrigger:** Messbericht zeigt keinen Qualitaetsgewinn (Recall/Belegquote, 9.6) gegenueber BM25+Vektor+RRF; oder Z7/Z16 sind fuer den Graphpfad nicht nachweisbar.
