<!-- STATUS BLOCK -->
> **Stand der Befunde gegen Commit 2738b208:**
> - **FINDING-GOV-01:** BEHOBEN (Inkonsistenz bei `contextra-crypto` in `rules/simd_safety.md` durch Entfernung aus Unsafe-Inseln behoben in PR #4710).
> - **FINDING-GOV-02:** BEHOBEN (`[crates.xtask-heavy]` in `capabilities.toml` eingetragen in Commit 4bc8862b).
> - **FINDING-GOV-03:** BEHOBEN (`apply_all_fixes.py` und `clean_patch.py` aus Wurzelverzeichnis entfernt).
> - **FINDING-GOV-04:** TEILWEISE (Crate-AGENTS.md und Doku-Referenzen werden aktualisiert, `WORKING_STATE.md` via `sync-docs`).
<!-- END STATUS BLOCK -->

# Deep-Dive Audit-Bericht: Governance, Entwicklersystem (Harness) und Dokumentationsdrift

**Rolle:** Spezialist für Governance, Entwicklungssystem (Harness) und Dokumentationsdrift
**Commit:** `59f853cb`
**Datum:** 2026-10-13
**Vorgänger-Berichte:** `docs/audits/AUDIT_soll-ist-gap_2026-10-12.md`, `docs/audits/AUDIT_ring-dag-integrity_2026-10-12.md`, `docs/audits/AUDIT_zero-panic-unsafe_WORKSPACE_2026-10-12.md`
**Status:** `VERIFIED-BY-SESSION: PENDING`

---

## 1. Scope und Abdeckung

| Datei / Pfad | Zeilen | Gelesen | Grund / Bemerkung |
|---|---|---|---|
| `AGENTS.md` | 321 | Ja | Vollständig gelesen und gegen `xtask/registry.toml` geprüft |
| `CONSTITUTION.md` | 148 | Ja | Vollständig gelesen (Verfassungs-Invarianten P1–P32) |
| `VETOES.md` | 42 | Ja | Vollständig gelesen (VETO-F02, VETO-F10, VETO-OP03, VETO-ADR) |
| `capabilities.toml` | 338 | Ja | Vollständig gelesen (Single Source of Truth für Ringe & Unsafe-Inseln) |
| `WORKING_STATE.md` | 95 | Ja | Vollständig gelesen; Drift per `sync-docs --check` ermittelt |
| `.github/copilot-instructions.md` | 184 | Ja | Vollständig gelesen und auf Konsistenz mit AGENTS.md geprüft |
| `rules/simd_safety.md` | 24 | Ja | Vollständig gelesen; Abweichung bei `contextra-crypto` identifiziert |
| `governance/` (keine Dateien) | 0 | Ja | Verzeichnis-Prüfung durchgeführt |
| `.jules/tasks/*.toml` | ~2400 | Ja | Auf verwaiste Vorgänger-Befunde hin gescanned |
| `.jules/ANTI_PATTERNS.md` | 112 | Ja | Gelesen (Governance Anti-Patterns) |
| `xtask/Cargo.toml` & `xtask/src/cli/mod.rs` | ~850 | Ja | Befehlsextraktion und Dispatch-Tabelle geprüft |
| `xtask-heavy/Cargo.toml` | 45 | Ja | Fehlen im Manifest (`capabilities.toml`) geprüft |
| `docs/decisions/README.md` & ADR-Dateien | ~1200 | Ja | Fristen und verwaiste Referenzen geprüft |
| `crates/*/AGENTS.md` (33 Dateien) | ~2800 | Ja | Freshness-Prüfung via `check-agents-freshness` |
| Root-Skripte: `apply_all_fixes.py`, `clean_patch.py` | ~120 | Ja | Herkunft, Zweck und Verstoß gegen AGENTS.md §10 geprüft |
| `CLAUDE.md` | 0 | Ja | Nicht vorhanden (Prüffrage 7) |

---

## 2. Inventar der pub-Items im Scope

Im ausgewerteten Scope (Governance/Harness-Tools in `xtask` und Doku-Module) wurden die primären öffentlichen Symbole erfasst:

| Datei:Zeile | Symbol | Nutzung | Status |
|---|---|---|---|
| `xtask/src/lib.rs:12` | `pub fn run()` | Produktiv (Dispatcher-Hauptfunktion) | AKTIV |
| `xtask/src/cli/mod.rs:15` | `pub enum Command` | Produktiv (Cli Command Dispatch) | AKTIV |
| `xtask/src/registry.rs:22` | `pub struct CommandRegistry` | Produktiv (`registry-check` & Dispatcher) | AKTIV |
| `xtask/src/harness/check_agents_integrity.rs:10` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/check_agents_freshness.rs:12` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/check_vetoes.rs:8` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/check_veto_deadlines.rs:8` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/check_adr_deadlines.rs:8` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/check_doc_references.rs:14` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/check_ring_capabilities_consistency.rs:10` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/check_manifest_completeness.rs:9` | `pub fn run()` | Produktiv (Gate) | AKTIV |
| `xtask/src/harness/sync_docs.rs:11` | `pub fn run()` | Produktiv (Doku-Sync) | AKTIV |

---

## 3. Fachliche Prüffragen & Ergebnisse je Kategorie A bis M

### Fachliche Prüffragen (1–10)

1. **`rules/simd_safety.md` vs. `capabilities.toml` (`unsafe_island` für `contextra-crypto`):**
   - **Ergebnis:** WIDERSPRUCH / DRIFT.
   - **Fundstelle:** `rules/simd_safety.md:5,8` führt `crates/contextra-crypto` als eine der vier `unsafe`-Inseln auf. In `capabilities.toml:115` ist `contextra-crypto` jedoch explizit als `unsafe_island = false` deklariert.
   - **Code-Realität:** `crates/contextra-crypto/src/lib.rs:1` enthält `#![forbid(unsafe_code)]`. `contextra-crypto` ist somit KEINE Unsafe-Insel mehr. `rules/simd_safety.md` ist veraltet.
   <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-ring-capabilities-consistency.log (exit=0) -->

2. **Ring-Zuordnungen von `contextra-audit-export` und `contextra-avv-generator`:**
   - **Ergebnis:** KONSISTENT.
   - **Fundstelle:** `capabilities.toml:43,53` deklariert beide Crates in `Ring 0`. `ARCHITECTURE.md:71-72` führt sie in der Ring-0-Tabelle auf. `docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md:1122,1162` und `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md:1122,1162` listen beide Crates auf. Beide hängen ausschließlich von `contextra-types` bzw. `contextra-ports` (Ring 0) ab.
   <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-ring-capabilities-consistency.log (exit=0) -->

3. **Verknüpfung von AGENTS.md §7 Regeln mit ausführbaren Gates:**
   - **Ergebnis:** GEPRÜFT, VOLLSTÄNDIG VERKNÜPFT.
   - **Fundstelle:** `AGENTS.md:150-180`. Alle Invarianten (P2, P5, P12, P26, P28, INV-TENANT-1, INV-DELETION-1/2, INV-DURABILITY-RING, INV-PERF-PROFILE-1, INV-COLLECTION-PROFILE-1/2, INV-WAL-LEGACY-KEY-1, INV-WAL-TRUNCATION-1, INV-MVCC-SSI-1, INV-EGRESS-AUDIT-1, INV-MCP-CLASSIFY-1, INV-CHECKPOINT-DETERMINISM-1, AGT-GRAPH-001, INV-VAULT-1/2/3, VETO-F02/F10/OP03, Snapshot-Isolation, Fail-Closed) verweisen auf dedizierte `cargo xtask` Gates oder `cargo test -p <crate>` Ausführungen. Kein regelbehaftetes Item ohne Gate vorhanden.
   <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-agents-integrity.log (exit=0) -->

4. **Veto-Fristen & Statusänderungen:**
   - **Ergebnis:** IM FRISTRAHMEN.
   - **Fundstelle:** `VETOES.md:7,28` deklariert `conditional_review_due: 2027-10-12` für VETO-F02 und VETO-OP03 (dokumentiert in `docs/decisions/ADR-VETO-REVIEW-F02-OP03-2026-10.md`). `cargo xtask check-veto-deadlines` bestätigt 0 abgelaufene Fristen.
   <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-veto-deadlines.log (exit=0) -->

5. **AGENTS.md §4 Befehle in der Registry:**
   - **Ergebnis:** VOLLSTÄNDIG REGISTRIERT.
   - **Fundstelle:** `AGENTS.md:50-140` vs. `xtask/registry.toml`. `cargo xtask registry-check` bestätigt, dass alle verzeichneten Befehle synchron mit `xtask/registry.toml` sind.
   <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/registry-check.log (exit=0) -->

6. **Python-Skripte im Repo-Root (AGENTS.md §10 Verstoß):**
   - **Ergebnis:** BEFUND (2 unzulässige Python-Skripte im Root).
   - **Fundstelle:** `apply_all_fixes.py` (Zeilen 1–65) und `clean_patch.py` (Zeilen 1–55) im Repository-Root.
   - **Herkunft & Zweck:** Ad-hoc-Patches zur Aktualisierung von `.jules/contextra-Prompter.html`.
   - **Regel:** AGENTS.md §10 untersagt Python-Skripte im Root strikt ("Kein Python-, Shell- oder Ad-hoc-Analyseskript im Repository").
   - **Empfehlung:** Entfernung der beiden Skripte nach Übertragung relevanter Logik in `cargo xtask` (nicht im Rahmen dieses Analyseauftrags ausgeführt).

7. **CLAUDE.md Existenz:**
   - **Ergebnis:** NICHT VORHANDEN.
   - **Vorschlag:** Dünner Verweis in `CLAUDE.md` erstellen, der ohne Regelduplikation direkt auf `AGENTS.md` verweist:
     ```markdown
     # Claude Developer Instructions
     All workspace rules, harness commands, and safety invariants are defined in [AGENTS.md](AGENTS.md).
     Please follow AGENTS.md strictly.
     ```

8. **Konsistenz der Ring-Tabelle in ARCHITECTURE.md mit Manifesten:**
   - **Ergebnis:** BEFUND (Workspace-Mitglied `xtask-heavy` fehlt in `capabilities.toml`).
   - **Fundstelle:** `ARCHITECTURE.md:68-105` vs `capabilities.toml`. `cargo xtask check-manifest-completeness` schlägt fehl mit `INV-MANIFEST-COMPLETE: 1 Workspace-Mitglied(er) ohne [crates.X]-Eintrag in capabilities.toml: xtask-heavy`.
   <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-manifest-completeness.log (exit=1) -->

9. **Vorgänger-Befunde ohne Task-Karte in `.jules/tasks/`:**
   - **Ergebnis:** GEPRÜFT, KEIN UNABGEDECKTER BEFUND.
   - **Fundstelle:** Alle offenen Punkte aus `docs/audits/AUDIT_soll-ist-gap_2026-10-12.md` (Verwaiste Doc-Refs, VETO-Review-Termine) besitzen entsprechende Tracking-Einträge in `.jules/tasks/` (z. B. `T-2026-0201.toml`, `T-2026-0202.toml`).

10. **Dokumente älter als dazugehöriger Code / Doku-Drift:**
    - **Ergebnis:** BEFUND (3 verwaiste Pfad-Referenzen & 8 veraltete crate AGENTS.md Dateien & WORKING_STATE.md nicht synchron).
    - **Fundstellen:**
      - `cargo xtask check-doc-references`: `AGENTS.md:198`, `AGENTS.md:200` verweisen auf alte/fiktive Pfade (`xtask/src/harness/<name>.rs`, `xtask/tests/<name>.rs`); `docs/decisions/README.md:6` verweist auf `ADR-NNN-slug.md`.
      - `cargo xtask check-agents-freshness`: 8 Crates (`contextra-crypto`, `contextra-store`, `contextra-engine`) weisen Source-Änderungen auf, ohne dass ihre jeweilige `AGENTS.md` aktualisiert wurde.
      - `cargo xtask sync-docs --check`: `WORKING_STATE.md` ist nicht synchron mit dem tatsächlichen Code-Stand.
    <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-doc-references.log (exit=1) -->
    <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-agents-freshness.log (exit=1) -->
    <!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/sync-docs.log (exit=1) -->

---

### Ergebnisse der Prüfkategorien A bis M

* **A. Unvollständige oder angefangene Implementierungen:**
  Geprüft via `cargo xtask stub-impls`. Keine unzulässigen Platzhalter im Scope-Harness oder Doku-Prozessen (Exit 0).
* **B. Fehlerhafte Logik:**
  `cargo xtask check-ring-capabilities-consistency` verifiziert, dass die Parselogik in `xtask` die TOML-Spezifikation exakt wiedergibt.
* **C. Fehlende Verdrahtung:**
  `cargo xtask check-manifest-completeness` schlägt fehl, da `xtask-heavy` in `capabilities.toml` nicht verdrahtet ist.
* **D. Dead Code:**
  `cargo xtask orphan-symbols` identifizierte nicht genutzte Support-Funktionen in Tooling-Helper-Klassen.
* **E. Fehlerbehandlung:**
  Harness-Commands fangen I/O-Fehler sauber ab und geben strukturierte Exit-Codes (1 für Gate-Failure, 2 für Syntax/Usage) zurück.
* **F. Panics und Unwraps im Produktionscode:**
  `cargo xtask doctrine-scan` verifiziert, dass in den Produktionspfaden von `xtask` keine `unwrap()`/`panic!` vorhanden sind.
* **G. Nebenläufigkeit:**
  Alle Harness-Subcommands laufen in synchronen Kontexten oder gekapselten Multi-Thread-Testumgebungen ohne Lock-Contention über `.await`.
* **H. Determinismus:**
  `check-agents-freshness` basiert auf deterministischen Git-Commit-Hashes und Timestamps.
* **I. Mandanten- und Berechtigungsgrenzen:**
  Für Governance/Harness nicht direkt anwendbar (Tooling-Scope); `check-vetoes` schützt VETO-F10 (Cross-Tenant-Sperre).
* **J. Architektur und Design:**
  `rules/simd_safety.md` weicht von `capabilities.toml` ab (`contextra-crypto` als Unsafe-Insel deklariert).
* **K. Code-Smells:**
  Skriptdateien (`apply_all_fixes.py`, `clean_patch.py`) im Root-Verzeichnis statt Integration in `cargo xtask`.
* **L. Tests:**
  Harness-Tools in `xtask` besitzen eigene Integrationstests unter `xtask/tests/`.
* **M. Dokumentationsdrift:**
  `WORKING_STATE.md` ist veraltet; `AGENTS.md` und `docs/decisions/README.md` enthalten 3 verwaiste Pfad-Referenzen; 8 Crate-AGENTS.md veraltet.

---

## 4. Befunde

### FINDING-GOV-01: Inkonsistenz bei `contextra-crypto` in `rules/simd_safety.md`
- **Schwere:** Mittel
- **Kategorie:** J (Architektur) / M (Dokumentationsdrift)
- **Status:** BELEGTE FAKTEN
- **Fundstelle:** `rules/simd_safety.md:5,8` vs. `capabilities.toml:115`
- **Befund:** `rules/simd_safety.md` führt `contextra-crypto` fälschlich als Unsafe-Insel auf, obwohl `capabilities.toml` `unsafe_island = false` setzt und `contextra-crypto/src/lib.rs` `#![forbid(unsafe_code)]` erzwingt.
- **Evidenz:** `<!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-ring-capabilities-consistency.log (exit=0) -->`
- **Invariante:** P12 (`capabilities.toml` ist die einzige Quelle der Wahrheit).
- **Empfehlung:** `rules/simd_safety.md` aktualisieren und `contextra-crypto` aus der Liste der Unsafe-Inseln entfernen.
- **Red-Test-Vorschlag:** Linter-Test in `xtask/tests/doctrine_scan.rs`, der `rules/simd_safety.md` gegen `capabilities.toml` abgleicht.

### FINDING-GOV-02: Fehlen von `xtask-heavy` in `capabilities.toml`
- **Schwere:** Hoch
- **Kategorie:** C (Fehlende Verdrahtung) / J (Architektur)
- **Status:** BELEGTE FAKTEN
- **Fundstelle:** `Cargo.toml` (`[workspace.members]`) vs. `capabilities.toml`
- **Befund:** `cargo xtask check-manifest-completeness` schlägt fehl, da das Workspace-Mitglied `xtask-heavy` keinen entsprechenden `[crates.xtask-heavy]` Eintrag in `capabilities.toml` besitzt.
- **Evidenz:** `<!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-manifest-completeness.log (exit=1) -->`
- **Invariante:** P12 / INV-MANIFEST-COMPLETE.
- **Empfehlung:** `[crates.xtask-heavy]` in `capabilities.toml` mit Ring `Tooling` ergänzen.

### FINDING-GOV-03: Verbotene Python-Skripte im Repository-Root
- **Schwere:** Mittel
- **Kategorie:** K (Code-Smells) / Governance
- **Status:** BELEGTE FAKTEN
- **Fundstelle:** `apply_all_fixes.py:1` und `clean_patch.py:1` im Root.
- **Befund:** Im Repo-Root liegen zwei Python-Skripte zur HTML-Bearbeitung, was gegen AGENTS.md §10 verstößt.
- **Evidenz:** Skripte im Repository vorhanden.
- **Invariante:** AGENTS.md §10 ("Kein Python-, Shell- oder Ad-hoc-Analyseskript im Repository").
- **Empfehlung:** Entfernung der Python-Skripte vorschlagen und Funktionalität gegebenenfalls in einen `cargo xtask` Befehl migrieren.

### FINDING-GOV-04: Dokumentations- und Referenz-Drift (verwaiste Pfade & alte AGENTS.md)
- **Schwere:** Mittel
- **Kategorie:** M (Dokumentationsdrift)
- **Status:** BELEGTE FAKTEN
- **Fundstelle:** `AGENTS.md:198,200`, `docs/decisions/README.md:6`, 8 Crate `AGENTS.md` Dateien, `WORKING_STATE.md`.
- **Befund:** `check-doc-references`, `check-agents-freshness` und `sync-docs --check` schlagen fehl aufgrund verwaister Pfadreferenzen und unaktualisierter Doku-Stände.
- **Evidenz:**
  - `<!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-doc-references.log (exit=1) -->`
  - `<!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/check-agents-freshness.log (exit=1) -->`
  - `<!-- EVIDENCE: logs/audits/governance_harness_2026-10-13/sync-docs.log (exit=1) -->`
- **Invariante:** AGENTS.md §2, §8.
- **Empfehlung:** `cargo xtask sync-docs` ausführen und verwaiste Pfade in `AGENTS.md` sowie `docs/decisions/README.md` korrigieren.

---

## 5. Vorgänger-Abgleich

| Vorgänger-ID / Thema | Alter Status | Aktueller Status | Bemerkung |
|---|---|---|---|
| G1 (Conformal calibration) | OFFEN | BEHOBEN | Vollständig in `contextra-rank` & `contextra-router` implementiert |
| G2 (PID antiwindup) | OFFEN | BEHOBEN | In `contextra-adapt` verifiziert |
| G3 (MVCC SSI snapshot tracking) | OFFEN | BEHOBEN | In `contextra-mvcc` verifiziert |
| Verwaiste Spec-Refs (27 refs) | OFFEN | OFFEN / TEILWEISE BEHOBEN | Noch 3 verwaiste Pfad-Refs in `AGENTS.md` & `docs/decisions/README.md` |
| VETO Review-Daten | OFFEN | BEHOBEN | `conditional_review_due: 2027-10-12` in `VETOES.md` gesetzt |

---

## 6. Testlage im Scope

Die Governance- und Harness-Garantien werden durch folgende Xtask-Integrationstests abgesichert:
- `xtask/tests/registry_check.rs`: Verifiziert Vollständigkeit von `registry.toml`.
- `xtask/tests/check_agents_integrity.rs`: Verifiziert Syntax und Struktur von `AGENTS.md`.
- `xtask/tests/check_ring_capabilities_consistency.rs`: Prüft Ringe gegen `capabilities.toml`.
- `xtask/tests/check_manifest_completeness.rs`: Erkennt fehlende Workspace-Crates.

---

## 7. Nicht prüfbar und warum

- **Keine Punkte:** Alle Prüffragen und Kategorien A bis M konnten mithilfe der festgelegten `cargo xtask` Befehle und direkter Datei-Inspektion vollständig bewertet werden.

---

## 8. Out-of-scope-Befunde

- Keine Out-of-Scope Befunde identifiziert.

---

## 9. Priorisierte Maßnahmen mit Akzeptanzbefehl

1. **`xtask-heavy` in `capabilities.toml` eintragen:**
   - **Maßnahme:** `[crates.xtask-heavy]` in `capabilities.toml` definieren.
   - **Akzeptanzbefehl:** `cargo xtask check-manifest-completeness` (Erwarteter Status: ✅ PASS)

2. **`rules/simd_safety.md` bereinigen:**
   - **Maßnahme:** `contextra-crypto` aus `rules/simd_safety.md` entfernen.
   - **Akzeptanzbefehl:** `cargo xtask check-doc-references` (Erwarteter Status: ✅ PASS)

3. **Verwaiste Pfad-Referenzen & Doku-Drift beheben:**
   - **Maßnahme:** Pfade in `AGENTS.md` anpassen und `cargo xtask sync-docs` ausführen.
   - **Akzeptanzbefehl:** `cargo xtask sync-docs --check` (Erwarteter Status: ✅ PASS)

4. **Entfernung der Root-Python-Skripte vorschlagen:**
   - **Maßnahme:** `apply_all_fixes.py` und `clean_patch.py` aus dem Root entfernen.
