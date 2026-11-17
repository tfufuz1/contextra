# contextra-engine — Implementierungsspezifikation

**Gegenstand:** Überführung von `contextra-engine` in den SOLL-Zustand nach `docs/SPEC.md` v6 (normativ).
**Basis:** `tfufuz1/contextra`, HEAD `24487fa` (16.11.2026) · Normquelle `docs/SPEC.md` v6 (16.11.2026, selbst gemessen auf `fcd387021`, 11.11.2026).
**Autor-Rolle:** Principal Rust Engineer · **Sprache der IDs:** `ENG-` Anforderung, `EW-` Arbeitspaket, `D-E` Entscheidung, `F-E` Befund.

---

## 0 Methode und Belegmarken

**Gelesen:** `docs/SPEC.md` (981 Zeilen, vollständig); `crates/contextra-engine/` (`Cargo.toml`, `src/lib.rs`, `SPEC.md`, `PLAN.md`, `README.md`, `AGENTS.md`, `capabilities.fragment.toml`, `docs/decoupling/*`, Test-Verzeichnis); `crates/contextra-db/` (`Cargo.toml`, `src/lib.rs`, Modulbaum, `SPEC.md`, `PLAN.md`, `DEPRECATED_DEBT.md`, Tests); `reports/*` (Audit, Verifikation, Fix-Report); `Cargo.lock`. Ergänzend: Python-Auswertung der `crate::`-Kanten zwischen `db`-Modulen und der Nutzung externer Crates, `grep`-Zählungen.

**Nicht möglich:** Bauen, Testen, Gates ausführen. Das Wurzel-`Cargo.toml` des Workspace fehlt im Repository (F-E02), die Toolchain 1.89.0 ist in der Analyseumgebung nicht installiert. **Jede Aussage zu Build- oder Testverhalten ist daher [A] oder [E].**

| Marke | Bedeutung |
|---|---|
| **[V]** | am HEAD per Datei, `grep` oder Skript geprüft; nicht gebaut |
| **[A]** | aus `reports/` übernommen, nicht nachgeprüft |
| **[E]** | Einschätzung, durch Messung in EW-00 zu bestätigen |
| **[D]** | Festlegung dieser Spezifikation |
| **[D?]** | Entscheidungsvorschlag, benötigt ADR vor Umsetzung |

**Rangfolge** (SPEC §1.2): `docs/SPEC.md` + Entscheidungsregister > ADRs > crate-lokale `SPEC.md`/`PLAN.md` > `AGENTS.md`/`docs/**`.

---

## 1 Kernbefund

1. **IST:** `contextra-engine` ist eine 15-zeilige, `#[deprecated]` markierte Façade (`pub use contextra_db::*;`) [V]. Der gesamte Engine-Code liegt in `contextra-db`: **21.419 LOC in 83 Dateien** (ohne `lib.rs`; `lib.rs` hat weitere 1.205 Zeilen, davon ≈690 Unit-Tests) [V]. Das deckt sich mit „Kern-LOC 21.935“ aus dem früheren Engine-Report [V, `docs/decoupling/REPORT.md` §6]: derselbe Code, verschoben.
2. **SOLL:** SPEC §2.2/§4.1/§9.4 führen `engine` als eigenständigen **E2-Baustein unterhalb von `db`** (`db` Kern: `engine, types, ports, vault`). S-33, S-60, S-61, S-63, S-64 setzen genau diese Struktur voraus [V].
3. **Widerspruch:** Crate-lokale `engine/SPEC.md` und `PLAN.md` melden „Erreicht“ bzw. „Abgeschlossen“ für einen Zustand, der nicht mehr existiert, und stützen sich auf `SPEC(11).md` und `ADR-0122`. Weder `SPEC(11).md` noch `docs/decisions/` existieren im Repository [V]. Nach §1.2 gilt SPEC v6; die Auflösung der Crate ist eine **Abweichung vom SOLL**, kein Erfolg.
4. **Machbarkeit der Umkehrung:** Die Modulkanten innerhalb `db/src` zeigen einen sauberen Schnitt: **0 Kanten von Engine-seitigen Modulen zu `db`-seitigen Modulen** [V, Skriptauswertung]. Nur `multistep` (→ `fusion`), `cognition/bridge` (→ `collection`) und `lib.rs` beziehen sich auf Engine-Typen.
5. **Ausgangslage ist rot** [A]: `engine` 110 grün / 1 rot (`test_durability_after_wal_commit_and_simulated_crash`); `db` 0 Tests ausgeführt (Kompilierfehler, 37 im Check); `contextra` 2 rot (u. a. `open_compliance_profile_succeeds_without_license`, T-DEL-1). Ohne Heilung bzw. Klassifikation ist „Verhalten unverändert“ (INV-1/INV-6) nicht beweisbar.
6. **Entscheidung (D-E1, [D?]):** Re-Extraktion nach SPEC v6 statt SPEC-Änderung. Begründung und Alternative in §5.

**Plan in einem Satz:** erst Verifikationsfähigkeit herstellen (Welle 0), dann Engine-Code mechanisch nach `engine` zurückverschieben und `db` zur dünnen Façade machen (Welle 1), dann die in SPEC §9.2 geforderten Entkopplungen ausführen (Welle 2), danach die Engine-Anteile der Kapitel 11–13 (Welle 3).

**Nicht-Ziele:** keine Verhaltensänderung, kein Formatbruch (INV-1), keine neuen Features (D-09 erlaubt Verschieben/Umbenennen), keine Zusammenlegung von `extraction/` mit `graph/entity_extraction.rs` (§9.4), keine In-Place-ID-Migration (D-16).

---

## 2 SOLL — normative Anforderungen an `contextra-engine`

Jede Anforderung hat ein Prüfverfahren (SPEC §1.3). Ohne laufendes Verfahren gilt sie nur als SOLL.

| ID | Anforderung | Quelle | Prüfverfahren |
|---|---|---|---|
| **ENG-01** | Ebene E2, Ring 3. Normale **und** dev-Kanten zeigen nur nach unten. `engine` hängt von **keinem** E2-Crate oberhalb (`db`, `contextra`, `agent`, `cognition`) und keinem E3-Crate ab. | §2.2, INV-8, R1 | G-L1 (normal, dev, Feature) |
| **ENG-02** | Kern direkt: `ids, env, types, ports, mvcc, store, vector, text, graph, rank, checkpoint`; transitiv `sys`, `crypto`. **Delta nach D-E6:** zusätzlich `adapt` und `ingest` (beide E0, `default-features = false`). Ungenutzte Abhängigkeiten entfallen. | §4.1, S-60, S-61 | G-L3, `tools/abhaengigkeits_budget.py` |
| **ENG-03** | Features: `adaptive-candidate-pool-sizing`→`adapt`; `onnx`→`infer-onnx`; `sandbox`→`sandbox`; `license`→`license`; `exp-extraction`. **`default` leer.** Experimente heißen `exp-*`. Ein Feature ohne `cfg(feature)` und ohne Weiterleitung entfällt. | §4.1, R10, R5 | G-H1, `feature_audit.py`, `cargo hack --each-feature` |
| **ENG-04** | Fehlerdomäne `ContextraError`; keine Sammelvariante `Other(String)`; `Display` bitgleich zur Vorversion; Abbildungstabelle `docs/decoupling/errors/engine.md`. | §4.1, §6.3 | G-E1, `s64_error_passthrough` |
| **ENG-05** | Bausteinvertrag: `rust-version`, `version` an jeder Pfad-Abhängigkeit, `#![forbid(unsafe_code)]`, README mit Beispiel ≤ 10 Zeilen, `examples/` (mind. 1), `capabilities.fragment.toml` mit `level`, `sublevel`, `core_deps`, `SPEC.md` ohne Widerspruch zu SPEC v6. | §5.4, D-17 | G-DOC1, `capabilities-consistency` |
| **ENG-06** | **Formatgleichheit:** kein persistiertes Byte ändert sich (Export-Schema v1, Commit-Intent, Orphan-Registry, Tenant-Index). Golden-Korpus für alle Engine-eigenen Formate. | INV-1, §6.1 | G-F1 |
| **ENG-07** | **Pfadgleichheit:** jeder öffentliche Pfad von `contextra_db` (vorher) und `contextra_engine` (vorher) bleibt auflösbar. | INV-2, R9 | G-A1 + Pfad-Anker-Test (EW-12) |
| **ENG-08** | **Determinismus:** gleiche Eingabe → gleiche Rangfolge, Tie-Break ID aufsteigend (`INV-RANK-DETERMINISM-1`). Zeit und Zufall nur über `env`-Ports. | INV-5 | G-D1, `determinism_guard.py` |
| **ENG-09** | **Durability unberührt:** WAL, Manifest, Flush, Recovery, Commit-Intent-Protokoll. | INV-6 | Crash-Suite (§3.3), G-F1 |
| **ENG-10** | **Neutralität:** keine Compliance-Aussage und keine Löschketten-Begriffe im Engine-Kern. | INV-7, `engine/SPEC` §3.2.4 | G-DOC1 |
| **ENG-11** | Kein `unsafe`; keine neuen `unwrap`/`expect`/`panic!`/`todo!` in Nicht-Test-Code. | INV-11, INV-12 | `unsafe-inventory`, `unwrap-gate` |
| **ENG-12** | **Ein Besitzer je Konzept:** Regler → `adapt`; Chunker → `ingest`; Temporalfilter/ValidityWindow → genau ein Besitzer (D-E5); Lizenz-Traits → `license`; Tombstone → `mvcc`. | R4, S-33/60/61/64 | `symbol-uniqueness`, Differenztests (§6.4) |
| **ENG-13** | **Nebenläufigkeit:** Lock-Reihenfolge `collections` → `kv_locks` → `embedder`; kein externer Aufruf (Embedder, LLM, VFS) unter synchronem Guard; `TxId` nur über `allocate_tx`; Fail-Closed bei I/O-/Index-/Integritätsfehlern. | `engine/SPEC` §3.2 | Loom, TSan (G-C1), `kv_lock_send_and_ordering`, `tx_allocation_concurrency` |
| **ENG-14** | **Löschkette, Engine-Anteil (Kap. 11):** Lesesperre vor Entschlüsselung/Ausgabe in O(1) ohne I/O auf **jedem** Lesepfad (Z-E1); Zugriffskontrolle wirkt vor Suche und Statistik, Rang und Zählwerte bleiben unverändert (Z-E8); Fail-Closed ohne Ersatzwerte (Z-E9); Metadaten und Logs ohne Roh-Kennungen (Z-E7). | §11.4.2, §11.5 | G-ER3 (Test je Zusage), Loom |
| **ENG-15** | **Klassentrennung (R-CLS):** jeder Schreibvorgang deklariert seine Datenklasse; fehlt sie → `Invalid`, nichts wird geschrieben. K-Schreiber nehmen nur `Validated<T>`. | §11.2, INV-13 | G-ER1 (Compile-Fail), G-PII1 |
| **ENG-16** | **Zeitreise (Kap. 12):** `as_of` (Transaktionszeit) und `valid_at` (Gültigkeitszeit, Kalendertage `i32`, `[valid_from, valid_until)`) unabhängig kombinierbar; Ausgaben deterministisch (Z-T1); Zugriffsrichtlinie nur im aktuellen Stand (Z-T5). | §12.2, §12.7 | G-TT1 |
| **ENG-17** | **Simulationsisolation (Kap. 13):** ein Szenario schreibt nie in `main`, P oder Rechte; temporärer Index nur unter dem Szenario und mit `retire` entfernbar. | §13.2–13.4, INV-16 | G-SIM1 |

ENG-14 bis ENG-17 sind **Engine-Anteile**; die Verantwortung für Gate, Saga, Ledger liegt in `erasure`, für Regeln/Versionsgraph in `rules`. Die Engine liefert die Durchsetzungspunkte (§6, EW-30 bis EW-33).

---

## 3 IST — verifizierter Stand

### 3.1 Verteilung des Codes (HEAD `24487fa`) [V]

| Modul in `db/src` | LOC | Seite im SOLL | Ziel-Besitzer nach SPEC |
|---|---:|---|---|
| `collection/` | 12.550 | engine | engine |
| `transaction/` | 1.658 | engine | engine |
| `chunker/` | 1.533 | engine | `ingest::chunking` (S-61), danach `pub use` |
| `contextra_impl/` | 1.452 | engine | engine |
| `background_workers/` | 1.298 | engine | engine |
| `extraction/` | 642 | engine (`exp-extraction`) | engine (§9.4) |
| `filter.rs` | 425 | engine | engine |
| `export.rs` / `import.rs` | 395 / 144 | engine | engine |
| `temporal_filter.rs` | 383 | engine | `rank::temporal` (S-33) / D-E5 |
| `orchestration/` | 330 | engine | engine |
| `merge_operator_adapter.rs` | 201 | engine (`sandbox`) | engine |
| `decay_controller.rs` | 198 | engine | `adapt` (S-60) |
| `fusion.rs` | 72 | engine | bereits Delegation an `rank` |
| `metrics_names.rs`, `backends.rs`, `license_seam.rs` | 66, 65, 7 | engine | engine; Lizenz → `license` |
| `adapters/` (3 Dateien) | 34 | engine [D] | engine (keine db-Verbraucher; `text.rs` braucht `text`) |
| `lib.rs`: `Contextra`, `ContextraConfig`, `Document`, `ContextraStats`, `SandboxBridge`, `Backends`-Re-Exports, Unit-Tests | ≈1.150 | engine | engine |
| `multistep.rs` | 442 | **db** | db (`multistep.rs bleibt`, §9.4) |
| `pid_latency_controller.rs` / `homeostat.rs` | 134 / 58 | **db** | `pub use` aus `adapt` (S-60) |
| `cognition/` | 19 | **db** | db (Feature `cognition`) |
| `volatile_vault.rs` (+ 3 leere `*.inc`) | 2 | **db** | `vault` (S-22), `*.inc` entfernen |

**Summe Engine-Seite:** 21.419 LOC / 83 Dateien (ohne `lib.rs`). **db-Rest:** ≈ 689 LOC plus Façade.

### 3.2 Schnittmatrix (Auszug der `crate::`-Kanten) [V]

```
collection  → decay_controller, metrics_names, transaction, filter, temporal_filter, fusion, chunker, extraction
contextra_impl → license_seam, transaction
background_workers → collection, decay_controller
export/import → collection, filter, export
multistep (db) → pid_latency_controller, fusion      ← einzige db→engine-Kante außer cognition
cognition (db) → collection
```

Engine-seitige Module referenzieren **kein** `multistep`, `homeostat`, `pid_latency_controller`, `cognition`, `volatile_vault`. `lib.rs` referenziert diese nur in `prelude`/`advanced`/Root-Re-Exports (Zeilen 458–511).

### 3.3 Abhängigkeiten, Features, Tests [V]

| Aspekt | `engine` (IST) | `db` (IST) |
|---|---|---|
| normale interne Kanten | nur `contextra-db` | `vault`, `cognition`(opt), `types`, `ports`, `mvcc`, `store`, `vector`, `text`, `checkpoint`, `rank`, `graph`, `sandbox`(opt), `infer-onnx`(opt), `adapt`(opt) |
| externe | — | `bytes, blake3, ahash, serde(_json), tokio, tokio-util, tracing, parking_lot, zeroize, thiserror` |
| Features | 11, alle leiten an `contextra-db/*` | 17: u. a. `bench`, `license`, `volatile-vault`, `coherence-bonus-fusion` |
| dev-Kanten | `types, ports, vector, store, text, graph, rank, checkpoint, ingest, testkit` | `vector, testkit, store, graph, text, adapt, checkpoint, ingest` |
| Tests | 50 Dateien / 8.630 LOC; `tests/golden/export_golden_v1.json` (+ `MANIFEST.sha256`); 2 nutzen `testkit`, 1 `ingest` | 53 Einträge; 8 Dateien nutzen Engine-interne Modulpfade (`collection`, `transaction`, `fusion`) |

**Externe Nutzung je Seite:** für `blake3`, `zeroize`, `thiserror` gibt es in keiner Seite einen Pfad-Treffer (`blake3::`, `zeroize::`, `thiserror::`) [V]; Makro-Nutzung ohne Pfad ist damit nicht ausgeschlossen → Entfernen nach `cargo check` prüfen (EW-10 B). `bytes`, `ahash`, `parking_lot`, `tokio-util`, `serde*`, `tracing`, `tokio` werden auf der Engine-Seite genutzt.

**Vorhandene Paritätsbeweise** (Voraussetzung für S-33/S-60/S-61): `proof_temporal_filter_vs_rank.rs`, `proof_regulators_vs_adapt.rs`, `proof_chunker_vs_ingest.rs`, `chunker_parity_ingest.rs`, `legacy_reference/*`. Namensschema weicht von §6.4/§7.3 ab (`diff_*` bzw. `differential_*`) → G-D1 erfasst sie sonst nicht.

**Crash-/Durability-Suite (INV-6):** `engine_commit_crash_points`, `commit_crash_consistency`, `engine_recovery_crash_windows`, `tx_fault_injection`, `invariant_durability`, `tx_recovery_and_erasure`, `multi_index_transaction`.

### 3.4 Befundregister `F-E`

| ID | Befund | Beleg | Behandlung |
|---|---|---|---|
| **F-E01** | `engine` ist Façade; SOLL verlangt eigenständigen E2-Baustein. | `engine/src/lib.rs`; SPEC §4.1 | EW-10 |
| **F-E02** | Wurzelartefakte fehlen: `Cargo.toml` (Workspace), `justfile`, `governance/`, `.github/`, `capabilities.toml`, `AGENTS.md`, `docs/decisions/`. Es existieren nur `.cargo/`, `Cargo.lock`, `rust-toolchain.toml`. | `git ls-files`, `ls -a` [V] | EW-00 (Blocker) |
| **F-E03** | Crate-lokale Doku widerspricht SPEC v6: „Erreicht“-Tabelle, Verweis auf `SPEC(11).md`, Modularitäts-Score 10/100, generierter Inventarblock („1 Datei, 7 LOC“ bei 15 Zeilen). | `engine/SPEC.md` §2, `PLAN.md` §1, `AGENTS.md` | EW-14 |
| **F-E04** | Feature-Drift: `license` fehlt in `engine`; `coherence-bonus-fusion` (3 `cfg`-Stellen) ist über `engine` nicht erreichbar; `edge-reinforcement-learning` hat 0 `cfg`-Stellen in `db` und leitet nichts weiter; `bench` und `license` haben 0 `cfg`-Stellen; `experimental-diskann` verletzt die `exp-*`-Konvention. | `db/Cargo.toml`, `grep` | EW-11 |
| **F-E05** | SPEC-Lücke §4.1: `adapt` und `ingest` sind keine Kernkanten von `engine`, obwohl S-60/S-61 `pub use` aus beiden verlangen und `decay_controller`/`chunker` immer verfügbar sind. Die `db`-Zeile nennt `adapt` nicht, obwohl `db` Regler per `pub use` aus `adapt` beziehen soll. Die „≤ 9 direkte“-Grenze aus `AGENTS.md` (Rang 3) wird mit `adapt`+`ingest` überschritten (11). | SPEC §4.1, §9.2, §9.4 | D-E6 |
| **F-E06** | `ValidityWindow` hat zwei Besitzer (`db/temporal_filter.rs`, `rank/temporal.rs`), künftig drei (`rules`). Die Einheit von `business_valid_*: Option<i64>` ist im Doc-Kommentar mit „ms/seconds“ unbestimmt; `as_of_business: Option<u64>`, `Query::as_of(u64)`. D-28 verlangt Kalendertage `i32`. | `db/src/temporal_filter.rs:20–60`, `builder.rs:246` | D-E5, EW-20, EW-32 |
| **F-E07** | Rote Ausgangslage: `db`-Tests referenzieren `contextra_db::memory_consolidation` (existiert nicht); `fault_injection_2pc` Typfehler; `engine`-Durability-Test rot; `contextra`: Compliance-Profil-Test und T-DEL-1 rot. | `grep`; `reports/AUDIT_REPORT.md` [A] | EW-02 |
| **F-E08** | INV-7: öffentliches Feld `ContextraConfig::deletion_proof_active`; 6 Treffer der Begriffssuche. | `db/src/lib.rs` (`ContextraConfig`), `lifecycle.rs` | EW-24, S-70 |
| **F-E09** | `ContextraConfig` ist nicht `#[non_exhaustive]`, alle Felder `pub`: jede Feldergänzung oder -umbenennung bricht Konsumenten mit Struct-Literal. | `db/src/lib.rs:221ff.` | D-E7 |
| **F-E10** | Dev-Kante `engine → testkit` (E2 → E3) verletzt INV-8; betrifft 2 von 50 Tests. | `engine/Cargo.toml`, `grep` | D-E3 |
| **F-E11** | §6.1 führt keine Engine-eigenen persistenten Formate (Commit-Intent, Orphan-Registry, Tenant-Index) im Golden-Korpus; `transaction/intent.rs` enthält keine Magic-Konstante [V] → Formatkennung unbestimmt. | `transaction/intent.rs` | EW-00, EW-02b |
| **F-E12** | `cognition ↔ db`: normale Kante `db → cognition` (Feature) plus dev-Kante `cognition → db`. Engine ist unberührt, S-63 bleibt offen. | `db/Cargo.toml`, `cognition/Cargo.toml` | außerhalb Scope, Nachführung |
| **F-E13** | Loom-Pfadfilter (`loom-gate.yml`) deckt `engine`/`db` nicht ab; Z-E1 verlangt Linearisierungstest. | SPEC §7.2 (G-C1) | EW-30 |

---

## 4 Zielarchitektur

### 4.1 Abhängigkeitsrichtung

```
contextra, mcp, py, agent ──► contextra-db ──► contextra-engine ──► store, vector, text, graph, rank,
                                  │   │                              checkpoint, mvcc, ports, types,
                                  │   └─► vault, cognition (opt)     adapt, ingest, [infer-onnx, sandbox, license]
                                  └─► adapt (Regler-Re-Exports, D-E6)
```

`engine` kennt weder `db` noch `cognition`. Der Re-Export-Pfad läuft ausschließlich `db → engine`.

### 4.2 Ziel-`Cargo.toml` `contextra-engine` (Entwurf, `[D]`)

```toml
[package]
name = "contextra-engine"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Contextra — Storage-, Index- und Transaktions-Orchestrierung (E2)"

[package.metadata.contextra]
ring = "3"

[lints]
workspace = true

[dependencies]
contextra-types      = { workspace = true }
contextra-ports      = { workspace = true }
contextra-mvcc       = { workspace = true }
contextra-store      = { workspace = true, features = ["workspace"] }
contextra-vector     = { workspace = true, features = ["workspace"] }
contextra-text       = { workspace = true, features = ["workspace"] }
contextra-graph      = { workspace = true, features = ["workspace"] }
contextra-rank       = { workspace = true, features = ["workspace"] }
contextra-checkpoint = { workspace = true }
# D-E6 (nach S-60/S-61):
contextra-adapt      = { workspace = true, default-features = false }
contextra-ingest     = { workspace = true, default-features = false, features = ["workspace"] }
# nur falls im Quelltext genutzt (EW-10 B): contextra-ids, contextra-env
contextra-sandbox    = { workspace = true, optional = true }
contextra-infer-onnx = { workspace = true, optional = true }
contextra-license    = { workspace = true, optional = true }   # nach S-50
bytes = { workspace = true }
ahash = { workspace = true }
parking_lot = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tokio-util = { version = "0.7.18", features = ["rt"] }
tracing = { workspace = true }

[features]
default = []
sandbox   = ["dep:contextra-sandbox"]
onnx      = ["dep:contextra-infer-onnx", "contextra-infer-onnx/onnx"]
license   = ["dep:contextra-license"]
adaptive-candidate-pool-sizing = []            # Verhalten; adapt ist Kernkante (D-E6)
exp-extraction = []
entity-extraction = ["exp-extraction"]
auto-extraction-opt-out = []
exp-diskann = ["contextra-vector/experimental-diskann"]  # Ziel: vector/exp-diskann nach S-12a; Alias experimental-diskann bis S-70
background-maintenance = []
graph-connectivity-health = ["contextra-graph/graph-connectivity-health"]
coherence-bonus-fusion = []
reranking = []
# edge-reinforcement-learning: nur behalten, wenn Weiterleitung/cfg belegt (F-E04)
```

Entfernt gegenüber `db`: `contextra-vault`, `contextra-cognition`, `zeroize`, `thiserror`, `blake3` (jeweils nach `grep`-Nachweis in EW-10 B). `dev-dependencies` ohne `db`, ohne `testkit` (D-E3).

### 4.3 Ziel-`Cargo.toml` `contextra-db` (Entwurf, `[D]`)

```toml
[dependencies]
contextra-engine = { workspace = true }
contextra-types  = { workspace = true }
contextra-ports  = { workspace = true }
contextra-vault  = { workspace = true }               # nach S-22 Workspace-Mitglied
contextra-adapt  = { workspace = true, default-features = false }   # D-E6
contextra-cognition = { workspace = true, optional = true }

[features]
default = []
cognition      = ["dep:contextra-cognition"]
volatile-vault = []
# Weiterleitung 1:1, damit bestehende Feature-Pfade gültig bleiben (INV-2):
sandbox        = ["contextra-engine/sandbox"]
onnx           = ["contextra-engine/onnx"]
license        = ["contextra-engine/license"]
adaptive-candidate-pool-sizing = ["contextra-engine/adaptive-candidate-pool-sizing"]
exp-extraction = ["contextra-engine/exp-extraction"]
entity-extraction = ["contextra-engine/entity-extraction"]
auto-extraction-opt-out = ["contextra-engine/auto-extraction-opt-out"]
background-maintenance  = ["contextra-engine/background-maintenance"]
graph-connectivity-health = ["contextra-engine/graph-connectivity-health"]
coherence-bonus-fusion  = ["contextra-engine/coherence-bonus-fusion"]
reranking      = ["contextra-engine/reranking"]
experimental-diskann = ["contextra-engine/exp-diskann"]   # Alias bis S-70
```

### 4.4 Ziel-Modulbild

```
contextra-engine/src
├─ lib.rs                 Contextra, ContextraConfig, Document, ContextraStats, SandboxBridge,
│                         Re-Exporte; prelude/advanced (ohne multistep/vault)
├─ backends.rs  collection/  transaction/  contextra_impl/  background_workers/
├─ export.rs  import.rs  filter.rs  fusion.rs  metrics_names.rs  orchestration/  adapters/
├─ chunker/        (→ Delegation an ingest::chunking, EW-22)
├─ decay_controller.rs   (→ pub use adapt, EW-21)
├─ temporal_filter.rs    (→ pub use Besitzer nach D-E5, EW-20)
├─ extraction/     (#[cfg(feature = "exp-extraction")])
├─ license_seam.rs (→ license, EW-23)
└─ merge_operator_adapter.rs (#[cfg(feature = "sandbox")])

contextra-db/src
├─ lib.rs          pub use contextra_engine::*; db-eigene prelude/advanced; deprecated Modul-Shims
├─ multistep.rs  homeostat.rs  pid_latency_controller.rs (→ pub use adapt)
├─ cognition/      (#[cfg(feature = "cognition")])
└─ volatile_vault.rs (→ pub use vault, S-22)
```

### 4.5 Pfadstabilität (ENG-07) und Deprecation

- `db/src/lib.rs` erhält `pub use contextra_engine::*;` und `pub use contextra_engine as engine;`. Explizite db-Items (`prelude`, `advanced`, `multistep`) überschreiben Glob-Importe; gleiche Items über zwei Globs sind in Rust kein Fehler.
- **R9 literal** verlangt `#[deprecated]`-Re-Exports. `#[deprecated]` auf `pub use` greift nach meinem Kenntnisstand nicht [E]. Verfahren: Shim-Module `#[deprecated(since="<nächste Version>", note="Pfad: contextra_engine::<modul>")] pub mod collection { pub use contextra_engine::collection::*; }` für **Modulpfade**; Root-Items (`contextra_db::Contextra`, `Collection`, …) bleiben **nicht** deprecated, weil `db` laut §4.1 die Verbraucher-Façade ist (D-E4, [D?]).
- `ContextraConfig` darf in diesem Vorhaben **nicht** um Felder erweitert oder umbenannt werden (F-E09, D-E7).
- `engine` behält seinen Namen und wird **nicht** mehr `#[deprecated]` (Gegenstück zu INV-2: der Name ist der Besitzer, nicht die Altlast). Die Façade-Semantik `pub use contextra_db::*` entfällt ersatzlos (Rückwärtspfeil).

---

## 5 Entscheidungen

| ID | Entscheidung | Begründung / Alternative | Status |
|---|---|---|---|
| **D-E1** | **Re-Extraktion:** Engine-Code zurück nach `contextra-engine`; `db` wird dünne Façade. | SPEC v6 §1.2 Rang 1; §4.1/§9.4/S-33/S-60/S-61/S-64 setzen `engine` voraus; Schnitt ist mechanisch (0 Rückkanten). *Alternative B:* SPEC ändern (`engine` streichen, `db` als Integrationsbaustein, §2.2, §4.1, §9.2, §9.4 anpassen). Kostet Änderungen an fünf SPEC-Abschnitten, belässt ≈ 22k LOC in der Verbraucher-Façade `db` und vergrößert deren Kern-Closure (G-L4). **B nur wählen, wenn ADR-0122 nachweislich Gründe nennt, die der SPEC widersprechen — ADR nicht auffindbar.** | **[D?]** ADR nötig |
| **D-E2** | `adapters/` wandert nach `engine` (34 LOC, keine `db`-Verbraucher). | `adapters/text.rs` braucht `contextra-text`, das `db` im SOLL-Kern nicht führt. | [D] |
| **D-E3** | Tests, die `testkit` brauchen (2 von 50), ziehen nach `testkit/tests/engine/` (E3 → E2). Kein `engine → testkit` dev-Kante. | INV-8; analog R12/D-12. Alternative: ADR-Ausnahme für E2 (Fragment führt `dev_allowed.testkit` bereits). | [D?] |
| **D-E4** | Deprecation nur für Modulpfad-Shims, nicht für Root-Items von `db`. | R9 vs. Façade-Zweck; vermeidet Warn-Flut in `contextra`/`mcp`/`py` unter `-D warnings`. | [D?] |
| **D-E5** | **Besitzer `ValidityWindow`/Zeitachsen:** `rules` besitzt den Typ (D-27/D-28, Kalendertage `i32`); `rank::temporal` und `engine::temporal_filter` konsumieren ihn. Bis S-104 bleibt `rank::temporal` Besitzer und `engine::temporal_filter` `pub use`. Da `rank` (E1a) `rules` (E1a) nicht im Kern führen darf (R13), übernimmt `rank::temporal` eine lokale Naht (`trait HasValidity`), `rules` implementiert sie in `compat/`. | R4 (ein Besitzer), R13, F-E06. Einheit `business_valid_*` ist vor S-105 durch Golden-Test festzustellen (Sekunden vs. Millisekunden); kein stilles Umdeuten (INV-1). | [D?] |
| **D-E6** | SPEC §4.1 wird ergänzt: `engine` führt `adapt` und `ingest` als Kernkanten (E0, `default-features = false`); `db` führt `adapt`. Die Grenze „≤ 9“ aus `AGENTS.md` wird auf „≤ 11 ohne `ids`/`env`“ angehoben. Kern-Closure von `engine` wächst um zwei E0-Mitglieder (E2 ist nicht budgetiert, §4.1). | Ohne Kernkante ist `pub use` in S-60/S-61 nicht ausführbar; ein Feature-Gate würde `AdaptiveDecayController`/`MarkdownChunker` aus dem Default entfernen (INV-2). | **[D?]** SPEC-Änderung via ADR |
| **D-E7** | `ContextraConfig` bleibt unverändert bis S-70; `deletion_proof_active` wird dann zu `integrity_audit_active` (Alias-Pfad über Builder-Methode, `#[non_exhaustive]` ab dieser Version). | F-E08/F-E09; Felder sind `pub` ohne `non_exhaustive`, Umbenennung ist Major-Änderung. | [D] |
| **D-E8** | **Datenklasse (R-CLS) wird je `Collection` deklariert** (`CollectionConfig::data_class`, Pflichtfeld ab Major), nicht je Einzelaufruf; K-Collections schreiben nur über `Validated<T>`-Pfade. | SPEC verlangt „jeder Schreibvorgang deklariert seine Klasse“ ohne Engine-Schnittstelle zu nennen; Collection-Ebene ist die kleinste Einheit, die Index, Schlüssel und Löschweg bindet. Per-Call-Deklaration vergrößert die Fehlerfläche (Z-E5). | **[D?]** |

---

## 6 Arbeitspakete

**Konvention:** Jedes Paket ist ein Commit-Paar `S-64E-<nn> A: …` (verschiebt/benennt um/sichert ab) und `S-64E-<nn> B: …` (baut um); beide einzeln revertierbar (INV-3, D-08). Nach **jedem** Commit gilt `cargo check --workspace --all-targets --locked` (INV-4). „DoD“ = SPEC §9.1 (8 Punkte) plus §8.

### Welle 0 — Verifikationsfähigkeit (kein Produktivcode)

#### EW-00 Baseline und Wurzelartefakte · **Blocker**
**Ziel:** Alle folgenden Aussagen werden messbar.
1. Fehlende Wurzelartefakte (F-E02) aus der Quelle des Eigentümers einspielen: Workspace-`Cargo.toml` (`members`, `exclude`, `default-members`, `[workspace.package]`, `[workspace.dependencies]`, `[workspace.lints]`), `justfile`, `governance/{gates,ratchet,protected-paths}.toml`, `.github/workflows/*`, `capabilities.toml`, `AGENTS.md`. Ist das nicht möglich: Rekonstruktion aus `Cargo.lock` und den Member-Manifesten, als Befund kennzeichnen, **kein Merge**, bis der Eigentümer bestätigt (INV-10).
2. Toolchain `1.89.0` bereitstellen (`static.rust-lang.org` ist laut `CLAUDE.md` nicht erreichbar; Weg über `apt`-Pakete `rustc-1.89`/`cargo-1.89` oder conda-forge).
3. Baseline-Läufe nach `docs/decoupling/baseline/engine/` ablegen: Workspace-Check; `--no-default-features` je Crate (`engine`, `db`); `cargo hack check --each-feature --no-dev-deps`; `cargo test -p contextra-engine --locked`; `cargo test -p contextra-db --no-run`. Ergebnis **je Testname**, nicht nur je Target.
4. API einfrieren: `cargo public-api -p contextra-db` und `-p contextra-engine` → `api_db_pre.txt`, `api_engine_pre.txt`. Daraus einen **Pfad-Anker-Test** generieren (eine `use`-Zeile je öffentlichem Pfad; Werkzeuge: `tools/generate_interface_spec.py`, `tools/contract_diff_detector.py`).
5. Golden: `sha256sum -c crates/contextra-engine/tests/golden/MANIFEST.sha256`. Engine-eigene persistente Formate erheben und mit 64- **und** 128-Bit-Korpus einfrieren (F-E11, §6.2): Commit-Intent-Datei, Orphan-Registry (`orphan_registry_path`), Tenant-Index (`tenant_index_persistence_reload`).
6. Messwerte: `tools/loc.py`, `zero_panic_analyzer.py` (klassifiziert `#[cfg(test)]` als Produktion → bereinigen), `determinism_guard.py`, `unsafe_boundary_auditor.py`, Zählung `unwrap/expect/panic` Engine-Seite (Rohwert 68 [V] inkl. Inline-Tests).
**Abnahme:** Bericht mit HEAD-Hash und Datum; jedes Test-Target als grün / rot / nicht kompilierbar; Pfad-Anker-Test kompiliert gegen `db` **und** `engine` (über Façade).

#### EW-01 Entscheidungen und SPEC-Deltas
1. ADR „ADR-0122-R1“ (D-E1) mit Kontext, Entscheidung, Revert-Pfad; ADRs für D-E3, D-E4, D-E5, D-E6, D-E8.
2. SPEC-Patch (D-E6): §4.1 Zeile `engine` + `adapt, ingest`; Zeile `db` + `adapt`; Anhang C `engine`; Anmerkung „≤ 9“ ersetzen. Eigentümer-Freigabe nötig (SPEC ist normativ).
3. Crate-lokale `engine/SPEC.md`/`PLAN.md` mit Kopfvermerk „überholt durch diese Spezifikation“ versehen (Ersatz in EW-14).
**Abnahme:** `docs/decisions/` existiert; G-DOC1 findet keinen toten Verweis.

#### EW-02 Rote Ausgangslage klassifizieren (F-E07)
a) `db`-Tests mit `contextra_db::memory_consolidation` (u. a. `docid_128_consolidation`, `consolidation_double_trigger_test`): Verbleib des Moduls per `git log -S memory_consolidation` klären; Tests zum Besitzer verschieben oder mit Nachweis nach `archive/tests-legacy/contextra-db/` (D-09).
b) `fault_injection_2pc`: Typfehler (Test-Veraltung) beheben, Assertions unverändert.
c) `invariant_durability::test_durability_after_wal_commit_and_simulated_crash` [A]: Der Fix-Report beschreibt einen Lock-Lebenszyklus-Defekt in `store` (`DirLock` am `Manifest`, `CompactionEngine`-Task, Re-Open nach Drop schlägt fehl; `store/src/lsm/recovery/startup.rs:466`). Ob derselbe Mechanismus den Engine-Test bricht, ist **[E]**. Test **nicht** abschwächen; als „bekannt rot“ in der Baseline führen, Befund an `store`.
d) `contextra`: `open_compliance_profile_succeeds_without_license` und T-DEL-1: Engine-Bezug sind der Lizenzpfad (`lifecycle.rs`) und die Delete→Purge-Orchestrierung; Befund an `erasure`/`store`, nicht Teil der Extraktion.
**Regel:** Test-Veraltung wird behoben, Produktivdefekte werden **nicht** in Extraktionscommits repariert (INV-6, INV-3). Die Extraktion beweist Neutralität über **Gleichheit der Ergebnismengen vorher/nachher**, rote Tests eingeschlossen.
**Abnahme:** jedes rote Target mit Klasse {Test-Veraltung behoben · Produktivdefekt + Owner · unklar}.

### Welle 1 — Re-Extraktion (D-E1)

#### EW-10 Kernumkehr
**Commit A (verschieben, Build grün):**
1. `git mv` der Module aus §3.1 (Engine-Seite inkl. `adapters/`) von `db/src/` nach `engine/src/`; Historie per `git log --follow` prüfen; keine Inhaltsänderung außer Pfaden und den Punkten 2–4.
2. `engine/src/lib.rs` := bisheriger `db/src/lib.rs` **ohne** `mod multistep`, `homeostat`, `pid_latency_controller`, `cognition`, `volatile_vault`, ohne Root-Re-Exports (Z. 501–511) und ohne die db-Zeilen in `prelude`/`advanced`. Alle `#[cfg(not(loom))]`-Gates 1:1 übernehmen; die ≈690 Zeilen Unit-Tests wandern mit (sie referenzieren keines der db-Module [V]).
3. `db/src/lib.rs` nach §4.5: `pub use contextra_engine::*;`, `pub use contextra_engine as engine;`, db-eigene `prelude`/`advanced` (Engine-Teile per Re-Export, plus `multistep`, Regler, `volatile_vault`).
4. `multistep.rs`: `crate::fusion` → `contextra_engine::fusion`. `cognition/bridge.rs`: `crate::collection::Collection` → `contextra_engine::collection::Collection`.
5. Manifeste: `engine/Cargo.toml` zunächst als **Obermenge** (alle bisherigen `db`-Abhängigkeiten, ohne `contextra-db`), `db/Cargo.toml` nach §4.3 mit 1:1-Feature-Weiterleitung. `Cargo.lock` im selben Commit mitführen (`--locked` muss bestehen).
6. `db/tests`, `db/examples`, `db/benches`, `engine/tests` bleiben vorerst unverändert.
**Abnahme A:** Workspace-Check grün; Pfad-Anker-Test (db + engine) grün; Ergebnismengen `-p contextra-engine` und `-p contextra-db` identisch zu EW-00; `git revert` des Commits baut (DoD 7).

**Commit B (umbauen):**
1. Unbenutzte Abhängigkeiten entfernen: Kandidaten `blake3`, `zeroize`, `thiserror` (kein Quelltext-Treffer auf beiden Seiten [V]); Abgleich mit `tools/abhaengigkeits_budget.py`. `contextra-ids`/`contextra-env` nur bei tatsächlicher Nutzung deklarieren, **kein** Massen-Rewrite von `contextra_types::{DocId,TenantId,TxId}`.
2. `db`-Manifest auf die Minimalmenge nach §4.3 reduzieren (`cargo check -p contextra-db --no-default-features`).
3. `capabilities.fragment.toml`: `engine` neu (`level`, `sublevel`, `core_deps`, `allowed_internal` inkl. `adapt`, `ingest`, `license`; `dev_allowed` ohne `testkit`); `db`-Fragment **neu anlegen** (fehlt [V]).
4. `cfg(loom)`-Builds für `engine` prüfen (Loom-Tests des `kv_lock`-Pfads).
**Abnahme B:** G-L1 (keine Kante `engine → db|cognition|contextra|testkit`, normal/dev/Feature), G-L3, G-A1 (siehe EW-12), Loom-Lauf grün.
**Rollback:** `git revert` B, dann A; Façade-Zustand `engine → db` ist wiederherstellbar.

#### EW-11 Feature-Modell (R10)
1. F-E04 abarbeiten: `license` aufnehmen (zunächst leer, mit `cfg`-Stelle in `license_seam`); `coherence-bonus-fusion` in `engine` aufnehmen; `edge-reinforcement-learning` entweder an `contextra-graph/edge-reinforcement-learning` weiterleiten (`cognition` tut das bereits [V]) oder entfernen; `bench` entfernen, sofern keine `required-features`-Nutzung; `experimental-diskann` → `exp-diskann` mit Alias bis S-70.
2. `default = []` in `engine` und `db`.
**Abnahme:** `cargo hack check --each-feature --no-dev-deps -p contextra-engine -p contextra-db` grün; `feature_audit.py` ohne Befund; jedes Feature mit ≥ 1 `cfg` oder Weiterleitung.

#### EW-12 Tests, Beispiele, Anker
1. Acht `db`-Tests mit Engine-Modulpfaden nach `engine/tests/`: `context_compaction_retry_test`, `cross_domain_chaos_matrix_test`, `fault_injection_2pc`, `fusion_edge_cases_test`, `graph_entity_delete_test`, `proptest_search_invariants`, `signal_kind_no_new_variant`, `split_brain_vector_orphan_test`; `diskann_collection_fallback` mit `required-features = ["exp-diskann"]`.
2. `s64_public_surface.rs` durch den generierten Pfad-Anker-Test (`engine/tests/api_anchor.rs`, `db/tests/api_anchor.rs`) ersetzen; der Konsumentenanker für `cognition` bleibt.
3. Die zwei `testkit`-Tests nach `testkit/tests/engine/` (D-E3).
4. Differenztests auf das Schema `differential_*` bringen (G-D1-Muster, F-E-Hinweis §3.3).
**Abnahme:** Testanzahl vorher = nachher (Muster S-12b); G-A1: jeder Pfad aus `api_db_pre`/`api_engine_pre` auflösbar.

#### EW-13 Façade-Shims und Konsumenten
1. Modul-Shims `#[deprecated] pub mod <modul> { pub use contextra_engine::<modul>::*; }` in `db` für alle verschobenen Modulpfade (D-E4); Root-Items unverändert.
2. In-Repo-Konsumenten (`contextra`, `mcp`, `py`, `agent`, `cognition`-Tests, `benchmarks/`) auf `contextra_engine::<modul>` umstellen, **wo Modulpfade** genutzt werden (Zählung in EW-00). Root-Imports bleiben.
3. `db/DEPRECATED_DEBT.md` (40 `#[allow(deprecated)]`-Stellen) entlang der Dateien aufteilen; `no_blanket_allow_deprecated` auf `engine` erweitern.
**Abnahme:** `cargo clippy --workspace --all-targets --locked -- -D warnings` frei von Deprecation-Warnungen aus Repo-Code; G-A1.

#### EW-14 Bausteinvertrag und Doku (ENG-05)
`engine/README.md` (Beispiel ≤ 10 Zeilen, nur `contextra_engine`), `examples/engine_minimal.rs` (fehlt [V]), `docs/decoupling/errors/engine.md` (aus `ERRORS.md`), neue `SPEC.md`/`PLAN.md` mit **Ist-Aussagen** und Verweis auf diese Spezifikation (INV-7), regenerierter `AGENTS.md`-Inventarblock (`tools/crate_inventory.py agents`), `REPORT.md` nach §8.4, `version` an allen Pfad-Abhängigkeiten.
**Abnahme:** G-DOC1, `capabilities-consistency`.

### Welle 2 — Entkopplung nach SPEC §9.2 (je Paket: Differenztest zuerst, dann Umbau)

#### EW-20 Temporalfilter (S-33 Anteil, D-E5)
A: Differenztest `engine::temporal_filter` ↔ `rank::temporal` aus `proof_temporal_filter_vs_rank.rs` erweitern: Grenzen von `[from, until)`, `None`, Legacy-Fail-Open, `tx`-Achse; **Einheit von `business_valid_*` feststellen** (Sekunden vs. Millisekunden, F-E06). B: `engine::temporal_filter` → `pub use contextra_rank::temporal::*`.
**Abnahme:** bitgleich (§6.4); `integration_temporal_filter`, `query_builder_time_travel`, `proof_temporal_filter_vs_rank` grün; G-D1.

#### EW-21 Regler (S-60) · benötigt S-23, D-E6
A: Methodenparität (decay 5, homeostat 3, pid 5 [Ü]) mit `ManualClock`, f64 bitgleich; fehlendes Feld `base_half_life_ms: None` in `tests/decay_and_expiry.rs:19` ergänzen. B: `engine::decay_controller` und `db::{homeostat, pid_latency_controller}` → `pub use contextra_adapt::…`.
**Abnahme:** G-D1; `decay_and_expiry`, `proof_regulators_vs_adapt`, `pid_pool_production_caller` grün.

#### EW-22 Chunker (S-61) · benötigt D-E6
A: `proof_chunker_vs_ingest` und `chunker_parity_ingest` auf die **Chunk-Folge samt Breadcrumbs** erweitern [E, §9.2 S-61]. B: `engine::chunker` delegiert an `ingest::chunking` (Brücke `workspace`); `contextra::chunker::MarkdownChunker` API-stabil.
**Abnahme:** G-D1; `chunker_golden_tests`, `chunker_hardening_tests`, `chunker_robustness`, `proof_chunker_properties`; `mcp`-Tests unverändert.

#### EW-23 Lizenz (S-50/S-64) · benötigt S-50
A: `license_seam.rs` auf `contextra_license` hinter Feature `license`; ohne Feature identische `OpenFastGate`-Semantik (`s64_license_default_semantics`). B: Feature in `db` und `contextra` weiterleiten (F-02 separat in S-01/S-64).
**Abnahme:** G-L1…G-L3; G-A1 ohne ungewollten Diff; Klassifikation aus EW-02 d).

#### EW-24 Hygiene und Neutralität
1. INV-12: 68 Rohtreffer klassifizieren; Test-Code ausnehmen; Produktivstellen **nur in angefassten Dateien** in `Result` überführen (Fail-Closed, ENG-13).
2. INV-7: Wortfilter-Lauf; `deletion_proof_active` bleibt bis S-70 (D-E7), Doku neutral.
3. ENG-08: einzige Zeitquelle im Engine-Code ist `Instant::now()` (`lifecycle.rs:255`) [V]; Rangfolgerelevanz ausschließen, sonst über `env::Clock::monotonic_nanos`.
4. ENG-04: Display-Tabelle je Variante tabellengetrieben (G-E1).
**Abnahme:** `unwrap-gate`, `determinism_guard.py`, G-E1, G-DOC1.

### Welle 3 — Engine-Anteile Kapitel 11–13 · benötigt S-100, S-101…S-103, S-104…S-106

#### EW-30 Lesepfad-Gate (ENG-14: Z-E1, Z-E8, Z-E9) [D?]
- **Port** `ReadGate` (Besitzer `erasure`-Naht nach S-50): `check(&self, tenant, target) -> GateVerdict { Allowed, Erased, NotFound }`, O(1), kein I/O, kein Await.
- **Einhängepunkte** (Vollständigkeit per Aufrufgraph, Z-E1 „jeder Lesepfad“): `Collection::{get, scan, search, query, hybrid, text, vector, graph-Traversal, relate}`, Export, Snapshot und `as_of`, `multistep`, Stats. Prüfung **vor Entschlüsselung** und **vor Kandidatenbildung**; kein Post-Filter, weil Overfetch (`hybrid_adaptive_overfetch`) Rang und Zählwerte sonst abhängig von gesperrten Dokumenten macht (Z-E8). Der Filter fließt in die Scope-/ACORN-Kandidatenmenge ein.
- **Fehlerbild:** Unberechtigte erhalten dasselbe Ergebnis wie bei nicht vorhandenem Ziel; Berechtigte `Erased`. Schreibpfade verweigern.
- **Tests:** Matrix `Lesepfad × {Erased, unberechtigt, erlaubt}`; Rangtest mit unzugänglichem Dokument; Fehlerinjektion (Gate-Fehler → fail-closed, keine Ersatzwerte); Loom-Modell Gate-Setzung vs. Lesepfad; `engine` in den Pfadfilter von `loom-gate.yml` (F-E13).
**Abnahme:** G-ER3 (Z-E1, Z-E8, Z-E9 je ≥ 1 grüner Test), G-C1.

#### EW-31 Klassendeklaration (ENG-15: R-CLS, Z-E5, Z-E7) [D-E8, D?]
- `DataClass { K, P, D, E }` mit Besitzer festlegen (ADR; R11 beachten). `CollectionConfig::data_class` Pflichtfeld; fehlende Klasse → `Invalid`, nichts wird geschrieben.
- K-Schreibpfad nimmt nur `Validated<T>` (privater Konstruktor, erzeugt ausschließlich über den `privacy::entity_recognizer`-Adapter). Compile-Fail-Tests (`trybuild`) für G-ER1, Mustertest G-PII1 mit synthetischen Canaries.
- Z-E7: `tracing`-Felder, `metrics_names`, Fehlertexte enthalten keine Roh-Kennungen; Canary-Suche über alle Ausgaben.
- **Migration:** kein stilles Default (fail-closed). Bestehende Collections nur über explizite, im Bericht ausgewiesene Übergangsklasse.
**Abnahme:** G-ER1, G-PII1.

#### EW-32 Zeitreise (ENG-16) · benötigt S-104, S-105, D-E5
- Zwei getrennte Achsen am Query-Builder: `as_of_tx` (Sequenz) und `valid_at` (Tag, `i32`). `as_of(u64)` bleibt `#[deprecated]` mit dokumentierter Abbildung (INV-2), Einheit gemäß EW-20.
- `ValidityWindow` aus `rules`; Golden-Test der Intervallgrenzen auf x86_64 und aarch64.
- Z-T5: Test mit älterem Stand und erweiterter Richtlinie (Zugriff bleibt aktuell).
**Abnahme:** G-TT1; `query_builder_time_travel`, `invariant_determinism`.

#### EW-33 Simulationsisolation (ENG-17) · benötigt S-107 [D?]
Szenario-Namespace `sim/<id>` mit temporärem Index (eigener Collection-/Verzeichnisscope); `retire` entfernt deterministisch. `SimHandle` besitzt **keinen** Schreibweg auf Nicht-`sim`-Collections (Typzustand statt Laufzeitprüfung). Proptest über Operationsfolgen: Hash von `main`-Stand, P und Rechten vor/nach identisch.
**Abnahme:** G-SIM1.

### Welle 4

#### EW-40 Abnahme und Entnehmbarkeit (§8.4)
`crates/contextra-engine/docs/decoupling/REPORT.md` mit HEAD-Hash und Datum: G-L2, G-L1, G-L3, G-L4, G-H1 grün; README-Beispiel kompiliert; Golden-/Breitentest grün; Bausteinvertrag vollständig; `cargo package -p contextra-engine --locked` als Probe. Fragment-Feld `extractable`/`verified_at`. Messbericht (S-111) für die Engine-Pfade: L1-Sperre p99 < 50 ms bei 1.000 Versionen, Zeitreise p95 < 200 ms bei 1 Mio. Blobs (§14.5, [Q]).

---

## 7 Reihenfolge und Abhängigkeiten

| Paket | hängt ab von | externe SPEC-Voraussetzung | Größe [E] | Welle |
|---|---|---|---|---|
| EW-00 | — | Wurzelartefakte vom Eigentümer | M | 0 |
| EW-01 | EW-00 | Eigentümer-Freigabe SPEC-Patch | S | 0 |
| EW-02 | EW-00 | — | M | 0 |
| EW-10 | EW-00, EW-01 (D-E1), EW-02 | S-22 empfohlen vor Änderungen an `db` (§9.3) | L | 1 |
| EW-11 | EW-10 | — | S | 1 |
| EW-12 | EW-10, EW-11 | — | M | 1 |
| EW-13 | EW-12 | — | M | 1 |
| EW-14 | EW-10 | — | S | 1 |
| EW-20 | EW-14 | S-33 (`rank::temporal`) | M | 2 |
| EW-21 | EW-14 | S-23, D-E6 | M | 2 |
| EW-22 | EW-14 | D-E6 | M | 2 |
| EW-23 | EW-11 | S-50 | S | 2 |
| EW-24 | EW-14 | — | M | 2 |
| EW-30 | EW-24 | S-101…S-103, S-50 | L | 3 |
| EW-31 | EW-30 | S-100, S-106, D-E8 | L | 3 |
| EW-32 | EW-20 | S-104, S-105, D-E5 | M | 3 |
| EW-33 | EW-31, EW-32 | S-107 | M | 3 |
| EW-40 | alle | S-111 | S | 4 |

**Parallel:** EW-20…EW-24 untereinander; EW-31 und EW-32 nach EW-30/EW-20. **Seriell zwingend:** EW-00 → EW-10 → EW-12; EW-10 A vor B.

---

## 8 Gates und Definition of Done

Je Paket gilt SPEC §9.1 (Commit-Paar, INV-4, Gates der Zeile, Score/Brückenzählung neu, lokale `SPEC`/`PLAN` aktuell, keine neue R10–R14-Verletzung, Revert baut, G-DOC1). Für `engine` zusätzlich:

| Nr. | Kriterium | Prüfung |
|---|---|---|
| E1 | API-Äquivalenz: jeder Pfad aus `api_db_pre` und `api_engine_pre` löst auf | Pfad-Anker-Test, G-A1 |
| E2 | Ergebnismengen je Testname vorher = nachher (rote Tests eingeschlossen) | Diff der Testlisten |
| E3 | Golden-Hashes unverändert, 64- und 128-Bit | G-F1, `docid-128`-Lauf |
| E4 | Crash-Suite (§3.3) unverändert | `cargo test -p contextra-engine --test engine_commit_crash_points …` |
| E5 | Loom/TSan der Lock-Pfade | G-C1 (Pfadfilter um `engine` ergänzt) |
| E6 | Keine Kante `engine → db`; kein Zyklus | G-L1 |
| E7 | Feature-Matrix | `cargo hack check --each-feature --no-dev-deps` |
| E8 | Keine neuen `unwrap`/`panic`/`unsafe`; Neutralität | `unwrap-gate`, `unsafe-inventory`, G-DOC1 |

---

## 9 Risiken und Rückfall

| Risiko | Wirkung [E] | Gegenmaßnahme / Rückfall |
|---|---|---|
| Wurzelmanifest nicht rekonstruierbar | Extraktion nicht verifizierbar | EW-00 ist Blocker; kein Merge ohne Eigentümer-Bestätigung |
| Rote Basis maskiert Regression | falsche Neutralitätsaussage | Vergleich je Testname (E2), nicht je Target |
| Glob-Re-Exports kollidieren (`prelude`, `advanced`, `engine`) | Pfad verschwindet oder wird mehrdeutig | Pfad-Anker-Test, `cargo doc` ohne Warnung |
| `cfg(loom)`/Feature-Gates beim Verschieben verloren | Loom-Tests laufen nicht mehr | 1:1-Übernahme, Loom-Lauf in EW-10 B |
| `Cargo.lock`/`--locked` bricht | CI rot | Lock im Commit A mitführen |
| `ContextraConfig` ohne `non_exhaustive` | SemVer-Bruch bei Feldänderung | D-E7: bis S-70 unverändert |
| D-E6 abgelehnt | S-60/S-61 für `engine` nicht ausführbar | `decay_controller`/`chunker` bleiben lokal, Differenztest als Wächter, R4-Ausnahme per ADR |
| D-E1 abgelehnt | Struktur bleibt | Alternative B: EW-10…EW-13 entfallen; ENG-01…ENG-17 gelten für `db`; SPEC §2.2, §4.1, §9.2, §9.4 ändern; `engine`-Façade bis S-70 (INV-2) |
| Z-E8-Vorfilter verändert Performance/Rang | Regression in Recall oder Latenz | Benchmarks (`benchmarks/`, `db`-Benches), Ziel §14.5, Differenz zu Baseline |
| Gate rot nach Commit B | — | SPEC §9.5: Revert B innerhalb eines Arbeitstags, A bleibt, Befund ins Register |

---

## 10 Offene Punkte (vor Start zu klären)

1. Inhalt von `ADR-0122` und Existenz von `SPEC(11).md`; zeitliche Einordnung der Auflösung relativ zu `fcd387021` (Hinweis: SPEC v6 misst den Stand vor der Auflösung, bezeichnet ihn aber als HEAD) [E].
2. Wurzelartefakte (F-E02) und Quelle der Wahrheit für `[workspace.dependencies]`.
3. Einheit von `ValidityWindow::business_valid_*` (F-E06).
4. Persistente Formate der Engine und deren Kennungen (F-E11).
5. Ursache des roten `invariant_durability`-Tests (F-E07 c).
6. Anzahl der Konsumenten-Aufrufstellen mit Modulpfaden `contextra_db::<modul>` (bestimmt Umfang EW-13); im Klon nur drei `volatile_vault`-Treffer in Nicht-Brace-Imports [V], Brace-Imports nicht ausgewertet.
7. Der Engine-Report `docs/decoupling/REPORT.md` (Gate-Matrix „Grün“, Score 10/100) bezieht sich auf den Stand vor der Auflösung und ist als Beleg ungültig.
8. Ob `D-E8` (Klasse je Collection) mit dem Konzept von `erasure`/`rules` verträglich ist; SPEC nennt keine Engine-Schnittstelle zu R-CLS.

---

## Anhang A Prüfbefehle

```bash
# Struktur / Kanten
python3 tools/gate_layering.py --crate contextra-engine
python3 tools/isolation_proof.py contextra-engine          # G-L4
python3 tools/gate_bridge_null.py                           # G-L2 (E2: informativ)
python3 tools/abhaengigkeits_budget.py
# Build / Features
cargo check --workspace --all-targets --locked
cargo check -p contextra-engine -p contextra-db --no-default-features --locked
cargo hack check --each-feature --no-dev-deps -p contextra-engine -p contextra-db
cargo check --workspace --features docid-128 --locked      # F-02: Feature-Pfad vorab klären
# Tests / Gates
cargo test -p contextra-engine --locked
cargo test -p contextra-db --locked
sha256sum -c crates/contextra-engine/tests/golden/MANIFEST.sha256
cargo public-api -p contextra-engine ; cargo public-api -p contextra-db
RUSTFLAGS="--cfg loom" cargo test -p contextra-engine --release --lib kv_lock
# Statik
python3 tools/zero_panic_analyzer.py --crate contextra-engine
python3 tools/determinism_guard.py --crate contextra-engine
python3 tools/decouple_inspector.py -c contextra-engine --erasure-leak-check
python3 tools/doc_lint.py
# Aufrufsyntax der Skripte vor Verwendung gegen `--help` prüfen (nicht ausgeführt)
# Messung Schnitt (reproduziert §3.2)
grep -rn 'contextra_db' crates/contextra-engine/src | wc -l   # Ziel: 0
```

## Anhang B Rückverfolgung SPEC → Arbeitspaket

| SPEC | Inhalt | Paket |
|---|---|---|
| §4.1 `engine`/`db` | Zielarchitektur | EW-10, EW-11 |
| §5.4 | Bausteinvertrag | EW-14, EW-40 |
| §6.1/§6.2 | Golden, ID-Breite | EW-00, EW-12 |
| §6.3 | Fehlerabbildung | EW-14, EW-24 |
| §6.4 | Differenztests | EW-20…EW-22 |
| S-33, S-60, S-61 | Temporal, Regler, Chunker | EW-20, EW-21, EW-22 |
| S-50, S-64 | Lizenz, `engine`-Features | EW-23, EW-11 |
| S-22 | `vault` Mitglied, `volatile_vault` | Voraussetzung EW-10 (db), Folgeschritt in `db` |
| S-63 | `cognition`-Inversion | außerhalb; EW-10 berührt nur `cognition/bridge.rs` |
| S-70 | Aufräumen (`deprecated`, Aliase, `deletion_proof_active`) | nach EW-40 |
| §11.4.2, Z-E1/E8/E9 | Lesesperre | EW-30 |
| §11.2, Z-E5/E7, G-ER1, G-PII1 | Klassentrennung | EW-31 |
| §12, Z-T1/T5, G-TT1 | Zeitreise | EW-20, EW-32 |
| §13, Z-S1, G-SIM1 | Simulation | EW-33 |
| §8.4, §14.5, S-111 | Entnehmbarkeit, Leistung | EW-40 |
