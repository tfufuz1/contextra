# contextra-sys — Umsetzungsplan (Baukasten-Spezifikation v3)

> Abgeleitet aus docs/SPEC.md v3 §3.1, §6.2 (S-03R, S-22R) und §6.4.
> Status: **contextra-sys ist als autarker E0-Klasse-A-Baustein vollständig entkoppelt**.

## 1. Einordnung im Baukastensystem (SPEC v3 §1.1, §3.1)

| Merkmal | Status / Festlegung | Nachweis |
|---|---|---|
| Ebene | **E0** (Kern-Closure leer) | `decouple_inspector.py -c contextra-sys` (0 interne Abhängigkeiten) |
| Klasse | **A** (Wiederverwendbarer Baustein) | Score 100/100, autarker System-FFI-Layer |
| Ring | **0** (Hardware- und OS-nahe Unsafe-Insel) | Kapselt FFI (`libc`, `windows-sys`, `memmap2`) |
| Unsafe-Status | Unsafe-Insel (Insel 3 von 3: `simd`, `wire`, `sys`) | Vollständig abgesichert mit `// SAFETY:`-Audits |
| Zero-Panic (R8) | 0 Panics/Unwraps/Expects in Produktionscode | `pattern_diagnostic.py -c contextra-sys --prod-only` (0 Befunde) |
| Fehlertyp (R3) | `SysError` (Alias für `contextra_sys::Error`) | `crates/contextra-sys/src/error.rs` |
| Baukasten-Export | Module `mem`, `fsmeta`, `map` sowie Direkt-Exporte | Ergonomische Schnittstellen für Konsumenten |

## 2. Schritte und Baukasten-Transformation

| Schritt | Inhalt | Voraussetzung | Nachweis | Status |
|---|---|---|---|---|
| S-03R | Doku/Metadaten: `sys/lib.rs:19` korrigieren (unbelegte durable-fs Behauptung entfernt); Baukasten-Rolle E0 festschreiben | — | `lib.rs` korrigiert; Ist-Zustand dokumentiert | ☑ abgeschlossen |
| S-22 | `vault` aus `db`, baut auf `sys::vault` auf | S-01 | `crates/contextra-vault` existiert und nutzt `sys::vault`; `sys::vault` autark | ☑ abgeschlossen |
| DOC-ANCHORS | Einführung des systematischen Comment-Anchors-Schemas | — | Vollständige, feingranulare Verankerung und Inline-Doku aller Dateien | ☑ abgeschlossen |
| VERIFY-ISO | Isolationsnachweis und Testsuite in autarkem Workspace | — | `test_isolated_crate.py`: alle 41 Tests grün, 0 Panics in Prod | ☑ abgeschlossen |

## 3. Definition of Done (DoD)

- [x] Autarkie: 0 interne Abhängigkeiten, vollständige E0-Closure.
- [x] Zero-Panic in Produktion (`unwrap`, `expect`, `panic!` ausschließlich in Tests).
- [x] Keine verbotenen Garantiewörter gemäß docs/SPEC.md §6.4 (`FORBIDDEN_WORDING` = 0).
- [x] Durchgängige `// SAFETY:`-Begründungen für jeden `unsafe`-Block.
- [x] Konsistente Comment-Anchor-Syntax (`ANCHOR`, `SECTION`, `LINK`, `NOTE`, `TODO`, `REVIEW`).
- [x] Alle Tests und Benchmarks lauffähig und grün.
