# contextra-types — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-types/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Gemeinsame Domänentypen und `ContextraError` (43 Varianten). Übergangsträger, wird über E1/E2 abgebaut. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | contracts |
| Ebene (Kern-Closure) | 2 |
| Kern-Closure | ids |
| Default-Features | — |
| Fehlertyp (R3) | `ContextraError` |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 4966 Zeilen in 22 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | keine |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 3 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 12 |
| `#[test]`-Vorkommen (grep, src + tests) | 186 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 0) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | HnswDeletionError, ContextraError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** keine
- **Ziel-Kanten, die im Ist fehlen:** `ids`
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: `DistanceMetric` in `src/types/domain/document.rs:20` (B09). `ModelFingerprint` wird in `src/lib.rs:74` re-exportiert. In `src/lib.rs` steht die Verwenderliste mit `contextra-agreement-generator`, das im Workspace nicht existiert (B06 bestätigt).
- Tombstone: siehe Befund bei `mvcc` (Modul-Imports 4 Stellen, Symbole 14 Crates).

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Bleibt: `error.rs` (`ContextraError`, 43 Varianten), `error_dto.rs`, `types/{budget,filter,importance,saos}.rs`, `types/domain/document.rs`, `types/domain/misc.rs` (abzüglich Auszüge), `schema.rs`, `RequestContext` aus `witness.rs`, `model_fingerprint.rs`, `retrieval_strategy.rs`. Geht: IDs/Refs/Tenant → `ids`; `tombstone.rs` → `mvcc`; `erasure.rs`, `SagaRecord`, `ScanMethod`, `ErasureId`, `GateId` → `erasure`; `PprConfig`/`PprAlgorithm`/`TlHfdParams` → `graph`. `lib.rs` Zeilen 20–28 (Verwenderliste) ersetzen durch generierten Verweis. Varianten `Sandbox*`, `Cluster`, `KvQuantization`, `KvDeleteModeConfig`, `HnswConnectivityDegraded`, `GraphRepairFailed` bleiben, **bis** alle lokalen Fehler existieren **und** G-E1 grün ist; erst dann entfernen (S-70), weil `#[non_exhaustive]` das Entfernen zu einer Verhaltensänderung für `Display`-Konsumenten macht.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `ContextraError` (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Fehler-Enums (Ist):**

- `crates/contextra-types/src/error.rs:38` — HnswDeletionError
- `crates/contextra-types/src/error.rs:129` — ContextraError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B03 — `docid-128` ändert die Typbreite und ist nicht additiv**
> **Entscheidung:** 1. `ids` besitzt `docid-128` allein. Jeder Crate mit Abhängigkeit auf `ids` leitet `docid-128 = ["contextra-ids/docid-128"]` weiter; leere/lokale Features entfallen. E0-Crates ohne `ids` (`ingest`, `erasure`) bekommen **kein** `docid-128`; ihre Konvertierung von `ids::DocId` ist `TryFrom` (fehlschlagend bei Überlauf), nie `as`. 2. `ids` führt `pub const ID_WIDTH_BITS: u32` (64 oder 128) und die Compile-Time-Prüfung `size_of`/`align_of` (8/8 bzw. 16/16). 3. Jedes persistierte Format, das IDs trägt, MUSS die Breite beim Öffnen prüfen (§5.2). Fehlt das Feld heute, wird die Formatversion erhöht und v(alt) als „Breite unbekannt“ nur mit expliziter Konfiguration geöffnet. 4. CI-Matrix: `cargo hack check --workspace --features docid-128` zusätzlich zum Default.

**B06 — `capabilities.toml` und `AGENTS.md` widersprechen sich**
> **Entscheidung:** Schema v3 (§8.3) ersetzt `klasse/ring`. `may_depend_on` wird aus dem Ziel (§3.1) erzeugt, nicht aus dem Ist.

**B09 — `ports` und `types` ohne Zielbild für `DistanceMetric`; mehrere Distanz-Implementierungen**
> **Entscheidung (R4-Ausnahme):** Die drei Enums bleiben je im Besitz ihres Crates. Erlaubt sind kleine geschlossene Wert-Enums (≤ 8 Unit-Varianten) als **gespiegelte Kopie**, wenn (a) Variantennamen und -reihenfolge identisch sind und (b) ein Konformitätstest die serde-Namen aller Kopien gleichsetzt. Verbindung per `From` in `compat/`. `vector` bekommt das optionale Feature `simd`, das `compute_distance` durch `contextra_simd::dispatch` ersetzt; die Skalarpfade bleiben als Orakel. Differenztest §5.4 (Toleranz relativ 1e-5, f32).

**B12 — Die Fehlerabbildung ist unterspezifiziert**
> **Entscheidung:** §5.3 (Abbildungsvertrag) und S-10 mit separatem Aufrufstellen-Commit. Ausgangspunkt der Tabelle: **Display-Text und Variante bleiben bitgleich** (Test je Variante).

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `ModelFingerprint` (Plan: → `kvcache`; Teil 2 K9: → `ports/kv`) — `ports/kv.rs` importiert ihn aus `types`. Alle Verbraucher der Typ-Variante sind Brücken oder Re-Exports (`crypto` wird auf Roh-Bytes umgestellt). — **Bleibt in `types`.** Kein Umzug. `kvcache` und `infer-candle` führen im Kern einen …
> `RetrievalStrategy` — `types` und `adapt` führen verschiedene Gestalten — Bleibt in `types` (K10); `router/arm_registry.rs` bildet ab.

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-03 | `ModelFingerprint`/`RetrievalStrategy` bleiben in `types` (B16) |

**Risiken (§9.2):**

| `ids`-Umzug ändert Rückgabetypen (B12) | S-10 C | Mehr als ~20 nicht-mechanische Aufrufstellen ⇒ Rückfall: `types` behält `DocId::from_key` als freie Funktion `types::doc_id_from_key` plus Methode via Extension-Trait (Trait-Import nötig) — nur mit ADR |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-types --no-default-features --locked
cargo hack check -p contextra-types --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-types --modes no-default,default,each-feature
```

