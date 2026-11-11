# Findings: contextra-store Modul F (LSM-Kern)

Dieses Dokument erfasst Beobachtungen, vorbestehende Mängel und Abweichungen, die während der Bearbeitung von Modul F festgestellt wurden.

## 1. Baseline-Defekt: Workspace-Feature & Dependencies in Cargo.toml
- **Datei:** `crates/contextra-store/Cargo.toml`
- **Beobachtung:**
  `workspace = ["dep:contextra-types", "dep:contextra-ports"]`
  definiert `contextra-types` und `contextra-ports` als optionale Features, während diese unter `[dependencies]` als reguläre Dependencies (`contextra-types = { workspace = true }`) deklariert sind.
- **Folge:** `cargo check -p contextra-store --locked` schlägt workspace-weit fehl mit exit code 101.
- **Lösungsvorschlag für Hülle (Agent G):** Entweder `contextra-types` und `contextra-ports` als `optional = true` deklarieren oder `workspace = []`.

## 2. Baseline-Defekt: Fehlende Module in `src/lib.rs`
- **Datei:** `crates/contextra-store/src/lib.rs`
- **Beobachtung:** `pub mod budget;` und `pub mod stats;` fehlen in `src/lib.rs`, obwohl `crate::budget` und `crate::stats` in mehreren Subsystemen referenziert werden.
- **Folge:** Unresolved import Fehler beim Kompilieren von `contextra-store`.
- **Lösungsvorschlag:** Die zwei Zeilen nach `pub use error::{StoreError, StoreResult};` nachtragen.

## 3. Sperrenhierarchie Doku-Abweichung
- **Datei:** `crates/contextra-store/src/lsm/mod.rs` vs `src/lsm/lock_order.rs`
- **Beobachtung:**
  In `mod.rs` wurde historisch eine 5-stufige Hierarchie beschrieben, während `lock_order.rs` (von Agent E) und `SPEC §3.5` ein 10-stufiges Rangregister (Ränge 100 bis 950) definieren.
- **Korrektur:** Die Moduldokumentation in `mod.rs` wurde in WP-F.5 präzise auf die Ränge aus `lock_order.rs` harmonisiert.

## 4. Test-Dateien mit ungültigem Golden Key
- **Datei:** `crates/contextra-store/tests/golden/.wal_integrity_key`
- **Beobachtung:** Der Key hat eine Byte-Länge von 59 anstelle der erwarteten 32 Bytes, was `test_golden_backward_compatibility_read` fehlschlagen lässt.
- **Zuständigkeit:** Agent G / Golden Corpus Maintainer.
