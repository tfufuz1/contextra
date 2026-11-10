# contextra-router — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-router/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Retrieval-Routing: Arm-Registry, Strategieauswahl, Bandit-Dispatch. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | llm |
| Ebene (Kern-Closure) | 2 |
| Kern-Closure | types, ports, adapt (`wire` nur `subprocess-dispatch`) |
| Default-Features | — |
| Fehlertyp (R3) | besteht |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 2280 Zeilen in 16 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | types, wire, ports, adapt |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 5 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 54 |
| `#[test]`-Vorkommen (grep, src + tests) | 75 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 14; 14 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 0) |
| Port-M3-Leaks (port_contract_audit) | 6 |
| Error-Enums (Ist) | ArmRegistryError, FcTsDispatchError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** `wire`
- **Ziel-Kanten, die im Ist fehlen:** keine
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: 6 Port-M3-Leaks, 14 Produktionsbefunde (zero-panic). `ports_local` wird in S-62 entfernt, sobald `mcp` umgestellt ist.

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

`ports_local.rs` und die deprecated Re-Exports (`lib.rs:32–45`) entfernen, **nachdem** die drei `mcp`-Importe von `contextra_router::ports_local::{HybridSearchProvider, ContextPreparer, CommunityResolver}` auf `contextra_ports` umgestellt sind. `wire` optional hinter `subprocess-dispatch` (`dispatch.rs`, `transport.rs`); `ports::vfs`-Nutzung (eine Stelle) auf lokale `&Path`-Funktion; `RetrievalStrategy`-Abbildung nur in `arm_registry.rs`. Features: `exp-fc-ts`, `exp-routing-strategy`, `cloud-egress-guard`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: besteht (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 14, davon 14 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).


**Port-M3-Leaks (6):**

- `crates/contextra-router/src/lib.rs:35` —  — `pub use contextra_adapt::bandit;`
- `crates/contextra-router/src/lib.rs:40` —  — `pub use contextra_adapt::lyapunov;`
- `crates/contextra-router/src/lib.rs:45` —  — `pub use contextra_adapt::offpolicy;`
- `crates/contextra-router/src/lib.rs:60` —  — `pub use contextra_adapt::{DriftReason, LyapunovDriftWatcher, LyapunovResult};`
- `crates/contextra-router/src/lib.rs:61` —  — `pub use contextra_adapt::{OffPolicyEvaluator, OffPolicyStats};`
- `crates/contextra-router/src/lib.rs:69` —  — `pub use contextra_adapt::{BanditImplementation, BanditPolicy, BanditProfileState};`

**Fehler-Enums (Ist):**

- `crates/contextra-router/src/arm_registry.rs:34` — ArmRegistryError
- `crates/contextra-router/src/fc_ts_dispatch.rs:21` — FcTsDispatchError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B08 — `Vfs` kann nicht verschoben werden, ohne den Fehlertyp zu ändern**
> **Entscheidung:** `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia` bleiben in `ports`. `durable-fs` bleibt eigenständig (E0, Tier experimental, kein Verwender). Die Plan-Schritte „`io_seam`/`fsync.rs` nach `durable-fs`“ und „`sys-flush`“ entfallen (Nicht-Ziel). Verbraucher mit Dateisystembedarf (`store`, `checkpoint`) halten eine **crate-lokale** schmale Naht (siehe §6), die Brücke bindet an `ports::Vfs`.

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `RetrievalStrategy` — `types` und `adapt` führen verschiedene Gestalten — Bleibt in `types` (K10); `router/arm_registry.rs` bildet ab.

## 10. Entscheidungen und Risiken [S]

Keine Einträge in §9.1 oder §9.2.

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-router --no-default-features --locked
cargo hack check -p contextra-router --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-router --modes no-default,default,each-feature
```
