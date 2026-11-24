//! Rauchtest-Fuzz-Target für Tokenizer und reguläre Schutzmuster-Kompilierung.
//!
//! # Ring-Zuordnung
//! Ring LAB / Fuzzing.
//!
//! # Invariante
//! Garantierte Panik-Freiheit (Fail-Closed) bei Regex-Kompilierung und Tokenisierung.

#![no_main]

use contextra_text::tokenizer::{
    compile_protected_regex, get_protected_regex, DefaultTokenizer, GermanMorphTokenizer,
    Tokenizer,
};
use libfuzzer_sys::fuzz_target;

// ANCHOR[contextra-text:FUZZ-002]
// Fuzzing-Harnisch zur Prüfung der Regex-Validierung und Tokenizer-Stabilität.
fuzz_target!(|data: &str| {
    // 1. Default Tokenizer execution with arbitrary string
    let default_tok = DefaultTokenizer;
    let _ = default_tok.tokenize(data);

    // 2. German Morphological Tokenizer execution
    let german_tok = GermanMorphTokenizer::new();
    let _ = german_tok.tokenize(data);

    // 3. Global tokenize function
    let _ = contextra_text::tokenizer::tokenize(data);

    // 4. Test protected regex compilation with arbitrary pattern string (must return Ok or Err, never panic)
    let _ = compile_protected_regex(data);

    // 5. Test default protected regex accessor
    let _ = get_protected_regex();
});
