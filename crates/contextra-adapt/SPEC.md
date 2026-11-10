# contextra-adapt — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-adapt/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Adaptive Regler und Bandits (PID, Homeostat, Thompson-Sampling, Off-Policy-Schätzer). *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | llm |
| Ebene (Kern-Closure) | 0 |
| Kern-Closure | — |
| Default-Features | `std` |
| Fehlertyp (R3) | besteht |
| Brücke (hinter Feature `workspace`, §1.1) | Blanket-Impl `env→lokal` |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 3808 Zeilen in 19 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | keine |
| Interne Dev-Abhängigkeiten (Ist) | keine |
| Features (Ist, Anzahl) | 6 |
| Default-Features (Ist) | std, bandit-routing, flow-corrected-thompson, rie-greedy-personalization |
| `cfg(feature = …)`-Stellen in src/ | 20 |
| `#[test]`-Vorkommen (grep, src + tests) | 142 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 0; 0 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 0 (Tool 0) |
| Port-M3-Leaks (port_contract_audit) | 0 |
| Error-Enums (Ist) | RieGreedyError, BanditError, FcTsError, OffPolicyError, AdaptError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** keine
- **Ziel-Kanten, die im Ist fehlen:** keine
- **Default-Features weichen ab:** Ist `std, bandit-routing, flow-corrected-thompson, rie-greedy-personalization` → Ziel `std`.

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: Feature `sketched-bandit` in `Cargo.toml:26` vorhanden, ohne `cfg(feature)`-Nutzung im Code (B14). Clock/Rng-Varianten in `src/clock.rs:11`, `src/contract.rs:11,19`, `src/rng.rs:8` (`next_u64(&mut self)`).

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

(1) `clock.rs`+`contract.rs` und `rng.rs`+`contract.rs` zu je **einem** Trait zusammenführen; `Rng` behält `&mut self` (R7). (2) `off_policy.rs` (6 Z.) entfernen, wenn `rg 'off_policy::'` leer. (3) `default = ["std"]`; `router` aktiviert `bandit-routing` und die Experimente gezielt. (4) `sketched-bandit` streichen (B16). (5) Features umbenennen: `exp-flow-thompson`, `exp-rie-greedy`; Aliasnamen **nicht** behalten (nur `router` setzt sie).

### 6.2 Brückenmuster [S]

Die Brücke liegt ausschließlich in `src/compat/` hinter dem Feature `workspace`. Verbindliche Muster: docs/SPEC.md §4.3 (a–d). R2 gilt: `contextra_types`, `contextra_ports`, `contextra_testkit` dürfen im Kern nirgends vorkommen, auch nicht in Doc-Kommentaren oder `#[cfg(test)]`.

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

### 6.5 Fehlerabbildung [S]

Eigener Fehlertyp: besteht (thiserror, `#[non_exhaustive]`). Jede heute genutzte `ContextraError`-Variante hat genau eine lokale Entsprechung; `Display` bleibt bitgleich; keine Sammelvariante `Other(String)` ohne Quelle. Tabelle je Baustein: `docs/decoupling/errors/<crate>.md` (aus S-xx erzeugt). Quelle: §5.3.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**Fehler-Enums (Ist):**

- `crates/contextra-adapt/src/rie_greedy.rs:12` — RieGreedyError
- `crates/contextra-adapt/src/bandit.rs:36` — BanditError
- `crates/contextra-adapt/src/flow_thompson.rs:29` — FcTsError
- `crates/contextra-adapt/src/flow_thompson.rs:695` — OffPolicyError
- `crates/contextra-adapt/src/error.rs:10` — AdaptError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B02 — Die `Clock`-/`Rng`-Kopien sind nicht kompatibel (Teil 2, G1 ist unvollständig)**
> **Entscheidung:** - `env::Clock` = `ports::Clock` (Nanosekunden, `monotonic_nanos`, `Send+Sync+'static`). `env::Rng` = `ports::Rng` mit `&self` (siehe §4.2). - Millisekunden-Verbraucher (`text`, `graph`, `vector/contract.rs`) benutzen `env::Clock` und `env::unix_ms(&dyn Clock) -> u64` = `now_unix_nanos() / 1_000_000` (Abrunden). Persistierte ms-Zeitstempel MÜSSEN im Differenztest (§5.4) bitgleich zu `now_unix_ms` der alten Implementierung bei identischer Quelle sein. - `&mut self`-Verbraucher `text`/`graph`/`vector` wechseln auf `&self`-`env::Rng`. Das ist für alle heutigen Aufrufstellen ein Superset (Mutex/Atomic innerhalb `SeededRng`). Der Umstieg ist ein **Umbau-Commit** (INV-3 B). - E0-Crates (`adapt`, `privacy`, `license`, `erasure`, `ingest`, `audit-export`) behalten ihre lokalen Traits (R7/K8). Ihre Brücke bindet per Blanket-Impl **von `env` auf lokal** (Richtung `&self → &mut …
> `adapt/{clock,contract,rng}.rs` — nanos + monotonic; (`clock.rs` mit `'static`) — **`&mut self`**, `next_f64`/`next_f32`

**B06 — `capabilities.toml` und `AGENTS.md` widersprechen sich**
> **Entscheidung:** Schema v3 (§8.3) ersetzt `klasse/ring`. `may_depend_on` wird aus dem Ziel (§3.1) erzeugt, nicht aus dem Ist.

**B14 — Defaults und leere Features (Teil 2, G5/G6) — bestätigt, plus Ergänzungen**
> **Entscheidung:** Zieldefaults §3.1. Da `default-members` diese Crates auslässt, genügt „wöchentlich `cargo check`“ nicht als Kompilierbarkeitsgarantie; CI führt bei **jedem** PR `cargo check --workspace --all-targets --locked` aus (S-01). Ein Feature ohne `cfg(feature = "x")` im Quelltext **und** ohne Weiterleitung in einem anderen Manifest wird entfernt; geprüft vom neuen `tools/feature_audit.py` (S-12).

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `RetrievalStrategy` — `types` und `adapt` führen verschiedene Gestalten — Bleibt in `types` (K10); `router/arm_registry.rs` bildet ab.
> `adapt/drift` vs. `rank/drift` — 2 gemeinsame Namen (`adapt`: 8 eigene, `rank`: 7 eigene) [V] — **Keine Dubletten**, keine Konsolidierung.
> `decay_controller` adapt/engine — 9 gemeinsam; `engine` hat 5 zusätzliche (`with_clock`, `set_clock`, `effective_half_life_ms`, `effective_score_wall_clock`, `should_evict_wall_clock`), `adapt` 1 (`config`) [V] — `adapt` bleibt Besitzer, **muss zuerst die 5 Methoden aufnehmen**; Zusammenlegung erst …
> `homeostat` adapt/db — 4 gemeinsam; `adapt` 11, `db` 3 zusätzliche (`is_expired`, `pool_size`, `remaining`) [V] — Wie oben: `adapt` nimmt die 3 Methoden auf, dann `pub use` in `db`.

## 10. Entscheidungen und Risiken [S]

**Entscheidungen (§9.1):**

| D-07 | `vault` baut auf `sys::vault`; `mcp::protocol` und `wire::jsonrpc` bleiben getrennt; `adapt::drift`/`rank::drift` und die beiden `query_rewriter` bleiben getrennt (B16) |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-adapt --no-default-features --locked
cargo hack check -p contextra-adapt --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-adapt --modes no-default,default,each-feature
```

