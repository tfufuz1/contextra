# Gate-Canary-Suite (gate-canary)

Stand: 2026-09-30 (Contextra Governance & Gate-Sicherheit)

## 1. Übersicht & Zielsetzung

Die **Gate-Canary-Suite** (`gate-canary`) stellt durch automatistische Tests sicher, dass Qualitäts-Gates im Contextra-Workspace im Falle eines echten Defekts oder Regelverstoßes zuverlässig anschlagen (**rot werden / Exit != 0**) und nicht fälschlicherweise stumm oder „immer grün" bleiben.

Jedes Canary-Testfall-Paar besteht aus:
1. **Seeded-Defect-Fixture**: Injiziert gezielt einen typischen Fehler oder Qualitätsverstoß in einem isolierten `TempDir`-Git-Repository. Das Gate **muss** fehlschlagen.
2. **Gegen-Fixture (Counter-Fixture)**: Prüft denselben Pfad auf einem sauberen Diff/Zustand. Das Gate **muss** erfolgreich bestehen (Exit 0 / grün).

Die Tests verändern zu keinem Zeitpunkt das echte Workspace-Repository.

---

## 2. Übersichtstabelle: Gate -> Fixture -> Erwartetes Ergebnis

| Gate | Testfunktion (`xtask/tests/canaries/main.rs`) | Seeded Defect (Fehler-Fixture) | Erwartetes Ergebnis (Defekt) | Gegen-Fixture (Sauberer Zustand) | Erwartetes Ergebnis (Sauber) |
|---|---|---|---|---|---|
| **`panic-inventory`** | `test_canary_panic_inventory_ring0_unwrap` | Hinzufügen von `.unwrap()` in Produktionscode eines Ring-0-Crates (`contextra-core`). | **FAIL** (`Err("Strict mode failure...")`) | Produktionscode verwendet Result-basiertes Error Handling. | **PASS** (`Ok(...)`, 0 Panics) |
| **`gate-weakening`** | `test_canary_gate_weakening` | Einfügen von `#[allow(unused)]` im Rust-Quellcode. | **FAIL** (Exit-Code != 0) | Hinzufügen einer sauberen Funktion ohne Linter-Absenkung. | **PASS** (Exit-Code == 0) |
| **`protected-paths`** | `test_canary_protected_paths` | Ändern von geschützten Dateien (`AGENTS.md`) ohne `Protected-Change: ADR-xxx` Trailer in der Commit-Message. | **FAIL** (Exit-Code != 0) | Ändern einer ungeschützten Datei (`src/lib.rs`). | **PASS** (Exit-Code == 0) |
| **`check-dag`** | `test_canary_check_dag_ring_layering` | Erzeugen einer Rückwärtskante in Ring 0 (`contextra-types` [Ordnung 0] hängt von `contextra-graph` [Ordnung 3] ab). | **FAIL** (Erkennt aktive Verstöße) | Valide Abwärts-Abhängigkeit (Ring 1 `contextra-store` -> Ring 0 `contextra-core`). | **PASS** (0 aktive Verstöße) |
| **`determinism-check`** | `test_canary_determinism_check` | Direkter Aufruf von `SystemTime::now()` im Produktionspfad eines Ring-0-Crates. | **FAIL** (Exit-Code != 0) | Deterministischer Funktionsaufruf ohne direkte Zeit/Zufall-Aufrufe. | **PASS** (Exit-Code == 0) |
| **`check-unsafe-islands`** | `test_canary_check_unsafe_islands` | Nutzung des `unsafe`-Keywords in einer Nicht-Insel-Crate (`contextra-core`). | **FAIL** (`errors` nicht leer) | `unsafe` ausschließlich in zulässiger Insel-Crate (`contextra-sys`); Nicht-Inseln nutzen `#![forbid(unsafe_code)]`. | **PASS** (`errors` ist leer) |
| **`scope-guard`** | `test_canary_scope_guard` | Bearbeiten einer Datei außerhalb des in der Task-Karte freigegebenen Scopes (`src/out_of_scope.rs`). | **FAIL** (Exit-Code != 0) | Bearbeiten ausschließlich freigegebener Dateien (`src/in_scope.rs`). | **PASS** (Exit-Code == 0) |
| **`test-integrity`** | `test_canary_test_integrity` | Löschen einer Testdatei ohne gleichwertigen Ersatz im selben Diff. | **FAIL** (Exit-Code != 0) | Hinzufügen eines validen Tests mit Assertionen. | **PASS** (Exit-Code == 0) |
| **`symbol-exists`** | `test_canary_symbol_exists` | Abfrage eines erfundene/nicht-existierenden Symbols (`contextra-core::InventedSymbol`). | **FAIL** (Exit-Code != 0) | Abfrage eines im Code existierenden Typs (`contextra-core::RealStruct`). | **PASS** (Exit-Code == 0) |
| **`ratchet`** | `test_canary_ratchet` | Erhöhung von `#[allow(...)]`-Attributen über die festgelegte `ratchet.toml`-Baseline. | **FAIL** (Exit-Code != 0) | Kennzahlen verbleiben strikt innerhalb/unterhalb der Baseline. | **PASS** (Exit-Code == 0) |

---

## 3. Anleitung für die Einrichtung des CI-Jobs (`gate-canary`)

*Hinweis: Dieser Auftrag erstellt KEINE CI-Workflow-Datei in `.github/workflows/`. Die Anbindung im CI-System erfolgt separat durch den Repository-Administrator.*

### 3.1 Ausführung
Die Canary-Suite wird über den Cargo-Test-Befehl für das `xtask`-Package aufgerufen:

```bash
cargo test --manifest-path xtask/Cargo.toml canaries
```

Oder gezielt der Test-Target-Name:

```bash
cargo test --manifest-path xtask/Cargo.toml --test canaries
```

### 3.2 Empfohlene CI-Pipeline-Integration
- **Frequenz**: Nightly Build oder als verpflichtendes Gate im `merge-gate.yml` Workflow.
- **Isolierung**: Die Tests nutzen `tempfile::TempDir` und eigene Git-Initialisierungen in `/tmp`. Es werden keine Umgebungsvariablen des Haupt-Repositories korrumpiert.
- **Erfolgskriterium**: Alle 10 Canary-Tests müssen den Status `ok` zurückliefern. Ein Fehlschlagen der Canary-Suite bedeutet, dass ein Qualitäts-Gate im Repository beschädigt wurde oder regrediert ist.

---

## 4. Ausgenommene / Nicht als separates Fixture geführte Gates

Für folgende spezialisierte Gates wurde begründet kein separates Canary-Fixture in dieser Suite angelegt:

1. **`lock-audit`**:
   - *Begründung*: Prüft verschachtelte Mutex-Locks und Sperrenhierarchien über mehrere Threads. Wird primär über Loom-Concurrency-Runs (`cargo xtask loom-run`) und direkte Harness-Tests (`xtask/tests/harness_lock_audit_test.rs`) abgedeckt.
2. **`diff-budget`**:
   - *Begründung*: Rein zahlenmäßiger Schwellenwert-Vergleich der geänderten Zeilen im Git-Diff gegenüber der Task-Karte. Bereits vollständig durch `xtask/tests/harness_diff_budget_test.rs` abgedeckt.
3. **`veto-deadline-gate`**:
   - *Begründung*: Überprüft das aktuelle Systemdatum gegen Veto-Ablaufdaten in `VETOES.md`. Ein Fixture würde künstliche Manipulationen der Systemzeit erfordern; bereits durch synthetische In-Memory-Datumstests in `xtask/tests/veto_deadline_gate_test.rs` geschützt.
4. **`doc-truth`**:
   - *Begründung*: Validiert Backtick-Pfade und Crate-Kataloge in Dokumenten gegen das reale Dateisystem. Bereits durch den dedizierten Harness-Test `xtask/tests/harness_doc_truth_test.rs` abgesichert.
