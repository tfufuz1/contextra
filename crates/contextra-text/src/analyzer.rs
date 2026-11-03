//! Analyzer-Implementierung nach SPEC D-81 (Analyzer de/1 und und/1).

use contextra_crypto::hierarchy::IdxKey;
use contextra_crypto::labels::TERM_TAG;
use contextra_crypto::tag::{tag, TagPreimage};
use unicode_segmentation::UnicodeSegmentation;

use crate::morphology::{is_german_stopword, normalize_umlauts};
use crate::morphology::{GermanCompoundSplitter, MorphologicalTokenizer};

/// Trait für Text-Analyzer.
pub trait Analyzer: Send + Sync {
    fn id(&self) -> u8;
    fn version(&self) -> u16;
    fn lang(&self) -> [u8; 3];
    fn analyze(&self, text: &str) -> Vec<String>;

    fn term_tag(&self, key: &IdxKey, term: &str) -> [u8; 16] {
        let mut pre = TagPreimage::new();
        pre.push(TERM_TAG);
        let id_bytes = [self.id()];
        pre.push(&id_bytes);
        let lang = self.lang();
        pre.push(&lang);
        pre.push(term.as_bytes());
        tag(key, &pre)
    }
}

pub struct GermanAnalyzer {
    splitter: GermanCompoundSplitter,
}

impl GermanAnalyzer {
    pub fn new() -> Self {
        Self {
            splitter: GermanCompoundSplitter::new_with_all_domains(),
        }
    }
}

impl Default for GermanAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for GermanAnalyzer {
    fn id(&self) -> u8 {
        1
    }
    fn version(&self) -> u16 {
        1
    }
    fn lang(&self) -> [u8; 3] {
        *b"de\0"
    }

    fn analyze(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        for word in text.unicode_words() {
            let folded = word.to_lowercase().replace('ß', "ss");
            let norm = normalize_umlauts(&folded);

            if is_german_stopword(&folded) || is_german_stopword(&norm) {
                continue;
            }

            let parts = self.splitter.decompose(&folded);
            if parts.len() > 1 {
                tokens.push(folded.clone());
                for part in parts {
                    let part_norm = part.to_lowercase().replace('ß', "ss");
                    if !is_german_stopword(&part_norm) && part_norm.len() >= 2 {
                        let stemmed = simple_german_stem(&part_norm);
                        tokens.push(stemmed);
                    }
                }
            } else {
                let stemmed = simple_german_stem(&folded);
                tokens.push(stemmed);
            }
        }
        tokens
    }
}

pub struct UndeterminedAnalyzer;

impl Analyzer for UndeterminedAnalyzer {
    fn id(&self) -> u8 {
        0
    }
    fn version(&self) -> u16 {
        1
    }
    fn lang(&self) -> [u8; 3] {
        *b"und"
    }

    fn analyze(&self, text: &str) -> Vec<String> {
        text.unicode_words()
            .map(|w| w.to_lowercase())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

fn simple_german_stem(word: &str) -> String {
    let mut w = word.to_string();
    const SUFFIXES: &[&str] = &["ern", "est", "en", "er", "es", "em", "e", "s"];
    for &suf in SUFFIXES {
        if w.ends_with(suf) && w.len() > suf.len() + 2 {
            let new_len = w.len() - suf.len();
            if w.is_char_boundary(new_len) {
                w.truncate(new_len);
                break;
            }
        }
    }
    w
}

pub fn get_analyzer(lang: &str, version: u16) -> Option<Box<dyn Analyzer>> {
    match (lang, version) {
        ("de", 1) => Some(Box::new(GermanAnalyzer::new())),
        ("und", 1) => Some(Box::new(UndeterminedAnalyzer)),
        _ => None,
    }
}
