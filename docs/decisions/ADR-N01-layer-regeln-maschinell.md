# ADR-N01: Maschinelle Durchsetzung der Layer-Regeln (DAG & Ring-Schichtenmodell)

* **Status:** Final
* **Datum:** 2026-09-22
* **Kontext / Auslöser:**
  In verteilten Entwicklungsumgebungen und bei der Zusammenarbeit mit automatisierten KI-Agenten reicht eine reine Freitext-Dokumentation von Architektur- und Schichtgrenzen (z. B. in `ARCHITECTURE.md` oder `README.md`) nicht aus, um architektonische Regelverstöße zuverlässig zu verhindern.
  Ohne maschinelle Sperren entstehen schleichend unzulässige Aufwärts-Abhängigkeiten (z. B. Ring-0-Domänenkerne, die auf höhergestellte Service- oder Engine-Crates zugreifen) oder unzulässige Laufzeit-Kopplungen (z. B. direkte `tokio`-Importe in synchronen Ring-0-Kernmodulen).

  Zur Vermeidung von Architektur-Drift und zur Einhaltung der DAG-Matrix (GESAMTSPEZIFIKATION §0.3, §1.1, §4.3) erfordert das MemFuse Cognitive OS ein automatisiertes, maschinell erzwungenes Gate.

## Entscheidungen

1. **Maschinell erzwungenes Layering-Gate:**
   Sämtliche Crate-Abhängigkeiten und Schichtenregeln werden automatisiert über das Test-Harness `tests/layering.rs` sowie die `xtask`-Befehle `just dag-check` / `cargo xtask check-ring-layering` auf Basis von `cargo metadata` verifiziert.

2. **Ring-Schichtenmodell (Ring 0 bis 4):**
   Das Repository unterliegt einer Fünf-Ring-Topologie:
   * **Ring 0 (Foundation & Core Domain Logic):** `memfuse-types`, `memfuse-ports`, `memfuse-mvcc`, `memfuse-vector`, `memfuse-rank`, `memfuse-adapt`, `memfuse-text`, `memfuse-graph`, `memfuse-crypto`, `memfuse-simd`, `memfuse-sys`, `memfuse-wire`, `memfuse-core`, `memfuse-calibration`.
     * *Invariante:* **Keine Aufwärts-Abhängigkeiten** und **keine `tokio`-Abhängigkeit** (ausgenommen `memfuse-core` als zentrales Trait-Definitions-Modul). Ring-0-Kerne operieren streng synchron.
   * **Ring 1 (Storage & Persistence):** `memfuse-store`, `memfuse-checkpoint`, `memfuse-kvcache`.
     * *Invariante:* Dürfen nur von Ring 0 abhängen; keine gegenseitigen Querverweise untereinander.
   * **Ring 2 (External Integrations & Execution Sandboxes):** `memfuse-sandbox`, `memfuse-infer-onnx`, `memfuse-infer-candle`, `memfuse-infer-ollama`.
     * *Invariante:* Nur Abhängigkeiten auf freigegebene Ring-0-Basismodule (`types`, `ports`, `crypto`, `core`, `simd`).
   * **Ring 3 (Engine, Reasoning & Cognition):** `memfuse-engine`, `memfuse-cognition`, `memfuse-privacy`, `memfuse-router`, `memfuse-agent`, `memfuse-db`.
     * *Invariante:* Dürfen auf Ring 0 und Ring 1 zugreifen, jedoch **nicht** auf konkrete Ring-2-Integrations-Crates.
   * **Ring 4 (Public Facade & Protocols):** `memfuse`, `memfuse-mcp`, `memfuse-py`.
     * *Invariante:* Konsumieren die darunterliegenden Schichten als öffentliche Schnittstelle.

3. **Gesteuerte Ausnahmen über `LAYER_ALLOWLIST`:**
   Temporäre Übergangs-Abhängigkeiten während Refactoring-Phasen dürfen ausschließlich über ein strukturiertes `AllowlistEntry`-Array in `tests/layering.rs` eingetragen werden. Jeder Eintrag erfordert:
   * `from_crate` und `to_crate`
   * `target_phase` (Ziel-Phase zur Beseitigung)
   * `reason` (explizite technische Begründung)

## Konsequenzen

* **CI-Pflicht:** Das Gate läuft verpflichtend in den CI-Pipelines (`context-gates.yml`, `rust-ci.yml`) und schlägt fehl (`exit status 1`), sobald eine unangemeldete Schichtgrenzen-Verletzung vorliegt.
* **Refactoring-Schutz:** Refactorings und Crate-Hinzufügungen müssen die Zuordnung in `Ring::for_crate()` pflegen und die Invarianten erfüllen.
* **Architektur-Transparenz:** Entwickler und Agenten erhalten bei Regelverstößen präzise Fehlermeldungen mit betroffenen Crates, Ringen und Regel-Nummern.
