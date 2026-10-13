// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Unabhaengiges Canary-Orakel und Rohsweep fuer die Loeschgarantie (Spezifikation T1).
// INVARIANTEN: No unsafe code allowed (#![forbid(unsafe_code)]).

//! # Deletion Oracle (Loeschorakel)
//!
//! Stellt das unabhaengige Test-Orakel und die Rohsweep-Infrastruktur fuer die
//! Verifikation der Loeschgarantie bereit.
//!
//! **Hinweis:** Der Rohsweep ist im verschluesselten Modus bedeutungslos (Spezifikation T1);
//! der RAM-Sweep ist NICHT Teil dieses Pakets.

pub mod marker;
pub mod rawsweep;
pub mod report;

pub use marker::{OrcFieldMarker, OrcMarkerSet};
pub use rawsweep::raw_sweep;
pub use report::{OrcHit, OrcSweepReport};
