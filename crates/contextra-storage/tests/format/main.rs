#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::collapsible_if,
    clippy::unnecessary_map_or
)]

//! # Format Tests
//!
//! Verifiziert SPEC §10.2 (FORMAT, Goldens aus tools/reference), §10.5.1 (Bloom),
//! F-15 Downgrade-Schutz, Feature-Bits und Primitiv-Vektoren für XXH64 und CRC-32C.

mod bloom_layout_test;
mod f15_test;
mod features_test;
mod format_codec_test;
mod golden_test;
mod primitive_vectors;
