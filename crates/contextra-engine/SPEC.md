# contextra-engine — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-engine/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Ausführungsschicht der Datenbank: Collections, Suche, Transaktionen, Chunking-Adapter. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | integration |
| Ebene (Kern-Closure) | 2 |
| Kern-Closure | alle benötigten |
| Default-Features | — |
| Fehlertyp (R3) | `ContextraError` |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 13333 Zeilen in 76 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports, mvcc, sandbox, infer-onnx, adapt, store, vector, text, checkpoint, rank, graph |
| Interne Dev-Abhängigkeiten (Ist) | testkit, vector |
| Features (Ist, Anzahl) | 11 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 37 |
| `#[test]`-Vorkommen (grep, src + tests) | 70 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 184; 184 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 21 (Tool 22) |
| Port-M3-Leaks (port_contract_audit) | 2 |
| Error-Enums (Ist) | keine |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ziel-Kern-Closure nicht maschinenlesbar** (§3.1: „alle benötigten“). Der Abgleich entfällt; die Festlegung erfolgt in S-00 bzw. im jeweiligen Schritt.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Zero-panic: 184 Produktionsbefunde (höchster Wert im Workspace). `FeatureRing` in `src/contextra_impl/lifecycle.rs:9` (Lizenz-Umstellung S-64).
- Engine ist der einzige Verwender von `chunker`-Logik, der S-61 betrifft. Die Paritätstests sind noch nicht geschrieben.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Herauslösen: `chunker/` → dünner Adapter über `ingest::chunking` (S-61); `temporal_filter.rs` → `rank`; `decay_controller.rs` → `adapt` (S-60); `extraction/` → `exp-extraction` (Vergleich mit `graph/entity_extraction.rs`: Namens-/API-Diff erheben, **keine** Zusammenlegung); `merge_operator_adapter.rs` bleibt. Lizenz: `contextra_impl/lifecycle.rs:9–55`, `lib.rs:17,22,373,410` → `contextra-license`. `VectorDeleteMode` aus `vector`. Leere Features (`reranking`) per Audit.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `ContextraError` (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 184, davon 184 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).


**Determinismus (Produktion, bereinigt: 21; die ersten 15):** Tool-Gesamt 22.

- `crates/contextra-engine/src/collection/maintenance.rs:172` — random_hashset — `let indexed_ids: std::collections::HashSet<DocId> =`
- `crates/contextra-engine/src/collection/maintenance.rs:209` — random_hashset — `indexed_ids: &std::collections::HashSet<DocId>,`
- `crates/contextra-engine/src/collection/maintenance.rs:816` — random_hashmap — `) -> Result<std::collections::HashMap<EntityId, u64>> {`
- `crates/contextra-engine/src/collection/maintenance.rs:818` — random_hashmap — `return Ok(std::collections::HashMap::new());`
- `crates/contextra-engine/src/collection/maintenance.rs:830` — random_hashset — `let target_set: std::collections::HashSet<&EntityId> = entity_ids.iter().collect();`
- `crates/contextra-engine/src/collection/maintenance.rs:831` — random_hashmap — `let mut map = std::collections::HashMap::new();`
- `crates/contextra-engine/src/collection/maintenance.rs:914` — random_hashmap — `let mut embeddings = std::collections::HashMap::new();`
- `crates/contextra-engine/src/collection/maintenance.rs:915` — random_hashmap — `let mut id_map = std::collections::HashMap::new();`
- `crates/contextra-engine/src/orchestration/mod.rs:59` — random_hashmap — `data: std::collections::HashMap<String, String>,`
- `crates/contextra-engine/src/orchestration/mod.rs:66` — random_hashmap — `data: std::collections::HashMap::new(),`
- `crates/contextra-engine/src/contextra_impl/collections.rs:249` — random_hashset — `let mut names = std::collections::HashSet::new();`
- `crates/contextra-engine/src/contextra_impl/collections.rs:280` — random_hashset — `let mut names = std::collections::HashSet::new();`
- `crates/contextra-engine/src/contextra_impl/lifecycle.rs:250` — instant_now — `let start_time = std::time::Instant::now();`
- `crates/contextra-engine/src/collection/search/filtered.rs:91` — random_hashset — `) -> Result<std::collections::HashSet<DocId>> {`
- `crates/contextra-engine/src/collection/search/filtered.rs:101` — random_hashset — `let mut matched = std::collections::HashSet::new();`

**Port-M3-Leaks (2):**

- `crates/contextra-engine/src/lib.rs:24` —  — `pub use contextra_store::lsm::DurabilityMode;`
- `crates/contextra-engine/src/lib.rs:97` —  — `pub use contextra_text::Language;`

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B07 — Eine Verzeichnis-Regruppierung bricht messbar viel**
> **Entscheidung (D-01):** Das Verzeichnislayout bleibt **flach** (`crates/contextra-*`). Die Gruppierung steht als Metadatum `group` in `capabilities.toml`. Die physische Regruppierung ist Nicht-Ziel (§9.3); sie liefert keinen Entnahmegewinn und kostet die oben genannten Brüche.

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `decay_controller` adapt/engine — 9 gemeinsam; `engine` hat 5 zusätzliche (`with_clock`, `set_clock`, `effective_half_life_ms`, `effective_score_wall_clock`, `should_evict_wall_clock`), `adapt` 1 (`config`) [V] — `adapt` bleibt Besitzer, **muss zuerst die 5 Methoden aufnehmen**; Zusammenlegung erst …

## 10. Entscheidungen und Risiken [S]

**Risiken (§9.2):**

| `cognition`-Port-Zuschnitt (`CommitIntent` aus `engine::transaction`, 1.652 Z. [E]) | S-63 | Port scheitert ⇒ `CommitIntent` als eigener Typ in `cognition`, Abbildung in `db`-Brücke |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-engine --no-default-features --locked
cargo hack check -p contextra-engine --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-engine --modes no-default,default,each-feature
```

