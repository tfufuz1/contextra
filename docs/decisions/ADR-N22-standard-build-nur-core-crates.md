# ADR-N22: F7 Standard-Build-Umfang — Nur Core-Crates in default-members

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  Im Workspace existieren über 35 Crates, darunter der Core-Laufzeitkern, spezialisierte Sprach-Bindings (Python PyO3), Prozess-Sandboxen (WASM), optionale ML-Inferenz-Backends (ONNX, Ollama) sowie Benchmarks und Entwurfswerkzeuge.
  Ein unselektiver Standard-Build (`cargo build` / `cargo test` ohne Paketbegrenzung) zwingt jeden Einbettesteller dazu, schwere C-Bibliotheken (ONNX Runtime), Python-Entwicklungs-Header oder WASM-Compiler mitzukompilieren. Dies verlängert die CI-Build-Zeiten massiv und widerspricht dem Grundsatz "Pure Rust, kein Server" (§0.5, §A.3).

## Entscheidung

1. **Einschränkung von `default-members` auf den Core-Laufzeitkern:**
   Der Standard-Build-Bereich im Wurzel-`Cargo.toml` (`default-members`) wird strikt auf den reinen Core-Laufzeitkern (Ring 0–1, Kerne, Engine, öffentliche Facette `contextra`) beschränkt [V].

2. **Crates außerhalb von `default-members`:**
   Folgende Nicht-Core-, FFI-, Inferenz- oder Tooling-Crates verbleiben in `workspace.members`, werden jedoch aus `default-members` ausgeschlossen [V]:
   * **`crates/contextra-sandbox`:** WASM-Ausführungsgrenze (Sandbox-Subsystem, opt-in).
   * **`crates/contextra-py`:** PyO3 / NumPy C-FFI-Bindings für Python.
   * **`crates/contextra-infer-ollama`:** Externer Prozess-Adapter für Ollama HTTP/JSON.
   * **`crates/contextra-infer-onnx`:** Natives ONNX-Runtime C-Bibliotheks-Backend (`ort`).
   * **`benchmarks/contextra-bench`:** Synthetische und Reallast-Benchmark-Suite.
   * **`xtask` / `xtask-heavy`:** Repository-Automatisierungs- und Governance-Tooling.

   Diese Crates werden weiterhin über explizite Paketaufrufe gebaut und getestet (z. B. `cargo build -p contextra-py`).

## Alternativen

### Option A: Alle Crates in `default-members` belassen
* **Vorteile:** Einfaches `cargo build` baut ausnahmslos alles im Repository.
* **Nachteile:** Extrem lange Build-Zeiten; Zwang zur Installation externer C-Toolchains; unnötiger Ballast für eingebettete Anwendungsfälle.
* **Bewertung:** Verworfen [S].

## Konsequenzen

* **Schnelle Entwicklungs-Zyklen:** Entwickler und CI-Jobs für den Core-Kern profitieren von drastisch reduzierten Kompilierzeiten [V].
* **Saubere Trennung:** Klare Entkopplung zwischen dem embedded Core-Kern und optionalen FFI-/Tooling-Adaptern [V].
* **Keine ungewollten C-FFI-Abhängigkeiten:** Einbettesteller von `contextra` erhalten ein reines Pure-Rust-Paket ohne fremde dynamische Bibliotheken [V].

## Reversibilitäts-Bedingung

Eine Wiederaufnahme eines ausgenommenen Crates in `default-members` erfordert den Nachweis, dass es zu einer universell zwingenden Kern-Abhängigkeit für alle Einbettesteller ohne C-FFI-Ballast geworden ist [S].
