# contextra — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Öffentliche Fassade: Builder, Profile, Lizenz-Gating. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | integration |
| Ebene (Kern-Closure) | 2 |
| Kern-Closure | db, types, ports (`license`, `infer-candle` optional) |
| Default-Features | `embedded` |
| Fehlertyp (R3) | — |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 1218 Zeilen in 8 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports, db, infer-candle |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 3 |
| Default-Features (Ist) | embedded |
| `cfg(feature = …)`-Stellen in src/ | 1 |
| `#[test]`-Vorkommen (grep, src + tests) | 16 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 2 (Tool 2) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | DurabilityConfigError, PerformanceProfileError, CollectionProfileError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `infer-candle` (optional)
- **Ziel-Kanten, die im Ist fehlen:** keine
- **Default-Features:** Ist = Ziel (embedded).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Keine zusätzlichen Abweichungen in dieser Prüfung festgestellt. Die Aussagen aus docs/SPEC.md sind nur dort übernommen, wo sie oben nicht widersprochen werden.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

Lizenz-Gating (`builder.rs:33–35,243–350`, `performance_profile.rs`, `collection_profile.rs:33,128`) hinter Feature `license`; ohne Feature baut die Fassade ohne Lizenzprüfung (Verhalten **ohne** Feature = `OpenFastGate`-Semantik, per Test belegt). `agent_memory.rs` bleibt. `query_rewriter.rs` bleibt (B16).

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Determinismus (Produktion, bereinigt: 2; die ersten 15):** Tool-Gesamt 2.

- `crates/contextra/src/query_rewriter.rs:32` — random_hashset — `use std::collections::HashSet;`
- `crates/contextra/src/query_rewriter.rs:248` — random_hashset — `let mut seen = HashSet::new();`

**Fehler-Enums (Ist):**

- `crates/contextra/src/performance_profile.rs:53` — DurabilityConfigError
- `crates/contextra/src/performance_profile.rs:128` — PerformanceProfileError
- `crates/contextra/src/collection_profile.rs:93` — CollectionProfileError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `contextra/query_rewriter` vs. `infer-ollama/query_rewriter` — 1 gemeinsamer Name [V] — **Keine Dubletten**, keine Konsolidierung.

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra --no-default-features --locked
cargo hack check -p contextra --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra --modes no-default,default,each-feature
```
