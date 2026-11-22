// FILE-CONTEXT
// ZWECK: Satzsegmentierung für übergroße Absätze mit Abkürzungsbehandlung (de/en) (WP M3 / SPEC C-11.7).
// INVARIANTEN: Zero-Panic, UTF-8-sicher, keine Satzteilung bei bekannten Abkürzungen.
// STAND: TS:2026-10-31T00:00:00Z

use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// Bekannte deutsche und englische Abkürzungen (in Kleinbuchstaben).
const ABBREVIATIONS: &[&str] = &[
    "z. b.", "z.b.", "dr.", "bzw.", "e. g.", "e.g.", "i. e.", "i.e.",
    "vs.", "mr.", "mrs.", "ms.", "prof.", "str.", "etc.", "ca.", "vgl.",
    "st.", "s.", "u. a.", "u.a.", "d. h.", "d.h.", "incl.", "excl.",
    "min.", "max.", "approx.", "nr.", "no.", "art.", "abs.", "sog.",
];

/// Repräsentiert einen Satz innerhalb eines Textes mit dessen Byte-Span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceSpan {
    pub text: String,
    pub span: Range<usize>,
}

/// Zerlegt einen Text unter Berücksichtigung von Abkürzungen in Sätze.
pub fn split_sentences(text: &str, base_offset: usize) -> Vec<SentenceSpan> {
    if text.is_empty() {
        return Vec::new();
    }

    let raw_sentences: Vec<&str> = text.unicode_sentences().collect();
    if raw_sentences.is_empty() {
        return vec![SentenceSpan {
            text: text.to_string(),
            span: base_offset..base_offset + text.len(),
        }];
    }

    let mut sentences = Vec::new();
    let mut current_sentence = String::new();
    let mut current_start: Option<usize> = None;
    let mut current_end = 0usize;

    let text_ptr = text.as_ptr() as usize;

    for raw in raw_sentences {
        let raw_ptr = raw.as_ptr() as usize;
        let start_rel = raw_ptr.saturating_sub(text_ptr);
        let end_rel = start_rel.saturating_add(raw.len());

        if current_start.is_none() {
            current_start = Some(start_rel);
        }

        current_sentence.push_str(raw);
        current_end = end_rel;

        if !ends_with_abbreviation(&current_sentence) {
            if let Some(start) = current_start {
                sentences.push(SentenceSpan {
                    text: std::mem::take(&mut current_sentence),
                    span: (base_offset + start)..(base_offset + current_end),
                });
                current_start = None;
            }
        }
    }

    if !current_sentence.is_empty() {
        let start = current_start.unwrap_or(0);
        sentences.push(SentenceSpan {
            text: current_sentence,
            span: (base_offset + start)..(base_offset + current_end),
        });
    }

    sentences
}

/// Prüft, ob ein Textende auf eine bekannte Abkürzung passt.
fn ends_with_abbreviation(text: &str) -> bool {
    let trimmed = text.trim_end();
    if !trimmed.ends_with('.') {
        return false;
    }

    let lower = trimmed.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let last_word = match words.last() {
        Some(w) => *w,
        None => return false,
    };

    for &abbr in ABBREVIATIONS {
        if !abbr.contains(' ') {
            if last_word == abbr {
                return true;
            }
        } else if lower.ends_with(abbr) {
            return true;
        }
    }

    // Einzelliteral + Punkt wie "A.", "B."
    let word_trim = last_word.trim_matches(|c: char| c.is_ascii_punctuation());
    if word_trim.len() == 1 && word_trim.chars().all(|c| c.is_alphabetic()) {
        return true;
    }

    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences_german_abbreviation() {
        let text = "Das ist z. B. ein Test. Dr. Müller geht nach Hause. Er kommt bzw. bleibt hier.";
        let result = split_sentences(text, 0);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap().text, "Das ist z. B. ein Test. ");
        assert_eq!(result.get(1).unwrap().text, "Dr. Müller geht nach Hause. ");
        assert_eq!(result.get(2).unwrap().text, "Er kommt bzw. bleibt hier.");
    }

    #[test]
    fn test_split_sentences_spans() {
        let text = "Erster Satz. Zweiter Satz.";
        let result = split_sentences(text, 10);
        assert_eq!(result.len(), 2);
        assert_eq!(
            text.get(result.get(0).unwrap().span.start - 10..result.get(0).unwrap().span.end - 10)
                .unwrap(),
            "Erster Satz. "
        );
        assert_eq!(
            text.get(result.get(1).unwrap().span.start - 10..result.get(1).unwrap().span.end - 10)
                .unwrap(),
            "Zweiter Satz."
        );
    }
}
