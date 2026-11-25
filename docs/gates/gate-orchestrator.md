# Gate Orchestrator Phase Levels (`gate-check`)

Das `gate-check`-Subkommando in `xtask` dient als zentraler Orchestrator zur automatisierten Prüfung der Phaseneintrittsbedingungen gemäß `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md` Teil A.3 (Phasen und Gates).

Es ermöglicht Entwicklern und CI-Pipelines die Ausführung von Phasen-Gates über ein einziges Kommando (`cargo run -p xtask -- gate-check --level <N>`).

---

## Levels & Befehlsfolgen

### Level 0 (Gate 0 - Je Crate)
- **Zweck**: Verifiziert die Code-Qualität, Formatierung und Lauffähigkeit der Tests für ein einzelnes Crate.
- **Voraussetzung**: Parameter `--crate <name>` ist obligatorisch.
- **Befehlsfolge**:
  1. `cargo fmt --check -p <name>`
  2. `cargo clippy -p <name> --all-targets --locked -- -D warnings`
  3. `cargo test -p <name> --locked`
- **Abbruchverhalten**: Bricht beim ersten fehlschlagenden Schritt sofort ab und gibt den genauen Befehl sowie Stderr/Stdout aus.
- **Implementierungsstatus**: **Automatisiert**. (Hinweis: P7 Marker-Generator wird als TODO im Bericht referenziert).

### Level 1 (Gate 1 - Fundament stabil)
- **Zweck**: Verifiziert die Stabilität des gesamten Fundaments (Ring 0 & Ring 1 Crates, Loom-Concurrency-Tests und Architecture/Ring-Layering).
- **Voraussetzung**: Keine gesonderten Argumente erforderlich (`--crate` wird ignoriert).
- **Befehlsfolge**:
  1. Gate 0 sequentiell für **alle** Ring 0/1 Crates ausführen:
     - Ring 0: `contextra-types`, `contextra-ports`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-adapt`, `contextra-simd`
     - Ring 1: `contextra-store`, `contextra-mvcc`, `contextra-checkpoint`, `contextra-kvcache`, `contextra-crypto`, `contextra-privacy`, `contextra-sys`, `contextra-wire`
  2. `cargo test --workspace --features loom -- --test-threads=1` mit `RUSTFLAGS="--cfg loom"` (Loom-Tests)
  3. `cargo test --manifest-path xtask/Cargo.toml --test layering` (Ring-Layering-Test)
- **Implementierungsstatus**: **Automatisiert**.

### Level 2 (Gate 2 - Phase 1 Kriterien)
- **Zweck**: Abnahme der Kriterien aus Phase 1 vor Beginn der Arbeiten an Phase 2.
- **Implementierungsstatus**: **Platzhalter**.
- **Verhalten**: Meldet einen klaren Fehler:
  `"Gate 2 ist spezifiziert, aber die zugehörigen Abnahmekriterien aus Phase 1 sind in dieser xtask-Version noch nicht als automatisierte Prüfung hinterlegt — siehe docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md Teil A.3"` (Exit-Code != 0).

### Level 3 (Gate 3 - Phase 2 Kriterien)
- **Zweck**: Abnahme der Kriterien aus Phase 2 vor Beginn der Arbeiten an Phase 3.
- **Implementierungsstatus**: **Platzhalter**.
- **Verhalten**: Meldet einen klaren Fehler:
  `"Gate 3 ist spezifiziert, aber die zugehörigen Abnahmekriterien aus Phase 2 sind in dieser xtask-Version noch nicht als automatisierte Prüfung hinterlegt — siehe docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md Teil A.3"` (Exit-Code != 0).

### Level 4 (Gate 4 - Phase 3 Kriterien)
- **Zweck**: Abnahme der Kriterien aus Phase 3 vor Beginn der Arbeiten an Phase 4.
- **Implementierungsstatus**: **Platzhalter**.
- **Verhalten**: Meldet einen klaren Fehler:
  `"Gate 4 ist spezifiziert, aber die zugehörigen Abnahmekriterien aus Phase 3 sind in dieser xtask-Version noch nicht als automatisierte Prüfung hinterlegt — siehe docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md Teil A.3"` (Exit-Code != 0).

---

## Verwendung

```bash
# Gate 0 für ein spezifisches Crate prüfen:
cargo run -p xtask -- gate-check --level 0 --crate contextra-types

# Gate 1 für das gesamte Fundament prüfen:
cargo run -p xtask -- gate-check --level 1

# Versuch, nicht-implementierte Gates auszuführen (schlägt mit Exit-Code 1 fehl):
cargo run -p xtask -- gate-check --level 2
```

---

## Status (Stand 2026-09-26)

Level 2–4 sind funktionale Platzhalter ohne automatisierte Kriterien; für produktive Arbeitspakete gelten stattdessen paket-lokale Akzeptanzkriterien (`cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `xtask jules-preflight`), bis ein gesondertes Nachfolgepaket (Referenz: "AP-001" aus der Arbeitspaket-Planung) Level 2–4 entweder real spezifiziert oder formal als entfallen markiert.
