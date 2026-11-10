# contextra-text — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-text/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Invertierter Textindex (Postings, WAND, Morphologie). *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | retrieval |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | ids, env |
| Default-Features | `std` |
| Fehlertyp (R3) | `TextError` (neu) |
| Brücke (hinter Feature `workspace`, §1.1) | `TextIndex`-Impl, Purge, `From` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 5886 Zeilen in 31 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 10 |
| Default-Features (Ist) | std, types-compat |
| `cfg(feature = …)`-Stellen in src/ | 55 |
| `#[test]`-Vorkommen (grep, src + tests) | 152 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 5 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 22 (Tool 25) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | TextError, Error |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types` (optional), `ports` (optional)
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env`
- **Default-Features weichen ab:** Ist `std, types-compat` → Ziel `std`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: 5 unsafe-Vorkommen, kein Insel-Status, `forbid(unsafe_code)` gesetzt. Clock: `now_unix_ms` (B02). Determinismus-Audit: 17 `HashSet`- und 7 `HashMap`-Treffer in Produktionscode (siehe Fundstellen).

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Neues lokales Trait `PostingStore` = **exakt** die in `src/` aufgerufenen `StorageEngine`-Methoden (Erhebung: `rg 'storage\.' src` und alle `StorageEngine`-Aufrufe in `wand.rs`, `inverted*`, `stream.rs`, `memory_index.rs`); keine Methode mehr als genutzt. Brücke implementiert es für `StorageEngine`. In `compat/`: `inverted/purge.rs`, `inverted/scanner.rs`, `inverted/morph_index.rs` (`TextIndex`), Fehlerkonvertierung, `wand.rs`-Teile, die `ports` direkt brauchen (nach B04). `types.rs` (130) und `contract.rs` (39) → `pub use` auf `ids`/`env`; `docid-128` an `ids` (B03). Leere Features (`experimental`, `bm25f`, `serde`, `tracing`, `inspect`) per Audit streichen. `default = ["std"]`.

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `TextError` (neu) (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

### 7.1 Persistierte Formate (§5.1 Golden-Korpus) [S]

| Format | Quelle / Kennung |
|---|---|
| Text-Postings, Graph-Persistenz (bincode), Checkpoint-Meta/Manifest | aus `text`, `graph`, `checkpoint` zu erheben |

Jeder Eintrag hat einen Erzeugertest (`#[ignore]`, Script `tools/golden_regen.sh`) und einen Prüftest, der immer läuft. Abweichung ⇒ Schritt abbrechen.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Determinismus (Produktion, bereinigt: 22; die ersten 15):** Tool-Gesamt 25.

- `crates/contextra-text/src/tokenizer.rs:23` — random_hashset — `use std::collections::HashSet;`
- `crates/contextra-text/src/tokenizer.rs:28` — random_hashset — `static STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();`
- `crates/contextra-text/src/tokenizer.rs:120` — random_hashset — `fn get_stopwords() -> &'static HashSet<String> {`
- `crates/contextra-text/src/tokenizer.rs:461` — random_hashset — `stopwords: Option<HashSet<String>>,`
- `crates/contextra-text/src/tokenizer.rs:493` — random_hashset — `pub fn with_stopwords(mut self, stopwords: HashSet<String>) -> Self {`
- `crates/contextra-text/src/morphology/compound_splitter.rs:6` — random_hashset — `use std::collections::HashSet;`
- `crates/contextra-text/src/morphology/compound_splitter.rs:81` — random_hashset — `pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self {`
- `crates/contextra-text/src/morphology/stopwords.rs:3` — random_hashset — `use std::collections::HashSet;`
- `crates/contextra-text/src/morphology/stopwords.rs:6` — random_hashset — `static GERMAN_STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();`
- `crates/contextra-text/src/morphology/stopwords.rs:13` — random_hashset — `pub fn get_german_stopwords() -> &'static HashSet<String> {`
- `crates/contextra-text/src/morphology/stopwords.rs:22` — random_hashset — `let mut set = HashSet::with_capacity(words.len());`
- `crates/contextra-text/src/morphology/trie.rs:10` — random_hashmap — `pub children: std::collections::HashMap<char, TrieNode>,`
- `crates/contextra-text/src/inverted/purge.rs:25` — random_hashset — `use std::collections::HashSet;`
- `crates/contextra-text/src/inverted/purge.rs:176` — random_hashset — `let mut terms_to_check: HashSet<String> = HashSet::new();`
- `crates/contextra-text/src/inverted/index_struct.rs:4` — random_hashmap — `use std::collections::HashMap;`

**unsafe-Dateien (1):**

- `crates/contextra-text/tests/alloc_profiler.rs`

**Fehler-Enums (Ist):**

- `crates/contextra-text/src/error.rs:8` — TextError
- `crates/contextra-text/src/contract.rs:3` — Error

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)**
> **Entscheidung:** - `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2). - Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein. - `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B). - E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut …
> `text/contract.rs:16/19`, `graph/contract.rs:15/17` — **`now_unix_ms`** — **`&mut self`**, `Send`

**B03 — `docid-128` ändert die Typbreite und ist nicht additiv**
> **Entscheidung:** 1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`. 2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16). 3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet. 4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

**B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab**
> **Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

**B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen**
> **Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-text --no-default-features --locked
cargo hack check -p contextra-text --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-text --modes no-default,default,each-feature
```

