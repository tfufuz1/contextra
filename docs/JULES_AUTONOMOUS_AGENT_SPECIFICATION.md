# Google-Jules Autonome Agenten- und Umgebungs-Spezifikation (v1.0)

**Titel:** Mikrospezifikation der Funktionsweise, Werkzeugnutzung, VM-Umgebung, Testverfahren und Repository-Optimierung von Google-Jules
**Projekt:** Contextra Engine & Ecosystem
**Autor:** Google-Jules (Autonomer Software-Ingenieur)
**Stand:** Oktober 2026
**Status:** Normativ für autonome Agentensitzungen in `contextra`

---

## 1. Präambel & Einleitung

Diese Spezifikation beschreibt mikrospezifisch und lückenlos die interne Funktionsweise, die Werkzeugnutzung, die VM-Infrastruktur, die Test- und Benchmark-Methodik sowie die Abgabe- und Git/GitHub-Prozesse von Google-Jules im Repository `contextra`.

Ziel dieses Dokuments ist es, Entwicklern und Maintainern maximale Transparenz darüber zu geben, wie Google-Jules als autonomer Software-Ingenieur agiert, welche Dateien er zu welchem Zeitpunkt liest, wie sein Kontextfenster aufgebaut ist, wie er den Quellcode modifiziert und verifiziert und mit welchen Maßnahmen das Repository für autonome Entwicklungszyklen weiter optimiert werden kann.

---

## 2. Autonomer Initialisierungs- & Leseablauf (Welche Dateien zuerst gelesen werden)

### 2.1 Erstes Lesen bei Aufgabenbeginn (Dateihierarchie)

Wenn Google-Jules eine neue Aufgabe startet, führt er **autonom und ohne Aufforderung** eine fest definierte Reihenfolge von Datei-Lese-Operationen durch. Diese Hierarchie stellt sicher, dass Jules die verbindlichen Regeln, Invarianten, Typdefinitionen und den aktuellen Projektzustand versteht, bevor er Zeilen Code berührt:

```
[1. AGENTS.md] (Root)
       │
       ▼
[2. README.md / .jules/tasks/T-XXXX.toml / JULES_CONTEXT.md]
       │
       ▼
[3. capabilities.toml]
       │
       ▼
[4. docs/TYPE_REGISTRY.md & docs/SOURCE_OF_TRUTH.md]
       │
       ▼
[5. crates/<target_crate>/AGENTS.md] (Bei Modifikation in spezifischem Crate)
```

### 2.2 Zweck und Auswertung der Einstiegsdateien

1. **`AGENTS.md` (Root-Ebene):**
   - **Priorität:** Höchste (nach den primären System-Prompts & Schutzregeln).
   - **Inhalt:** Betriebsanleitung für Agenten, Rangfolge der Regeldokumente, Werkzeug-Verwendungsregeln (`cargo xtask` als verbindliche Wahrheit), Liste zulässiger Harness-Befehle, Lifecycle-Gates, Nicht-Verhalten (z. B. Verbot von `sh -c` oder Ad-hoc-Python-Skripten) und verbindliche Architektur-Invarianten (P2, P5, P12, P26, P28, INV-*).
   - **Autonome Auswertung:** Jules prüft, welche Gates, Linter und Prüfbefehle für die aktuellen Änderungen zwingend durchzuführen sind.

2. **`README.md` / Task-Card (`.jules/tasks/*.toml` oder `.jules/JULES_CONTEXT.md`):**
   - **Inhalt:** Projektüberblick, Task-Beschreibung, Akzeptanzkriterien, Scope-Einschränkungen (welche Dateien geändert werden dürfen und welche protected sind).
   - **Autonome Auswertung:** Jules extrahiert den genauen Auftrag, identifiziert betroffene Crates und prüft zugewiesene Scope-Budgets.

3. **`capabilities.toml`:**
   - **Inhalt:** Die einzige normative Quelle der Wahrheit für Ring-Zuweisungen (Ring 0 bis Ring 10), Unsafe-Inseln und Crate-Klassifizierungen.
   - **Autonome Auswertung:** Jules verifiziert, in welchem Architektur-Ring sich das betroffene Crate befindet, ob Unsafe-Code erlaubt ist und welche Abhängigkeiten erlaubt sind.

4. **`docs/TYPE_REGISTRY.md` & `docs/SOURCE_OF_TRUTH.md`:**
   - **Inhalt:** Vollständige Inventarisierung aller Domänen-Typen, Crate-LOCs, Reifegrade und Modulstrukturen.
   - **Autonome Auswertung:** Vermeidung doppelter Typdefinitionen und Sicherstellung der Wiederverwendung bestehender Schnittstellen.

5. **Lokale Crate-Dokumente (`crates/<crate_name>/AGENTS.md`):**
   - **Inhalt:** Lokale, crate-spezifische Betriebsanleitungen und Invarianten (übersteuern im Konfliktfall übergeordnete Anleitungen für Dateien innerhalb dieses Ordners).

---

## 3. Operations-LOG & Lebenszyklus einer Aufgabenlösung

Die Bearbeitung jeder Aufgabe durch Google-Jules folgt einem strengen 7-Phasen-Lebenszyklus:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Phase 1: Preflight & Environment Attestation                           │
│   └─ cargo xtask env-attest · cargo xtask repo-doctor                     │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 2: Context Exploration & Discovery                               │
│   └─ list_files · read_file · cargo xtask harness-list                    │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 3: Drei-Wege-Planung                                             │
│   └─ request_plan_review ──> Feedback ──> set_plan                     │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 4: Schrittweise Implementierung & Verifikation                   │
│   └─ write_file / replace_with_git_merge_diff ──> read_file (Guard)    │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 5: Testing, Benchmarking & Gate-Validierung                      │
│   └─ cargo test · cargo xtask gate-guard / gate-arch / gate-integrity  │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 6: Pre-Commit & Memory Recording                                 │
│   └─ pre_commit_instructions · initiate_memory_recording               │
├────────────────────────────────────────────────────────────────────────┤
│ Phase 7: Finale Abgabe                                                 │
│   └─ submit(branch_name, commit_message, title, description)           │
└────────────────────────────────────────────────────────────────────────┘
```

### Detaillierter Ablaufplan:

1. **Phase 1: Preflight & Umgebungsvalidierung**
   - Jules prüft den Toolchain-Zustand, Git-Status und offene Symbole über `cargo xtask repo-doctor` und `cargo xtask env-attest`.
2. **Phase 2: Kontext-Exploration**
   - Jules erkundet mit `list_files` und `read_file` betroffene Code-Dateien. Er prüft mit `cargo xtask harness-list` verbleibende verfügbare Analyse-Werkzeuge.
3. **Phase 3: Fünf-Sterne-Planung**
   - Jules verfasst einen nummerierten Markdown-Plan.
   - **Regel:** Er ruft *immer* zuerst `request_plan_review(plan)` auf und baut Feedback bezüglich Verifikations- und Testschritten ein.
   - Erst nach Genehmigung setzt er den Plan aktiv mit `set_plan(plan)`.
4. **Phase 4: Schrittweise Implementierung mit Invarianten-Prüfung**
   - Code-Modifikationen erfolgen mit `replace_with_git_merge_diff` oder `write_file`.
   - **Read-Back Guard:** Nach *jeder* Dateimodifikation liest Jules die Datei mit `read_file` zurück, um festzustellen, dass keine Syntaxfehler oder Marker-Fehler entstanden sind.
5. **Phase 5: Verifikation & Gate-Enforcement**
   - Jules führt gezielte Tests aus: `cargo test -p <crate>`.
   - Er ruft die aggregierten Gates auf: `cargo xtask gate-guard`, `cargo xtask gate-arch`, `cargo xtask gate-integrity`.
6. **Phase 6: Pre-Commit & Reflection**
   - Jules ruft `pre_commit_instructions()` auf und befolgt die 4-Schritte-Anweisung (Tests, Frontend-Check falls relevant, Code Review via Tooling, Memory Recording).
   - Er speichert wichtige Erkenntnisse für künftige Tasks via `initiate_memory_recording()`.
7. **Phase 7: Task-Abgabe (`submit`)**
   - Jules ruft das `submit`-Tool mit explizitem Branch-Namen, kurzem Git-Subject, ausführlichem Body und PR-Beschreibung auf.

---

## 4. Dauerhafter Kontext & Kontextfenster-Verwaltung

Während der gesamten Sitzung bleiben folgende Informationen im Kontextfenster von Google-Jules erhalten:

1. **System Prompt & Kern-Direktiven:**
   - Sicherheits- und Ethik-Regeln.
   - Git merge diff Format-Vorgaben (`<<<<<<< SEARCH`, `=======`, `>>>>>>> REPLACE`).
   - Standard-Anweisungen zu Planning, Testing, Verification und `pre_commit_instructions`.
2. **Memory Bank (Erinnerungsregister aus früheren Tasks):**
   - Historische Erkenntnisse über behobene Fehler, Architektur-Entscheidungen, Audits und Invarianten (z. B. Task T-2026-9106 Purge-Compaction, AUFGABE 07 Jitter, P03 Audit, P05 WASM Policy, J35 Symbol Wiring, Ring-DAG Verifizierungen).
3. **Werkzeug-Deklarationen (JSON Schemas):**
   - Schemas aller verfuegbaren Werkzeuge (`read_file`, `run_in_bash_session`, `submit`, etc.).
4. **Aktueller Plan-Zustand (`set_plan`):**
   - Der aktuelle, mit `set_plan` eingestellte Fortschrittsplan.
5. **Konversationshistorie & Dateiinhalte:**
   - Verlauf aller Prompts, Tool-Calls, Tool-Antworten und Dateiinhalte, die während der Session gelesen wurden.

---

## 5. Werkzeugkatalog & Nutzungsmethodik

Google-Jules stehen zwei Klassen von Werkzeugen zur Verfügung: **Plattform-Tools** (direkte Function Calls) und **Workspace Harness-Tools** (`cargo xtask`).

### 5.1 Agent-Plattform-Tools

| Tool-Name | Typ | Hauptzweck |
|---|---|---|
| `list_files(path)` | Read-only | Listet Dateien & Ordner im Repository |
| `read_file(filepath)` | Read-only | Liest Textinhalte einer Datei |
| `write_file(filepath, content)` | Mutating | Erstellt oder überschreibt eine Datei vollständig |
| `replace_with_git_merge_diff` | Mutating | Gezielter Search-and-Replace im Git Merge Diff Format |
| `delete_file(filepath)` | Mutating | Löscht eine Datei |
| `rename_file(old, new)` | Mutating | Benennt Datei/Ordner um |
| `restore_file(filepath)` | Recovery | Stellt geänderte Datei auf Git-HEAD wieder her |
| `reset_all()` | Recovery | Setzt das gesamte Repo auf Git-HEAD zurück |
| `run_in_bash_session(cmd)` | Execution | Führt Bash-Befehle aus (Kompilierung, Tests, xtask) |
| `request_plan_review(plan)` | Planning | Blendet Plan zur Vorab-Prüfung ein |
| `set_plan(plan)` | Planning | Setzt den verbindlichen Markdown-Plan |
| `plan_step_complete(msg)` | Planning | Markiert einen Plan-Schritt als abgeschlossen |
| `request_user_input(msg)` | Interactive | Stellt eine Frage an den Benutzer |
| `message_user(msg, cont)` | Interactive | Sendet Status-Informationen an den Benutzer |
| `pre_commit_instructions()` | Workflow | Holt verbindliche Checkliste vor Task-Abgabe |
| `initiate_memory_recording()`| Memory | Startet Aufzeichnung von Lerneffekten für künftige Tasks |
| `submit(...)` | Submission | Committet Code und beantragt Push/PR-Merge |
| `read_pr_comments()` | GitHub | Liest eingegangene Pull Request Kommentare |
| `reply_to_pr_comments(...)` | GitHub | Antwortet auf spezifische PR-Kommentare |
| `google_search` / `view_text_website` | Web | Liest Dokumentation oder externe Quellen |

### 5.2 Contextra Harness-Tools (`cargo xtask`)

Die verbindliche Werkzeug-Quelle für Repo-Analyse ist `cargo xtask`. Wichtige Tiers:

- **Tier 0 (Build-frei, sofort):**
  - `cargo xtask harness-list`: Verbindliches Verzeichnis aller Harness-Befehle.
  - `cargo xtask fast-diff-lint`: Schneller Diff-Linter für Doktrin-Verstöße.
  - `cargo xtask repo-doctor`: Dashboard für Session-Start.
  - `cargo xtask env-attest`: Validierung der Build-Umgebung.
- **Tier 1 (Workspace-Scan via `syn` AST-Analyse, build-frei):**
  - `cargo xtask orphan-symbols`: Erkennt ungenutzte pub-Symbole.
  - `cargo xtask unwired-ports`: Erkennt nicht-implementierte Traits.
  - `cargo xtask doctrine-scan`: Prüft Zero-Panic, Unsafe-Inseln und Allow-Overrides.
  - `cargo xtask dependency-graph-audit`: Prüft Ring-DAG (P5) und Tokio in Ring 0 (P26).
  - `cargo xtask check-ring-layering`: Layering-Check.
- **Tier 2 (Build-abhängig):**
  - `cargo xtask check-compile`: Prüft Fehlerfreiheit beim Compilieren.
  - `cargo xtask bench-gate --tolerance 0.05`: Benchmark-Regressionstests.
  - `cargo xtask feature-matrix`: Testet Feature-Kombinationen.
  - `cargo xtask mutants-diff`: Mutation Testing Score auf Diffs.
  - `cargo xtask wal-replay-verify`: WAL-Replay-Crash-Konsistenz.


---

## 6. VM-Infrastruktur & Laufzeitumgebung

Jules führt alle Operationen in einer hochperformanten, isolierten Sandbox-VM aus.

### 6.1 Hardware-Topologie

- **Betriebssystem:** Ubuntu 24.04.4 LTS (Noble Numbat), Linux Kernel 6.8.0 x86_64 PREEMPT_DYNAMIC
- **CPU:** Intel(R) Xeon(R) CPU @ 2.30GHz (4 vCPUs, NUMA Node 0)
- **RAM:** 7.8 GiB gesamt (~7.4 GiB verfügbar), Swappiness 0
- **Disk:** 98 GiB OverlayFS auf `/` (~93 GiB verfügbar für `/app/target` Artefakte)
- **Arbeitsverzeichnis:** `/app` (Symlink auf den Contextra Git Workspace)

### 6.2 Multi-Language Runtimes & Tooling

- **Rust Toolchain:** Rust 1.89.0 (`cargo`, `rustc`, `miri`, `clippy`, `rustfmt`)
- **Cargo-Utilities:** `cargo-nextest` (parallele Testausführung), `cargo-llvm-cov`, `cargo-audit`, `cargo-deny`, `just`, `ast-grep` (`sg`)
- **Node.js Ecosystem:** Node v22.22.1 LTS, npm v11.11.0, pnpm v10.30.3, yarn v1.22.22
- **Bun:** v1.2.14 Runtime
- **Python:** v3.12.13 (Pyenv managed), NumPy, PyO3 Bindings
- **C/C++ Compiler & Libraries:** GCC 13.3.0, Clang 18.1.3, CMake 3.28.3, Ninja 1.11.1, FlatBuffers Compiler (`flatc`) v24.3+, Tauri System-GTK/WebKit Entsorgungslibs.

---

## 7. Quellcode-Interaktion, Doktrin & Invarianten

### 7.1 "Edit Source, Not Artifacts"-Prinzip
Jules editiert **niemals** generierte Artefakte (z. B. `target/`, `dist/`, oder auto-generierten Code wie `contextra_generated.rs`). Änderungen erfolgen ausschließlich in Quellcodedateien (`.rs`, `.fbs`, etc.). Nach Schema-Änderungen in FlatBuffers führt Jules `cargo xtask regenerate-flatbuffers` aus.

### 7.2 Zero-Panic-Doktrin (P2)
In Produktionscode sind `unwrap()`, `expect()`, `panic!()`, `todo!()` und `unimplemented!()` strikt verboten. Alle Fehler werden als `Result<T, ContextraError>` propagiert.

### 7.3 Ring-DAG-Architektur & Import-Regeln (P5, P12, P26, P28)
- **P5:** Abhängigkeiten dürfen nur zu niedrigeren Ringen verlaufen. Ring 0 darf keine höheren Ringe importieren.
- **P12:** `capabilities.toml` ist die alleinige Quelle der Wahrheit für Ring-Zuordnungen.
- **P26:** Ring 0 bindet `tokio` nicht als Produktions-Abhängigkeit ein (Async-Reinheit).
- **P28:** Determinismus über injizierte Ports (`Clock`, `Rng`, `IdGenerator`).

### 7.4 Unsafe-Inseln & Safety-Audit
Unsafe Code ist nur in explizit deklarierten Unsafe-Inseln erlaubt (`contextra-simd`, `contextra-sys`, `contextra-wire`). Jedes `unsafe`-Block erfordert einen strukturierten `# Safety` Doc-Comment sowie direkt darüber liegende `// SAFETY:` Absicherungen innerhalb von 3 Zeilen.

---

## 8. Test- & Benchmark-Spezifikation

### 8.1 Testausführung
- **Standard Unit & Integrationstests:**
  ```bash
  cargo test -p contextra-store
  cargo nextest run -p contextra-engine
  ```
- **Prüfung ohne Domain-Build:**
  ```bash
  cargo xtask check-compile
  cargo xtask fast-diff-lint
  ```

### 8.2 Loom, TSan, Fuzzing & Mutation-Testing
- **Concurrency Verification (Loom):** `cargo xtask loom-run`
- **ThreadSanitizer (TSan):** Wird in CI über `.github/workflows/tsan.yml` mit `nightly-2026-10-06` und `-Zbuild-std` ausgeführt.
- **Fuzzing Smoke Tests:** `cargo xtask fuzz-smoke`
- **Diff-Mutation Testing:** `cargo xtask mutants-diff`

### 8.3 Benchmark-Ausführung
- **Benchmarking einzelner Crates:**
  ```bash
  cargo bench -p contextra-simd
  ```
- **Automatisierte Benchmark-Regressionstests:**
  ```bash
  cargo xtask bench-gate --tolerance 0.05
  cargo xtask bench-trend
  ```
- **Reproduzierbare Genauigkeits-Benchmarks:**
  Ausführung des spezialisierten Harnesses `crates/contextra-bench`.

---

## 9. Abgabe-Prozess, Submit-Tool & Pull-Requests

### 9.1 Ablauf vor der Abgabe
Vor dem Aufruf von `submit` muss Jules die Anweisungen von `pre_commit_instructions` abarbeiten:
1. Relevante Tests ausführen.
2. Code Review Tooling aufrufen & Feedback umsetzen.
3. Memory Recording via `initiate_memory_recording()` durchführen.

### 9.2 Funktionsweise von `submit`
Der Tool-Aufruf `submit` verlangt vier Parameter:
- `branch_name`: Kurzer, sprechender Branch-Name (z. B. `feat/purge-compaction-fix`).
- `commit_message`: Standard Git Commit Message (50 Zeichen Subject, Leerzeile, Body).
- `title`: Titel des Pull Requests.
- `description`: Detaillierte Zusammenfassung der Änderungen, Testergebnisse und Verifikationen.

Intern committet die Umgebung den Code lokal und fordert die Freigabe des Benutzers zum Push auf den Remote-Branch an.

### 9.3 Bearbeitung von PR-Kommentaren
Wenn der Benutzer Feedback zu einem PR gibt, nutzt Jules:
- `read_pr_comments()`: Um ausstehende Kommentare abzurufen.
- `reply_to_pr_comments(replies)`: Um strukturierte Antworten auf Kommentare zu verfassen.

---

## 10. GitHub-Integration & Grenzen

### 10.1 Handlungsspielraum auf GitHub
- **Lesen:** Jules kann PR-Kommentare, Workflows (`.github/workflows/*`), Git-Branches, Tag-Historien und Commits lesen.
- **Schreiben:** Jules kann Commits erstellen, Zweige pushen (über `submit`), PRs erstellen/aktualisieren und auf PR-Kommentare antworten.

### 10.2 Grenzen
- Jules besitzt keinen direkten administrativen GitHub API-Token für destruktive Aktionen (z. B. Repository löschen, Repository-Settings ändern, Force-Merge ohne Status-Checks).
- Jules kann keine Secrets oder Repository-Tokens einsehen.

---

## 11. Wiederherstellung von Git-Verlauf & Artefakten

### 11.1 Lokale Rekonstruktion
Google-Jules kann den Git-Verlauf und gelöschte/veränderte Dateien jederzeit wiederherstellen:

1. **Datei auf HEAD zurücksetzen:** `restore_file(filepath)`
2. **Gesamtes Repository zurücksetzen:** `reset_all()`
3. **Gelöschte/Verlorene Commits über Reflog finden:**
   ```bash
   git reflog
   git checkout <commit_sha>
   ```
4. **Verlorene Dateien aus alten Commits wiederherstellen:**
   ```bash
   git checkout <commit_sha> -- <path/to/file>
   ```

### 11.2 Grenzen bei Remote-Löschungen
Wenn ein Branch auf Remote (GitHub) gelöscht wurde und lokal nicht im Git-Cache (`.git/objects`) vorhanden ist, kann Jules diesen Remote-Branch nicht autonom wiederherstellen, es sei denn, er existierte während der aktuellen VM-Sitzung im lokalen Git-Reflog.

---

## 12. Repository-Optimierungen für Contextra (Implementierungsstatus)

Um autonome Agenten-Sitzungen (bis zu 165 Sitzungen/Tag) sowie menschliche Entwicklungszyklen in `contextra` weiter zu optimieren:

### ✅ Implementiert (Oktober 2026)

1. **`harness-phases.toml` vollständig** (`.jules/harness-phases.toml`):
   - `cargo xtask repo-doctor` in `start`-Phase ergänzt (Dashboard beim Session-Start).
   - `cargo xtask gate-integrity` in `verify`-Phase ergänzt (fehlte vorher).
   - `cargo xtask fast-diff-lint` in `check`-Phase ergänzt.
   - Alle `risk_gates`-Klassen vollständig: `concurrency` → `loom-run`, `utf8`/`fusion`/`calib`/`none` → Standard.

2. **`environment_script.sh` AGENTS.md-konform** (`.jules/setup/environment_script.sh`):
   - Alle `bash -c '...'` und `sh -c`-Aufrufe durch direkte Argument-Arrays ersetzt (AGENTS.md §2).
   - `--exclude contextra-py` bei `cargo check --workspace` (vermeidet PyO3/C-Erweiterungen in Routinechecks).
   - `--jobs 4` bei `cargo fetch` (nutzt alle 4 vCPUs der VM).
   - Nightly-TSan-Check im `full`-Profil ergänzt.

3. **`required-tools.toml` vollständig** (`.jules/setup/required-tools.toml`):
   - `cargo-audit` als `required = true` ergänzt (fehlte trotz Installation).
   - Alle `version_cmd`-Felder als Argument-Arrays (kein Shell-String).

4. **Neue Hilfsdateien für Jules**:
   - [`.jules/QUICK_REFERENCE.md`](../.jules/QUICK_REFERENCE.md): Kompakter Spickzettel aller Lifecycle-Befehle, Top-10-APMs, Invarianten-IDs, Crate-Schnellindex.
   - [`.jules/TASK_TEMPLATE.md`](../.jules/TASK_TEMPLATE.md): Vollständige TOML-Schablone für neue Task-Karten mit Risiko-Klassen-Tabelle und Planmuster.

5. **Abschnitt 5.3 (eigene Python-Werkzeuge) entfernt** (Widerspruch zu AGENTS.md §10).

### 🔲 Geplant (noch nicht implementiert)

6. **`WorkspaceIndex` für xtask vervollständigen** (Tier 1 Beschleunigung):
   - Das Caching des AST-basierten Workspace-Index in `target/xtask-cache/workspace-index.json` reduziert die Ausführungszeit von `orphan-symbols`, `unwired-ports` und `doctrine-scan` von ~2.5s auf < 150ms.

7. **Kompilier-Caches & Target-Warmup**:
   - Bereitstellung eines vorgebauten Cargo target Warmup-Archives (`/target`), damit frische Agenten-VMs nicht wiederholt Abhängigkeiten wie `syn`, `quote`, `serde` oder `tokio` von Grund auf neu kompilieren müssen.

8. **Entkopplung der Heavy Gates**:
   - Beibehaltung der Aufteilung von Schnellprüfungen (Tier 0 & Tier 1) für Routine-Iterationen und Auslagerung von Loom/Mutation/TSan auf Scheduled/Nightly-Runs bzw. PRs mit Label `run-deep`.
