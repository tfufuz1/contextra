# contextra-wire — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-wire/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

FlatBuffers- und JSON-RPC-Wire-Format. Enthält generierten Code. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | foundation |
| Ebene (Kern-Closure) | 0 |
| Kern-Closure | — |
| Default-Features | `std` |
| Fehlertyp (R3) | besteht |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 2474 Zeilen in 11 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | keine |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 5 |
| Default-Features (Ist) | std |
| `cfg(feature = …)`-Stellen in src/ | 0 |
| `#[test]`-Vorkommen (grep, src + tests) | 110 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 17 (Tool 17; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 54 / 9 / True |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 0) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | FlatBufferError, Error |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** keine
- **Ziel-Kanten, die im Ist fehlen:** keine
- **Default-Features:** Ist = Ziel (std).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: `build.rs` verweist auf `../../schemas/contextra.fbs`. Im Repo-Root existiert kein `schemas/`-Verzeichnis. Das Schema liegt unter `crates/contextra-wire/schemas/contextra.fbs` zusammen mit `contextra.fbs.sha256` (Anforderung G-F2 ist also vorbereitet).
- Unsafe: 54 Vorkommen, davon 9 ohne `// SAFETY:`-Kommentar (Audit). Das ist der einzige Stellen-Befund mit fehlendem Nachweis im Workspace.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

unverändert; generierten Code im LOC-Report getrennt führen; `router`-Referenzen: nur `dispatch.rs:23` [E] ⇒ `wire` bei `router` optional (S-62).

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: besteht (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 17; die ersten 15):** Tool-Gesamt 17, davon 0 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).

- `crates/contextra-wire/build.rs:46` — assert — `assert!(status.success(), "flatc failed to generate code");`
- `crates/contextra-wire/src/contextra_generated.rs:142` — unwrap — `unsafe { self._tab.get::<i8>(Embedding::VT_METRIC, Some(0)).unwrap() }`
- `crates/contextra-wire/src/contextra_generated.rs:336` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:569` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:583` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:956` — unwrap — `unsafe { self._tab.get::<u32>(RoleId::VT_ID, Some(0)).unwrap() }`
- `crates/contextra-wire/src/contextra_generated.rs:1077` — unwrap — `unsafe { self._tab.get::<u64>(HyperEdgeId::VT_ID, Some(0)).unwrap() }`
- `crates/contextra-wire/src/contextra_generated.rs:1204` — unwrap — `unsafe { self._tab.get::<u32>(RoleBinding::VT_ROLE, Some(0)).unwrap() }`
- `crates/contextra-wire/src/contextra_generated.rs:1217` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:1391` — unwrap — `unsafe { self._tab.get::<u64>(HyperEdge::VT_ID, Some(0)).unwrap() }`
- `crates/contextra-wire/src/contextra_generated.rs:1404` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:1435` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:1449` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:1463` — unwrap — `.unwrap()`
- `crates/contextra-wire/src/contextra_generated.rs:1477` — unwrap — `.unwrap()`

**unsafe-Dateien (3):**

- `crates/contextra-wire/build.rs`
- `crates/contextra-wire/src/contextra_generated.rs`
- `crates/contextra-wire/tests/schema_drift.rs`

**Fehler-Enums (Ist):**

- `crates/contextra-wire/src/verification.rs:34` — FlatBufferError
- `crates/contextra-wire/src/lib.rs:61` — Error

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B15 — Mehr nicht verifizierbare Plan-Annahmen**

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `mcp/protocol.rs` vs. `wire/jsonrpc.rs` — 5 gemeinsame, 7 nur-mcp (`McpError`, Tiefen-/Batch-Validierung), 10 nur-wire [V] — **Kein Merge**; beide bleiben. Dublette nur im Typnamen, nicht in der API.

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-07 | `vault` baut auf `sys::vault`; `mcp::protocol` und `wire::jsonrpc` bleiben getrennt; `adapt::drift`/`rank::drift` und die beiden `query_rewriter` bleiben getrennt (B16) |
| D-10 | `wire/build.rs` unverändert; Prüfsumme in CI (B15) |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-wire --no-default-features --locked
cargo hack check -p contextra-wire --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-wire --modes no-default,default,each-feature
```

