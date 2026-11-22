//! # FilterBlock und PrefixFilterBlock
//!
//! FilterBlock (SPEC §9.5):
//! Bloom-Filter über distinkte Nutzerschlüssel, `bits_per_key = 10`.
//! Größe berechnet aus der tatsächlichen Anzahl distinkter Schlüssel.
//! PrefixFilterBlock (SPEC §9.5):
//! Gleiches Layout über die ersten `n` Bytes jedes Schlüssels.
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use crate::codec::bloom::BloomFilter;

/// Standardmäßig 10 Bits je Nutzerschlüssel.
pub const BLOOM_BITS_PER_KEY: usize = 10;

/// Ersteller für den Bloom-Filter über distinkte Nutzerschlüssel.
#[derive(Debug, Clone, Default)]
pub struct FilterBlockBuilder {
    keys: Vec<Vec<u8>>,
}

impl FilterBlockBuilder {
    /// Erstellt einen neuen leeren Filter-Builder.
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Fügt einen Nutzerschlüssel hinzu (doppelte Schlüssel werden beim Bau dedupliziert).
    pub fn add_key(&mut self, user_key: &[u8]) {
        if self.keys.last().map(|k| k.as_slice()) != Some(user_key) {
            self.keys.push(user_key.to_vec());
        }
    }

    /// Schließt den Bloom-Filter ab und kodiert ihn nach SPEC §9.5.
    pub fn finish(mut self) -> Vec<u8> {
        self.keys.sort_unstable();
        self.keys.dedup();

        let count = self.keys.len();
        let mut filter = BloomFilter::new(count, BLOOM_BITS_PER_KEY);
        for k in &self.keys {
            filter.insert(k);
        }
        filter.encode()
    }
}

/// Ersteller für den Präfix-Bloom-Filter über die ersten `n` Bytes jedes Schlüssels.
#[derive(Debug, Clone)]
pub struct PrefixFilterBlockBuilder {
    prefix_len: usize,
    prefixes: Vec<Vec<u8>>,
}

impl PrefixFilterBlockBuilder {
    /// Erstellt einen neuen Builder für Präfixlänge `n` (1 <= n <= 64).
    pub fn new(prefix_len: usize) -> Self {
        Self {
            prefix_len,
            prefixes: Vec::new(),
        }
    }

    /// Fügt einen Schlüssel hinzu (die ersten `prefix_len` Bytes werden extrahiert).
    pub fn add_key(&mut self, user_key: &[u8]) {
        let len = std::cmp::min(user_key.len(), self.prefix_len);
        if let Some(slice) = user_key.get(..len) {
            if self.prefixes.last().map(|p| p.as_slice()) != Some(slice) {
                self.prefixes.push(slice.to_vec());
            }
        }
    }

    /// Schließt den Präfix-Bloom-Filter ab und kodiert ihn.
    pub fn finish(mut self) -> Vec<u8> {
        self.prefixes.sort_unstable();
        self.prefixes.dedup();

        let count = self.prefixes.len();
        let mut filter = BloomFilter::new(count, BLOOM_BITS_PER_KEY);
        for p in &self.prefixes {
            filter.insert(p);
        }
        filter.encode()
    }
}

/// Leser für einen Bloom-Filter-Block.
#[derive(Debug, Clone)]
pub struct FilterBlockReader {
    filter: BloomFilter,
}

impl FilterBlockReader {
    /// Dekodiert einen Filter-Block aus den Rohdaten.
    pub fn decode(bytes: &[u8]) -> Result<Self, RunError> {
        let filter = BloomFilter::decode(bytes)
            .map_err(|e| RunError::Corrupt(format!("Bloom-Filter Dekodierfehler: {e}")))?;
        Ok(Self { filter })
    }

    /// Prüft, ob der Nutzerschlüssel möglicherweise vorhanden ist.
    pub fn may_contain(&self, user_key: &[u8]) -> bool {
        self.filter.may_contain(user_key)
    }
}

/// Leser für einen Präfix-Bloom-Filter-Block.
#[derive(Debug, Clone)]
pub struct PrefixFilterBlockReader {
    prefix_len: usize,
    filter: BloomFilter,
}

impl PrefixFilterBlockReader {
    /// Dekodiert einen Präfix-Filter-Block mit angegebener Präfixlänge `n`.
    pub fn decode(bytes: &[u8], prefix_len: usize) -> Result<Self, RunError> {
        let filter = BloomFilter::decode(bytes)
            .map_err(|e| RunError::Corrupt(format!("Präfix-Bloom-Filter Dekodierfehler: {e}")))?;
        Ok(Self {
            prefix_len,
            filter,
        })
    }

    /// Liefert die konfigurierte Präfixlänge.
    pub const fn prefix_len(&self) -> usize {
        self.prefix_len
    }

    /// Prüft, ob ein Präfix möglicherweise im Run vorhanden ist.
    ///
    /// Hat das Suchpräfix weniger als `prefix_len` Bytes, kann der Filter
    /// nicht genutzt werden und liefert konservativ `true` (SPEC §9.5).
    pub fn may_contain_prefix(&self, prefix: &[u8]) -> bool {
        if prefix.len() < self.prefix_len {
            return true;
        }
        if let Some(slice) = prefix.get(..self.prefix_len) {
            self.filter.may_contain(slice)
        } else {
            true
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder_and_reader() {
        let mut builder = FilterBlockBuilder::new();
        builder.add_key(b"apple");
        builder.add_key(b"banana");
        builder.add_key(b"cherry");

        let encoded = builder.finish();
        let reader = FilterBlockReader::decode(&encoded).unwrap();

        assert!(reader.may_contain(b"apple"));
        assert!(reader.may_contain(b"banana"));
        assert!(reader.may_contain(b"cherry"));
        assert!(!reader.may_contain(b"durian"));
    }

    #[test]
    fn test_prefix_filter_builder_and_reader() {
        let mut builder = PrefixFilterBlockBuilder::new(4);
        builder.add_key(b"user:1001");
        builder.add_key(b"user:1002");
        builder.add_key(b"order:2001");

        let encoded = builder.finish();
        let reader = PrefixFilterBlockReader::decode(&encoded, 4).unwrap();

        assert!(reader.may_contain_prefix(b"user:1001"));
        assert!(reader.may_contain_prefix(b"user:"));
        assert!(reader.may_contain_prefix(b"orde"));
        assert!(!reader.may_contain_prefix(b"item:9999"));

        // Kürzere Präfixe als n (4) liefern immer true (kann nicht filtern)
        assert!(reader.may_contain_prefix(b"us"));
    }
}
