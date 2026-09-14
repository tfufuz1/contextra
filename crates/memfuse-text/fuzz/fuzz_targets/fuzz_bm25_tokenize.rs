#![no_main]
use libfuzzer_sys::fuzz_target;
use memfuse_text::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};

fuzz_target!(|data: &str| {
    // Arbiträre UTF-8-Strings an die reale Tokenizer-Einstiegsfunktion.
    let german_tok = GermanMorphTokenizer::new();
    let _ = german_tok.tokenize(data);
    let _ = DefaultTokenizer.tokenize(data);
    let _ = memfuse_text::tokenizer::tokenize(data);
});
