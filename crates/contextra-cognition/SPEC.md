# Spezifikation: `contextra-cognition`

**Ring:** Ring 3 (Orchestration & Cognition)
**Normativität:** Normativ gemäß `docs/SPEC.md` §1.1, §6.3
**Klasse:** C (Kognitive Verdichtung & Semantische Aggregation)
**MSRV:** 1.89.0
**Code-Disziplin:** `#![forbid(unsafe_code)]`, Zero-Panic

---

## 1. Zweck & Systemgrenzen
`contextra-cognition` stellt Algorithmen zur kognitiven Verdichtung, semantischen Aggregation, Hyperedge-Synthese und Wissensnetz-Konsolidierung bereit:
- **Scope:** Erkennung emergenter Beziehungen, automatische Zusammenfassung von Kontextknoten, Feedback-gestützte Kantengewichtung.
- **Entkopplung:** Vermeidung von direkten zirkulären Abhängigkeiten zu `contextra-engine`; Nutzung lokaler Abstraktionsports (`DocumentSource`, `ConsolidationSink`, `HyperedgeSink`).

---

## 2. Metriken & Werkzeug-Audit-Befunde
*Erfasst via `tools/decouple_inspector.py -c contextra-cognition`:*
- **LOC:** 5.335 LOC über 24 Rust-Dateien.
- **Typen-Inventar:** 3 Traits, 45 Structs, 1 Enum, 77 öffentliche Funktionen (`pub fn`).
- **Abhängigkeiten:** 4 interne Crates (`contextra-types`, `contextra-ports`, `contextra-graph`, `contextra-engine`).
- **Audit-Befund:** 1 Legacy-Verweis auf Purge-Begriffe; Ziel ist die vollständige Isolation von `engine`.

---

## 3. Architektur & Modulkarte

| Modul | Verantwortung | Primäre Typen |
|---|---|---|
| `aggregation/` | Semantische Clusterung und Kondensation von Wissensfragmenten | `SemanticAggregator`, `ClusterSummary` |
| `consolidation/` | Periodische Wissenskonsolidierung und Decay-Bereinigung | `ConsolidationPipeline`, `DecayPolicy` |
| `hyperedge/` | Synthese von N-stelligen Relationen und Hypergraphen | `HyperedgeSynthesizer`, `EdgeReinforcement` |
| `ports.rs` | Lokale Entkopplungs-Ports | `DocumentSource`, `ConsolidationSink`, `HyperedgeSink` |
| `error.rs` | Domänenspezifischer Fehlertyp | `CognitionError`, `Result<T>` |

---

## 4. Invarianten (INV-COG-*)

| Invariante | Beschreibung | Verifikation |
|---|---|---|
| `INV-COG-ZERO-UNSAFE` | `#![forbid(unsafe_code)]` wird strikt durchgesetzt. | `cargo check -p contextra-cognition` |
| `INV-COG-DETERMINISM-1` | Aggregations- und Ranking-Schritte liefern für bit-identische Graphen identische Konsolidierungszustände. | `tests/consolidation_determinism.rs` |
| `INV-COG-FAIL-CLOSED` | Fehlerhafte Konsolidierungsschritte brechen sicher ab, ohne den persistenten Graphen zu korrumpieren. | `tests/tx_resilience.rs` |
| `INV-COG-ISOLATION` | Engine-gebundene Ausführer verbleiben im Crate `db` unter dem Feature `cognition`. | `docs/SPEC.md` §6.3 |

---

## 5. Zielzustand & Migrationspfad
Gemäß `docs/SPEC.md` §6.3:
1. Lokale Ports `DocumentSource`, `ConsolidationSink`, `HyperedgeSink` etablieren.
2. Einführung des dedizierten `CognitionError` anstelle generischer `ContextraError`-Varianten.
3. Verlagerung engine-spezifischer Brücken nach `contextra-db` hinter Feature `cognition`.
