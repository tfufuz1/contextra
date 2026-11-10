# contextra-cognition — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-cognition/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Konsolidierung und Aggregation von Erinnerungen. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | agentic |
| Ebene (Kern-Closure) | 2 |
| Kern-Closure | types, ports, graph (nur `exp-hyperedge`) |
| Default-Features | — |
| Fehlertyp (R3) | `CognitionError` (neu) |
| Brücke (hinter Feature `workspace`, §1.1) | Bindung an `engine` liegt in **`db`** (Feature `cognition`) |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 5377 Zeilen in 25 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | engine, types, ports, graph |
| Interne Dev-Abhängigkeiten (Ist) | testkit |
| Features (Ist, Anzahl) | 3 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 11 |
| `#[test]`-Vorkommen (grep, src + tests) | 52 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 0) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | keine |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `engine`
- **Ziel-Kanten, die im Ist fehlen:** keine
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert (Cargo.toml): interne Normal-Abhängigkeit auf `contextra-engine`. Das widerspricht dem Ziel (Kern: types, ports, graph). Die Umkehrung ist in S-63 geplant.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Lokale Ports `DocumentSource`, `ConsolidationSink` (inkl. `CommitIntent`-Ersatztyp, falls der Port-Zuschnitt scheitert), `HyperedgeSink` (`graph_sink.rs` als Ausgangspunkt), `DecayPolicy`; `CognitionError` statt `ContextraError::{Internal, PolicyViolation, InvalidInput}` (~30 Stellen); engine-gebundene Dateien (`consolidation_executor.rs`, `context.rs`, `semantic_aggregation_facade.rs`) → Brücke **im Crate `db`** (Feature `cognition`), damit `cognition` kein `compat/`-Zielcrate `engine` braucht. Features `graph-connectivity-health`, `edge-reinforcement-learning` leiten danach nur an `graph` weiter. Löschen der `*_local.rs`-Umbenennungsdateien (G7).

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `CognitionError` (neu) (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

Keine Befunde in den Audit-Berichten.

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

## 10. Entscheidungen und Risiken [S]

**Risiken (§9.2):**

| `cognition`-Port-Zuschnitt (`CommitIntent` aus `engine::transaction`, 1.652 Z. [E]) | S-63 | Port scheitert ⇒ `CommitIntent` als eigener Typ in `cognition`, Abbildung in `db`-Brücke |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-cognition --no-default-features --locked
cargo hack check -p contextra-cognition --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-cognition --modes no-default,default,each-feature
```
