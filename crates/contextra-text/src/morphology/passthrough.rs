use super::compound_splitter::MorphologicalTokenizer;

/// Passthrough morphological tokenizer for languages without compound word splitting.
///
/// Serves as a fallback [`MorphologicalTokenizer`] strategy for non-compound languages
/// (such as English, French, Spanish, or Japanese) where morphological decomposition
/// is not performed. Returns the input token unchanged as a single element slice.
pub struct PassthroughTokenizer {
    lang: String,
}

impl PassthroughTokenizer {
    /// Creates a passthrough tokenizer for the given language ISO code.
    pub fn new(lang: impl Into<String>) -> Self {
        Self { lang: lang.into() }
    }
}

impl MorphologicalTokenizer for PassthroughTokenizer {
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str> {
        vec![token]
    }

    fn language(&self) -> &str {
        &self.lang
    }
}
