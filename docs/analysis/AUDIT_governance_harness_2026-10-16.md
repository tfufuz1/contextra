# Deep-Dive Audit-Bericht: Governance, Harness, Doku-Wahrheit und Schuldenabbau

**Rolle:** M1 — Spezialist für Governance, Entwicklungsharness, Dokumentationswahrheit und Schuldenabbau
**Scope:** `AGENTS.md`, `CLAUDE.md`, `CONSTITUTION.md`, `VETOES.md`, `rules/`, `governance/`, `.jules/`, `xtask/`, `docs/`, `capabilities.toml`, Workflow-Dateien, Crate-`AGENTS.md`
**Datum:** 2026-10-16
**Delta-Basis:** `docs/analysis/AUDIT_governance_harness_DEEP_2026-10-13.md` & `docs/analysis/AUDIT_GESAMTSYNTHESE_2026-10-14.md` (Commit: `f9429808`)

---

## 1. Zusammenfassung

| Prüfbereich | Ampel | Grund / Status |
|---|---|---|
| **Dokumentationswahrheit & Drift** | 🔴 ROT | 50+ veraltete Befehls- & Pfadreferenzen in Crate-AGENTS.md ohne `<!-- harness:planned -->` / `<!-- doc-ref-ignore -->` |
| **Harness & Registry-Konsistenz** | 🔴 ROT | `registry-check` & `gates-catalog-sync` schlagen fehl (`check-port-names`, `check-no-stubs`, `fast-diff-lint`, `context-card`) |
| **Port- & Feature-Isolierung** | 🔴 ROT | `check-port-features` schlägt fehl (Selbst-Dependency in `contextra-ports`, `test-utils` in Prod-Deps, `backend-impl` in Facade) |
| **Architektur & Single Source** | 🔴 ROT | `gc-floor-single-source` (60 Verstöße) und `determinism-check` (`Instant::now()` in Ring 0/1) schlagen fehl |
| **Doktrin & Unsafe-Inseln** | 🟡 GELB | `fast-diff-lint`, `check-no-downcast`, `check-loom-mocks` grün; status_contradiction in `capabilities.toml` bzgl. P1/P2 Warnungen |
| **Vetoes & Fristen** | 🟢 GRÜN | VETO-F02, VETO-F10, VETO-OP03 intakt; Review-Fristen bis 2027-10-12 verlängert (ADR-VETO-REVIEW-F02-OP03-2026-10.md) |
| **Supply Chain & Workflows** | 🟡 GELB | Actions gepinnt, `--locked` in CI; `env-attest` meldet fehlendes `cargo-fuzz` im Host-Container |

---

## 2. Delta zum Vorgängerbericht (13./14.10.2026)

- **Neu identifiziert:**
  - `registry-check` Kollisionen & Fehlstellen: `check-port-names` und `check-no-stubs` fehlen in `xtask/registry.toml`.
  - `gates-catalog-sync` Entkopplung: `fast-diff-lint` und `context-card` fehlen im zentralen Gate-Katalog `governance/gates.toml`.
  - `check-port-features` Verstöße in `contextra-ports/Cargo.toml`, `contextra-mcp/Cargo.toml`, `contextra/Cargo.toml`.
  - 60 dezentrale `min_active_seqno()` Aufrufe außerhalb `crates/contextra-mvcc/src/floor.rs`.
- **Behoben seit 13.10.2026:**
  - FINDING-GOV-01: `rules/simd_safety.md` stellt `contextra-crypto` explizit als Nicht-Unsafe-Insel klar.
  - FINDING-GOV-02: `[crates.xtask-heavy]` in `capabilities.toml` eingetragen.
  - FINDING-GOV-03: Wurzel-Skripte `apply_all_fixes.py` und `clean_patch.py` aus Repo-Root gelöscht.
- **Offen:**
  - Systemweite Doku-Drift in Crate-AGENTS.md bezüglich geplanter xtask-Befehle (`check-ring0-async-purity`, `check-unsafe-islands` etc.).
  - Determinismus-Verstöße (`Instant::now()`, `SystemTime::now()`, `Uuid::new_v4()`) in Ring 0 und Ring 1 Prod-Pfaden.
- **Verschlechtert:**
  - Keines.

---

## 3. Befunde (ID, Schwere, Datei:Zeile, Beleg, Risiko, Empfehlung, Aufwand)

### FINDING-GOV-20261016-01: Unregistrierte Harness-Kommandos & Registry-Kollision
- **Schwere:** P1
- **Datei:Zeile:** `xtask/registry.toml:1`, `xtask/src/harness/check_port_names.rs:1`, `xtask/src/harness/check_no_stubs.rs:1`
- **Beleg:** `cargo xtask registry-check` (Exit-Code 1). `check-port-names` und `check-no-stubs` sind als Module/Arms implementiert, fehlen aber in `registry.toml`.
- **Risiko:** Nicht-Registrierung führt zur Umgehung von CI-Aggregat-Gates und verzerrt die Befehlswahrheit.
- **Empfehlung:** Einträge für `check-port-names` und `check-no-stubs` in `xtask/registry.toml` ergänzen und CLI-Match-Arm-Kollision mit dem Harness-Generator auflösen.
- **Aufwand:** S

### FINDING-GOV-20261016-02: Gate-Katalog Katalog-Drift (`gates-catalog-sync`)
- **Schwere:** P1
- **Datei:Zeile:** `governance/gates.toml:1` vs `.jules/harness-phases.toml:40`
- **Beleg:** `cargo xtask gates-catalog-sync` (Exit-Code 1). `fast-diff-lint` und `context-card` sind in `.jules/harness-phases.toml` als `required = true` deklariert, fehlen jedoch im zentralen Katalog `governance/gates.toml`.
- **Risiko:** Inkonsistenzen zwischen lokalen Jules-Harness-Phasen und CI-Sicherheits-Gate-Katalogen.
- **Empfehlung:** `[[gate]]` Blöcke für `fast-diff-lint` und `context-card` in `governance/gates.toml` eintragen.
- **Aufwand:** S

### FINDING-GOV-20261016-03: Port-Feature & Dependency-Invariante gebrochen (`check-port-features`)
- **Schwere:** P1
- **Datei:Zeile:**
  - `crates/contextra-ports/Cargo.toml:30`
  - `crates/contextra-mcp/Cargo.toml:19`
  - `crates/contextra/Cargo.toml:38`
- **Beleg:** `cargo xtask check-port-features` (Exit-Code 1).
  - `contextra-ports` hängt in `[dev-dependencies]` von sich selbst ab.
  - `contextra-mcp` zieht `test-utils` im Produktionsbereich `[dependencies]`.
  - `contextra` schaltet `backend-impl` ein (nur für `contextra-store` erlaubt).
- **Risiko:** Brechen der Ring-0 Entkopplung, Aufblähen von Production-Binaries mit Test-Mocks.
- **Empfehlung:** Selbst-Abhängigkeit in `contextra-ports` entfernen, `test-utils` in `contextra-mcp` nach `[dev-dependencies]` verschieben, `backend-impl` aus `contextra/Cargo.toml` entfernen.
- **Aufwand:** S

### FINDING-GOV-20261016-04: Verletzung des Single-Source GC Floor Prinzips (`gc-floor-single-source`)
- **Schwere:** P2
- **Datei:Zeile:** `crates/contextra-store/src/lsm/ops/purge.rs:28` und 59 weitere Stellen in `contextra-store` und `contextra-mvcc`.
- **Beleg:** `cargo xtask gc-floor-single-source` (Exit-Code 1). 60 direkte Aufrufe von `min_active_seqno()`.
- **Risiko:** Dezentrale Abfragen der MVCC-Garbage-Collection-Grenze können zu Inkonsistenzen bei Compaction und Purge führen.
- **Empfehlung:** Abfragen zentral über `crates/contextra-mvcc/src/floor.rs` leiten.
- **Aufwand:** M

### FINDING-GOV-20261016-05: Systemweite Doku-Drift in AGENTS.md Dateien (`doc-truth`)
- **Schwere:** P2
- **Datei:Zeile:** `crates/contextra-*/AGENTS.md` (z.B. `contextra-vector/AGENTS.md:12`, `contextra-crypto/AGENTS.md:45`)
- **Beleg:** `cargo xtask doc-truth` (Exit-Code 1). 50+ Warnungen wegen veralteter Pfade und ungekennzeichneter geplanter Befehle.
- **Risiko:** Agenten führen nicht-existierende Befehle aus oder suchen nach gelöschten Moduldateien.
- **Empfehlung:** Geplante Befehle mit `<!-- harness:planned -->` annotieren und gelöschte Pfadreferenzen mit `<!-- doc-ref-ignore -->` versehen oder korrigieren.
- **Aufwand:** M

### FINDING-GOV-20261016-06: Non-Deterministic Time/RNG Calls in Ring 0/1 (`determinism-check`)
- **Schwere:** P2
- **Datei:Zeile:** `crates/contextra-ports/src/clock.rs:55`, `crates/contextra-store/src/lsm/ops/write.rs:323`, `crates/contextra-sys/src/determinism.rs:26`
- **Beleg:** `cargo xtask determinism-check` (Exit-Code 1). Ungekapselte Aufrufe von `Instant::now()`, `SystemTime::now()` und `Uuid::new_v4()`.
- **Risiko:** Brechen der P28 Determinismus-Garantie für Simulation, Replay und Property-Testing.
- **Empfehlung:** Systemaufrufe hinter die Ports `Clock`, `EntropyRng`, `IdGenerator` legen.
- **Aufwand:** L

---

## 4. Testlücken und Testverbesserungsvorschläge

1. **`xtask` Harness Fixture-Tests für Subkommandos:**
   - **Lücke:** Subkommandos wie `check-port-names`, `check-no-stubs`, `fast-diff-lint` besitzen keine synthetischen Test-Fixtures für den Positiv- (Pass) und Negativfall (Fail mit Exit 1/2).
   - **Vorschlag:** Integrationstests in `xtask/tests/` erstellen, die temporäre Verzeichnisstrukturen mit fehlerhaften und korrekten Manifesten/Dateien anlegen, das Subkommando ausführen und Exit-Code + JSON-Ausgabe verifizieren.
2. **`doc-truth` automatischer Regressionstest:**
   - **Lücke:** Keine automatische CI-Garantie gegen schleichende Pfad-Drift in neu erstellten Crate-AGENTS.md Dateien.
   - **Vorschlag:** `doc-truth` im Phase check in `.jules/harness-phases.toml` verpflichtend schalten.

---

## 5. Schuldenregister (Technical Debt Register)

| Typ | Ort | Risiko | Aufwand | Reihenfolge | Delta vs 13.10. |
|---|---|---|---|---|---|
| **Doku-Drift** | `crates/*/AGENTS.md` | P2 (Agenten-Verwirrung) | M | 3 | Gleich (gehäuft durch Crate-Splits) |
| **Metadata-Drift** | `xtask/registry.toml`, `governance/gates.toml` | P1 (CI-Bypass) | S | 1 | Neu identifiziert |
| **Dependency-Debt** | `contextra-ports`, `contextra-mcp`, `contextra` | P1 (Architektur-Bruch) | S | 2 | Neu identifiziert |
| **Architektur-Schuld** | `contextra-store`, `contextra-mvcc` (`min_active_seqno`) | P2 (Single Source) | M | 4 | Wächst durch Store-Refactoring |
| **Determinismus-Schuld** | `contextra-ports`, `contextra-store`, `contextra-sys` | P2 (Replay-Sicherheit) | L | 5 | Gleich |
| **Altlast** | `capabilities.toml` Status-Konditionierung | P3 (Warn-Rauschen) | M | 6 | Wächst durch Lints |

---

## 6. Vorschlag von 5 Schuldenabbau-Task-Karten (nach ROI sortiert)

### Karte 1: `registry-check` & `gates-catalog-sync` Ausrichtung (ROI: EXTREM HOCH)
- **Scope:** `xtask/registry.toml`, `governance/gates.toml`, `xtask/src/harness/check_port_names.rs`, `xtask/src/harness/check_no_stubs.rs`
- **Ziel:** Registrierung von `check-port-names` & `check-no-stubs` in `registry.toml`, Behebung der Match-Arm-Kollision, Eintragen von `fast-diff-lint` & `context-card` in `gates.toml`.
- **Akzeptanzbefehl:** `cargo xtask registry-check && cargo xtask gates-catalog-sync`

### Karte 2: Port-Feature & Dependency-Isolierung (ROI: HOCH)
- **Scope:** `crates/contextra-ports/Cargo.toml`, `crates/contextra-mcp/Cargo.toml`, `crates/contextra/Cargo.toml`
- **Ziel:** Bereinigung der Selbst-Dependency in `contextra-ports`, Verschiebung von `test-utils` in `contextra-mcp` nach `dev-dependencies`, Entfernung von `backend-impl` in `contextra/Cargo.toml`.
- **Akzeptanzbefehl:** `cargo xtask check-port-features`

### Karte 3: Doku-Wahrheit & AGENTS.md Bereinigung (ROI: HOCH)
- **Scope:** `AGENTS.md`, `crates/*/AGENTS.md`, `docs/decisions/README.md`
- **Ziel:** `<!-- harness:planned -->` Annotierung aller geplanten Subkommandos, Korrektur veralteter Pfad-Referenzen in `contextra-vector/AGENTS.md`.
- **Akzeptanzbefehl:** `cargo xtask doc-truth`

### Karte 4: Zusammentragen des GC-Floor Single-Source Zugriffs (ROI: MITTEL)
- **Scope:** `crates/contextra-store/`, `crates/contextra-mvcc/`
- **Ziel:** Kapselung aller `min_active_seqno()` Aufrufe über `crates/contextra-mvcc/src/floor.rs`.
- **Akzeptanzbefehl:** `cargo xtask gc-floor-single-source`

### Karte 5: P28 Determinismus-Härtung in Ring 0/1 (ROI: MITTEL)
- **Scope:** `crates/contextra-ports/`, `crates/contextra-store/`, `crates/contextra-sys/`, `crates/contextra-vector/`
- **Ziel:** Ersetzen ungekapselter `Instant::now()` / `SystemTime::now()` Aufrufe durch `Clock` / `Rng` Port-Injektionen.
- **Akzeptanzbefehl:** `cargo xtask determinism-check`

---

## 7. Hypothesen

1. **Hypothese 1:** Das Fehlen von `flatc` im Host-Container beeinträchtigt nicht die Sicherheit, da FlatBuffers-Generates beim Build gepost-prozessiert und mit `// SAFETY:` annotiert vorliegen. (Bestätigt durch `cargo xtask check-loom-mocks` und `cargo test -p contextra-wire`).
2. **Hypothese 2:** Die `min_active_seqno()` Verstöße in Testdateien (`tests/*.rs`) verursachen im laufenden Betrieb keine Datenkorruption, verletzen aber das Kapselungs-Invarianzprinzip G-11.

---

## 8. Quellenkonflikte und Doku-Drift

1. **`governance/gates.toml` vs. `.jules/harness-phases.toml`:**
   - `.jules/harness-phases.toml:40` verlangt `fast-diff-lint` und `context-card` als verpflichtende lokale Gates.
   - `governance/gates.toml` enthält beide Gates nicht.
2. **`xtask/registry.toml` vs. `xtask/src/harness/`:**
   - Module `check_port_names.rs` und `check_no_stubs.rs` existieren im Dateisystem und sind in `main.rs` verdrahtet, fehlen jedoch in `xtask/registry.toml`.
3. **Crate-`AGENTS.md` vs. `xtask harness-list`:**
   - Zahlreiche Crate-AGENTS.md (z.B. `contextra-types/AGENTS.md`) dokumentieren Befehle wie `cargo xtask check-type-registry` oder `cargo xtask check-agents-integrity` als direkt ausführbar, obwohl diese in `harness-list` als planned markiert oder nicht registriert sind.

---

## 9. Ausgeführte Befehle und Exit-Codes

| Befehl | Exit-Code | Status / Bemerkung |
|---|---|---|
| `cargo xtask harness-list` | 0 | Verbindliche Befehlsliste abgerufen |
| `cargo xtask registry-check` | 1 | FAIL (Fehlende Einträge & Arm-Kollisionen) |
| `cargo xtask gates-catalog-sync` | 1 | FAIL (Fehlende Pflichtgates in gates.toml) |
| `cargo xtask doc-truth` | 1 | FAIL (Veraltete Befehls- & Pfad-Referenzen) |
| `cargo xtask repo-doctor` | 1 | FAIL (Workspace-Verify Drift & missing tools) |
| `cargo xtask orphan-symbols` | 2 | FAIL (80+ ungenutzte pub Symbole) |
| `cargo xtask unwired-ports` | 0 | PASS (Keine unverdrahteten Ports im Fokus) |
| `cargo xtask stub-impls` | 0 | PASS (Warnmodus) |
| `cargo xtask doctrine-scan` | 1 | FAIL (Status-Widersprüche) |
| `cargo xtask dependency-graph-audit` | 0 | PASS (Keine DAG-Zyklen) |
| `cargo xtask determinism-check` | 1 | FAIL (Instant::now / Uuid in Ring 0) |
| `cargo xtask fast-diff-lint` | 0 | PASS (Sauberes Diff) |
| `cargo xtask gc-floor-single-source` | 1 | FAIL (60 min_active_seqno Verstöße) |
| `cargo xtask env-attest` | 1 | FAIL (cargo-fuzz fehlt) |
| `cargo xtask check-loom-mocks` | 0 | PASS (Keine verbotenen Loom-Mocks) |
| `cargo xtask check-no-downcast` | 0 | PASS (Keine verbotenen Downcasts) |
| `cargo xtask check-no-stubs` | 0 | PASS (G-30 Warn-Modus) |
| `cargo xtask check-port-features` | 1 | FAIL (3 Feature-Invarianz-Brüche) |
| `cargo xtask check-port-names` | 0 | PASS (Standard-Ports §7.4 vorhanden) |

---

## 10. Nicht Geprüftes und Gründe

- **Volle Mutations-Tests (`cargo xtask mutants-diff`):** Ausgeführt als diff-scoped, jedoch kein vollständiger Workspace-Lauf wegen mehrstündiger Laufzeit (gehört zu Nightly/CI matrix).
- **Loom Concurrency Model Checks (`cargo xtask loom-run`):** Werden im PR-Risk-Tier `concurrency` getriggert; statischer Scanner `check-loom-mocks` wurde erfolgreich ausgeführt.
