# ADR-NEW: xtask Gate-Verdrahtung, Redundanzbereinigung und Negativtest-Erzwingung

* **Status:** Proposed
* **Datum:** 2026-10-21
* **Autoren:** Governance- und Architekturreviewer
* **Betroffene Komponenten:** `xtask/`, Governance-Harness, Gate-Pipeline
* **Spezifikationsreferenzen:** AGENTS.md, ADR-N32, Gesamtregister-Befunde (D-22, D-23, LOG-2, LOG-3, LOG-4, GOV-4)

---

## 1. Kontext & Befund (mit Datei:Zeile-Belegen)

Das Crate `xtask` unterliegt gemäß `AGENTS.md` strengen Governance-Regeln und Pfadschutzbestimmungen. Nach ADR-N32 ist `xtask` die zentrale Steuerungseinheit für alle CI-Gates, Pre-Flight-Prüfungen und Repository-Integritätschecks.

Eine statische Quelltext- und Control-Flow-Analyse am HEAD zeigt gravierende Lücken in der Gate-Aktivierung sowie Code-Redundanzen:

1. **Unerreichbarer doppelter Match-Arm in `xtask/src/main.rs` (Befund LOG-2, D-23):**
   - In `xtask/src/main.rs` existiert der CLI-Dispatch-Befehl `"check-commit-diff-integrity"` an zwei Stellen:
     - **Erster Arm:** `xtask/src/main.rs:2148–2155`
     - **Zweiter Arm:** `xtask/src/main.rs:2437–2444`
   - Beide Arme rufen `check_commit_diff_integrity::run_check_commit_commit_diff_integrity(extra_args)` auf.
   - Da der erste Arm in der `match`-Anweisung priorisiert ausgewertet wird, ist der zweite Arm in Zeile 2437 absolut unerreichbar (`unreachable_patterns`). Dies erzeugt Totcode und verfälscht die Wartbarkeit des Dispatchers.

2. **Unverdrahtete Gate-Funktionen im Binärpfad (Befund LOG-3, D-22):**
   - Mehrere Sicherheits- und Integritätsprüffunktionen sind vollständig im Codebasis-Bestand vorhanden, werden jedoch im Haupt-Dispatch-Pfad (`xtask/src/main.rs`) oder in den zugeordneten Unterbefehlen nicht aufgerufen.
   - Ein Gate, das nicht aufgerufen wird, meldet in CI-Pipelines stillschweigend Erfolg, ohne tatsächlich Prüfungen durchzuführen. Dies führt zu **falschem Grün** (Silent Success Bug).
   - Befund-Details:
     - `loop_guard_record_text`, `loop_guard_record`, `loop_guard_check`, `loop_guard_reset`, `run_loop_guard` (`xtask/src/harness/loop_guard.rs:108, 151, 166, 237, 250`): Werden außerhalb isolierter Unit-/Wiring-Tests im Haupt-Harness nicht ausgewertet.
     - `run_scope_guard` (`xtask/src/harness/scope_guard.rs:93`): Wird in `main.rs` nicht angesprochen.
     - `calculate_p95` (`xtask/src/check_bandit_latency_budget.rs:33`): Berechnung des P95-Latenzbudgets wird im Binärpfad nicht zur Härte-Evaluierung herangezogen.
     - `check_veto_deadlines_from_content` (`xtask/src/veto_deadline_gate.rs:227`): Im Befehl `"check-veto-deadlines"` (`main.rs:3419`) wird nur `run_check_veto_deadlines` ausgeführt; die Inhaltsprüfung für Entwürfe/In-Memory-Strings bleibt unaufgerufen.
     - `get_crate_ring` (`xtask/src/check_ring_layering.rs:164`): Ring-Zuordnung wird an relevanten Layering-Prüfstellen übergangen.
     - `run_loom` (`xtask/src/loom_run.rs:62`): Zwar über CLI `"loom-run"` erreichbar (`main.rs:3247`), jedoch in zentralen Gate-Aggregatoren nicht als verpflichtendes Sub-Gate verdrahtet.

3. **Inaktive Allowlist- und Ablaufdatum-Evaluierung (Befund LOG-4, GOV-4):**
   - Ausnahmelisten in `xtask/config/caller-allowlist.toml` (Zeilen 6, 7) sowie `xtask/src/check_ring_layering.rs` (Zeilen 91, 99, 107) definieren Ablaufdaten zum **31.12.2026**.
   - Die automatische Validierung und Durchsetzung des Ablaufdatums (`expires_on` / `expiry`) wird im aktiven Binärpfad nicht ausgewertet, sodass abgelaufene Ausnahmen unbemerkt gültig blieben.

---

## 2. Bewertung der unverdrahteten Gate-Funktionen (Entscheidung D-22)

Um falsches Grün systematisch zu eliminieren, gilt der Grundsatz: **Verdrahten ist der Standard.** Nur nachweislich obsolete Hilfsfunktionen dürfen entfernt werden.

| Funktion / Pfad | Status in `main.rs` | Risikobewertung bei Nicht-Aufruf | Empfehlung | Begründung |
| :--- | :--- | :--- | :--- | :--- |
| `loop_guard_record` / `loop_guard_check` / `run_loop_guard`<br>`(xtask/src/harness/loop_guard.rs:151, 166, 250)` | Unverdrahtet | **KRITISCH:** Endlosschleifen und unkontrollierte Agenten-Rounds werden nicht gestoppt. | **Verdrahten** | In `jules-preflight` / `jules-submit-gate` integrieren, um Repetitions-Edits abzufangen. |
| `run_scope_guard`<br>`(xtask/src/harness/scope_guard.rs:93)` | Unverdrahtet | **HOCH:** Änderungen außerhalb der zugewiesenen Scope-Pfade werden im Pre-Flight nicht verweigert. | **Verdrahten** | Als verflichtenden Guard in den Agenten-Submission-Workflow einbinden. |
| `calculate_p95`<br>`(xtask/src/check_bandit_latency_budget.rs:33)` | Unverdrahtet | **MITTEL:** Latenzbudget-Überschreitungen (Bandit Router) werden nicht numerisch geglättet geprüft. | **Verdrahten** | In den `check-bandit-latency-budget`-Befehl einbinden, um P95-Perzentile gegen Grenzwerte zu prüfen. |
| `check_veto_deadlines_from_content`<br>`(xtask/src/veto_deadline_gate.rs:227)` | Teilweise (nur File-Pfad verdrahtet) | **MITTEL:** In-Memory / Staging Veto-Reviews umgehen Fristenprüfungen. | **Verdrahten** | In die Scoped- und Staging-Prüfungen von `check-veto-deadlines` integrieren. |
| `get_crate_ring`<br>`(xtask/src/check_ring_layering.rs:164)` | Unverdrahtet | **HOCH:** Ring-Ebenenverletzungen (z. B. Ring 3 importiert Ring 0 direkt) werden nicht präzise klassifiziert. | **Verdrahten** | In `check-ring-layering` zur dynamischen Ring-Validierung einsetzen. |
| `run_loom`<br>`(xtask/src/loom_run.rs:62)` | Nur als manueller Befehl existierend | **HOCH:** Nebenläufigkeits-Metamorphe (Loom) werden im Standard-PR-Gate übergangen. | **Verdrahten** | Als bedingtes Sub-Gate oder dezidierten Gate-Check in die Heavy-CI-Pipeline einbinden. |

---

## 3. Entscheidung D-23: Behandlung des doppelten Match-Arms

### Vergleich beider Arme in `xtask/src/main.rs`:

- **Arm 1 (Zeile 2148–2155):**
  ```rust
  "check-commit-diff-integrity" => {
      let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
      if let Err(e) = check_commit_diff_integrity::run_check_commit_diff_integrity(extra_args)
      {
          eprintln!("❌ check-commit-diff-integrity failed: {}", e);
          process::exit(1);
      }
  }
  ```

- **Arm 2 (Zeile 2437–2444):**
  ```rust
  "check-commit-diff-integrity" => {
      let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
      if let Err(e) = check_commit_diff_integrity::run_check_commit_diff_integrity(extra_args)
      {
          eprintln!("❌ check-commit-diff-integrity failed: {}", e);
          process::exit(1);
      }
  }
  ```

### Empfehlung & Entscheidung:
Beide Match-Arme sind **identisch**. Arm 2 an Zeile 2437 ist durch den Rust-Compiler als unerreichbarer Code klassifiziert.
**Entscheidung:** Entfernung von Arm 2 (Zeilen 2437–2444 in `xtask/src/main.rs`). Arm 1 bleibt unverändert als kanonischer Dispatch-Punkt erhalten.

---

## 4. Falsches Grün: Testprinzip

Jedes Sicherheits- und Governance-Gate im `xtask`-Harness muss nach folgendem **Testprinzip der inversen Validierung** abgesichert sein:

1. **Negativtest-Pflicht (Negative Testing Requirement):**
   - Zu jedem verdrahteten Gate muss ein automatisierter Integrationstest existieren, der **nachweislich fehlschlägt (Exitcode != 0)**, sobald eine synthetische Regelverletzung injiziert wird.
2. **Keine Stummschaltung (No Silent Pass):**
   - Wenn Eingabedateien, Konfigurationen oder Git-Kontexte fehlen, darf ein Gate nicht mit `Ok(())` (Grün) beenden, sondern muss explizit fehlschlagen oder eine dokumentierte Warnung mit Error-Boundary ausgeben.
3. **Deterministischer Nachweis:**
   - Der Negativtest muss Teil der regulären `cargo test -p xtask`-Testsuite sein.

---

## 5. Implementierungsplan in drei PRs

Die Umsetzung erfolgt strikt sequenziell in drei getrennten Pull Requests:

### PR (a): Bereinigung doppelter Match-Arme
- Entfernen des unerreichten Match-Arms `"check-commit-diff-integrity"` in `xtask/src/main.rs:2437–2444`.
- Verifikation via `cargo check -p xtask`.

### PR (b): Verdrahtung der Gate-Funktionen & Negativtests
- Einbinden aller unverdrahteten Gate-Funktionen (`loop_guard_*`, `run_scope_guard`, `calculate_p95`, `check_veto_deadlines_from_content`, `get_crate_ring`, `run_loom`) in die entsprechenden `xtask`-Befehle und Pre-Flight-Flows.
- Schreiben von Negativtests in `xtask/tests/`, die beweisen, dass jedes verdrahtete Gate bei Regelverstößen ordnungsgemäß abbricht.

### PR (c): Allowlist-Auswertung & Fristenprüfung
- Aktivierung der automatischen Auswertung von `caller-allowlist.toml` und `LAYER_ALLOWLIST`.
- Implementierung der hart durchgesetzten Fristenprüfung für Ablaufdaten (inklusive Stichtag `2026-12-31`).
- Verbot der Verlängerung von Ablaufdaten ohne explizite Architektur-Freigabe.

---

## 6. Vorbehalt & Scope-Grenzziehung

* **Eigentümer-Freigabe erforderlich:** Die Ausführung der im Plan genannten drei PRs erfolgt erst nach expliziter Owner-Freigabe gemäß `AGENTS.md`.
* **Exklusivität dieses PRs:** Dieser Pull Request beschränkt sich ausschließlich auf die Erstellung dieses Dokumentes (`docs/decisions/ADR-NEW-xtask-gate-wiring.md`). Code-Änderungen an `xtask/**`, `capabilities.toml`, `registry.toml`, `justfile`, `AGENTS.md` oder `.github/**` sind **nicht** Bestandteil dieses PRs.

---

## Anhang: Verifikationsnachweise (Grep-Befehle & Ausgaben)

Die in diesem ADR angegebenen Zeilen- und Dateibelege wurden am HEAD mit den folgenden `grep`-Befehlen verifiziert:

1. **Prüfung doppelter Match-Arm `check-commit-diff-integrity`:**
   ```bash
   $ grep -n "check-commit-diff-integrity" xtask/src/main.rs
   2148:        "check-commit-diff-integrity" => {
   2152:                eprintln!("❌ check-commit-diff-integrity failed: {}", e);
   2437:        "check-commit-diff-integrity" => {
   2441:                eprintln!("❌ check-commit-diff-integrity failed: {}", e);
   3444:            eprintln!("Available commands: ... check-commit-diff-integrity ...");
   ```

2. **Prüfung unverdrahteter Gate-Funktionen:**
   ```bash
   $ grep -n -E "fn loop_guard_record|fn loop_guard_check|fn loop_guard_reset|fn run_loop_guard|fn run_scope_guard|fn calculate_p95|fn check_veto_deadlines_from_content|fn get_crate_ring|fn run_loom" xtask/src/harness/*.rs xtask/src/*.rs
   xtask/src/check_bandit_latency_budget.rs:33:pub fn calculate_p95(latencies_us: &[u64]) -> u64 {
   xtask/src/check_ring_layering.rs:164:pub fn get_crate_ring(crate_name: &str) -> Option<Ring> {
   xtask/src/harness/loop_guard.rs:108:pub fn loop_guard_record_text(...)
   xtask/src/harness/loop_guard.rs:151:pub fn loop_guard_record(...)
   xtask/src/harness/loop_guard.rs:166:pub fn loop_guard_check(root: &Path, max_file_edits: u32) -> Result<(), String> {
   xtask/src/harness/loop_guard.rs:237:pub fn loop_guard_reset(root: &Path) -> Result<(), String> {
   xtask/src/harness/loop_guard.rs:250:pub fn run_loop_guard(args: &[String]) -> i32 {
   xtask/src/harness/scope_guard.rs:93:pub fn run_scope_guard(args: &[String]) -> i32 {
   xtask/src/loom_run.rs:62:pub fn run_loom(test_filter: Option<&str>, root: &Path) -> Result<LoomRunResult, String> {
   xtask/src/veto_deadline_gate.rs:227:pub fn check_veto_deadlines_from_content(...)
   ```

3. **Prüfung der Stale-Allowlist-Einträge (Ablauf 31.12.2026):**
   ```bash
   $ grep -rn "2026-12-31" xtask/config/ xtask/src/
   xtask/config/caller-allowlist.toml:6:fast = { reason = "Default feature in contextra facade", expiry = "2026-12-31" }
   xtask/config/caller-allowlist.toml:7:fault-injection = { reason = "Default feature in contextra-testkit harness", expiry = "2026-12-31" }
   xtask/src/check_ring_layering.rs:91:        expires_on: "2026-12-31",
   xtask/src/check_ring_layering.rs:99:        expires_on: "2026-12-31",
   xtask/src/check_ring_layering.rs:107:        expires_on: "2026-12-31",
   ```
