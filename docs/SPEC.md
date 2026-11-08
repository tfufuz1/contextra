# Contextra Baukasten – Spezifikation v4.0

```
Dokument:  Normative Spezifikation des internen Baukastens (Repository-Ebene)
Fassung:   4.0 – ersetzt v3.0 und alle früheren Fassungen vollständig
Stand:     2026-11-08
Status:    Entscheidungsfassung (bindend für WP-0 bis WP-6)
Ablage:    docs/SPEC.md   (Vorgängerfassung: docs/archive/SPEC_v3_oekosystem.md, nicht normativ)
```

## 0. Zweck und Geltung

**Contextra ist ein privates Repository aus isolierten, wiederverwendbaren Rust-Bausteinen** für LLM-bezogene Systeme (Retrieval, Speicher, Inferenz, Sandboxing, Compliance-Primitive). Es ist **kein Produkt, keine Bibliothek mit Fassade und keine Plattform.** Aus den Bausteinen werden in *anderen* Repositories Produkte komponiert.

Diese Spezifikation regelt ausschließlich:

1. Was ein Baustein ist und welche Eigenschaften er garantiert (§2–§4).
2. Welche Crates welcher Klasse angehören und welche Zielabhängigkeiten gelten (§5).
3. Wie Löschlogik (DSGVO) getrennt wird (§6).
4. Wie Bausteine exportiert, geprüft und gemessen werden (§7–§9).
5. Welche Entscheidungen getroffen sind und was umzusetzen ist (§10–§12).

**Nicht Gegenstand** (gegenüber v3.0 gestrichen): Produktdefinition, Deployment-Tiers, Endnutzer-Schnittstellen, Python-Wheels, Release-/Ökosystem-Governance, Gesamtanforderungskatalog des Produkts. Bausteinspezifische Fachdetails (Persistenzformat, Saga-Zustände, Startwerte, Kalibrierung) wandern in `crates/<name>/SPEC.md` (§9.2) und sind **keine** Aufgabe dieses Dokuments.

**Evidenzregel.** Zahlen in diesem Dokument sind entweder *gemessen* (mit Befehl) oder als *Analyse, ungeprüft* markiert. Handgepflegte Scores, LOC- und „Relikt“-Zähler sind verboten; Kennzahlen entstehen ausschließlich durch Skripte (§8.3).

---

## 1. Ausgangslage (Ist-Zustand, belegt)

Messung 2026-11-08: Rust 1.89, `cargo metadata` (Kanten `normal`+`build`, Dev ausgeklammert), `cargo check --workspace --exclude contextra-bench --exclude contextra-py` = **grün, 68 Warnings, davon 48 „use of deprecated“**. Tests, Benchmarks, Clippy: **nicht ausgeführt**.

| Befund | Beleg |
|---|---|
| Nur `types`, `sys`, `wire`, `sandbox` haben 0 interne Abhängigkeiten. | `cargo metadata` |
| `simd` hängt an `contextra-types` (`ContextraError`, `DistanceMetric`); kein lokales `Metric`-Enum existiert. | `grep`, `Cargo.toml` |
| `rank` hängt an `types` **und** `ports`; `rrf.rs` re-exportiert `types::FusionStrategy`. | `Cargo.toml`, `grep` |
| `durable-fs`: `ports`, `types`, `sys`. `vector`: 5 interne Deps, `store`: 5, `engine`: 12 transitiv, `mcp`: 19. | `cargo metadata` |
| Selbst `simd` zieht transitiv 67 externe Crates (durch `types`). | `cargo metadata` |
| `contextra-erasure` deklariert `contextra-crypto`, nutzt es nirgends (0 Treffer in `src`). | `grep` |
| Löschartefakte außerhalb `erasure`: `Purger`/`LayerScanner`/`WitnessLayer` in `vector`, `text`, `graph`, `store`; `ShreddableSegmentKey` in `kvcache`; `ErasureId` in `types`. | `grep` |
| Referenzierte Crates `contextra-erasure-spi`, `contextra-erasure-text` existieren nicht. | Verzeichnis, Workspace |
| `AGENTS.md` enthält ungelöste Merge-Marker (Zeilen 244, 262, 282). | `grep` |
| `crates/contextra-core` liegt als Verzeichnis vor, ist kein Workspace-Member. | Verzeichnis, `Cargo.toml` |
| `VectorIndex`/`TextIndex` enthalten `commit`, `rollback`, `rollback_to_tx`, `last_tx_id` (an `TxId` gebunden). | `ports/src/text_index.rs` |
| Benchmarks existieren nur in 11 von 36 Crates (`checkpoint`, `crypto`, `erasure`, `graph`, `infer-candle`, `infer-onnx`, `mvcc`, `router`, `simd`, `store`, `vector`). | Verzeichnis `benches/` |
| Manifeste erben per `workspace = true`; ein herauskopiertes Crate baut nicht ohne Manifest-Reparatur. | `Cargo.toml` |

**Konsequenz:** Der Abschlussbericht „Transformation erfolgreich abgeschlossen“ und die Zielarchitektur-Scores sind **nicht belastbar** und werden durch §1 und §8.3 ersetzt. *Analyse, ungeprüft* (aus separater Statikanalyse, vor Umsetzung per `grep` zu verifizieren): Löschlogik in `crypto` (`deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`, ca. 2,8k Zeilen) und `engine` (`contextra_impl/erasure.rs`, 619 Zeilen); Erasure-Typen in fünf Dateien (ca. 2,7k Zeilen) in `types`, `ports`, `erasure`.

---

## 2. Definition „Baustein“

Ein **Baustein** ist ein Crate, das die folgenden Eigenschaften **maschinell prüfbar** erfüllt (Klasse A, §5):

| ID | Eigenschaft | Prüfung |
|---|---|---|
| **B1** | **Keine internen Abhängigkeiten** (`contextra-*`) in `[dependencies]` und `[build-dependencies]`, auch nicht optional. | Gate G1 |
| **B2** | **Keine internen Dev-Abhängigkeiten.** Testhilfen liegen im Crate (`tests/common/` oder `#[cfg(test)]`-Modul). `contextra-testkit` ist für Klasse A verboten. | Gate G1 |
| **B3** | **API in Standardtypen** (`&[f32]`, `&[u8]`, `&str`, `u64`, `usize`, `core::time::Duration`) und crate-eigenen Typen. Kein Typ aus einem anderen `contextra-*`-Crate in der öffentlichen API. | Gate G2 (`cargo public-api`) |
| **B4** | **Ein Fehlertyp** je Crate (`thiserror`, `#[non_exhaustive]`), implementiert `std::error::Error`; kein Wiederverwenden eines globalen Fehler-Enums. | Gate G2 |
| **B5** | **Isolierter Build:** Aus leerem Verzeichnis kopiert und exportiert (§7) baut und testet das Crate mit `--no-default-features` **und** Default-Features. | Gate G3 |
| **B6** | **Schwere Abhängigkeiten hinter Features** (`wasmtime`, `candle`, `ort`, `reqwest`/`tokio`, `pyo3`, Dokumentparser, `rayon`). `default = []` oder `["std"]`. | Gate G4 |
| **B7** | **Keine Fachlogik fremder Domänen.** Indizes kennen kein DSGVO, keine Mandanten, keine Transaktions-IDs, keine Audit-Ketten (§6). | Review + `grep`-Gate G5 |
| **B8** | **Eigenständig dokumentiert und gemessen:** `README.md`, `//!`-Moduldoku, `SPEC.md` (§9.2), Unit-/Property-Tests, mindestens ein Criterion-Benchmark je öffentlicher Hauptfunktion. | Gate G6 |
| **B9** | **Zero-Panic in Produktion:** kein `unwrap`/`expect`/`panic!`/unchecked Slicing in Nicht-Test-Code (`clippy` auf `deny`); Mutex-Poisoning ist typisierter Fehler. | Clippy-Gate |
| **B10** | **`unsafe`** nur in ausgewiesenen Inseln (`simd`, `sys`, `wire`, ggf. `ingest`-FFI); jeder Block mit `// SAFETY:` und Test/Miri-Abdeckung. | `tests/unsafe_islands.rs` |
| **B11** | **Determinismus:** Zeit, Zufall und IDs kommen als Parameter oder Trait (`Clock`, `Rng`) **aus dem Crate selbst**; keine `SystemTime::now`-Aufrufe in Kernlogik. | Clippy `disallowed_methods` |
| **B12** | **Eigene Lizenzdateien** (`LICENSE-MIT`, `LICENSE-APACHE`) und `rust-version` im Manifest des Crates. | Gate G3 |

**Klasse B (Storage-Bundle)** erfüllt B3–B12, B1 nur gegenüber Klasse-B-Mitgliedern (§5.2). **Klasse C** (Anwendungsschicht) ist kein Baustein und unterliegt nur G7.

---

## 3. Leitprinzipien

1. **Vertikale Autarkie vor DRY.** Duplikation trivialer Typen (`Metric`, `ScoredId`) ist billiger als jeder Hub. Konvertierung (`From`) lebt im **Adapter**, nie im Baustein.
2. **Abhängigkeitsrichtung:** *Adapter/Komposition → Baustein.* Nie Baustein → Adapter, nie Baustein → Baustein (Klasse A).
3. **Dependency Inversion pro Konsument:** Ein Baustein definiert die schmale Trait-Oberfläche, die er **benötigt** oder **anbietet**, selbst. Es gibt keinen globalen Port-Hub für Klasse A.
4. **Statischer Dispatch** (Generics) auf Hot-Paths (SIMD, HNSW, WAND, RRF); `dyn Trait` nur an Plugin-Grenzen.
5. **Sync-Kern, Async-Schale:** Algorithmen sind synchron und laufzeitfrei. Async-Schnittstellen (`tokio`, `reqwest`) nur in Bausteinen, deren Zweck I/O ist (`infer-ollama`), und nur hinter Feature.
6. **State-Passing statt Trait-Import** für Zeit und Metriken: Parameter oder Closure (`impl Fn() -> u64`) statt Port-Abhängigkeit.
7. **Keine Löschung von Algorithmen.** Isolieren, verschieben, hinter Feature stellen – nie entfernen (§10, E-07).
8. **Compiler vor Heuristik:** `rustc`, `clippy`, `cargo metadata`, `cargo public-api` entscheiden; Text-Audits ergänzen nur. Ein nicht ausgeführtes Prüfwerkzeug ist **rot**.
9. **Evidenz vor Behauptung:** Aussagen über Zustand (grün, autark, schnell) brauchen Befehl, Commit, Exit-Code. Benchmark-Zahlen in Doku nur mit reproduzierbarem Befehl.
10. **Mandantenneutralität:** Klasse-A-Bausteine kennen keinen Tenant. Isolation erfolgt durch Instanzbildung beim Aufrufer. Die Mandantentrennung (frühere VETO-F10) gilt in Klasse B und `erasure`.

---

## 4. Fehler-, Feature- und API-Konventionen

### 4.1 Fehler
```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    // … bausteinspezifisch
}
pub type Result<T> = core::result::Result<T, Error>;
```
Kein Erzeugen von Fehlern anderer Crates über Grenzen; Konvertierung ausschließlich in Adaptern.

### 4.2 Features
| Feature | Bedeutung |
|---|---|
| `std` (Default, wo sinnvoll) | Standardbibliothek; Grundlage für späteres `no_std`/`alloc` (Kandidaten: `simd`, `rank`, `wire`). |
| `serde` | Serialisierung der öffentlichen Typen. |
| `tracing` | Optionale Instrumentierung; ohne Feature kein `tracing`-Import. |
| `inspect` | Neutrale Lesesicht auf Rohdaten/IDs für Scan- und Audit-Adapter (§6.3). |
| `experimental` | Quarantäne für nicht verdrahtete/ungetestete Algorithmen (§10, E-07). |
| Schwere Deps | je ein Feature mit dem Namen der Abhängigkeit (`wasmtime`, `candle`, `onnx`, `http`, `pdf`, …). |

`--all-features` und `--no-default-features` müssen beide bauen (Gate G4). Features sind **additiv**.

### 4.3 Index-Oberfläche (Klasse A: `vector`, `text`, `graph`)
```rust
pub trait Index<Id, Query> {
    type Hit;
    fn insert(&mut self, id: Id, /* Nutzlast */) -> Result<()>;
    fn remove(&mut self, id: &Id) -> Result<bool>;
    fn search(&self, q: &Query, k: usize) -> Result<Vec<Self::Hit>>;
}
/// Neutrale physische Wartung; keine Gesetzes-/Beweislogik.
pub trait Maintain {
    fn compact(&mut self) -> Result<MaintainStats>;   // Slots/Bytes tatsächlich freigeben
}
```
Verboten in Indizes: `commit`, `rollback`, `rollback_to_tx`, `last_tx_id`, `TxId`, `Purger`, `LayerScanner`, `WitnessLayer`, `DeletionProof`, Mandanten-IDs. Der Transaktionsvertrag ist ein **eigener Trait** in Klasse B (`Transactional`), implementiert vom Adapter.

---

## 5. Crate-Klassifikation (Zielzustand)

### 5.1 Klasse A – Isolierte Bausteine (B1–B12)

| Domäne | Crate | Ist (intern transitiv) | Zielmaßnahme |
|---|---|---:|---|
| System | `contextra-sys` | 0 | Referenz; Doku/Bench ergänzen. |
| Protokoll | `contextra-wire` | 0 | Bench ergänzen. |
| Sandbox | `contextra-sandbox` | 0 | `wasmtime` hinter Feature prüfen. |
| Vektor-Math | `contextra-simd` | 1 | `types` entfernen; lokaler `Metric`, lokaler `Error`; `bytemuck`+`thiserror` einziges Pflicht-Set. |
| Fusion | `contextra-rank` | 2 | `types`/`ports` entfernen; lokale `FusionStrategy`; `serde` hinter Feature. |
| Volltext | `contextra-text` | 2 | `types`/`ports` entfernen; Löschlogik raus (§6); Index-Oberfläche §4.3. |
| Vektorindex | `contextra-vector` | 5 | `mvcc`/`sys`/`types`/`ports` entfernen; `simd` nur hinter Feature oder per Trait-Parameter; Löschlogik raus; VETO-F02 (Tombstone+Slot-Nullung+Rebuild) bleibt Invariante des Crates. |
| Graph | `contextra-graph` | 2 | `types`/`ports` entfernen; Kaskaden-Löschfunktionen (`cascade_invalidate_*`, `process_cascade_queue`) nach §6; Funktionsinventar vor Verschiebung. |
| Cache | `contextra-kvcache` | 3 | `types`/`ports`/`crypto` entfernen; AEAD optional hinter Feature mit eigener Trait-Schnittstelle; `ShreddableSegmentKey` → neutraler `SegmentKey::destroy()`. |
| Crypto | `contextra-crypto` | 2 | Nur Primitive (AEAD, KDF, HMAC/Chain, Signatur, Anti-Tamper, Secret); Löschbeweis-Code nach `erasure` (§6.2). |
| Privacy | `contextra-privacy` | 2 | `types`/`ports`/`crypto` entfernen; reine PII-Erkennung. |
| Lizenz | `contextra-license` | 2 | `types`/`ports` entfernen; bleibt erhalten (Beratungsempfehlung „entfernen“ abgelehnt, E-08). |
| Fs | `contextra-durable-fs` | 3 | Lokaler `Vfs`-Trait; `sys` optional hinter Feature; `ports`/`types` entfernen. |
| Ingest | `contextra-ingest` | 1 | `types` entfernen (billigster Gewinn); Parser je Format hinter Feature (`pdf`, `docx`, `xlsx`, `html`, `email`). |
| Compliance | `contextra-audit-export`, `contextra-avv-generator` | 1 | `types` entfernen; bleiben erhalten (E-08). |
| Adaption | `contextra-adapt` | 2 | `types`/`ports` entfernen. |
| Inferenz | `contextra-infer-onnx`, `-ollama`, `-candle` | 2–3 | Je ein lokaler `Embedder`/`Generator`-Trait; `rank`/`crypto`/`ports`/`types` entfernen; Backend hinter Feature. |
| Löschkern | `contextra-erasure` | 3 | Siehe §6; ungenutzte `crypto`-Dep entfernen. |
| Routing | `contextra-router` | 4 | Entkoppeln (`types`, `wire`, `ports`, `adapt`); bis dahin Klasse C. |

### 5.2 Klasse B – Storage-Bundle (gemeinsam kopiert)

`contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-store`, `contextra-checkpoint`.

- Gelten als **eine Einheit**: `docs/bundle-storage.md` listet sie, ihre gegenseitigen Kanten und den Export (`tools/export-bundle storage`).
- `types` und `ports` bleiben **nur** für dieses Bundle bestehen. Klasse A importiert sie nicht (B1).
- `ports` verliert alle Erasure-Typen (§6) und die an Indizes gekoppelten Methoden (§4.3); `Transactional` lebt hier.
- `mvcc`/`checkpoint`: `Clock`/Metriken per State-Passing (Prinzip 6) statt Port-Import.
- `store` (Ist ca. 28k LOC): interne Zerlegung in `wal`, `memtable`, `sstable`, `lsm` **als Module mit klaren Grenzen**; Aufspaltung in Crates erst nach Inventar (E-06) und nur, wenn je Teil Gate G3 bestanden wird.

### 5.3 Klasse C – Anwendungsschicht (kein Baustein)

`contextra`, `contextra-db`, `contextra-engine`, `contextra-cognition`, `contextra-agent`, `contextra-mcp`, `contextra-py`, `contextra-router` (bis entkoppelt).

- Referenz-Komposition und Beispielquelle, **nicht** Kopierziel.
- Dürfen alles verdrahten; dürfen **keine** Fachlogik behalten, die einem Baustein gehört (§10, E-05).
- Pflicht-Dependencies auf alle Bausteine entfallen: generische Backends, Features je Subsystem. Gate G7: Minimal-Komposition (z. B. `text` + `rank` ohne `store`) kompiliert.

### 5.4 Klasse T – Testwerkzeuge
`contextra-testkit` ist Dev-only und von Klasse A verboten. Für Klasse B/C erlaubt. `contextra-bench` ist reines Messwerkzeug.

### 5.5 Entfernt / archiviert
- `crates/contextra-core` (Orphan, kein Workspace-Member): nach Git-Tag `archive/contextra-core-final` und Inventarprüfung (WP-0) entfernen.
- `docs/meta/` Duplikate (`ENTKOPPEL-Prompts.md` ≡ `MODULAR_DECOUPLING_PROMPTS.md`, ältere Audit-Kopie) entfernen.

---

## 6. Löschlogik und DSGVO

### 6.1 Zuständigkeit
**Alle** Löschketten, Beweis-, Nachweis- und DSGVO-Logik gehört in `contextra-erasure` (bzw. §6.3). Datenhaltende Bausteine bieten nur §4.3.

### 6.2 `contextra-erasure` (autark, Klasse A)
Enthält: Saga-Zustandsmaschine, `FileVault`/Schlüsselverwahrung, ReadGate/GateLog, RestoreGuard, Messung/Witness-Aggregation, `DeletionReport` (Ed25519-signiert), Revocation-Log, kryptografisches Shredding, Löschbeweis-Format.

- **Eigene Traits** `Purger`, `LayerScanner`, `KeyScope`, `WitnessLayer`, `ErasureId`, `PurgeOutcome` – aus `ports`/`types` hierher verschoben. Es gibt **eine** Definition je Typ (heute fünf Dateien).
- **Importiert keine Index-/Store-Crates.** `crypto` nur, wenn tatsächlich genutzt; sonst lokale Primitive (`crypto_local.rs` existiert bereits). Aus `types`/`ports` nichts.
- Bestehende Löschinvarianten (Ebene A physisch + Ebene B Schlüsselvernichtung wirken nur gemeinsam; Berichtsformat V5; Wirkungsklassen; `INV-WITNESS-MINT-1`) **bleiben erhalten** und werden mit nach `crates/contextra-erasure/SPEC.md` übernommen. Verschieben ändert keine Semantik.

### 6.3 `contextra-erasure-adapters` (neues Crate, Klasse A-Adapter)
- Je Backend ein Feature: `vector`, `text`, `graph`, `store`, `kvcache`.
- Implementiert `Purger`/`LayerScanner` **für** die Indextypen, indem es deren §4.3-Oberfläche plus `inspect`-Feature nutzt.
- Abhängigkeitsrichtung: `adapters → {erasure, Baustein}`. Weder `erasure` noch Bausteine kennen den Adapter.
- Ersetzt die nicht existierenden `contextra-erasure-spi` / `contextra-erasure-text`.

### 6.4 Physische Löschwirkung (Entscheidung E-04)
`remove` entfernt logisch. **Physische Wirkung** liefert die neutrale Wartung `Maintain::compact()` bzw. crate-spezifisch `reclaim()` (mmap-Graph, WAL-Segmente, SSTables, Slots) **in Kombination** mit Schlüsselvernichtung durch `erasure` (bestehende Doktrin Ebene A + B). Kryptografisches Shredding **allein** ersetzt die physische Bereinigung nicht und erzeugt nie allein eine Löschbehauptung. *Ob dies rechtlich für Art. 17 DSGVO genügt, ist nicht beurteilt und liegt außerhalb dieser Spezifikation.*

### 6.5 Migrationsreihenfolge (jeweils kompilierbar und getestet)
1. `erasure`: ungenutzte `crypto`-Dep entfernen (Einzeiler).
2. Typen/Ports in `erasure` bündeln; `types`/`ports` re-exportieren **temporär** mit `#[deprecated]`.
3. `crypto/deletion_proof`, `ed25519_proof`, `kv_shredding`, `revocation_log` nach `erasure` verschieben (nach Inventar, §10 E-06).
4. `engine/contextra_impl/erasure.rs` nach `erasure`/`erasure-adapters`.
5. `Purger`/`LayerScanner`-Implementierungen aus `vector`, `text`, `graph`, `store`, `kvcache` in `erasure-adapters`; Index-Crates erhalten `Maintain` + `inspect`.
6. Temporäre Re-Exporte entfernen; Warnungen `use of deprecated` = 0.

Abnahme: `grep -riE "witness|purger|layerscanner|deletion_?proof|shred|erasure"` in `vector`, `text`, `graph`, `store`, `kvcache`, `crypto` = 0 Treffer (außer dokumentierte `Maintain`-Texte).

---

## 7. Export (Copy-Paste-Readiness)

Das Workspace-Erbe (`workspace = true`) verhindert heute das bloße Kopieren. Pflichtwerkzeug:

**`tools/export-crate <name> [--out DIR]`**
1. Prüft Klassenzugehörigkeit und B1/B2 (bricht bei Verstoß ab).
2. Kopiert `Cargo.toml`, `src/`, `tests/`, `benches/`, `examples/`, `README.md`, `SPEC.md`, Lizenzdateien.
3. **Löst Workspace-Vererbung auf** (Version, Edition, Lints, `rust-version`, Dependencies mit expliziten Versionen aus `Cargo.lock`/`[workspace.dependencies]`).
4. Schreibt `rust-toolchain.toml`-freies, eigenständiges Manifest.
5. Baut und testet im leeren Verzeichnis (`cargo test --no-default-features`, danach Default, danach `--all-features`).

`tools/export-bundle storage` erzeugt dasselbe für Klasse B als Mini-Workspace.

*Nicht verifiziert:* ob `cargo package` die Vererbung zuverlässig auflöst; es gilt als Alternative erst nach Test. Lösung ist selbst ein Testobjekt (Meta-Gate G8).

---

## 8. Gates und Kennzahlen

### 8.1 Gate-Tabelle (CI-Pflicht, rot bei Nichtausführung)

| Gate | Prüft | Quelle |
|---|---|---|
| **G1** Isolation | Je Crate Soll-Liste interner Deps (Klasse A: leer, B: Bundle-Liste) in `capabilities.toml` (`may_depend_on`); Ist aus `cargo metadata` (normal+build+dev, mit aufgelösten Features). | `tests/layering.rs`, `tools/` |
| **G2** API | `cargo public-api`: keine `contextra-*`-Typen in Klasse-A-APIs; `#[non_exhaustive]`-Fehler. | `cargo public-api` |
| **G3** Export | `export-crate` + Build/Test im leeren Verzeichnis. | `tools/export-crate` |
| **G4** Features | `--no-default-features` und `--all-features` bauen und testen; schwere Deps nur hinter Feature (`cargo tree -e normal` ohne Feature). | CI-Matrix |
| **G5** Domänenreinheit | `grep`-Verbotsliste (§4.3, §6.5) je Crate. | `tools/` |
| **G6** Qualität | `README`, `SPEC.md`, Doc-Tests, Benchmarks vorhanden und kompilierbar (`cargo bench --no-run`). | CI |
| **G7** Komposition | Minimal-Kompositionen kompilieren (§5.3). | CI |
| **G8** Meta | Gate-Skripte haben eigene Negativtests (absichtlicher Verstoß → rot). | `tests/` |
| **G9** Warnungen | `RUSTFLAGS="-D warnings"`; `#[deprecated]`-Nutzung = 0. | CI |
| **G10** Unsafe/Panic | `tests/unsafe_islands.rs`, Clippy-Lints auf `deny`. | CI |

**Bekannte Gate-Schwäche (zuerst beheben):** Der bisherige Abhängigkeitsreport führt den Ring je Crate als `?` und meldet „0 Verletzungen“ ohne Prüfung. G1 ersetzt ihn; `capabilities.toml` wird auf Klassen (A/B/C/T) und `may_depend_on` umgestellt (Ringe nur noch für Klasse B/C).

### 8.2 Ratchet
Grenzwerte (Warnungen, Panic-Stellen, Dep-Zahl, Benchmark-Regression) stehen in `governance/ratchet.toml` und dürfen nur besser werden.

### 8.3 Generierte Kennzahlen
`tools/loc.py`, `cargo_dependency_graph.py`, `abhaengigkeits_budget.py` erzeugen `docs/generated/baukasten-status.md` (Klasse, interne/externe Deps, LOC, Features, Test-/Bench-Anzahl). Handschriftliche Scores, Crate-Zählungen („35/36/37“), LOC- und Relikt-Zahlen in Fließtext sind verboten.

---

## 9. Qualität pro Baustein

### 9.1 Tests (mikrofeingranular)
- **Unit-Tests** je öffentlicher Funktion und je Fehlerpfad.
- **Property-Tests** (`proptest`) für Algorithmen mit Invarianten (RRF-Determinismus, Metrik-Symmetrie, Index-Recall vs. Brute Force).
- **Orakel** je Kerngarantie: unabhängiges Referenzmodell (Brute-Force-Suche, Naiv-BM25, Skalar-Distanz).
- **Fault-Injection** lokal im Crate für I/O-Bausteine (`durable-fs`, `store`), ohne `testkit`.
- Jede dokumentierte Invariante `INV-*` hat einen Test, dessen Verletzung rot wird.

### 9.2 Dokumentation
Je Crate:
- `README.md`: Zweck, **Copy-Paste-Anleitung** (Export-Befehl, benötigte Features), Abhängigkeitsliste (extern), Minimalbeispiel, Nicht-Ziele.
- `SPEC.md` (kurz): Verhalten, Invarianten, Konstanten, Fehlermodell, Concurrency-Garantien, Durability-Garantien (falls zutreffend).
- `//!` Moduldoku, `# Errors`/`# Panics`/`# Safety` je öffentlicher Funktion.
- `examples/` mit lauffähigem Minimalbeispiel.

### 9.3 Benchmarks
- Criterion-Benchmarks je Hauptfunktion; Ziel: alle Klasse-A/B-Crates (Ist 11/36).
- Veröffentlichte Zahl nennt Commit, Hardware, Workload, Vergleichsbasis. **Nicht verifizierte Zahlen** aus v3.0/`BAUSTEIN_DOKUMENTATION_UND_BENCHMARKS.md` (z. B. „<40 ns Cosine-768“, „>500k docs/s“) werden bis zur Messung gestrichen.
- Prioritär fehlend: `text`, `rank`, `kvcache`, `wire`, `ingest`, `sys`, `sandbox`, `durable-fs`, `privacy`, `license`.

### 9.4 Concurrency und Durability
- Concurrency-Modell je Baustein dokumentiert (Send/Sync-Garantien, Lock-Reihenfolge, lock-frei oder nicht).
- Keine Locks über `await`; Stichprobenprüfung der in der Diagnose gemeldeten Stellen (`engine` ~40, `checkpoint` ~23, `erasure` ~20) – *Heuristik, unverifiziert*.
- Durability-Verträge (fsync-Semantik, atomares Rename, Fsync-Poisoning `INV-FSYNC-1`) gehören in `durable-fs`/`store`-SPEC und sind mit Crash-Tests belegt; der gemeldete verworfene `fsync` in `crypto` ist zu prüfen (*unverifiziert*).

---

## 10. Entscheidungsregister (bindend, ADR-pflichtig bei Änderung)

| ID | Entscheidung | Verworfene Alternative | Begründung |
|---|---|---|---|
| **E-01** | **Drei Klassen A/B/C** (+T). Alle 36 Crates in 3 Tagen vollständig zu entkoppeln ist unrealistisch; Klasse B wird bewusst als Einheit geführt. | Vollständige Auflösung von `types`/`ports` für alle. | Hebel/Risiko; Storage-Kern teilt echte Semantik (`TxId`, Snapshot). |
| **E-02** | **`types` und `ports` bleiben, aber nur für Klasse B.** Klasse A importiert sie nicht. | Globale Auflösung; oder Hub behalten. | Löst Copy-Paste-Problem, ohne Storage-Semantik zu duplizieren. |
| **E-03** | **Adapter-Prinzip:** Verbindung zwischen Baustein und Port/Fremdtyp liegt in Adapter-Crates/Features; Abhängigkeit zeigt vom Adapter zum Baustein. | Port-Impl im Baustein. | Baustein bleibt autark. |
| **E-04** | **Löschung:** Logik zentral in `contextra-erasure`; Anbindung über `contextra-erasure-adapters`; Indizes bieten nur `insert/remove/search` + `Maintain::compact()` (+ `inspect`). Physische Bereinigung + Schlüsselvernichtung wirken gemeinsam. | `erasure` importiert Indizes; Shredding-only. | `erasure` bleibt autark; `remove` allein löscht physisch nichts. |
| **E-05** | **`engine` und Klasse C:** Algorithmen werden inventarisiert und in Bausteine verschoben, nichts gelöscht; `engine` wird dünne Komposition. | `engine` einfrieren; oder löschen. | Algorithmen sind Wert; Monolith ist nicht entnehmbar. |
| **E-06** | **Inventar vor Verschiebung:** Für `engine` (≈30k LOC), `crypto/deletion_proof`, `store` liegt vor jeder Verschiebung eine Funktionsliste Funktion → Zielbaustein vor (`docs/generated/funktionsinventar.md`), geprüft per Diff nach der Verschiebung (keine verlorene öffentliche Funktion). | Verschieben nach Augenmaß. | Schutz vor stillem Funktionsverlust. |
| **E-07** | **„Kein Algorithmus wird gelöscht“ ersetzt „Zero Dead Code“.** Nicht verdrahteter Code erhält Feature `experimental` und Test oder Verweis im Inventar. | Dead-Code-Pruner automatisch (löscht). | Vorgabe des Eigentümers; Scanner meldet viele nicht verdrahtete Algorithmen. |
| **E-08** | **Keine Fusion/Archivierung** von `license`, `erasure`, `audit-export`, `avv-generator` und keine Verschmelzung zu 7 Crates. | Empfehlung der Beratung. | Widerspricht feingranularer Baustein-Vorgabe. |
| **E-09** | **Index-Traits getrennt:** Kern (`insert/remove/search`), `Maintain`, `Transactional` (Klasse B). | Ein Trait mit allem. | Index ohne MVCC nutzbar. |
| **E-10** | **Export per eigenem Tool** (`export-crate`), nicht per Handarbeit. | `cargo package`-Annahme. | Vererbung muss belegt aufgelöst werden. |
| **E-11** | **Quelle der Wahrheit:** dieses Dokument + generierte Kennzahlen. `AGENTS.md` verweist hierher; widersprüchliche Altdokumente werden archiviert. | Parallele Dokumente. | Aktuell widersprüchliche Zahlen/Aussagen. |
| **E-12** | **Test-Hilfen lokal** in Klasse A; `testkit` nur für B/C. | `testkit` überall. | Heute in 11 Crates Dev-Dep; bricht B2/B5. |

---

## 11. Offene Entscheidungen des Eigentümers

| ID | Frage | Default, falls keine Antwort |
|---|---|---|
| **O-1** | `no_std`/`alloc`-Fähigkeit für `simd`, `rank`, `wire` (Robotik/Embedded)? | Nein in v4.0; `std`-Feature als Vorbereitung. |
| **O-2** | MSRV je Baustein niedriger als 1.89? | Nein; 1.89 einheitlich. |
| **O-3** | `store` in Crates aufteilen (`wal`/`memtable`/`sstable`/`lsm`)? | Nur Modulgrenzen in v4.0, Crate-Split nach E-06. |
| **O-4** | Rechtliche Würdigung der Löschwirkung (Art. 17 DSGVO)? | Außerhalb dieser Spezifikation; Dokumente tragen keine Rechtsaussage. |
| **O-5** | `contextra-router` als Baustein (Klasse A) oder Komposition (C)? | C bis entkoppelt. |

---

## 12. Arbeitspakete (9.–11. November, Akzeptanz jeweils maschinell)

### WP-0 Leitplanken (Mo, Vormittag)
- `AGENTS.md`: Merge-Marker (Z. 244/262/282) auflösen; Verweis auf diese SPEC.
- `docs/meta/` Duplikate entfernen; Jules-Prompts umschreiben: „Zero Dead Code“ streichen, aktuelle §1-Befunde statt Scores vom 6.11.
- Altdokumente (`ZIELARCHITEKTUR_BAUKASTEN`, `…_ABSCHLUSSBERICHT`, `MODULAR_BAUKASTEN_ANALYSEBERICHT`, `BAUSTEIN_DOKUMENTATION_UND_BENCHMARKS`) nach `docs/archive/` mit Kopfzeile „nicht normativ, widerlegt durch SPEC v4.0 §1“.
- `capabilities.toml` auf Klassen/`may_depend_on` umstellen; **Gate G1** implementieren (rot für heutige Verstöße).
- Funktionsinventar-Generator (E-06) für `engine`, `crypto`, `store` lauffähig.
- **Abnahme:** G1 meldet exakt die Verstöße aus §1; `cargo check --workspace` unverändert grün.

### WP-1 Billige Blätter (Mo)
- `simd`, `rank`, `durable-fs`, `license`, `privacy`, `audit-export`, `avv-generator`, `ingest`: interne Deps → 0, lokaler Fehler/Typen, Adapter-Feature `types-compat` wo Bestandsnutzer existieren.
- `tools/export-crate` + G3 für diese Crates.
- **Abnahme:** `cargo tree -p X -e normal` ohne `contextra-*`; G3 grün; vorhandene Tests grün.

### WP-2 Löschlogik (Di)
- §6.5 Schritte 1–6. Funktionsinventar-Diff: keine verlorene öffentliche Funktion.
- Neues Crate `contextra-erasure-adapters`.
- **Abnahme:** §6.5 Abnahme-`grep` = 0; `deprecated`-Warnungen 0; Erasure-Tests grün.

### WP-3 Große Bausteine (Di–Mi)
- `rank`/`text` (bereits WP-1/2 teils), danach `vector` (mechanisch, 38/45 Dateien), `graph`, `kvcache`, `crypto`, `adapt`, `infer-*`.
- Index-Traits trennen (E-09); `Transactional` nach Klasse B.
- **Abnahme:** G1/G2/G3/G4 grün für alle Klasse-A-Crates außer `router`.

### WP-4 Storage-Bundle und Komposition (Mi)
- `mvcc`/`checkpoint` State-Passing; Bundle-Dokument und `export-bundle`.
- `engine`/`db`/`contextra`: Features je Subsystem; Minimal-Komposition G7.
- **Abnahme:** G7 grün; Bundle-Export baut isoliert.

### WP-5 Messung und Doku (Mi)
- `README.md`, `SPEC.md`, Examples je Baustein; Benchmarks für die Prioritätsliste (§9.3).
- Nicht verifizierte Benchmark-Zahlen aus Altdokumenten entfernt.
- **Abnahme:** G6 grün; `docs/generated/baukasten-status.md` erzeugt.

### WP-6 Restschulden (nach 11.11.)
- Stichprobenprüfung Heuristik-Funde (Panics, Locks über `await`, `fsync` in `crypto`).
- Entscheidungen O-1…O-5.
- `store`-Split nach E-06/O-3.

---

## 13. Änderungsverlauf

| Fassung | Datum | Änderung |
|---|---|---|
| 4.0 | 2026-11-08 | Vollständige Neufassung: Produkt-/Ökosystem-Spezifikation (v3.0) ersetzt durch Baukasten-Spezifikation. Klassen A/B/C/T, Bausteindefinition B1–B12, Adapter-Prinzip, Löschlogik-Umzug, Export, Gates G1–G10, Entscheidungen E-01–E-12. Korrektur der unbelegten Aussagen des Abschlussberichts. |
