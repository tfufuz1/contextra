//! Adapter bridges connecting Class A / external building blocks to `contextra-ports`.
//!
//! Per SPEC E-05 (Adapter-Übernahme), bridges from Class A components are hosted here
//! so the core building blocks remain completely decoupled from port traits.

pub mod text;
pub mod privacy;
pub mod ollama;
