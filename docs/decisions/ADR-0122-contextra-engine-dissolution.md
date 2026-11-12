# ADR-0122: Auflösung von `contextra-engine` und Konsolidierung in `contextra-db`

**Status:** Akzeptiert  
**Datum:** 2026-11-12  
**Autor:** Principal Rust Engineer  
**Bezug:** SPEC(11).md §4.1 engine, §9.4 engine, S-63, S-64, HUB_REMOVAL.md §2.3, INV-2, INV-5, INV-12

---

## 1. Kontext und Problemstellung

Die Crate `contextra-engine` (E2, ~13.300 Zeilen) fungiert historisch als intermediärer Orchestrierungs- und Speicherhub zwischen den Fachindizes (Ring 1) und den Integrationsfassaden `contextra-db` und `contextra-cognition`.

Gemäß HUB_REMOVAL.md §2.3 und SPEC(11).md S-63 / S-64 existieren nur noch zwei direkte Verbraucher von `contextra-engine`:
- `contextra-db`: Verwendet `engine` fast ausschließlich als Re-Export-Basis für seine öffentliche API (`prelude` und `advanced`).
- `contextra-cognition`: Verwendet `engine` verdeckt über `cognition/src/engine_local.rs` für `Collection`, `decay_controller`, `CommitIntent`, `SearchResult` und `ProvenanceRecord`.

Ziel ist die vollständige Auflösung von `contextra-engine` als physische Abhängigkeit.

---

## 2. Entscheidung

### 2.1 Modul-Verschiebung nach `contextra-db` statt Erstellung eines neuen Crates
Die Module von `contextra-engine`:
- `adapters/`, `backends.rs`, `background_workers/`, `chunker/`
- `collection/`
- `contextra_impl/` (Orchestrierung und Fassade `Contextra`)
- `decay_controller.rs`
- `export.rs`, `import.rs`
- `extraction/`
- `filter.rs`, `temporal_filter.rs`
- `fusion.rs`
- `license_seam.rs`
- `merge_operator_adapter.rs`
- `metrics_names.rs`
- `orchestration/`
- `transaction/`

werden vollständig in `contextra-db` integriert.

**Begründung:**
`contextra-db` ist auf Ebene E2 bereits als eingebettete Datenbankfassade definiert. Ein neuer separater E2-Crate würde lediglich einen neuen Integrationshub schaffen. Durch die Integration in `contextra-db` fällt ein kompletter Layer weg und `contextra-db` wird zur autarken Speicher- und Abfrage-Engine.

### 2.2 Feature-Zuordnung
Alle Features von `contextra-engine` werden direkt von `contextra-db` übernommen:
- `onnx` -> `dep:contextra-infer-onnx`, `contextra-infer-onnx/onnx`
- `sandbox` -> `dep:contextra-sandbox`
- `license` -> `dep:contextra-license` (S-64)
- `exp-extraction` / `entity-extraction` -> bedingte Kompilierung von `extraction/`
- `background-maintenance`, `graph-connectivity-health`, `adaptive-candidate-pool-sizing`

### 2.3 Entkopplung von `contextra-cognition` via Trait-Inversion (S-63)
`contextra-cognition` darf `contextra-engine` nicht mehr als Abhängigkeit führen.
1. `Collection`-Operationen werden über das Trait `CollectionOps` in `contextra-cognition::ports` abstrahiert.
2. `AdaptiveDecayController` und `DecayControllerConfig` erhalten eine lokale, deterministische Repräsentation in `cognition` (INV-5, replay-safe).
3. `CommitIntent` nutzt die in `cognition::ports` definierte Struktur.
4. `SearchResult` und `ProvenanceRecord` stammen kanonisch aus `contextra-rank` (E1).
5. In `contextra-db::cognition::bridge` wird der Adapter implementiert, der `Collection` auf `CollectionOps` abbildet.

### 2.4 Erhalt von `contextra-engine` als `#[deprecated]` Fassade (INV-2)
`contextra-engine` bleibt als Übergangs-Crate mit minimalem Inhalt bestehen, re-exportiert `contextra-db` mit `#[deprecated]` und wird aus dem normalen Abhängigkeitsgraphen der Workspace-Crates entfernt.

---

## 3. Invarianten

- **INV-2:** Fassade bleibt mit `#[deprecated]`-Markierung erhalten.
- **INV-5:** Determinismus der Rangfolge und Decay-Berechnung bleibt exakt identisch.
- **INV-12:** Kein `unwrap()`, `expect()` oder `panic!` in neuem Produktionscode.
