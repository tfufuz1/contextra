# contextra-testkit — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-testkit/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Testwerkzeuge: Konformitätssuiten, Fakes, FaultVfs. Nur E3, nie im Kern. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | testing |
| Ebene (Kern-Closure) | 3 |
| Kern-Closure | ports, types, env |
| Default-Features | `testkit` (ohne `erasure-conformance`) |
| Fehlertyp (R3) | — |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 2723 Zeilen in 21 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | ports, types |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 3 |
| Default-Features (Ist) | testkit, erasure-conformance |
| `cfg(feature = …)`-Stellen in src/ | 1 |
| `#[test]`-Vorkommen (grep, src + tests) | 24 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 0) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | keine |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** keine
- **Ziel-Kanten, die im Ist fehlen:** `env`
- **Default-Features weichen ab:** Ist `testkit, erasure-conformance` → Ziel `testkit`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: Default-Features Ist = `testkit`, `erasure-conformance`. Das Ziel (§3.1) verlangt nur `testkit`.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

`conformance/{clock,rng,metrics}.rs` und `manual_clock.rs` → `env` (`test-utils`); `conformance/vfs.rs` und `fault_vfs.rs` **bleiben** (B08); `erasure-conformance` raus aus `default`, Suite → `erasure` (Feature `conformance`). `ports/conformance.rs`, `contract_tests.rs` hierher.
---

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

Keine Befunde in den Audit-Berichten.

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B08 — `Vfs` kann nicht verschoben werden, ohne den Fehlertyp zu ändern**
> **Entscheidung:** `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` bleiben in `ports`. `durable-fs` bleibt eigenständig (E0, Tier experimental, kein Verwender). Die Plan-Schritte „`io_seam`/`fsync.rs` nach `durable-fs`“ und „`sys-flush`“ entfallen (Nicht-Ziel). Verbraucher mit Dateisystembedarf (`store`, `checkpoint`) halten eine **crate-lokale** schmale Naht (siehe §6), die Brücke bindet an `ports::Vfs`.

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen**
> **Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-testkit --no-default-features --locked
cargo hack check -p contextra-testkit --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-testkit --modes no-default,default,each-feature
```

