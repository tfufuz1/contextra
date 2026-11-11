# Baseline-Bericht: contextra-store (Modul F: LSM-Kern)

Datum: 2026-11-11
Branch: jules-13220193073890637661-01dea5fe (Base: 922ee2694)

## 1. Ausgangszustand Build & Manifest
Befehl: `cargo check -p contextra-store --locked`
Ergebnis: FEHLGESCHLAGEN (Baseline-Defekt)
Fehler:
```
error: failed to load manifest for workspace member `/app/crates/contextra-store`
referenced by workspace at `/app/Cargo.toml`

Caused by:
  failed to parse manifest at `/app/crates/contextra-store/Cargo.toml`

Caused by:
  feature `workspace` includes `dep:contextra-types`, but `contextra-types` is not an optional dependency
  A non-optional dependency of the same name is defined; consider adding `optional = true` to its definition.
```
Ursache: In `crates/contextra-store/Cargo.toml` wurde das Feature `workspace = ["dep:contextra-types", "dep:contextra-ports"]` hinzugefügt, aber `contextra-types` und `contextra-ports` sind unter `[dependencies]` als reguläre (nicht-optionale) Dependencies deklariert.
Zudem ist `Cargo.lock` im Workspace nicht synchronisiert mit `--locked`.

## 2. Baseline-Defekt: Fehlende Module in `src/lib.rs`
Wie im Auftrag beschrieben (`BASELINE-VORBEDINGUNG`):
In `crates/contextra-store/src/lib.rs` fehlen:
```rust
pub mod budget;
pub mod stats;
```
Sobald der Manifest-Fehler isoliert wird, scheitert der Build an 13 Stellen (`unresolved import crate::stats`, `crate::budget`).
Mit Ergänzung von `pub mod budget; pub mod stats;` und Behebung des Features baut `contextra-store` mit Feature `workspace` erfolgreich.

## 3. Test-Status Baseline
Befehl: `cargo test -p contextra-store --test golden_corpus`
Ergebnis: Fehlgeschlagen mit:
- `open wal: Storage("WAL integrity key has unexpected length: 59 (expected 32)")`
(Datei `tests/golden/.wal_integrity_key` hat Länge 59 statt 32 im Checkout).
- Bounded-Flush- und Recovery-Tests zeigen Abhängigkeiten von Salt-Generierung und fsync-Hooks im Test-Umfeld.

## 4. Eigentums-Prüfung Baseline
Befehl: `/tmp/check_owner.sh`
Ergebnis: Leer (100% sauber, keine unberechtigten Dateien modifiziert).
