//! Port-Brücke für BM25-Scoring.
//!
//! Die Implementierung liegt in `contextra-text` (`compat::scorer`). Diese Datei hält
//! nur den Re-Export, damit bestehende Engine-Pfade gültig bleiben. Eine zweite,
//! abweichende Implementierung existiert bewusst nicht mehr (Ein-Besitzer-Regel).

#[allow(deprecated)]
pub use contextra_text::compat::scorer::Bm25Scorer;
