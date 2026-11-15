//! acta-graph: Segmentierter Graphindex (CSR) mit Lineage fuer die Abrufpipeline.
//!
//! Level L3 (Vorschlag, ADR-pflichtig) · Budget 4000 LOC · Erweiterung zu docs/SPEC.md Kap. 5.5 und 9.4
//!
//! Extraktionsarchiv: `archive/acta-graph/MANIFEST.md`.
#![forbid(unsafe_code)]

pub mod error;
