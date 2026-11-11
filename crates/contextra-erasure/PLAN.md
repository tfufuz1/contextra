# contextra-erasure — Umsetzungsplan

> Abgeleitet aus docs/SPEC.md v3 sowie der Löschketten-Implementierungsspezifikation (`docs/erasure/Loeschkette_Implementierungsspezifikation.md`). Status-Stand: **Vollständig entkoppelt und als autarker Klasse-A Baustein verifiziert**.

## 1. Übersicht & Status

| Schritt | Inhalt | Voraussetzung | Nachweis | Status |
|---|---|---|---|---|
| S-12 | Feature-Hygiene: Features `signing`, `vault`, `async` definiert und entkoppelt | S-01 | `cargo check --no-default-features` & je Feature grün | ☑ abgeschlossen |
| S-24 | Feature-Gating & Datei-Zuordnung | S-12 | `Cargo.toml` strukturiert, Default-Features aktiv | ☑ abgeschlossen |
| S-50 | Port-Entkopplung: Autarke Definitionen in `types_local.rs` | S-24 | 0 Abhängigkeiten zu `contextra-ports` | ☑ abgeschlossen |
| S-51 | Typen-Entkopplung: Autarke Domänenmodelle in `types_local.rs` | S-50 | 0 Abhängigkeiten zu `contextra-types` | ☑ abgeschlossen |
| S-70 | Bereinigung und Code-Kommentierung | S-51 | Systematisches Comment-Anchor System, Null-Panic, Null-Unsafe | ☑ abgeschlossen |

## 2. Architekturkonsistenz

- **Klasse:** Baustein Klasse A (B1–B12 konform).
- **Abhängigkeiten:** 0 interne Abhängigkeiten zu `contextra-*`.
- **Lints:** `#![forbid(unsafe_code)]`, Zero-Panic in non-test Code (`#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]`).
- **Dateistruktur:** Keine `mod.rs` (D-90), alle Submodule folgen dem `<modul>.rs` + `<modul>/` Schema.

## 3. Definition of Done

- Autarkie-Score von ≥ 95/100 (`tools/decouple_inspector.py`).
- Standalone-Kompilierbarkeit und Unabhängigkeit im Workspace.
- Vollständige Einhaltung von SPEC §6.4 (Wording-Neutralität).
- Durchgängiges, verständliches Kommentierungssystem nach Comment-Anchors Standard für Junior- und Senior-Entwickler.

## 4. Löschketten-Arbeitspakete (WP-0 bis WP-12, WP-K)

Die detaillierte Umsetzung der Löschkette erfolgt gemäß `docs/erasure/Loeschkette_Implementierungsspezifikation.md`:
- **WP-0:** Baubarkeit und Baseline (P0: Status erfasst in `docs/erasure/BASELINE.md` und `docs/erasure/handoff/P0.md`).
- **WP-1:** Fehlertaxonomie des Purge (lokal in `error.rs`).
- **WP-2:** Einheitliche `PurgePhase` (`purge/`).
- **WP-3:** Dauerhafter Scope und Single-Flight (INV-ERA-3).
- **WP-4:** Resume führt den Purge aus (INV-ERA-2).
- **WP-5 / WP-5a:** `purge_keys` und RAM-Swap-Absicherung (`vault/`).
- **WP-6:** Fence-Protokoll (INV-ERA-5).
- **WP-7:** WAL-Entsorgung und Scope-Purge (INV-ERA-6).
- **WP-8:** Persistenter Ledger (INV-ERA-4).
- **WP-9:** Zeugen und Beweis (INV-ERA-7).
- **WP-10:** Ein Koordinator, ein Pfad (INV-ERA-1).
- **WP-11:** Restore und Anti-Resurrection (INV-NO-RESURRECTION-1).
- **WP-12:** Restspuren im Index (`measure/`).
- **WP-K:** KEK/DEK Schlüsselhierarchie (`kv_shredding`, `vault/keyring`).
