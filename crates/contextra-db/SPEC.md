# contextra-db — Spezifikation

> **Geltung:** Crate-lokale Spezifikation. Normativ bleibt `docs/SPEC.md` (v2, 10.11.2026). Dieses Dokument überträgt die Anforderungen für dieses Crate, ergänzt sie um verifizierte Ist-Fakten und markiert Abweichungen.
> **Faktenbasis:** statische Analyse mit `tools/` am geklonten Repo-Stand (`main`, HEAD `f0eda1cb9`). `cargo` ist in der Analyseumgebung nicht verfügbar. **Es wurde nichts gebaut, getestet oder ausgeführt.**
> **Beleg-Marken:** *[V]* am Quelltext/Manifest verifiziert, *[T]* aus einem tools/-Bericht, *[S]* aus docs/SPEC.md übernommen, *[E]* Festlegung dieser Spezifikation.

**Pfad:** `crates/contextra-db/` · **Umsetzungsplan:** `PLAN.md` (im selben Verzeichnis)

## 1. Zweck

Datenbank-Fassade: Collections, Vault-Anbindung, Cognition-Brücke. *[S]*

## 2. Zielzustand (docs/SPEC.md §3.1) [S]

| Merkmal | Ziel |
|---|---|
| Gruppe | integration |
| Ebene (Kern-Closure) | 2 |
| Kern-Closure | engine, types, ports |
| Default-Features | — |
| Fehlertyp (R3) | — |
| Brücke (hinter Feature `workspace`, §1.1) | — |

Ebenen (SPEC §1.1): E0 = Kern-Closure leer · E1 = Kern-Closure ⊆ E0, höchstens 4 Mitglieder · E2 = Integration · E3 = Anwendungen und Testwerkzeuge. Der Kern ist der Code mit `--no-default-features` ohne Feature `workspace`.

## 3. Ist-Zustand (tools/)

| Messgröße | Wert |
|---|---|
| Lines of Code (Produktion, `loc.py`) | 1797 Zeilen in 10 Dateien |
| Version / publish | workspace-vererbt / <unset> |
| Interne Normal-Abhängigkeiten (Ist, Cargo.toml) | engine, types, ports |
| Interne Dev-Abhängigkeiten (Ist) | vector, testkit, store, graph, text, adapt, checkpoint |
| Features (Ist, Anzahl) | 12 |
| Default-Features (Ist) | keine |
| `cfg(feature = …)`-Stellen in src/ | 5 |
| `#[test]`-Vorkommen (grep, src + tests) | 35 |
| zero-panic: Produktionsbefunde, bereinigt (Tool-Gesamt; davon in Testdateien) | 0 (Tool 1; 1 in Testdateien) |
| unsafe: Vorkommen / ohne SAFETY / Insel | 0 / 0 / False |
| Determinismus: Produktionsbefunde, bereinigt (Tool-Gesamt) | 5 (Tool 5) |
| Port-M3-Leaks (port_contract_audit) | 7 |
| Error-Enums (Ist) | VaultError |

**Tool-Caveat:** `zero_panic_analyzer.py` klassifiziert `#[cfg(test)]`-Module (`tests.rs`) als Produktion. Im Repo betrifft das 449 von 478 Treffern. Die Zahlen oben sind deshalb bereinigt; die Rohzahl steht in Klammern. Die Korrektur am Tool ist eigener Schritt (S-00).

**Quellen:** `tools/loc.py`, `tools/zero_panic_analyzer.py`, `tools/unsafe_boundary_auditor.py`, `tools/determinism_guard.py`, `tools/port_contract_auditor.py`, `tools/error_topology_mapper.py`, `tools/cargo_dependency_graph.py`, Cargo.toml-Auswertung.

## 4. Ist-vs-Ziel-Abgleich [V]

- **Ist-Kanten, die das Ziel nicht vorsieht:** keine
- **Ziel-Kanten, die im Ist fehlen:** keine
- **Default-Features:** Ist = Ziel (leer).

## 5. Verifizierte Befunde und Abweichungen zu docs/SPEC.md [V]

- Verifiziert: 7 Port-M3-Leaks (port_contract_audit). `volatile_vault` ist nach `vault` auszulagern (S-22).

## 6. Anforderungen

### 6.1 Crate-Spezifikation (docs/SPEC.md §6) [S]

`volatile_vault.rs` → `vault`; `pid_latency_controller.rs`, `homeostat.rs` → `pub use` aus `adapt` nach S-60; `multistep.rs` (442) bleibt. `experimental-diskann` zeigt auf `contextra-vector-diskann` als **echte optionale** Abhängigkeit (heute Dev-Abhängigkeit [V]). Neues Feature `cognition` (Brücke zu `cognition`, siehe unten).

### 6.3 Regeln für jeden Schritt [S]

INV-1 Formatgleichheit · INV-2 Pfadgleichheit (Re-Export mit `#[deprecated]`) · INV-3 Ein Schritt = Commit-Paar · INV-4 `cargo check --workspace --all-targets --locked` grün · INV-5 Determinismus (Tie-Break nach ID aufsteigend) · INV-6 Durability-Pfade unberührt · INV-7 keine Compliance-Aussagen · INV-8 keine Rückwärtspfeile. Vollständig: docs/SPEC.md §1.2, Regeln R1–R9: §3.2.

## 7. Schnittstellen und Persistenz

Keine Zeile aus dem Golden-Korpus (§5.1) bezieht sich auf dieses Crate.

### 7.2 Öffentliche Schnittstelle

Ist-Stand und Ziel-Schnittstelle sind in den Abschnitten 3 bis 6 festgehalten. Die exakte Liste der `pub`-Symbole erhebt S-00 über `cargo public-api` (Gate G-A1); erlaubte Differenzen sind nur Re-Export-Pfade mit `#[deprecated]`.

## 8. Fundstellen (tools/, exakt) [T]

**zero-panic (Produktion, bereinigt: 0; die ersten 15):** Tool-Gesamt 1, davon 1 in Testdateien (`tests.rs` unter `#[cfg(test)]`, Tool-Fehlklassifikation).


**Determinismus (Produktion, bereinigt: 5; die ersten 15):** Tool-Gesamt 5.

- `crates/contextra-db/src/volatile_vault.rs:289` — instant_now — `purged_at: Instant::now(),`
- `crates/contextra-db/src/homeostat.rs:17` — instant_now — `deadline: Instant::now() + timeout,`
- `crates/contextra-db/src/homeostat.rs:22` — instant_now — `Instant::now() >= self.deadline`
- `crates/contextra-db/src/homeostat.rs:26` — instant_now — `self.deadline.saturating_duration_since(Instant::now())`
- `crates/contextra-db/src/multistep.rs:434` — random_hashset — `let mut seen = std::collections::HashSet::new();`

**Port-M3-Leaks (7):**

- `crates/contextra-db/src/lib.rs:44` —  — `pub use contextra_engine::collection::{self, Collection, CollectionConfig};`
- `crates/contextra-db/src/lib.rs:45` —  — `pub use contextra_engine::{`
- `crates/contextra-db/src/lib.rs:73` —  — `pub use contextra_engine::DurabilityMode;`
- `crates/contextra-db/src/lib.rs:75` —  — `pub use contextra_engine::{`
- `crates/contextra-db/src/lib.rs:82` —  — `pub use contextra_engine::SandboxBridge;`
- `crates/contextra-db/src/lib.rs:92` —  — `pub use contextra_engine::decay_controller::*;`
- `crates/contextra-db/src/lib.rs:94` —  — `pub use contextra_engine::fusion;`

**Fehler-Enums (Ist):**

- `crates/contextra-db/src/volatile_vault.rs:154` — VaultError

## 9. Befunde aus docs/SPEC.md (B-Nummern) [S]

**B07 — Eine Verzeichnis-Regruppierung bricht messbar viel**
> **Entscheidung (D-01):** Das Verzeichnislayout bleibt **flach** (`crates/contextra-*`). Die Gruppierung steht als Metadatum `group` in `capabilities.toml`. Die physische Regruppierung ist Nicht-Ziel (§9.3); sie liefert keinen Entnahmegewinn und kostet die oben genannten Brüche.

**B13 — Dev-Kanten auf `testkit` und Entnahmenachweis**
> **Entscheidung:** Dev-Kanten zu `testkit` sind erlaubt, stehen aber im Manifest `dev_allowed` (§8.3). Der Isolationsnachweis (G-L4) prüft `--edges normal` und baut zusätzlich `--no-dev-deps`; Tests laufen im Export nur mit `--with-dev` (nimmt dann `testkit`-Closure mit). `env` (Feature `test-utils`) übernimmt `ManualClock` und die Clock/Rng-Konformitätssuite, damit E1-Crates dafür nicht auf `testkit` angewiesen sind.

**B16 — Einzelentscheidungen, die der Entwurf offen ließ**
> `contextra-vault` vs. `sys/vault.rs` — Öffentliche Namen gemeinsam: 3; `sys::vault` = gesperrter Speicher (`allocate`, `as_slice`, `is_locked`), `db::volatile_vault` = Chunk-Tresor (`ingest`, `commit`, `SignalModality`) [V] — **Aufbauend, nicht ersetzend:** `contextra-vault` (E1, Kern-Closure …
> `homeostat` adapt/db — 4 gemeinsam; `adapt` 11, `db` 3 zusätzliche (`is_expired`, `pool_size`, `remaining`) [V] — Wie oben: `adapt` nimmt die 3 Methoden auf, dann `pub use` in `db`.

## 10. Entscheidungen und Risiken [S]

**Risiken (§9.2):**

| `cognition`-Port-Zuschnitt (`CommitIntent` aus `engine::transaction`, 1.652 Z. [E]) | S-63 | Port scheitert ⇒ `CommitIntent` als eigener Typ in `cognition`, Abbildung in `db`-Brücke |

Nicht-Ziele (§9.3), gelten für alle Crates: physische Regruppierung, Mikro-Crate-Zerlegung (F-1), Verschieben von `Vfs`/`io_seam`, crates.io-Veröffentlichung, Formatänderungen außer ID-Breite, Änderung der FlatBuffers-Schemas.

## 11. Abnahme [S]

Ein Crate gilt als entnehmbar (R6), wenn in dieser Reihenfolge grün: **G-L2** (Null-Prüfungen, nur bei Brücke) → **G-L1** (Kern- und Feature-Closure) → **G-L3** (`--no-default-features`, `--each-feature`) → **G-L4** (isolierter Export). Danach **G-F1/G-D1/G-E1/G-C1** für den betroffenen Schritt (§8.4).

```bash
cargo check -p contextra-db --no-default-features --locked
cargo hack check -p contextra-db --each-feature --no-dev-deps --locked
python3 tools/isolation_proof.py --crate contextra-db --modes no-default,default,each-feature
```

