# FINDINGS — contextra-store Modul A (WAL)

Beobachtungen und Defekte außerhalb des zugewiesenen Eigentums von Agent A:

## Finding #1: Ungültiges `workspace`-Feature in `crates/contextra-store/Cargo.toml`
- **Datei:** `crates/contextra-store/Cargo.toml:17`
- **Beobachtung:**
  ```toml
  [features]
  workspace = ["dep:contextra-types", "dep:contextra-ports"]
  ```
  `contextra-types` und `contextra-ports` sind unter `[dependencies]` nicht mit `optional = true` deklariert.
  Cargo meldet Fehler:
  `feature 'workspace' includes 'dep:contextra-types', but 'contextra-types' is not an optional dependency`
- **Reproduktion:** `cargo check -p contextra-store`
- **Vorschlag (Übergabe an Agent G):**
  In `crates/contextra-store/Cargo.toml`:
  Entweder `optional = true` bei beiden Abhängigkeiten ergänzen:
  ```toml
  contextra-types = { workspace = true, optional = true }
  contextra-ports = { workspace = true, optional = true }
  ```
  oder das ungenutzte `workspace`-Feature bereinigen:
  ```toml
  workspace = []
  ```

## Finding #2: Fehlende Modul-Deklarationen in `crates/contextra-store/src/lib.rs`
- **Datei:** `crates/contextra-store/src/lib.rs`
- **Beobachtung:** `mod budget;` und `mod stats;` fehlen in `src/lib.rs`, obwohl `crate::budget::ResourceTracker` und `crate::stats::StorageStats` referenziert werden.
- **Reproduktion:** Nach Auflösung des Manifest-Fehlers scheitert der Build an fehlenden Modulen `budget` und `stats`.
- **Vorschlag:** Laut Baseline-Vorbedingung in `lib.rs` nach `pub use error::{StoreError, StoreResult};` einfügen:
  ```rust
  pub mod budget;
  pub mod stats;
  ```
