# Google-Jules Diagnose- & Optimierungsbericht in der Contextra-Entwicklung

**Datum:** 2026-03-06
**Ziel:** Vollständige Diagnose der VM-Umgebung, Kompilierung, Testabdeckung, Abhängigkeiten, xtasks und Gate-System für hochfrequenten Entwicklungsbetrieb (bis zu 165 Sitzungen/Tag).

---

## 1. Diagnose der VM-Umgebung & Abhängigkeiten

### 1.1 System- & Hardwareressourcen
* **CPU:** Intel(R) Xeon(R) CPU @ 2.30GHz (4 vCPUs, NUMA 0)
* **RAM:** 7.8 GiB gesamt, 7.4 GiB frei (kein SWAP)
* **OS:** Ubuntu 24.04.4 LTS (Linux Kernel 6.8.0 x86_64)
* **Disk Overlay:** 98 GiB Root OverlayFS (93 GiB frei für Build-Artefakte)

### 1.2 Verfügbare vs. Fehlende Werkzeuge & Abhängigkeiten
| Werkzeug / Library | Status | Pfad / Version | Bewertung |
| :--- | :--- | :--- | :--- |
| **rustc / cargo / rustup** | ✅ Verfügbar | `/home/jules/.cargo/bin/` (Rust 1.89.0) | Absolut stabil |
| **gcc / g++ / clang** | ✅ Verfügbar | `/usr/bin/` (GCC 13.3, Clang 18.1) | Vollständig C++20-fähig |
| **make / cmake / ninja** | ✅ Verfügbar | `/usr/bin/` | Vollständig |
| **Node / npm / pnpm / yarn / bun** | ✅ Verfügbar | Node v22.22, Bun v1.2.14 | Ausgezeichnet |
| **Python3 / pip** | ✅ Verfügbar | Python 3.12.13 (`.pyenv`) | Ausgezeichnet |
| **sg (ast-grep)** | ✅ Verfügbar | `/usr/bin/sg` | Funktional (Symlink `ast-grep` wird von Setup erstellt) |
| **flatc (FlatBuffers Compiler)** | ⚠️ Optional / Gefehlt | Nicht vorinstalliert | In `.jules/setup/environment_script.sh` integriert |
| **Tauri System Libs (WebKit/GTK)** | ⚠️ Gefehlt | Nicht vorinstalliert | In `.jules/setup/environment_script.sh` integriert |
| **just / cargo-nextest / cargo-audit** | ⚠️ Gefehlt | Nicht in PATH | In `.jules/setup/environment_script.sh` integriert |

---

## 2. Kompilierungs- & Testabdeckungsanalyse

### 2.1 Workspace-Kompilierung (`cargo check --workspace`)
* **Ergebnis:** Erfolgreich (Duration: ~1m 41s bei Kaltstart, < 5s inkrementell).
* **Workspace-Umfang:** 31 Rust Workspace Crates kompiliert.
* **Architektur-Isolation:** Ring 0 Unsafe Islands (`contextra-simd`, `contextra-sys`, `contextra-wire`) und Ring 0 Sync-Kerne halten alle Invarianten ein.

### 2.2 Test-Abdeckung & Test-Laufzeit
* **Core Crates (`contextra-types`, `contextra-ports`, etc.):** Passieren in < 0.2s (137 Unit- & Integrationstests).
* **Workspace-Weiter Testlauf (`cargo test --workspace`):**
  * Das Ausführen von `cargo test --workspace` ohne Einschränkungen führt bei unoptimierten Debug-Builds und sequenzieller Kompilierung auf 4 vCPUs zu einem **Command Timeout (> 400 Sekunden)**.
  * **Empfehlung & Optimierung:** Testläufe müssen gezielt pro Crate (`cargo test -p contextra-types -p contextra-core ...`) oder mittels `cargo-nextest` in parallelen Chunks ausgeführt werden.

---

## 3. Analyse des Gate-Systems & xtask Optimierungen

Der Durchlauf von `cargo run --manifest-path xtask/Cargo.toml -- jules-preflight --fast` ergab wertvolle Erkenntnisse über Performance und Flaschenhälse im Gate-System:

### 3.1 Gefundene Engpässe in xtask
1. **`check-crate-references` Bottleneck (~118.7 Sekunden Laufzeit):**
   * *Ursache:* Durchsucht rekursiv alle Markdown-, Cargo.toml- und Source-Dateien im Repository nach historischen/umbenannten Crate-Namen (`contextra-security`, `contextra-index`, `contextra-embed`, `contextra-ollama`, `contextra-calibration`).
   * *Optimierungsvorschlag:* Cachen der bekannten Legacy-Realisierungen / Ignore-Pfade in `xtask` oder Einführung einer beschleunigten Index-Tabelle, um die Validierungszeit von 118s auf < 2s zu reduzieren.
2. **`check-module-reachability` (~3.8 Sekunden Laufzeit):**
   * *Status:* Hat 12 tote/unerreichbare Moduldateien identifiziert (z.B. `crates/contextra-agent/src/engine/condition.rs`).
   * *Optimierungsvorschlag:* Diese Dateien sollten entweder in die Modul-Hierarchie eingebunden oder als verwaiste Test-Mocks bereinigt werden.

### 3.2 Gate-Performance Übersicht

| Gate Check | Dauer | Status | Befund / Massnahme |
| :--- | :--- | :--- | :--- |
| **Gate 1: Kritische AI-TAGs** | 0.2s | ✅ PASSED | Keine offenen Kritischen SMELLs |
| **Check Crate References** | 118.7s | ❌ FAILED | 1332 veraltete Crate-Referenzen in Docs/AGENTS.md |
| **Module Reachability Check** | 3.8s | ❌ FAILED | 12 unerreichbare Dateien |
| **Unsafe Islands Check** | 4.0s | ✅ PASSED | `unsafe` streng isoliert |
| **Ring Layering Check** | 0.1s | ⚠️ WARNING | 2 undokumentierte Ring-Abhängigkeiten |
| **Gate 3: Silent IO** | 0.2s | ✅ PASSED | Keine unkontrollierten Print-Statements |
| **Gate 4: MCP axum Guard** | 0.0s | ✅ PASSED | MCP ist reines Stdio |
| **Duplicate Symbols** | 2.9s | ✅ PASSED | Keine Symbolduplikate |
| **DAG-Integrität** | 0.1s | ✅ PASSED | Layer-Hierarchie valide |

---

## 4. Empfohlener Handlungsplan für High-Frequency Agent Sessions (165/Tag)

1. **Schneller Sitzungs-Start:**
   * Jeder Agent führt zu Beginn seiner Sitzung `.jules/setup/environment_script.sh` aus.
2. **Inkrementelle Testabwicklung:**
   * Vermeide globale Unscoped-Tests (`cargo test --workspace`). Nutze stattdessen targeted Cargo Commands oder `cargo nextest run -p <crate>`.
3. **xtask Refactoring:**
   * Beschleunigung von `check-crate-references` in `xtask` zur Reduzierung der CI/Preflight-Zeiten für schnelle Turnarounds.
