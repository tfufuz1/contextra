//! # Schlüsselbereiche
//!
//! Definition von `KeyRange` für Bereichsabfragen und Tombstones.

/// Ein Schlüsselbereich (start inklusiv, end exklusiv oder unbegrenzt).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRange {
    /// Untere Grenze (inklusiv).
    pub start: Vec<u8>,
    /// Obere Grenze (exklusiv). `None` bedeutet unbegrenzt bis Space-Ende.
    pub end: Option<Vec<u8>>,
}

impl KeyRange {
    /// Erstellt einen Bereich von `start` (inkl.) bis `end` (exkl.).
    pub fn new(start: Vec<u8>, end: Vec<u8>) -> Self {
        Self {
            start,
            end: Some(end),
        }
    }

    /// Erstellt einen Bereich für alle Schlüssel mit dem gegebenen Präfix.
    pub fn prefix(p: &[u8]) -> Self {
        let start = p.to_vec();
        let end = prefix_end(p);
        Self { start, end }
    }

    /// Erstellt einen Bereich von `start` bis zum Ende des Space.
    pub fn from_start(start: Vec<u8>) -> Self {
        Self { start, end: None }
    }

    /// Prüft, ob ein Schlüssel innerhalb dieses Bereichs liegt.
    pub fn contains(&self, key: &[u8]) -> bool {
        if key < &self.start[..] {
            return false;
        }
        match &self.end {
            Some(e) => key < &e[..],
            None => true,
        }
    }
}

/// Berechnet den kleinsten Schlüssel, der größer als alle Schlüssel mit Präfix `p` ist.
pub fn prefix_end(p: &[u8]) -> Option<Vec<u8>> {
    let mut end = p.to_vec();
    while let Some(last) = end.pop() {
        if last < 0xFF {
            end.push(last + 1);
            return Some(end);
        }
    }
    None
}
