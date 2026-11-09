# Pending API Changes (P14)

Symbol | Betroffene Crates | Gewünschte Änderung
---|---|---
`SearchStrategy` | `crates/contextra/src/lib.rs`, `contextra-engine` | `pub use contextra_db::SearchStrategy` (oder direkt aus `contextra_engine::SearchStrategy`) in der Fassade `contextra` re-exportieren, damit Benchmarks und Konsumenten direkt gegen die Fassade `contextra` statt `contextra-db` bauen können.
`SignalWeights` | `crates/contextra/src/lib.rs`, `contextra-engine` | `pub use contextra_engine::SignalWeights` in der Fassade `contextra` re-exportieren für feinjustierte Signal-Gewichtung.
`HybridQueryBuilder` | `crates/contextra/src/lib.rs`, `contextra-engine` | In `contextra::lib.rs` re-exportieren für flüssige Suchstrategie-Konfiguration.
