//! acta-ingest: Chunking, Normalisierung, Ressourcenlimits, Worker-Protokoll (ohne Parser-Abhaengigkeiten).
//!
//! Level L1 · Budget 3000 LOC · Spezifikation docs/SPEC.md Kap. 9.1
//!
//! Extraktionsarchiv: `archive/acta-ingest/MANIFEST.md` (falls vorhanden).
#![forbid(unsafe_code)]

pub mod error;
