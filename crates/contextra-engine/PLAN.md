# contextra-engine — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §2.1 und `docs/crates/contextra-engine.md`. **Ausgangslage:** Dieses Crate ist ausschließlich als Deprecated-Fassade vorgesehen.

1. Ersetze `src/lib.rs` durch `#![forbid(unsafe_code)]`, dokumentiertes `#[deprecated]` und genau `pub use contextra_db::*;`.
2. Reduziere `Cargo.toml` auf die einzige normale Abhängigkeit `contextra-db`, `default = []` und Workspace-Lints; entferne Features, Tests und Dokumentation für eigene Logik.
3. Ergänze einen Compile-/API-Test, der das Re-Export-Verhalten prüft, sowie einen Dependency-Gate-Test gegen weitere Crates.

**Abnahme:** Bytekleine Fassade, `cargo tree -p contextra-engine` zeigt nur DB als internen Normal-Dependency, Re-export Compile-Test grün.
