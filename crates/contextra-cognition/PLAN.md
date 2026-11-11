# contextra-cognition — Umsetzungsplan

> Abgeleitet aus `docs/SPEC.md` v3 §6 (Umsetzungsreihenfolge). Status-Stand: 2026-11-11.

## 1. Einordnung im Gesamtsystem (docs/SPEC.md v3 §6.2) [S]

`contextra-cognition` ist der Ebene E2 (Integration) zugeordnet. Seine Invertierung (Entkopplung von `contextra-engine`, Delegation der konkreten Bindung an `contextra-db` hinter Feature `cognition`) ist im Hauptphasenplan unter Schritt **S-63** verankert.

| Schritt | Inhalt | Voraussetzung | Nachweis | Status |
|---|---|---|---|---|
| **S-03R** | Doku/Metadaten, Bereinigung veralteter Vorgaben und Vereinheitlichung der Spezifikationen | — | `AGENTS.md`, `SPEC.md`, `PLAN.md` aktuell | In Arbeit |
| **S-63** | `cognition` invertieren (Brücke in `db`, Ports `DocumentSource`/`ConsolidationSink` nutzen) | S-37, S-50 | `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine` | Vorgemerkt für S-63 |

## 2. Aktuelle Modulentkopplung und Code-Dokumentation

- **Modul-Dokumentation:** Vollständige Rustdoc-Abdeckung (`//!` und `///`) ohne Panics oder unzureichende Erklärungen.
- **Comment-Anchors Schema:** Durchgängige Etablierung des einheitlichen Kommentierungsschemas (`// SECTION`, `// !SECTION`, `// NOTE:`, `// TODO:`, `// FIXME:`, `// STUB:`, `// REVIEW:`, `// ANCHOR`, `// FILE-CONTEXT`).
- **Verifikation:** Zero-Panic (`zero_panic_analyzer.py`), Unsafe-Verbot (`unsafe_boundary_auditor.py`), Determinismus (`determinism_guard.py`).
