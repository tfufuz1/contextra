//! # Modul: estimate.rs
//!
//! **Zweck:** Heuristische Token- und Zeichengrößen-Schätzung für Chunks.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, deterministische Textverarbeitung.

// FILE-CONTEXT
// ZWECK: Heuristische Token-Schätzung für Fließtext, Codeblöcke und Tabellen.
// INVARIANTEN: Deterministisches BPE-Verhältnis (4 Zeichen pro Token Grundwert).
// NICHT-OFFENSICHTLICH: Codeblöcke nutzen erhöhten Multiplikator (1.8x).
// STAND: TS:2026-10-29T00:00:00Z

use super::blocks::{is_closing_fence, parse_opening_fence, CodeFence};

/// Approximate character count per token (BPE ratio).
pub(crate) const CHARS_PER_TOKEN: usize = 4;

/// Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln.
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let mut tokens: f64 = 0.0;
    let mut active_fence: Option<CodeFence> = None;
    let code_multiplier = 1.8f64;

    for line in text.lines() {
        if let Some(active) = &active_fence {
            if is_closing_fence(line, active) {
                active_fence = None;
                tokens += 1.0;
                continue;
            }
        } else if let Some(fence_info) = parse_opening_fence(line) {
            active_fence = Some(fence_info);
            tokens += 1.0;
            continue;
        }

        let multiplier = if active_fence.is_some() {
            code_multiplier
        } else {
            1.0
        };

        let mut char_iter = line.chars().peekable();
        while let Some(c) = char_iter.next() {
            match c {
                '\u{4E00}'..='\u{9FFF}'
                | '\u{3400}'..='\u{4DBF}'
                | '\u{20000}'..='\u{2A6DF}'
                | '\u{3040}'..='\u{309F}'
                | '\u{30A0}'..='\u{30FF}' => {
                    tokens += 1.0 * multiplier;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut word_len = 1usize;
                    while char_iter
                        .peek()
                        .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                    {
                        char_iter.next();
                        word_len += 1;
                    }
                    let word_tokens = if word_len <= 4 {
                        1.0
                    } else {
                        1.0 + (word_len as f64 - 4.0) / 4.0
                    };
                    tokens += word_tokens * multiplier;
                }
                '0'..='9' => {
                    let mut num_len = 1usize;
                    while char_iter.peek().is_some_and(|c| c.is_ascii_digit()) {
                        char_iter.next();
                        num_len += 1;
                    }
                    tokens += ((num_len as f64) / 3.0).ceil() * multiplier;
                }
                ' ' | '\t' => {}
                _ => {
                    tokens += 0.25 * multiplier;
                }
            }
        }
        tokens += 0.1;
    }

    (tokens.ceil() as usize).max(1)
}
