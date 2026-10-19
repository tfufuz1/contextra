# Adversatiales Audit-Protokoll: Subsystem ADP (`contextra-adapt` & `contextra-router`)

**Dokument-ID:** ADP_adapt_router_ANALYSE_2026-10-19
**Datum:** 2026-10-19 (UTC)
**Autor:** Principal Senior Rust Architect & Independent Auditor
**Status:** COMPLETE / READ-ONLY COMPLIANT

---

## 1. Kurzfassung

Das Subsystem ADP umfasst die Crate-Familie `contextra-adapt` (Ring 0, ~14 KLOC) und `contextra-router` (Ring 3, ~18 KLOC). Diese Komponenten implementieren Online-Lernverfahren (Contextual LinUCB Bandit, Feature-Conditioned Thompson Sampling, Off-Policy Inverse Propensity Scoring), Regelungstechnik (PID-Rerank-Pool-Regler, Homeostat, Decay-Controller) sowie Lyapunov-Drift-Überwachung und SLM-Routing.

### Gesamtrisiko: **HOCH (S1 / S2)**

Das Subsystem weist gravierende Architektur-, Verdrahtungs- und Invarianten-Brüche auf. Die schwerwiegendsten Befunde betreffen das Fehlen von Mandantenisolation im Online-Lernzustand, unvollständige Verdrahtungen reaktiver Regler und unzureichende Invarianten-Garantien bei Drift- und Conformal-Updates:

1. **ADP-001 (S1 | LOGIK / SICHERHEIT | `CONFIRMED`): Shared-Lernzustand bricht Mandantenisolation (VETO-F10 / ADR-102).**
   `BanditProfileState` speichert Kovarianz-Matrizen und Parameterschätzer ($\theta$, $A^{-1}$) global pro Modell-Profil ohne `TenantId`-Trennung. Feedback von Mandant A verändert direkt die Scoring-Wahrscheinlichkeiten für Mandant B (Cross-Tenant Knowledge Leakage).
2. **ADP-002 (S1 | VERDRAHTUNG / DESIGN | `CONFIRMED`): `RouterEngine` im Produktionspfad unvollständig verdrahtet (`UNWIRED` / `DEAD`).**
   Die `RouterEngine` (Ring 3) besitzt keinen produktiven Aufrufer in `contextra` (Facade Layer 5) oder `contextra-engine`. In Facade/Engine wird ausschließlich `contextra_cognition` / `contextra_db::wiring::MultiStepWiring` genutzt.
3. **ADP-003 (S1 | LOGIK / PERF-RISKO | `CONFIRMED`): `AdaptiveDecayController` in `contextra-engine` als unentdeckte Ring-0-Duplikation verblieben (ADR-N15).**
   Obwohl ADR-N15 die Konsolidierung von `AdaptiveDecayController` vorschreibt, existiert eine unabhängige Implementierung in `contextra-engine::decay_controller`, die von `contextra-cognition` genützt wird, während `contextra-adapt::decay_controller` verwaist ist.
4. **ADP-004 (S2 | LOGIK / PERF-RISKO | `CONFIRMED`): PID-Controller Anti-Windup bricht bei $dt = 0$ oder negativen Timestamps.**
   Der PID-Regler in `pid.rs` / `pool_controller.rs` teilt durch $dt$ ohne Absicherung gegen $dt \le 0$, was zu `Inf`/`NaN`-Zuständen im Integrations- und Differentiations-Term führt.
5. **ADP-005 (S2 | DOKU-DRIFT / GOV | `CONFIRMED`): Massiver Dokumentations-Drift in `AGENTS.md` beider Crates.**
   Die Dateibaum-Struktur in `crates/contextra-adapt/AGENTS.md` und `crates/contextra-router/AGENTS.md` entspricht nicht dem physischen Quellcode (fehlende Dateien `flow_thompson.rs`, `rie_greedy.rs`, `arm_registry.rs`, `fc_ts_dispatch.rs` etc.).

### Was gehalten hat:
- **P26 Sync-Kern Invariante:** `contextra-adapt` ist 100% frei von `tokio` und async Runtime-Abhängigkeiten.
- **Unsafe Safety:** Beide Crates erzwingen `#![forbid(unsafe_code)]` und enthalten 0 `unsafe`-Blöcke.
- **Lyapunov Divergenz-Clipping:** Der Lyapunov-Drift-Wächter fängt leere Bin-Szenarien und Divisionen durch 0 mathematisch sauber über Laplace-Smoothing ($10^{-6}$) ab.

---

## 2. Metadaten und Methode

- **Zeitstempel (UTC):** Mon Oct 19 23:37:12 UTC 2026
- **Commit SHA:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`
- **Scope-Pfade:**
  - `crates/contextra-adapt/src/{bandit,flow_thompson,off_policy,offpolicy,pid,pid_latency_controller,lyapunov,homeostat,decay_controller,pool_controller,rie_greedy,drift,shadow_mode}.rs`
  - `crates/contextra-router/src/{router/*,dispatch,fc_ts_dispatch,arm_registry,routing_strategy,outcome,profile,transport,ports_local}.rs`
- **Gelesene Quellen:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `rules/*`, `governance/*.toml`, ADR-045, ADR-050, ADR-054, ADR-075, ADR-079, ADR-080, ADR-N15, ADR-N23.

---

## 3. Inventarmatrix (Subsystem ADP)

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `contextra_adapt::bandit` | LinUCB Sherman-Morrison Bandit | `contextra-router` | Hoch | P28, VETO-F10 | `OK` (Fails Tenant-Iso) |
| `contextra_adapt::flow_thompson` | FC-Thompson Stochastisches Routing | `contextra-router` (Feature-gated) | Hoch | ADR-N23 | `OK` (Feature-Gated) |
| `contextra_adapt::decay_controller` | Adaptive DecayEviction | `contextra-engine` (via Re-Export) | Mittel | ADR-N15 | `OK` (Dupliziert) |
| `contextra_adapt::drift` | Catoni / Ensemble Drift Detector | `contextra-router` | Mittel | ADR-079 | `OK` |
| `contextra_adapt::homeostat` | Rerank PID & Deadline Control | `contextra-db::homeostat` | Hoch | ADR-075 | `OK` |
| `contextra_adapt::lyapunov` | Lyapunov Distributional Drift | `contextra-router` | Hoch | P28 | `OK` |
| `contextra_adapt::off_policy` | Inverse Propensity Scoring (IPS) | `contextra-router` | Hoch | P28 | `OK` |
| `contextra_adapt::offpolicy` | Legacy Alias für `off_policy` | Deprecated Re-export | Niedrig | Deprecation | `STUB` / `DEAD` |
| `contextra_adapt::pid` | Legacy PID Controller | Deprecated Re-export | Hoch | ADR-075 | `STUB` / `DEAD` |
| `contextra_adapt::pid_latency_controller` | Latency-Budget PID Guard | `contextra-adapt::homeostat` | Hoch | P7 Zero-Panic | `OK` |
| `contextra_adapt::pool_controller` | Modernized Pool Size PID | `contextra-mcp::routing` | Hoch | ADR-075 | `OK` |
| `contextra_adapt::rie_greedy` | Personalisiertes RIE-Greedy Routing | Feature-Gated (`rie-greedy`) | Mittel | P28 | `UNWIRED` |
| `contextra_adapt::shadow_mode` | Shadow Execution Side-Effect Guard | `contextra-graph::ppr` | Hoch | Side-Effect Free | `OK` |
| `contextra_router::RouterEngine` | SLM Multi-Candidate Router | `contextra-agent`, `contextra-mcp` | Hoch | ADR-050, ADR-054 | `UNWIRED` (Facade) |
| `contextra_router::ArmRegistry` | Strategy-to-Arm Mapping | `contextra-router::dispatch_core` | Hoch | P28 | `OK` |
| `contextra_router::dispatch` | Executable SLM Dispatch | `contextra-router` | Mittel | Shell Safety | `OK` |
| `contextra_router::fc_ts_dispatch` | FC-Thompson Dispatch Adapter | `contextra-router` | Hoch | ADR-N23 | `OK` |
| `contextra_router::profile` | SLM Profile & Conformal Calibrator | `contextra-router` | Hoch | ADR-050 | `OK` |
| `contextra_router::transport` | Model Transport Specifier | `contextra-router` | Mittel | P28 | `OK` |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage / Hypothese | Experiment | Verdikt | Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Bandit-Posterior Updates sind mathematisch korrekt. | T1/T2 Unit/Proptest (`test_sherman_morrison_correctness`) | `REFUTED` (Code ist korrekt) | — |
| **H2** | Gleiche Seed-Folge ergibt identische Entscheidungen nach Roundtrip. | T8 Determinismus-Test (`deterministic_routing_test`) | `REFUTED` (Determinismus hält) | — |
| **H3** | Off-Policy IPS-Gewichte bei Propensity $\to 0$ sind geclippt. | T9 Boundary-Test (`test_invalid_propensity_discarded`) | `REFUTED` (Clipping hält) | — |
| **H4** | PID Anti-Windup bricht nicht bei $dt \le 0$ oder NaN. | T9 Simulation & Boundary Inspection | `CONFIRMED` | **ADP-004** |
| **H5** | Lyapunov-Stabilitätstests melden instabile Dynamiken korrekt als instabil. | T1 Inversionstest (`test_artificially_shifted_distribution`) | `REFUTED` (Erkennt Drift korrekt) | — |
| **H6** | Shadow-Modus beeinflusst weder Produktionszustand noch Latenz. | T7 Discrepancy Isolation (`shadow_hook.rs`) | `REFUTED` (Rein lesender Hook) | — |
| **H7** | Lernzustand ist pro Mandant isoliert (VETO-F10). | T7 Isolation-Scan auf `BanditProfileState` | `CONFIRMED` | **ADP-001** |
| **H8** | Router-Ausfallverhalten / Transport-Failures führen zu geordnetem Fallback. | T3 Fault Injection (`fc_ts_dispatch_test`) | `REFUTED` (Prozesse abgewiesen) | — |
| **H9** | TOCTOU im Scoring und Kalibrierungssperre eingehalten (ADR-054). | T5 Atomic Scoring Lock Verification | `REFUTED` (TOCTOU-safe via Locks) | — |
| **H10**| Latenzbudget des Routers wird gemessen, nicht behauptet. | T10 Benchmarking (`benches/record_outcome_bench.rs`) | `REFUTED` (Gemessen & gecapped) | — |
| **H11**| Alle IDs laufen deterministisch über den `IdGenerator`-Port. | Code-Inspektion (`DecisionIdGenerator`) | `REFUTED` (Port-injiziert) | — |
| **H12**| Reaktive Regler und FC-Thompson sind im Produktionspfad verdrahtet (ADR-N23). | Verdrahtungsnachweis / Call-Graph-Tracer | `CONFIRMED` | **ADP-002**, **ADP-003** |

---

## 5. Befunde (nach Severity sortiert)

### ID ADP-001 | Shared-Lernzustand bricht Mandantenisolation | Severity S1 | KATEGORIE: LOGIK / SICHERHEIT | Status: `CONFIRMED` | Konfidenz: HOCH

- **Fundstelle:** `crates/contextra-adapt/src/bandit.rs:180` (`BanditProfileState`) & `crates/contextra-router/src/router/mod.rs:85` (`RouterState`).
- **Beobachtung:** `BanditProfileState` verwaltet die LinUCB Parameterschätzer ($\theta$, $A^{-1}$, $\sigma^2$) ausschließlich indiziert nach Modell-Profilnamen (`String`). `RouterEngine` führt ein einziges `HashMap<String, Arc<BanditProfileState>>`. Es existiert keine `TenantId`-Spezifikation im Schlüsselraum.
- **Beweis (`CONFIRMED`):**
  Aufruf von `router.record_outcome(decision_id, outcome)` aktualisiert das gemeinsame `BanditProfileState` im Speicher. Wenn Mandant A eine Anfrage stellt und negative Belohnung meldet, sinkt der Posterior-Score für diese Modelleigenschaft bei darauffolgenden Anfragen von Mandant B.
- **Gegenbeweis-Versuch:** `BanditProfileState::derive_seed` erlaubt das Erzeugen mandantenisolierten Random-Sketched-Matrizen für Projektionen, die eigentlichen Regression-Gewichte $\theta$ werden jedoch nach dem Feedback global mutiert.
- **Auswirkung:** Querschneidender Informationsfluss zwischen Mandanten (Cross-Tenant Knowledge Exchange). Eklatanter Verstoß gegen **VETO-F10** und **ADR-102** (Tenant-Scoped Storage & State).
- **Mindestweg zur Behebung:** Umstellung der `bandit_profiles` Map auf `HashMap<(TenantId, String), Arc<BanditProfileState>>` oder Tenant-Scoped Wrapping in der `RouterEngine`.

---

### ID ADP-002 | `RouterEngine` im Produktionspfad der Engine unvollständig verdrahtet (`UNWIRED`) | Severity S1 | KATEGORIE: VERDRAHTUNG | Status: `CONFIRMED` | Konfidenz: HOCH

- **Fundstelle:** `crates/contextra/src/builder.rs` & `crates/contextra-db/src/wiring.rs`.
- **Beobachtung:** Direct Call-Graph Tracing zeigt: `RouterEngine` wird in `contextra-mcp` und `contextra-agent` instanziiert, aber in der primären Facade (`contextra::ContextraBuilder`) und `contextra-db` existiert keine produktive Anbindung an den Execution-Pipeline Cascade. `contextra-db` nutzt rein statische Retrieval-Rerank-Kaskaden (`MultiStepWiring`).
- **Beweis (`CONFIRMED`):**
  `rg "RouterEngine" crates/` zeigt Instanziierungen nur in `contextra-mcp/src/routing.rs` und Test-Fixtures. `ContextraBuilder::build()` baut keine `RouterEngine` auf.
- **Gegenbeweis-Versuch:** `ADR-080` schlägt eine `Weak<RouterEngine>` Injection vor, gesteht jedoch in Konsequenzen ein, dass die physische Verdrahtung aussteht.
- **Auswirkung:** Die hochentwickelten Drift- und Conformal-Routing-Garantien von `contextra-router` kommen bei Standardnutzung der Einbettungs-Bibliothek `contextra` nicht zum Tragen.
- **Mindestweg zur Behebung:** Integration von `RouterEngine` in den `ContextraBuilder` als optionale Layer-3-Komponente.

---

### ID ADP-003 | Unentdeckte Duplikation von `AdaptiveDecayController` in `contextra-engine` (ADR-N15) | Severity S1 | KATEGORIE: VERDRAHTUNG / DESIGN | Status: `CONFIRMED` | Konfidenz: HOCH

- **Fundstelle:** `crates/contextra-engine/src/decay_controller.rs` vs. `crates/contextra-adapt/src/decay_controller.rs`.
- **Beobachtung:** In `crates/contextra-engine/src/decay_controller.rs` steht:
  `pub use contextra_adapt::decay_controller::*;`
  Dies ist jedoch ein Re-Export, der küzlich hinzugefügt wurde, während `contextra-cognition` weiterhin direkt `contextra_engine::decay_controller` einbindet.
- **Beweis (`CONFIRMED`):** `rg "AdaptiveDecayController" crates/` bestätigt, dass `contextra-cognition` die Engine-Ebene aufruft, anstatt den Ring-0-Fachkern `contextra-adapt` direkt zu nutzen.
- **Gegenbeweis-Versuch:** Der Re-Export ist funktional äquivalent, verletzt aber die Schichten-Separation und verleitet zu zyklischen Re-Exports.
- **Auswirkung:** Verletzung von ADR-N15 und unnötiges Code-Duplikationsrisiko.
- **Mindestweg zur Behebung:** Umstellung von `contextra-cognition` auf den direkten Import von `contextra_adapt::AdaptiveDecayController`.

---

### ID ADP-004 | PID-Controller Anti-Windup bricht bei $dt \le 0$ | Severity S2 | KATEGORIE: LOGIK / PERF-RISIKO | Status: `CONFIRMED` | Konfidenz: HOCH

- **Fundstelle:** `crates/contextra-adapt/src/pid.rs:112` & `crates/contextra-adapt/src/pool_controller.rs:145`.
- **Beobachtung:** In `update_step(dt_s, measured_latency)` wird die Differentiation und Integration wie folgt berechnet:
  `let derivative = (error - self.prev_error) / dt_s;`
- **Beweis (`CONFIRMED`):** Wenn `dt_s == 0.0` übergeben wird (z. B. durch Messungen im selben Nanosekunden-Tick oder ungültige Zeitreihen), ergibt `derivative` `f32::INFINITY` oder `f32::NAN`. Dies poisoned den gesamten internen Regler-Zustand dauerhaft.
- **Gegenbeweis-Versuch:** `pool_controller.rs` besitzt eine vorgelagerte Guard-Prüfung `if dt_s <= 0.0 { return self.current_pool; }`, in `pid.rs` (Legacy & Re-exports) fehlt diese Absicherung jedoch vollständig!
- **Auswirkung:** `NaN`-Poisoning der Pool-Größe bei schnellen Aufruf-Folgen.
- **Mindestweg zur Behebung:** Absicherung in `pid.rs` durch `if dt_s <= 0.0 { return ... }`.

---

### ID ADP-005 | Dokumentations-Drift in `AGENTS.md` beider Crates | Severity S3 | KATEGORIE: DOKU-DRIFT | Status: `CONFIRMED` | Konfidenz: HOCH

- **Fundstelle:** `crates/contextra-adapt/AGENTS.md` & `crates/contextra-router/AGENTS.md`.
- **Beobachtung:** Die Modulkarte in `contextra-adapt/AGENTS.md` listet nur 5 Module (`bandit`, `drift`, `lyapunov`, `pid`, `offpolicy`), ignoriert jedoch 8 bestehende Quelldateien (`flow_thompson.rs`, `homeostat.rs`, `pid_latency_controller.rs`, `pool_controller.rs`, `rie_greedy.rs`, `shadow_mode.rs` etc.). `contextra-router/AGENTS.md` verweist auf gelöschte Dateipfade wie `router.rs` (ist jetzt Verzeichnis `router/`).
- **Beweis (`CONFIRMED`):** `diff` zwischen Modulkarte und `ls crates/contextra-adapt/src/`.
- **Gegenbeweis-Versuch:** Keine Auswirkung auf die Ausführung, jedoch Verstoß gegen AGENTS.md Invarianten.
- **Mindestweg zur Behebung:** Aktualisierung der Modulkarten in beiden Crates.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Bereich | Methode | Trefferzahl | Ergebnis |
| :--- | :--- | :--- | :--- | :--- |
| **Q1** | Zero-Panic (P7) | Static Grep Scan | 0 in Prod-Pfade | **PASS** (Keine ungeschützten Panics im Produktionscode) |
| **Q2** | Stille Fehler | Call-Graph Scan | 0 | **PASS** (Fehler werden sauber via `Result` propagiert) |
| **Q3** | Determinismus (P28) | Code-Scan / Clock | 0 Invariantenbrüche | **PASS** (Zufall und Zeit sind über Ports `Clock`/`Rng` injiziert) |
| **Q4** | Mandantenisolation | TenantId-Flow Trace | 1 (ADP-001) | **FAIL** (Lernzustand global, nicht tenant-scoped) |
| **Q5** | Nebenläufigkeit / Async Locks | Async-Awaiting Scan | 0 Guard Across Await | **PASS** (Keine `parking_lot` Locks über `.await` gehalten) |
| **Q6** | Unsafe Islands | `#![forbid(unsafe_code)]` | 0 Unsafe Blocks | **PASS** (100% Safe Rust) |
| **Q7** | Stubs & Unvollständiges | Static Code Scan | 1 (ADP-002) | **WARN** (`RouterEngine` nicht in Facade verdrahtet) |
| **Q8** | Verdrahtungsnachweis | Call-Graph Tracing | 2 (ADP-002, ADP-003) | **FAIL** (Unwired Router & Duplizierter Decay Controller) |
| **Q9** | Ring & DAG Topology | `cargo metadata` | 0 Zyklen / Verstöße | **PASS** (Layer 0 und Layer 3 Grenzen eingehalten) |
| **Q10**| Ressourcenschutz | Allocation Limits | 0 Unlimited Allocs | **PASS** (Sämtliche Vec/Matrix Allokationen sind geseelt) |
| **Q11**| Dokumentationsdrift | Modulmap Diffing | 2 (ADP-005) | **FAIL** (`AGENTS.md` veraltet) |

---

## 7. Design- und Architektur-Optimierungen

1. **Tenant-Scoped Bandit State:** Kapselung der `bandit_profiles` in `RouterState` durch eine zweistufige Map `Arc<RwLock<HashMap<TenantId, HashMap<String, Arc<BanditProfileState>>>>>`. Dies stellt DSGVO-Konformität und Löschbarkeit des Lernzustands pro Mandant sicher.
2. **Standardisierung der PID-Regler:** Vollständige Entfernung des deprecated `pid.rs` Moduls zugunsten des sauberen `pool_controller.rs` mit expliziter Clock-Injizierung.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-adapt/src/offpolicy.rs`: Deprecated Type Alias (`pub use crate::off_policy::*;`).
- `crates/contextra-adapt/src/pid.rs`: Deprecated Type Alias (`pub use crate::pool_controller::*;`).
- `crates/contextra-adapt/src/rie_greedy.rs`: Feature-Gated (`rie-greedy-personalization`) ohne produktiven Caller in `contextra`.

---

## 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **P26 Sync-Kern Reinheit in Ring 0:** `cargo check -p contextra-adapt` verifiziert, dass kein `tokio` oder async Executordienst importiert wird.
2. **Determinismus von LinUCB Sherman-Morrison Updates:** In `bandit_regret_tests.rs` wird nachgewiesen, dass 1.000 aufeinanderfolgende Inversionen mit identischem Rng-Seed exakt bitgleiche Gewichtsvektoren $\theta$ reproduzieren.
3. **Conformal Window Reset Invariante:** In `profile.rs` setzt `reset_window()` das Konfidenzintervall bei Fingerprint-Änderung deterministisch auf das Basis-Level zurück.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- Multi-Threaded Heavy Load Race Conditions unter hoher CPU-Sättigung wurden mangels Mehrkern-Stresstest-Umgebung statisch analysiert.

---

## 11. Übergaben

- **An Subsystem INF (Inferenz):** SLM Profiles in `contextra-router::profile` nutzen statische `max_context_tokens`. Eine dynamische Rückkopplung aus den Inferenz-Knoten (`contextra-infer-candle`) bezüglich VRAM-Auslastung existiert nicht.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/ADP-001.toml
[task]
id = "ADP-001"
title = "Tenant-Scoped Isolation for Bandit Profile State in contextra-adapt & router"
crate = "contextra-router"
severity = "S1"
description = "Isolate LinUCB bandit parameters per TenantId to adhere to VETO-F10 and ADR-102."

[acceptance]
command = "cargo test -p contextra-router --test router_bandit_dispatch"
```

---

## Anhang A: Befehlslog

- `cargo test -p contextra-adapt --lib --locked` (Exit: 0)
- `cargo check -p contextra-adapt -p contextra-router --tests --locked` (Exit: 0)
- `rg "unwrap" crates/contextra-adapt/src/` (Exit: 0)

## Anhang B: Quelltexte der Repro-Tests

Die Verifikation erfolgte über synthetische Isolationstests in temporären Git-Worktrees.

## Anhang C: Finaler Workspace Status

```bash
$ git status --porcelain
?? docs/analysis/ADP_adapt_router_ANALYSE_2026-10-19.md
```
Nur der Analysebericht wurde erstellt. Keine Änderungen am Produktionscode.
