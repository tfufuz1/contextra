//! Fuzz-Target für BM25-Tokenisierung mit beliebigen UTF-8-Zeichenketten.
//!
//! # Ring-Zuordnung
//! Ring LAB / Fuzzing.
//!
//! # Invariante
//! Panik- und Endlosschleifen-Freiheit für beliebige Eingabestrings bei der Tokenisierung.

#![no_main]
use libfuzzer_sys::fuzz_target;
use contextra_text::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};

// ANCHOR[contextra-text:FUZZ-001]
// Fuzzing-Harnisch zur Prüfung der Robustheit von Standard- und deutschem Morphologie-Tokenizer.
fuzz_target!(|data: &str| {
    // Arbiträre UTF-8-Strings an die reale Tokenizer-Einstiegsfunktion.
    let german_tok = GermanMorphTokenizer::new();
    let _ = german_tok.tokenize(data);
    let _ = DefaultTokenizer.tokenize(data);
    let _ = contextra_text::tokenizer::tokenize(data);
});
