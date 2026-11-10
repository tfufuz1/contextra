# contextra-checkpoint — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-checkpoint/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Snapshot-/Checkpoint-Mechanik (Hardlinks, Meta, Manifest). *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | retrieval |
| Ebene (Kern-Closure) | 1 |
| Kern-Closure | ids, env |
| Default-Features | — |
| Fehlertyp (R3) | `CheckpointError` (neu) |
| Brücke (hinter Feature `workspace`, §1.1) | `Checkpoint`-Impls, `From` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 3614 Zeilen in 12 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports |
| Interne Dev-Abhängigkeiten (Ist) | testkit |
| Features (Ist, Anzahl) | 0 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 2 |
| `#[test]`-Vorkommen (grep, src + tests) | 48 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 39; 39 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 8) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | keine |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `types`, `ports`
- **Ziel-Kanten, die im Ist fehlen:** `ids`, `env`
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert (Cargo.toml): interne Abhängigkeiten nur auf `types`, `ports`. Die Ziel-Kern-Closure verlangt `ids`, `env`; dieser Schnitt ist in S-35 geplant.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

lokales Trait `SnapshotSource` = exakt die in `src/` aufgerufenen `StorageEngine`-Methoden (29 Referenzen [E]); lokale schmale Dateisystemnaht (`hardlink_cloner.rs`) statt `ports::Vfs` (B08); `CheckpointError`; `WorkflowState` lokal abbilden; Brücke: `ports::{Checkpoint, CheckpointCoordinator}`-Impls, `From`. Kern-Closure: ids, env. Nicht mit `mvcc` zusammenlegen.

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.4 ID-Breite [S]

Persistierte Formate mit IDs tragen `id_width_bits: u16` (64|128). Öffnen mit abweichender Breite ⇒ `…::IdWidthMismatch { stored, build }`, kein Panic, keine Teilmutation (docs/SPEC.md §5.2, B03).

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `CheckpointError` (neu) (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

### 7.1 Persistierte Formate (§5.1 Golden-Korpus) [S]

| Format | Quelle / Kennung |
|---|---|
| Text-Postings, Graph-Persistenz (bincode), Checkpoint-Meta/Manifest | aus `text`, `graph`, `checkpoint` zu erheben |

Jeder Eintrag hat einen Erzeugertest (`#[ignore]`, Script `tools/golden_regen.sh`) und einen Prüftest, der immer läuft. Abweichung ⇒ Schritt abbrechen.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 39, davon 39 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).


**Determinismus (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 8.


## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B04 — Entkopplungsstand von `text` ist falsch eingeschätzt; Zählungen weichen ab**
> **Entscheidung:** Zahlen sind Orientierung. Abnahme erfolgt ausschließlich über die Null-Prüfung G-L2 (§8.2) und den Build ohne Features, nie über Zählwerte. Aufwandsklasse `text`: mittel.

**B08 — `Vfs` kann nicht verschoben werden, ohne den Fehlertyp zu ändern**
> **Entscheidung:** `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` bleiben in `ports`. `durable-fs` bleibt eigenständig (E0, Tier experimental, kein Verwender). Die Plan-Schritte „`io_seam`/`fsync.rs` nach `durable-fs`“ und „`sys-flush`“ entfallen (Nicht-Ziel). Verbraucher mit Dateisystembedarf (`store`, `checkpoint`) halten eine **crate-lokale** schmale Naht (siehe §6), die Brücke bindet an `ports::Vfs`.

**B11 — Nebenläufigkeits-Abdeckung fehlt genau dort, wo umgebaut wird**
> **Entscheidung:** S-01 nimmt `mvcc`, `kvcache`, `checkpoint`, `ids`, `env` in die Pfadfilter von `tsan.yml`/`loom-gate.yml`, `mvcc`+`kvcache` in die Crate-Liste der Mutation-Tests und `rust-ci.yml` auf. Enthält `crates/contextra-mvcc/tests` kein Loom-Modell (`rg -l loom crates/contextra-mvcc` leer), ist vor S-30 ein Modell für `SequenceLog` (Publish/Visibility, zwei Schreiber, ein Leser) zu schreiben; sonst ist S-30 nicht abnehmbar.

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> Optionale Plan-Schritte `store→durable-fs`, `checkpoint` in `mvcc` — — — Entfallen (B08) bzw. abgelehnt (Datei-Snapshot-Mechanik ≠ Transaktionsisolation).

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-checkpoint --no-default-features --locked
cargo hack check -p contextra-checkpoint --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-checkpoint --modes no-default,default,each-feature
```

