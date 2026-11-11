# contextra-agent — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v3, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** Statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `278a3673`).
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-agent/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Agenten-Workflow-Engine: Graph, Budget, DLQ, Scratchpad, Recovery, Checkpoint-Loop. *[S]*

## 2. Zielzustand (docs/SPEC.md v3 §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | agentic |
| Ebene (Kern-Closure) | E2 (Integration) |
| Kern-Closure | types, ports |
| Default-Features | leer (`default = []`) |
| Fehlertyp (R3) | `AgentError` (`thiserror`, `#[non_exhaustive]`) [V] |
| Optionale Feature-Closures | `routing` → `router` |
| Brücke | Speicherbindung an `db`/`checkpoint` via Port-Abstraktionen |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1a/E1b = Bausteine mit ⊆ E0/E1a Abhängigkeiten · E2 = Integration (types, ports, agent, engine, db, contextra) · E3 = Anwendungen und Testwerkzeuge.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 4.394 Zeilen in 22 Quelldateien |
| Version / publish | workspace-vererbt / `<unset>` |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, ports, db, checkpoint, router |
| Interne Dev-Abhängigkeiten (Ist) | graph, store, testkit, vector, text |
| Features (Ist, Anzahl) | 2 (`default`, `test-utils`) |
| Default-Features (Ist) | keine (`[]`) |
| `#[test]`-Vorkommen (grep, src + tests) | 46+ |
| zero-panic: Produktionsbefunde, bereinigt | 0 (59 in internen Testmodulen) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False (`#![forbid(unsafe_code)]`) |
| Determinismus: Produktionsbefunde, bereinigt | 0 |
| Eigener Fehlertyp (Ist) | `AgentError` in `src/error.rs` vorhanden |

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/decouple_inspector.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten zur Invertierung/Kapselung:** `db`, `checkpoint`, `router`
- **Kern-Closure (Soll):** `types`, `ports`
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Architekturprinzipien [V]

- Lokale Port-Definitionen: `ContextStore` und `ScratchpadStore` sind in `src/ports.rs` deklariert.
- Brückenadapter: `src/compat/store_adapter.rs` adaptiert `contextra-db` Collections auf `ContextStore`.
- Graph-Entkopplung: `contextra-graph` ist bereits vollständig aus den Normal-Dependencies entfernt und in `[dev-dependencies]` isoliert.
- Eigener Fehlertyp: `AgentError` ist in `src/error.rs` implementiert mit lückenloser `From<AgentError> for ContextraError` Abbildung in `compat/error.rs` bzw. `error.rs`.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6.2, §6.4) [S]

Lokale Ports `ContextStore`, `ScratchpadStore`; Speicherbindung (`context.rs`, `clm_scratchpad.rs`, `tools/retrieve.rs`, `graph.rs`, `engine.rs`) kapseln und über Ports entkoppeln; `router`-Kopplung hinter Feature `routing`. Kern-Closure: types, ports.

### 6.2 Brückenmuster [S]

Optionale Speicher- und Integrationsbindungen liegen in `src/compat/`.
Kern-Trait-Methoden und Core-Logik exponieren keine Fremd-Storage-Strukturen (`Arc<LsmStorage>`).

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit · INV-3 Commit-Paare · INV-4 Workspace baut · INV-5 Determinismus (Tie-Break aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 kein Rückwärtspfeil. Vollständig: docs/SPEC.md §1.2, Regeln R1–R11.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: `AgentError` (`thiserror`, `#[non_exhaustive]`). Display-Formatierungen bleiben bitgleich zu `ContextraError`.

## 7. Abnahme [S]

```bash
cargo check -p contextra-agent --no-default-features --locked
cargo clippy -p contextra-agent --all-targets --locked -- -D warnings
cargo test -p contextra-agent --locked
```
