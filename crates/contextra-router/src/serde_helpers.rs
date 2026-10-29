// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Serde-Hilfsfunktionen für deterministische JSON-Serialisierung.
// INVARIANTEN: BTreeSet<u64> wird in natürlicher Reihenfolge serialisiert.

//! Serde helpers for contextra-router.

pub mod sorted_u64_set {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeSet;

    pub fn serialize<S: Serializer>(set: &BTreeSet<u64>, s: S) -> Result<S::Ok, S::Error> {
        let v: Vec<u64> = set.iter().copied().collect();
        v.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u64>, D::Error> {
        Vec::<u64>::deserialize(d).map(|v| v.into_iter().collect())
    }
}
