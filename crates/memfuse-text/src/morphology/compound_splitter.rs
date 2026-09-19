use super::stopwords::normalize_umlauts;
use super::trie::Trie;
use std::collections::HashSet;

/// KMU-Fachvokabular und allgemeiner deutscher Wortschatz.
///
/// Zur Kompilierzeit eingebettetes Wörterbuch (aus `data/german_words.txt`).
/// Verhindert Hartcodierung im Quelltext und ermöglicht einfache Erweiterbarkeit.
const DEFAULT_GERMAN_WORDS: &str = include_str!("../data/german_words.txt");



/// Trait for morphological tokenization.
///
/// Decomposes compound words into constituent morphemes.
///
/// # Input Contract
/// Input tokens MUST be lowercased before passing to `decompose`. Passing
/// mixed-case or uppercase tokens causes silent dictionary misses and returns
/// the original token without decomposition. Use `normalize_umlauts()` from
/// this module to prepare input correctly.
///
/// Example: "bundesverfassungsgericht" -> ["bundes", "verfassungs", "gericht"]
pub trait MorphologicalTokenizer: Send + Sync {
    /// Decomposes a token into its morphological components.
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str>;

    /// Returns the language code of this tokenizer (e.g. "de", "en").
    fn language(&self) -> &str;
}

/// German compound word splitter (*Komposita-Zerleger*).
///
/// Uses dictionary-based recursive segmentation powered by Dynamic Programming (DP).
///
/// # Architecture & Algorithm
/// - **Embedded Dictionary**: Thousands of common German stems, root words, prefixes, and KMU
///   enterprise vocabulary loaded at compile time via `include_str!("../data/german_words.txt")`.
/// - **Dynamic Programming (DP)**: Evaluates candidate segmentations of a token to find valid
///   stem paths, preferring decompositions with fewer segments and longer constituent components.
/// - **Interfix Candidates (Fugenelemente)**: Supports interfixes `-s-`, `-en-`, `-e-`, `-er-`,
///   `-n-`, and `-es-` occurring strictly between dictionary-matched components.
///
/// # Explicit Linguistic Limitations
/// 1. **Homograph Ambiguity**: Words with identical spellings that yield multiple valid split paths
///    (e.g., *Wachstube* $\rightarrow$ *Wachs-Tube* vs. *Wach-Stube*) are resolved deterministically
///    by segment count and stem length heuristic. Context-aware semantic disambiguation requires an
///    upstream LLM/POS tagger.
/// 2. **Unseen Stems & Proper Nouns**: Unknown company names, foreign loanwords, or unlisted stems
///    will fail dictionary lookup and safely fall back to returning the full original token unsplit.
/// 3. **Interfix Overgeneration Guard**: Interfixes are strictly constrained between recognized
///    dictionary stems, preventing invalid splitting of non-compound words ending in `-es` or `-en`.
///
/// # Input Contract
/// Input tokens MUST be lowercased (and ideally normalized with
/// [`normalize_umlauts`]) before calling [`MorphologicalTokenizer::decompose`].
/// Uppercase input triggers a `debug_assert!` panic in debug builds and
/// causes silent dictionary misses (fallback: token returned unsplit) in
/// release builds.
///
/// Fallback: returns the original token unsplit.

pub struct GermanCompoundSplitter {
    /// Minimum component length for splitting.
    min_component_len: usize,
    /// Trie data structure for fast prefix checking and dictionary matching.
    trie: Trie,
}

impl GermanCompoundSplitter {
    /// Creates a new German compound splitter loaded with the embedded German vocabulary.
    pub fn new() -> Self {
        Self::with_min_length(3)
    }

    /// Creates a splitter with custom minimum component length and default embedded vocabulary.
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

    /// Creates a splitter with custom minimum component length and custom dictionary set.
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

    /// Returns the minimum component length.
    pub fn min_component_len(&self) -> usize {
        self.min_component_len
    }

    /// Checks if a slice is a valid dictionary stem or stem + interfix.
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
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str> {
        debug_assert!(
            token.chars().all(|c| !c.is_uppercase()),
            "GermanCompoundSplitter::decompose received non-lowercase input: {:?}. \
             Call normalize_umlauts() before decompose().",
            token
        );

        // Guard against oversized single tokens (e.g., > 128 bytes) to avoid O(n^2) DP overhead
        if token.len() <= self.min_component_len || token.len() > 128 {
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

    fn language(&self) -> &str {
        "de"
    }
}
