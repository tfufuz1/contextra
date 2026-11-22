/// A prefix trie node for fast lookup and prefix matching of German dictionary words.
#[derive(Default, Debug, Clone)]
pub struct TrieNode {
    /// Indicates if a word ends at this node.
    pub is_terminal: bool,
    /// Child nodes keyed by character.
    pub children: std::collections::HashMap<char, TrieNode>,
}

/// A prefix trie data structure for dictionary lookup.
#[derive(Default, Debug, Clone)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    /// Creates a new empty Trie.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a word into the Trie.
    pub fn insert(&mut self, word: &str) {
        let mut curr = &mut self.root;
        for ch in word.chars() {
            curr = curr.children.entry(ch).or_default();
        }
        curr.is_terminal = true;
    }

    /// Checks if a word exists in the Trie.
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

    /// Checks if any word in the Trie starts with the given prefix.
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
