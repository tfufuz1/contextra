# Symbol Exists Gate (`symbol-exists`)

## Zweck
Löst Pfade wie `contextra_store::wal::WalWriter` auf und verifiziert deren Existenz im Workspace.

## Funktionsweise
- Parst den Symbolpfad (`crate::module::Item`).
- Mappt den Crate-Namen (Bindestrich <-> Unterstrich) auf den Verzeichnispfad über `capabilities.toml`.
- Trassiert die Modulhierarchie (`lib.rs`, `mod.rs`, `<module>.rs`, `#[path]`) und löst `pub use` Re-Exports bis Tiefe 3 auf.
- Erkennt Structs, Enums, Traits, Funktionen, Type-Aliase, Konstanten, Statics und Methoden in `impl`-Blöcken.
- Bei Nicht-Finden gibt das Gate Exit-Code 1 sowie die bis zu 5 Levenshtein-ähnlichsten Symbole desselben Crates aus.
