// FILE-CONTEXT
// ZWECK: Platzhalter-Modul für sichere Parsing-Helfer (Archiv, XML, OPC) (WP M4 / Platzhalter für P00B).
// INVARIANTEN: Zero-Panic, bedingte Kompilation.
// STAND: TS:2026-10-31T00:00:00Z

#[cfg(feature = "zip-container")]
pub mod archive;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(all(feature = "zip-container", feature = "xml"))]
pub mod opc;
