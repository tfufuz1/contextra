//! Kanonische Entry-Ordnung: Cluster, Branch, Schlüssel, Tag absteigend.

use contextra_storage_ports::{BranchId, ClusterId};
use core::cmp::Ordering;

/// Sortierschlüssel für einen Eintrag. `tag` wird als opaker Wire-Tag verglichen.
#[derive(Clone, Copy)]
pub struct EntryKey<'a> {
    pub cluster: ClusterId,
    pub branch: BranchId,
    pub user_key: &'a [u8],
    pub tag: u64,
}

impl core::fmt::Debug for EntryKey<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("EntryKey(<redacted>)")
    }
}

/// Vergleicht Felder einzeln; Konkatenation wäre wegen Präfixen mehrdeutig.
pub fn cmp_entry(left: &EntryKey<'_>, right: &EntryKey<'_>) -> Ordering {
    left.cluster
        .as_bytes()
        .cmp(right.cluster.as_bytes())
        .then_with(|| left.branch.cmp(&right.branch))
        .then_with(|| left.user_key.cmp(right.user_key))
        .then_with(|| right.tag.cmp(&left.tag))
}
