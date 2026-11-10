# contextra-vector — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-vector/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Vektorindex (HNSW). DiskANN und RaBitQ wandern in Experimente. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | retrieval |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | ids, env, mvcc (`sys` nur `mmap`, `simd` nur `simd`) |
| Default-Features | `std` |
| Fehlertyp (R3) | `VectorError` |
| Brücke (hinter Feature `workspace`, §1.1) | `VectorIndex`-Impls, Purger, `From` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 13800 Zeilen in 56 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports, mvcc, sys |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 16 |
| Default-Features (Ist) | std, ports-compat |
| `cfg(feature = …)`-Stellen in src/ | 66 |
| `#[test]`-Vorkommen (grep, src + tests) | 127 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 1 (Tool 16; 15 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 6 (Tool 6) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | VectorError, Error, AcornError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types` (optional), `ports` (optional), `sys` (optional)
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env`
- **Default-Features weichen ab:** Ist `std, ports-compat` → Ziel `std`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Determinismus-Treffer in `src/fixed_hasher.rs` sind die Schutzschicht selbst (`FixedHashMap` mit festem Hasher). Sie sind Fehlalarme des Tools und werden nicht behoben.
- Verifiziert: `HNSW_VERSION = 2` in `src/persistence/header.rs:11`. `DistanceMetric` in `src/types.rs:130` (B09).
- **Abweichung:** Der in §6.2 genannte Test `partial_rebuild_veto_f02_guard` existiert im Quelltext nicht. Der Name taucht nur in `crates/contextra-vector/SPEC.md` auf. Das V-VETO-Gate ist ohne Test nicht durchsetzbar, bis der Test angelegt oder umbenannt ist.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Brücke: `hnsw/vector_index_impl.rs`, `diskann/vector_index_impl.rs`, `hnsw/purge.rs`, `hnsw/scanner.rs`, `error.rs:61–91`, `types.rs:279ff`. Kern: `hnsw/types.rs` (5, 128–143, 203–210, 241–242, 279), `quantize/mod.rs`, `hnsw/core_{rebuild,search,insert}.rs`, `persistence/*`, `distance.rs`, `candidate_stream.rs` auf `VectorError`/`ids`/`env`/`mvcc`. `contract.rs` (119 Z.) wird zu `env`/`ids`-basiert vereint: `Clock` (ms→`unix_ms`), `Rng` (`&self`), `ExternalId(u128)` → `ids::DocId`-Pfad nur, wenn serde/`from_key` unberührt; sonst Kopie entfällt **nicht** (Differenztest entscheidet, Abbruch bei Abweichung). `VectorDeleteMode` hierher (S-50). `default = ["std"]` (kein `ports-compat`). `mvcc` bleibt Kern-Abhängigkeit (`SequenceLog`/`TxBuffer`-Nutzung) — die Schwester-Spec M-5-Alternative (eigener `VisibilityLog`) ist Nicht-Ziel. Feature `simd` (B09). Feature-Schema: `exp-diskann`, `exp-rabitq`, `exp-acorn`, `experimental-all`; `diskann/` (12 Dateien, 4.168 Z.) → Crate `contextra-vector-diskann` (S-80), `quantize_rabitq.rs` → `experimental/rabitq.rs`; Regel V-VETO: `PartialRebuildConfig` bleibt aus (Test `partial_rebuild_veto_f02_guard` MUSS grün bleiben, Pfad per `rg` bestätigen [E]).

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `VectorError` (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

### 7.1 Persistierte Formate (§5.1 Golden-Korpus) [S]

| Format | Quelle / Kennung |
|---|---|
| HNSW | `vector/persistence/header.rs`: `0x484E5357`, Version 2 |
| DiskANN | `vector/diskann/format.rs`: `DANN` v1, `DANF`, `PWAL` v1, `TWAL` v1 |

Jeder Eintrag hat einen Erzeugertest (`#[ignore]`, Script `tools/golden_regen.sh`) und einen Prüftest, der immer läuft. Abweichung ⇒ Schritt abbrechen.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 1; die ersten 15):** Tool-Gesamt 16, davon 15 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).

- `crates/contextra-vector/src/diskann/build.rs:759` — assert — `assert!(`

**Determinismus (Produktion, bereinigt: 6; die ersten 15):** Tool-Gesamt 6.

- `crates/contextra-vector/src/fixed_hasher.rs:4` — random_hashmap — `use std::collections::{HashMap, HashSet};` _(Schutzschicht, Fehlalarm)_
- `crates/contextra-vector/src/fixed_hasher.rs:4` — random_hashset — `use std::collections::{HashMap, HashSet};` _(Schutzschicht, Fehlalarm)_
- `crates/contextra-vector/src/fixed_hasher.rs:19` — random_hashset — `pub type FixedHashSet<T> = HashSet<T, RandomState>;` _(Schutzschicht, Fehlalarm)_
- `crates/contextra-vector/src/fixed_hasher.rs:22` — random_hashmap — `pub type FixedHashMap<K, V> = HashMap<K, V, RandomState>;` _(Schutzschicht, Fehlalarm)_
- `crates/contextra-vector/src/fixed_hasher.rs:26` — random_hashset — `HashSet::with_capacity_and_hasher(capacity, fixed_hasher())` _(Schutzschicht, Fehlalarm)_
- `crates/contextra-vector/src/fixed_hasher.rs:31` — random_hashmap — `HashMap::with_capacity_and_hasher(capacity, fixed_hasher())` _(Schutzschicht, Fehlalarm)_

**Fehler-Enums (Ist):**

- `crates/contextra-vector/src/error.rs:10` — VectorError
- `crates/contextra-vector/src/contract.rs:8` — Error
- `crates/contextra-vector/src/acorn/mod.rs:30` — AcornError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B01 — Drei Dokumente, drei Architekturen**
> **Entscheidung:** - Es gibt kein `modules/`-Verzeichnis. Die Brücke ist ein **Modul** `compat/` im Crate (kein Extra-Crate). - `ids`/`env` sind für alle Bausteine erlaubte E0-Abhängigkeiten. Die Seam-Definitionen `Clock`, `Rng`, `MetricsSink`, `IdGen` der Schwester-Specs entfallen zugunsten `env`; `DocKey`/`ExternalId`-Typen zugunsten `ids`. `docid-128` gehört **ausschließlich** `ids` (B03). - Die Mikro-Crate-Zerlegung der Schwester-Specs ist **Folgeprojekt F-1** und nicht Teil dieser Spezifikation (§9.3).

**B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)**
> **Entscheidung:** - `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2). - Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein. - `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B). - E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut …
> `vector/types.rs:169/203` — wie ports, ohne `'static` — `next_u64(&self)`, `next_f64(&self)`
> `vector/contract.rs:39/45` — **`now_unix_ms`** (Millisekunden) — **`&mut self`**, `Send`

**B03 — `docid-128` ändert die Typbreite und ist nicht additiv**
> **Entscheidung:** 1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`. 2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16). 3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet. 4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

**B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab**
> **Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

**B07 — Eine Verzeichnis-Regruppierung bricht messbar viel**
> **Entscheidung (D-01):** Das Verzeichnislayout bleibt **flach** (`crates/contextra-*`). Die Gruppierung steht als Metadatum `group` in `capabilities.toml`. Die physische Regruppierung ist Nicht-Ziel (§9.3); sie liefert keinen Entnahmegewinn und kostet die oben genannten Brüche.

**B09 — `ports` und `types` ohne Zielbild für `DistanceMetric`; mehrere Distanz-Implementierungen**
> **Entscheidung (R4-Ausnahme):** Die drei Enums bleiben je im Besitz ihres Crates. Erlaubt sind kleine geschlossene Wert-Enums (≤ 8 Unit-Varianten) als **gespiegelte Kopie**, wenn (a) Variantennamen und -reihenfolge identisch sind und (b) ein Konformitätstest die serde-Namen aller Kopien gleichsetzt. Verbindung per `From` in `compat/`. `vector` bekommt das optionale Feature `simd`, das `compute_distance` durch `contextra_simd::dispatch` ersetzt; die Skalarpfade bleiben als Orakel. Differenztest §5.4 (Toleranz relativ 1e-5, f32).

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
cargo check -p contextra-vector --no-default-features --locked
cargo hack check -p contextra-vector --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-vector --modes no-default,default,each-feature
```
