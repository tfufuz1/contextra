//! Präfixbaum (Trie) für effiziente Wörterbuchabfragen und Teilwortsuche.

// ANCHOR[contextra-text:MORPH-010]
/// Knoten im Präfixbaum zur Repräsentation von Buchstabenfolgen deutscher Wörter.
#[derive(Default, Debug, Clone)]
pub struct TrieNode {
    /// Zeigt an, ob an diesem Knoten ein vollständiges Wort aus dem Wörterbuch endet.
    pub is_terminal: bool,
    /// Kindknoten, indiziert nach Folgezeichen.
    pub children: std::collections::HashMap<char, TrieNode>,
}

// ANCHOR[contextra-text:MORPH-011]
/// Präfixbaum (Trie) für schnelle O(m)-Wörterbuchabfragen (m = Wortlänge).
#[derive(Default, Debug, Clone)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    // ANCHOR[contextra-text:MORPH-012]
    /// Erzeugt einen leeren Präfixbaum.
    pub fn new() -> Self {
        Self::default()
    }

    // ANCHOR[contextra-text:MORPH-013]
    /// Fügt ein Wort zeichenweise in den Baum ein und markiert den Endknoten als terminal.
    pub fn insert(&mut self, word: &str) {
        let mut curr = &mut self.root;
        for ch in word.chars() {
            curr = curr.children.entry(ch).or_default();
        }
        curr.is_terminal = true;
    }

    // ANCHOR[contextra-text:MORPH-014]
    /// Prüft, ob ein Wort exakt als terminaler Eintrag im Baum enthalten ist.
    pub fn contains(&self, word: &str) -> bool {
        let mut curr = &self.root;
        for ch in word.chars() {
            if let Some(next) = curr.children.get(&ch) {
                curr = next;
            } else {
                return false;
            }
        }
        curr.is_terminal
    }

    // ANCHOR[contextra-text:MORPH-015]
    /// Prüft, ob im Baum mindestens ein Wort existiert, das mit dem übergebenen Präfix beginnt.
    pub fn starts_with(&self, prefix: &str) -> bool {
        let mut curr = &self.root;
        for ch in prefix.chars() {
            if let Some(next) = curr.children.get(&ch) {
                curr = next;
            } else {
                return false;
            }
        }
        true
    }
}
