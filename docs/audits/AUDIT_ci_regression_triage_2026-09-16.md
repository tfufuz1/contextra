# Audit-Report: CI Post-Merge-Regression Triage & Root-Cause Triage

**Datum:** 2026-09-16
**Auditor:** Google-Jules (Principal Rust Systems Engineer)
**Ziel-Crate:** Repo-weit (`workspace-audit` / `ci-triage`) — Untersuchungsobjekt: `jules-preflight` Gate-Pipeline
**Prüfobjekt:** Preflight-Pipeline & Regression Issues (#2458, #2585, #2636, #2657, #2659, #2661, #2664, #2693, #2705, #2707, #2716, #2731, #2737, #2738, #2740, #2743, #2754, #2767, #2808, #2811, #2818, #2827, #2831, #2837, #2838, #2846, #2854)
**Referenz-Spezifikation:** `CONSTITUTION.md`, `.github/workflows/post-merge-verification.yml`, `xtask/src/main.rs`
**VERDICT: REJECTED / ACTION_REQUIRED** (ID: AGT-AUDIT-CI-20260916) (TS: 2026-09-16T12:00:00Z) (SESSION: acf8fe72) (VERIFIED-BY-SESSION: PENDING)

---

## 1. Executive Summary

Die automatische Post-Merge-Verifikation (`.github/workflows/post-merge-verification.yml`) schlägt aktuell auf `main` fehl, woraufhin automatisierte `post-merge-regression` Issues (#2458 ff.) ohne detaillierte Triage-Angaben eröffnet wurden.

Die lokale, deterministische Re-Ausführung von `cargo xtask jules-preflight` auf dem aktuellen `main`-HEAD ergab ein **100% reproduzierbares Fehlschlagen** über 5 verschiedene Gate-Kategorien.

### Hauptbefund & Zusammenfassung der Root Causes
Die Fehlschläge verteilen sich auf 5 klar abgrenzbare Fehlercluster:
1. **Unwrap-Baseline Drift (Gate 2):** Unregistrierte `.unwrap()` Aufrufe in `crates/memfuse-sandbox`.
2. **Silent I/O Discards (Gate 3):** Stille Verwerfung von `Result`/`Option` Rückgabewerten in `crates/memfuse-graph/src/csr.rs`.
3. **TODO-Grammatik Policy (Gate 6):** Fehlender `AI-TAG` Metadaten-Block an einem `// TODO` Kommentarknoten in `crates/memfuse-graph/src/csr.rs`.
4. **Code-Formatierungs-Drift (`cargo fmt`):** Unformatierter Code in `crates/memfuse-store/src/sstable.rs` und `xtask/src/`.
5. **Clippy Redundant-Cast & Docs-Sync Kaskade (Gate 5 / Clippy):** Redulanter `doc_id.inner() as u64` Cast in `crates/memfuse-core/src/types/domain.rs` löst `-D clippy::unnecessary_cast` Fehler aus, welcher `cargo clippy` abbricht und dadurch `cargo xtask sync-docs` behindert (`WORKING_STATE.md` und `SOURCE_OF_TRUTH.md` Drift).

---

## 2. Exakte Preflight-Fehlerausgabe (Terminal Log)

```text
=== xtask jules-preflight (FULL) ===
=== Running xtask check-unwrap-baseline ===
=== Running xtask validate-tags (fix=false) ===
✅ Alle Tags sind gültig.
=== xtask check-dag ===
=== xtask check-dag PASSED ===
=== xtask check-agents-integrity ===
=== xtask check-agents-integrity PASSED ===
=== Running xtask sync-docs (check_only=true) ===
Found 338 code tags across crates/.
Parsed 18 workspace crates.
✅ docs/CHANGELOG.md is in sync.
✅ Section 'DAG_TOPOLOGY' in docs/ARCHITECTURE.md is in sync.
✅ Section 'INVARIANTS_TABLE' in docs/ARCHITECTURE.md is in sync.
=== Running xtask check-review-coverage ===
✅ ANCHOR 'TEST:CKPT-001' in crates/memfuse-checkpoint/tests/cache_concurrency_pinning.rs:1 passed review coverage (2/2 independent sessions)
...
=== xtask check-review-coverage PASSED ===
=== xtask check-consistency ===
Actual workspace crate count: 18
=== xtask check-consistency PASSED ===
=== xtask check-jules-context-freshness ===
Last code change date in crates/: 2026-09-17
✅ AGENTS.md is fresh (Stand: 2026-09-14, crates/ change: 2026-09-17)
✅ .jules/JULES_CONTEXT.md is fresh (Stand: 2026-09-16, crates/ change: 2026-09-17)
=== xtask check-jules-context-freshness PASSED ===

┌───────────────────────────────────────────────────────────┐
│  Jules Preflight (FULL)  —  18.4s gesamt                      │
├───────────────────────────────────────────────────────────┤
│  ✅ Gate 1: Kritische AI-TAGs (0.3s)
│  ✅ Orphan Modules Check (0.9s)
│  ✅ Claim-Check (0.0s)
│  ❌ Gate 2: Unwrap-Baseline (0.2s)
│  ❌ Gate 3: Silent IO (0.2s)
│     Silent IO-Fehler gefunden:
│       crates/memfuse-graph/src/csr.rs:4976
│       crates/memfuse-graph/src/csr.rs:4977
│  ✅ Gate 4: Kein axum in MCP (0.0s)
│  ❌ Gate 6: TODO-Grammatik (0.1s)
│     1 TODOs ohne AI-TAG Grammatik:
│       crates/memfuse-graph/src/csr.rs:1445
│  ✅ Gate 7: ISO-8601 Tags (1.0s)
│  ✅ Duplicate Symbols (1.8s)
│  ✅ DAG-Integrität (0.1s)
│  ✅ AGENTS.md Integrität (0.1s)
│  ❌ cargo fmt (3.5s)
│     Formatierung nicht korrekt — `cargo fmt --all` ausführen
│  ❌ cargo clippy (7.7s)
│     Clippy-Warnungen gefunden
│  ❌ Gate 5: Docs-Sync (0.6s)
│  ✅ Gate 8: Review-Coverage (0.4s)
│  ✅ Gate 9: Konsistenz (1.7s)
│  ✅ Gate 10: Jules Context (0.0s)
├───────────────────────────────────────────────────────────┤
│  ❌ GATES FEHLGESCHLAGEN — Fixes erforderlich
└───────────────────────────────────────────────────────────┘

❌ [GATE-2]: crates/memfuse-sandbox/src/executor.rs:429 — new .unwrap() not in baseline
❌ [GATE-2]: crates/memfuse-sandbox/src/executor.rs:456 — new .unwrap() not in baseline
❌ [GATE-2]: crates/memfuse-sandbox/tests/wasm_boundary_tests.rs:23 — new .unwrap() not in baseline

error: casting to the same type is unnecessary (`u64` -> `u64`)
   --> crates/memfuse-core/src/types/domain.rs:384:14
    |
384 |         Self(doc_id.inner() as u64)
    |              ^^^^^^^^^^^^^^^^^^^^^ help: try: `doc_id.inner()`

error: casting to the same type is unnecessary (`u64` -> `u64`)
   --> crates/memfuse-core/src/types/domain.rs:396:43
    |
396 |         DocId::from_key(key).map(|d| Self(d.inner() as u64))
    |                                           ^^^^^^^^^^^^^^^^ help: try: `d.inner()`

❌ WORKING_STATE.md is out of sync!
❌ Section 'CRATE_INVENTORY' in docs/SOURCE_OF_TRUTH.md is out of sync!
❌ Documentation drift detected. Run `cargo xtask sync-docs` to fix.
```

---

## 3. Detail-Analyse & Kategorisierung pro Fehlercluster

### Cluster A: Gate 2 — Unwrap-Baseline Drift
* **Betroffene Dateien:**
  * `crates/memfuse-sandbox/src/executor.rs:429`
  * `crates/memfuse-sandbox/src/executor.rs:456`
  * `crates/memfuse-sandbox/tests/wasm_boundary_tests.rs:23`
* **Ursache:** Neue `.unwrap()` Aufrufe wurden hinzugefügt, ohne dass `.unwrap_baseline.json` aktualisiert oder defensive Fehlerbehandlung (`unwrap_or_else` / `match`) eingesetzt wurde.
* **Kategorie:** (c) Strukturelle Abweichung / Ausstehende Code-Korrektur oder Baseline-Update.

### Cluster B: Gate 3 — Silent I/O Discards
* **Betroffene Dateien:**
  * `crates/memfuse-graph/src/csr.rs:4976`
  * `crates/memfuse-graph/src/csr.rs:4977`
* **Ursache:** In `csr.rs` werden Rückgabewerte von I/O- oder Speicher-Operationen mit `let _ = ...` stumm verworfen, was gegen die Invariante verstößt, dass alle I/O-Fehler geloggt oder explizit behandelt werden müssen.
* **Kategorie:** (c) Fehler in `crates/memfuse-graph` (High-Traffic/Gesperrte Datei).

### Cluster C: Gate 6 — TODO-Grammatik Richtlinienverstoß
* **Betroffene Dateien:**
  * `crates/memfuse-graph/src/csr.rs:1445`
* **Ursache:** Ein reines `// TODO ...` ohne das vorgeschriebene Format `// AI-TAG[...] (ID: ...) (TS: ...) (SESSION: ...)` verstößt gegen die Repository-Invariante für Quelltext-Kommentare.
* **Kategorie:** (c) Fehler in `crates/memfuse-graph`.

### Cluster D: Code-Formatierung (`cargo fmt`)
* **Betroffene Dateien:**
  * `crates/memfuse-store/src/sstable.rs`
  * `xtask/src/check_flatbuffers_drift.rs`
  * `xtask/src/main.rs`
* **Ursache:** Zeilenumbrüche und Modul-Reihenfolge entsprechen nicht dem Standard-Stil von `rustfmt`.
* **Kategorie:** (b) Trivialer Fix durch `cargo fmt --all`.

### Cluster E: Clippy Redundant Cast & Documentation Drift Kaskade
* **Betroffene Dateien:**
  * `crates/memfuse-core/src/types/domain.rs:384,396`
  * `WORKING_STATE.md`
  * `docs/SOURCE_OF_TRUTH.md`
* **Ursache:** `doc_id.inner()` liefert auf `u64` konfigurierter Toolchain bereits `u64`. Der Cast `doc_id.inner() as u64` löst Clippy `unnecessary_cast` aus. Da Clippy mit `-D warnings` kompiliert, bricht die Build-Pipeline ab. Dies führt kaskadierend dazu, dass `cargo xtask sync-docs` die Generierung nicht abschließen kann.
* **Kategorie:** (c) Fehler in `crates/memfuse-core` (Zentrale Core-Typen).

---

## 4. Zuordnung der GitHub Issues zu den Root-Cause-Clustern

Sämtliche offenen `post-merge-regression` Issues (#2458, #2585, #2636, #2657, #2659, #2661, #2664, #2693, #2705, #2707, #2716, #2731, #2737, #2738, #2740, #2743, #2754, #2767, #2808, #2811, #2818, #2827, #2831, #2837, #2838, #2846, #2854) basieren auf demselben Post-Merge Pipeline-Fehlschlag. Sie teilen sich die oben analysierten 5 Root-Causes.

Sobald die 5 isolierten Folge-Prompts (siehe Abschnitt 5) abgearbeitet sind, können alle genannten Issues geschlossen werden.

---

## 5. Konkrete, isolierte Folge-Prompt-Empfehlungen

Gemäß der Rollen-Sperre (Modus **AUDIT**) werden die Behebungen als dedizierte, fokussierte Arbeitsaufträge vorgeschlagen:

### Folge-Prompt 1: Clippy Cast Fix & Docs Sync
* **Ziel-Datei:** `crates/memfuse-core/src/types/domain.rs`
* **Beschreibung:** Entferne den redundanten `as u64` Cast in Zeilen 384 und 396 (`doc_id.inner()`). Führe anschließend `cargo xtask sync-docs` aus, um `WORKING_STATE.md` und `SOURCE_OF_TRUTH.md` zu aktualisieren.
* **Verifikation:** `cargo clippy --workspace --all-targets -- -D warnings` und `cargo xtask sync-docs --check`.

### Folge-Prompt 2: Sandbox Unwrap Baseline Cleanup
* **Ziel-Datei:** `crates/memfuse-sandbox/src/executor.rs`, `crates/memfuse-sandbox/tests/wasm_boundary_tests.rs`, `.unwrap_baseline.json`
* **Beschreibung:** Ersetze die unwrap-Aufrufe in `executor.rs` (Zeile 429, 456) durch defensive `unwrap_or_else` Fehlerbehandlung. Aktualisiere für den Testfall `cargo xtask update-unwrap-baseline`.
* **Verifikation:** `cargo xtask check-unwrap-baseline`.

### Folge-Prompt 3: Graph CSR Silent IO & TODO Tagging
* **Ziel-Datei:** `crates/memfuse-graph/src/csr.rs`
* **Beschreibung:** Behebe die stummen I/O-Verwerfungen in Zeilen 4976–4977 durch Tracing-Logging (`tracing::warn!`/`tracing::error!`) oder explizite Propagierung. Formatiere den TODO-Kommentar in Zeile 1445 gemäß der AI-TAG Grammatik.
* **Verifikation:** `cargo xtask jules-preflight`.

### Folge-Prompt 4: Workspace Formatting Sync
* **Ziel-Dateien:** `crates/memfuse-store/src/sstable.rs`, `xtask/src/check_flatbuffers_drift.rs`, `xtask/src/main.rs`
* **Beschreibung:** Führe `cargo fmt --all` aus und überprüfe saubere Diff-Struktur.
* **Verifikation:** `cargo fmt --all -- --check`.

---

## 6. Audit Verdict

* **STATUS:** **REJECTED / ACTION_REQUIRED**
* **GRUND:** Das `jules-preflight` Gate schlägt auf `main` verlässlich fehl. Fünf Fehlercluster verhindern die erfolgreiche CI-Durchführung.
* **NEXT STEPS:** Abarbeitung der 4 Folge-Prompts und anschließendes Schließen der `post-merge-regression` Issues.
