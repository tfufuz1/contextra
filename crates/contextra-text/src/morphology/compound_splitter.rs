//! Zerlegung deutscher Komposita mittels Dynamischer Programmierung (DP) und Wörterbuch-Trie.

use super::stopwords::normalize_umlauts;
use super::trie::Trie;
use crate::domain::{DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary};
use std::collections::HashSet;

// ANCHOR[contextra-text:MORPH-016]
/// KMU-Fachvokabular und allgemeiner deutscher Wortschatz.
///
/// Zur Kompilierzeit eingebettetes Wörterbuch (aus `data/german_words.txt`).
/// Verhindert Hartcodierung im Quelltext und ermöglicht einfache Erweiterbarkeit.
const DEFAULT_GERMAN_WORDS: &str = include_str!("../data/german_words.txt");

// ANCHOR[contextra-text:MORPH-017]
/// Trait für morphologische Zerlegung von Wörtern in ihre Grundbestandteile.
///
/// # Eingabevertrag
/// Eingegebene Wörter MÜSSEN vor dem Aufruf von `decompose` in Kleinbuchstaben
/// vorliegen und idealerweise mit [`normalize_umlauts`] vorbereitet sein.
/// Bei Großbuchstaben oder ungültigen Zeichenfolgen wird das Token unzerlegt zurückgegeben.
///
/// Beispiel: "bundesverfassungsgericht" -> \["bundes", "verfassungs", "gericht"\]
pub trait MorphologicalTokenizer: Send + Sync {
    /// Zerlegt ein Token in morphologische Komponenten (Teilwörter).
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str>;

    /// Liefert den Sprachcode (z. B. "de").
    fn language(&self) -> &str;
}

// ANCHOR[contextra-text:MORPH-018]
/// Deutscher Komposita-Zerleger basierend auf Dynamischer Programmierung und Fugenanalyse.
///
/// # Funktionsweise & Algorithmus
/// 1. **Eingebettetes Wörterbuch**: Tausende deutsche Stämme, Grundwörter und Affixe im [`Trie`].
/// 2. **Dynamische Programmierung**: Sucht den optimalen Pfad durch das Wort mit minimaler
///    Segmentanzahl und maximaler Mindestsegmentlänge.
/// 3. **Fugenelemente**: Berücksichtigt Zwischenfugen (`-s-`, `-en-`, `-e-`, `-er-`, `-n-`, `-es-`).
///
/// Fallback: Bei unbekannten Wörtern oder fehlenden Treffern wird das Ursprungswort unzerlegt zurückgegeben.
pub struct GermanCompoundSplitter {
    /// Minimum component length for splitting.
    min_component_len: usize,
    /// Trie data structure for fast prefix checking and dictionary matching.
    trie: Trie,
}

impl GermanCompoundSplitter {
    // ANCHOR[contextra-text:MORPH-019]
    /// Erzeugt einen neuen Zerleger mit der Standard-Mindestlänge (3 Zeichen) und Basis-Wortschatz.
    pub fn new() -> Self {
        Self::with_min_length(3)
    }

    // ANCHOR[contextra-text:MORPH-020]
    /// Erzeugt einen Zerleger mit konfigurierbarer Mindest-Segmentlänge.
    pub fn with_min_length(min_len: usize) -> Self {
        let mut trie = Trie::new();
        for line in DEFAULT_GERMAN_WORDS.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let norm = normalize_umlauts(trimmed);
            if norm.len() >= 2 {
                trie.insert(&norm);
            }
        }
        Self {
            min_component_len: min_len,
            trie,
        }
    }

    // ANCHOR[contextra-text:MORPH-021]
    /// Erzeugt einen Zerleger mit benutzerdefiniertem Wörterbuch.
    pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self {
        let mut trie = Trie::new();
        for word in custom_words {
            let norm = normalize_umlauts(&word);
            if norm.len() >= 2 {
                trie.insert(&norm);
            }
        }
        Self {
            min_component_len: min_len,
            trie,
        }
    }

    // ANCHOR[contextra-text:MORPH-022]
    /// Erzeugt einen Zerleger erweitert um spezifische Fachvokabulare (z. B. Recht, Medizin).
    pub fn with_domains(min_len: usize, domains: &[&dyn DomainVocabulary]) -> Self {
        let mut splitter = Self::with_min_length(min_len);
        for domain in domains {
            for stem in domain.compound_stems() {
                let norm = normalize_umlauts(stem);
                if norm.len() >= 2 {
                    splitter.trie.insert(&norm);
                }
            }
            for term in domain.protected_terms() {
                let norm = normalize_umlauts(term);
                if norm.len() >= 2 {
                    splitter.trie.insert(&norm);
                }
            }
        }
        splitter
    }

    // ANCHOR[contextra-text:MORPH-023]
    /// Erzeugt einen Zerleger mit allen eingebauten Fachdomänen (Recht & Medizin).
    pub fn new_with_all_domains() -> Self {
        let legal = LegalDomainVocabulary;
        let medical = MedicalDomainVocabulary;
        Self::with_domains(3, &[&legal, &medical])
    }

    // ANCHOR[contextra-text:MORPH-024]
    /// Erweitert das Wörterbuch des Zerlegers dynamisch um zusätzliche Begriffe.
    pub fn extend_vocabulary(&mut self, words: impl IntoIterator<Item = impl AsRef<str>>) {
        for word in words {
            let norm = normalize_umlauts(word.as_ref());
            if norm.len() >= 2 {
                self.trie.insert(&norm);
            }
        }
    }

    // ANCHOR[contextra-text:MORPH-025]
    /// Liefert die Mindestlänge für Teilkomponenten.
    pub fn min_component_len(&self) -> usize {
        self.min_component_len
    }

    // ANCHOR[contextra-text:MORPH-026]
    // Prüft, ob ein Teilabschnitt ein gültiges Wörterbuch-Wort oder Stamm + Fugenelement darstellt.
    fn is_valid_component(&self, sub: &str, is_last: bool) -> bool {
        let norm_sub = normalize_umlauts(sub);
        if norm_sub.len() < 2 {
            return false;
        }

        // Direct dictionary match via Trie/HashSet
        if self.trie.contains(&norm_sub) {
            return true;
        }

        // Interfix candidates (Fugenelemente) — allowed strictly between components.
        // Längste-Zuerst-Prüfung verhindert, dass ein kürzeres Fugenelement (z. B. 's') vorzeitig matcht, obwohl ein längeres, spezifischeres Element (z. B. 'es') das eigentlich korrekte Fugenelement wäre.
        if !is_last {
            const INTERFIXES: &[&str] = &["en", "er", "es", "e", "n", "s"];
            for &fuge in INTERFIXES {
                if norm_sub.ends_with(fuge) && norm_sub.len() > fuge.len() {
                    let stem_len = norm_sub.len() - fuge.len();
                    if norm_sub.is_char_boundary(stem_len) {
                        let norm_stem = &norm_sub[..stem_len];
                        if norm_stem.len() >= 2 && self.trie.contains(norm_stem) {
                            return true;
                        }
                    }
                }
            }
        } else {
            // Also allow a component with an interfix if it matches a known stem + interfix pattern
            // or if it was matched as part of backtracking.
        }

        false
    }
}

impl Default for GermanCompoundSplitter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
struct PathNode {
    prev: usize,
    segment_count: usize,
    min_segment_len: usize,
}

impl MorphologicalTokenizer for GermanCompoundSplitter {
    // ANCHOR[contextra-text:MORPH-027]
    // Ablauf der Komposita-Zerlegung:
    // 1. Validierung: Abbruch falls Großbuchstaben vorhanden, Token zu kurz (<= min_component_len) oder > 128 Bytes.
    // 2. DP-Tabelle initialisieren (dp[i] speichert besten Pfad bis Byte-Offset i).
    // 3. Vorwärts-Scanning über alle gültigen UTF-8-Zeichengrenzen.
    // 4. Backtracking: Falls dp[n] mindestens 2 Segmente aufweist, Komponenten rekonstruieren.
    // 5. Bei Misserfolg Rückgabe des unveränderten Tokens.
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str> {
        // Fallback safely for non-lowercase or oversized single tokens
        if token.chars().any(|c| c.is_uppercase())
            || token.len() <= self.min_component_len
            || token.len() > 128
        {
            return vec![token];
        }

        let n = token.len();
        let mut dp: Vec<Option<PathNode>> = vec![None; n + 1];
        dp[0] = Some(PathNode {
            prev: 0,
            segment_count: 0,
            min_segment_len: usize::MAX,
        });

        for i in 0..n {
            if !token.is_char_boundary(i) {
                continue;
            }

            let current_node = match &dp[i] {
                Some(node) => node.clone(),
                None => continue,
            };

            for j in (i + 2)..=n {
                if !token.is_char_boundary(j) {
                    continue;
                }

                if !token.is_char_boundary(i) {
                    continue;
                }

                let sub = &token[i..j];
                let is_last = j == n;

                if self.is_valid_component(sub, is_last) {
                    let sub_char_count = sub.chars().count();
                    let new_seg_count = current_node.segment_count + 1;
                    let new_min_len = current_node.min_segment_len.min(sub_char_count);

                    let candidate = PathNode {
                        prev: i,
                        segment_count: new_seg_count,
                        min_segment_len: new_min_len,
                    };

                    let update = match &dp[j] {
                        None => true,
                        Some(existing) => {
                            if candidate.segment_count < existing.segment_count {
                                true
                            } else if candidate.segment_count == existing.segment_count {
                                candidate.min_segment_len > existing.min_segment_len
                            } else {
                                false
                            }
                        }
                    };

                    if update {
                        dp[j] = Some(candidate);
                    }
                }
            }
        }

        // Backtrack optimal path if compound decomposition (>= 2 segments) was found
        if let Some(ref target_node) = dp[n] {
            if target_node.segment_count >= 2 {
                let mut path = Vec::with_capacity(target_node.segment_count);
                let mut curr = n;
                while curr > 0 {
                    if let Some(ref node) = dp[curr] {
                        let prev = node.prev;
                        if token.is_char_boundary(prev) && token.is_char_boundary(curr) {
                            path.push(&token[prev..curr]);
                        } else {
                            break;
                        }
                        curr = prev;
                    } else {
                        break;
                    }
                }
                path.reverse();
                return path;
            }
        }

        vec![token]
    }

    // ANCHOR[contextra-text:MORPH-028]
    fn language(&self) -> &str {
        "de"
    }
}
