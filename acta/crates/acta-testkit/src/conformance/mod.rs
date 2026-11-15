//! Konformitätsprüfsuiten für Acta-Port-Traits (Spec 16.1, 16.2).
//!
//! Jede Suite liefert ein typisiertes `ConformanceResult` und panict nicht selbst.
//! Erst der Testläufer macht daraus mit `assert_conformance!` einen Fehlschlag.
//!
//! Umgesetzt: `CryptoRng` (Z10). Noch nicht umgesetzt: Suiten für `Vfs`, Textindex,
//! Vektorindex und Speicher; sie folgen mit ihren Crates. Für `Clock` gibt es bewusst
//! keine Suite: Der Port garantiert nach A-CLOCK nur `now_ms`, keine Monotonie.

pub mod crypto_rng;
pub mod failure;
pub mod macros;

pub use crypto_rng::run_crypto_rng_suite;
pub use failure::{ConformanceFailure, ConformanceResult};
