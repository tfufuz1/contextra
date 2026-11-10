# contextra-graph — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-graph/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Graphindex (CSR, Personalized PageRank) und Experimente. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | retrieval |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | ids, env |
| Default-Features | **leer** (kein `std`-Feature vorhanden; ohne `experimental-graph`) |
| Fehlertyp (R3) | `GraphError` (neu) |
| Brücke (hinter Feature `workspace`, §1.1) | `GraphIndex`, Purger, `From` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 13382 Zeilen in 71 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 11 |
| Default-Features (Ist) | types-compat, ports-compat, experimental-graph |
| `cfg(feature = …)`-Stellen in src/ | 44 |
| `#[test]`-Vorkommen (grep, src + tests) | 210 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 43; 43 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 5 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 137 (Tool 151) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | HyperedgeSuggestError, GraphError, GraphMutationError, Error, ApprhError, TlHfdError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types` (optional), `ports` (optional)
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env`
- **Default-Features weichen ab:** Ist `types-compat, ports-compat, experimental-graph` → Ziel `leer`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: `Clock` mit `now_unix_ms` und `Rng` mit `&mut self` in `src/contract.rs:15,17` (B02). Determinismus-Audit: 93 `HashMap`- und 58 `HashSet`-Treffer in Produktionscode. Das ist der höchste Einzelwert im Workspace. docs/SPEC.md adressiert ihn nicht, er ist ein eigener Befund.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Brücke: `provider.rs`, `purger.rs`, `csr/graph_index/*` und alle Importe von `ports::{graph_index,graph}`; lokale `DocId`/`TxId` (`types.rs:58,104`) → `ids`; `default = []`. `PprConfig`, `PprAlgorithm`, `TlHfdParams` (aus `types/misc.rs` ab Z. 375) wandern hierher (S-51); `engine`/`cognition` importieren dann aus `graph`. Experimente: Klasse A/B laut Entwurf (Blätter `hyperedge_suggest`, `entity_extraction`, `percolation`, `edge_reinforcement_buffer`; mit Naht `apprh`, `tl_hfd`, `session_dag`, `edge_reinforcement`, `community`); Klasse C (`hyperedge` 19 Inbound-Dateien, `path_rag` 14, `consistency_enforcement` 6) bleibt im Kern, optional `exp-hyperedge`. Ab > 2.000 Z. (X4): Crate `contextra-graph-research` (S-81). `ports/graph.rs` gibt `HyperEdgeId`/`RoleBinding`/`RoleId` hinter `exp-hyperedge` frei.

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `GraphError` (neu) (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

### 7.1 Persistierte Formate (§5.1 Golden-Korpus) [S]

| Format | Quelle / Kennung |
|---|---|
| Text-Postings, Graph-Persistenz (bincode), Checkpoint-Meta/Manifest | aus `text`, `graph`, `checkpoint` zu erheben |

Jeder Eintrag hat einen Erzeugertest (`#[ignore]`, Script `tools/golden_regen.sh`) und einen Prüftest, der immer läuft. Abweichung ⇒ Schritt abbrechen.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 43, davon 43 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).


**Determinismus (Produktion, bereinigt: 137; die ersten 15):** Tool-Gesamt 151.

- `crates/contextra-graph/src/provider.rs:146` — random_hashmap — `) -> HashMap<EntityId, f32> {`
- `crates/contextra-graph/src/provider.rs:150` — random_hashmap — `return HashMap::new();`
- `crates/contextra-graph/src/provider.rs:152` — random_hashmap — `let node_to_idx: HashMap<EntityId, usize> = nodes`
- `crates/contextra-graph/src/provider.rs:215` — random_hashmap — `let node_to_idx: HashMap<EntityId, usize> =`
- `crates/contextra-graph/src/provider.rs:242` — random_hashmap — `let mut comm_degs: HashMap<u64, f32> = communities`
- `crates/contextra-graph/src/provider.rs:255` — random_hashmap — `let mut weights: HashMap<u64, f32> = HashMap::new();`
- `crates/contextra-graph/src/ppr_stream.rs:15` — random_hashset — `use std::collections::{HashSet, VecDeque};`
- `crates/contextra-graph/src/ppr_stream.rs:145` — random_hashset — `let seed_set: HashSet<EntityId> = self.seeds.iter().copied().collect();`
- `crates/contextra-graph/src/cascade.rs:194` — random_hashset — `let node_set: std::collections::HashSet<_> = closure_nodes.iter().copied().collect();`
- `crates/contextra-graph/src/cascade.rs:196` — random_hashmap — `let mut in_degree = std::collections::HashMap::new();`
- `crates/contextra-graph/src/cascade.rs:197` — random_hashmap — `let mut children_map: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::ne`
- `crates/contextra-graph/src/cascade.rs:238` — random_hashset — `let placed: std::collections::HashSet<_> = topo_order.iter().copied().collect();`
- `crates/contextra-graph/src/cascade.rs:279` — random_hashset — `let mut visited = std::collections::HashSet::new();`
- `crates/contextra-graph/src/cascade.rs:280` — random_hashset — `let mut on_stack = std::collections::HashSet::new();`
- `crates/contextra-graph/src/cascade.rs:285` — random_hashset — `visited: &mut std::collections::HashSet<crate::hyperedge::HyperEdgeId>,`

**unsafe-Dateien (1):**

- `crates/contextra-graph/tests/ppr_alloc_test.rs`

**Fehler-Enums (Ist):**

- `crates/contextra-graph/src/hyperedge_suggest.rs:45` — HyperedgeSuggestError
- `crates/contextra-graph/src/error.rs:11` — GraphError
- `crates/contextra-graph/src/error.rs:48` — GraphMutationError
- `crates/contextra-graph/src/contract.rs:4` — Error
- `crates/contextra-graph/src/apprh/error.rs:8` — ApprhError
- `crates/contextra-graph/src/tl_hfd/error.rs:8` — TlHfdError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B01 — Drei Dokumente, drei Architekturen**
> **Entscheidung:** - Es gibt kein `modules/`-Verzeichnis. Die Brücke ist ein **Modul** `compat/` im Crate (kein Extra-Crate). - `ids`/`env` sind für alle Bausteine erlaubte E0-Abhängigkeiten. Die Seam-Definitionen `Clock`, `Rng`, `MetricsSink`, `IdGen` der Schwester-Specs entfallen zugunsten `env`; `DocKey`/`ExternalId`-Typen zugunsten `ids`. `docid-128` gehört **ausschließlich** `ids` (B03). - Die Mikro-Crate-Zerlegung der Schwester-Specs ist **Folgeprojekt F-1** und nicht Teil dieser Spezifikation (§9.3).

**B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)**
> **Entscheidung:** - `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2). - Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein. - `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B). - E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut …
> `text/contract.rs:16/19`, `graph/contract.rs:15/17` — **`now_unix_ms`** — **`&mut self`**, `Send`

**B03 — `docid-128` ändert die Typbreite und ist nicht additiv**
> **Entscheidung:** 1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`. 2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16). 3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet. 4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

**B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab**
> **Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

**B07 — Eine Verzeichnis-Regruppierung bricht messbar viel**
> **Entscheidung (D-01):** Das Verzeichnislayout bleibt **flach** (`crates/contextra-*`). Die Gruppierung steht als Metadatum `group` in `capabilities.toml`. Die physische Regruppierung ist Nicht-Ziel (§9.3); sie liefert keinen Entnahmegewinn und kostet die oben genannten Brüche.

**B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird**
> **Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

**B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen**
> **Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-graph --no-default-features --locked
cargo hack check -p contextra-graph --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-graph --modes no-default,default,each-feature
```

