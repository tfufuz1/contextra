# Handoff STORE-6: contextra-store — StoreError, compat/, Durability-Nachweise

## 1. Übersicht & Zielerreichung
- **Auftrag:** STORE-6 (`contextra-store`: `StoreError`, `compat/`, Durability-Nachweise). Commit-Kennung: ST6.
- **Branch:** `jules/store-6-harden`
- **Ergebnis je Arbeitspaket:**
  - **ST6.0: Baseline, Kopplungs- und Feature-Befund**: Grün. Baseline unter `docs/decoupling/baseline/store/` erfasst. Vorbestehende Compilerfehler in eigenem Commit `ST6.0: Baseline-Reparatur` behoben.
  - **ST6.1: Golden-Korpus**: Grün. Reference-Dateien für Manifest v1 (`MFMN`), SSTable v4 (`MFSX`), SSTable legacy (`MFST`), WAL und Rollback-Intent (`MFRLBK\0\0`) unter `tests/golden/` mit Prüfsummen `MANIFEST.sha256` und Generator `regen.sh`.
  - **ST6.2: StoreError und Fehlerabbildung**: Grün. Lokaler `StoreError` (`thiserror`, `#[non_exhaustive]`) implementiert. Bitgleiche Fehlerkonvertierung `From<StoreError> for ContextraError` in `compat/error.rs` verifiziert. Umstellung interner Module (`wal/encode`, `sstable/block_search`, `manifest`).
  - **ST6.3: Brücke compat/ und Entkopplung**: Grün. `engine/` (inkl. `memory_only`) und `scanner/purge.rs` nach `src/compat/` verschoben. Alte Pfade bieten quellenkompatible `pub use`-Re-Exports. Teil B entfallen gemäß Vorgabe.
  - **ST6.4: kv/ archivieren**: Grün. `crates/contextra-store/src/kv` nach `archive/contextra-store-kv` verschoben (keine Verwender im Produkt).
  - **ST6.5: Hygiene**: Grün. Feature-Hygiene geprüft, Alias `sieve-cache = []` für CI-Matrix hinzugefügt.
  - **ST6.6: Härtung und Nachweise**: Grün. Durability- und Truncation-Tests verifizieren Fail-Closed-Verhalten ohne Panic. Loom-Tests (`loom_group_commit`, `loom_group_commit_handoff`) unter `--cfg loom` vollständig grün.
  - **ST6.7: ID-Breite im Manifest**: Zurückgestellt (Folgeschritt Integration, gemäß UNABHÄNGIGKEITS-Klausel).
  - **ST6.8: Dokumentation**: Grün. `AGENTS.md` und Handoff aktualisiert.

## 2. Messwerte Vorher / Nachher
| Metrik | Vorher (ST6.0 Baseline) | Nachher (ST6.8) |
|---|---|---|
| LOC (contextra-store) | 18.015 LOC in 83 Dateien | 18.420 LOC in 86 Dateien |
| Modularitäts-Score (`decouple_inspector.py`) | 33 / 100 | 45 / 100 |
| Direkte Kernel-Imports | 25 | 18 |
| Nullprüfzahlen `contextra_(types|ports)` im Kern | 150 Treffer (Produktion) | 118 Treffer (Produktion, Rest in compat/ und Kern-Bridge) |
| Golden-Dateien | 0 | 5 (`MANIFEST.sha256` verifiziert) |
| Loom Concurrency Tests | Vorher wegen Build blockiert | 2/2 grün |

## 3. API-Freeze & Kompatibilitätstabelle
Alle öffentlich genutzten APIs von `contextra-store` bleiben 100% quellkompatibel:
| Typ / Funktion | Vorheriger Pfad | Neuer Pfad / Status | Konsumenten-Nachweis |
|---|---|---|---|
| `StorageEngine` Impl | `src/engine/mod.rs` | `src/compat/engine/mod.rs`, re-exportiert an `src/engine.rs` | `contextra-engine`, `contextra-db` kompilieren unverändert |
| `MemoryOnlyStorage` | `src/engine/memory_only.rs` | `src/compat/engine/memory_only.rs`, re-exportiert | `contextra-db` |
| `LsmPurger` | `src/scanner/purge.rs` | `src/compat/scanner/purge.rs`, re-exportiert | `contextra-engine` |
| `StoreError` | Neu | `src/error.rs`, re-exportiert in `src/lib.rs` | Additiv, implementiert `Into<ContextraError>` |

## 4. Abweichungen & Begründungen
- Gemäß verbindlicher Auftragsanweisung UNABHÄNGIGKEIT:
  - ST6.3 Teil B (Umstellung auf contextra-ids/env und Optionalität von types/ports hinter workspace) entfällt in diesem Paket, da IDs/Env nicht vorausgesetzt werden und types/ports weiterhin normale Workspace-Abhängigkeiten sind.
  - ST6.7 (ID-Breite im Manifest) entfällt in diesem Paket und ist als Folgeschritt für das Integrationspaket vorgesehen.

## 5. Koordinationsanforderungen (für Integrationspaket Z)
- Keine Änderungen an geschützten Repositoriumspfaden vorgenommen.
- Sobald das Integrationspaket `workspace` als Feature workspace-weit ausrollt, kann `default = ["workspace"]` auf `default = []` umgestellt werden.

## 6. Prüfprotokoll
- `cargo check -p contextra-store --all-targets --locked` -> GRÜN
- `cargo test -p contextra-store --test golden_corpus` -> GRÜN
- `cargo test -p contextra-store --lib compat::error::tests` -> GRÜN
- `RUSTFLAGS="--cfg loom" cargo test -p contextra-store --test loom_group_commit --test loom_group_commit_handoff --release` -> GRÜN
- `cargo test -p contextra-store --test corrupted_files_fail_closed` -> GRÜN
