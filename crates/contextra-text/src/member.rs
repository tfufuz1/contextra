//! Mitgliedschaftsblock nach SPEC D-121 Abs. 3 und F.7.
//!
//! sortierte u64-Werte: m(t) = u64_be(BLAKE3-keyed(K_idx, "ctx/member/v1" || t)[0..8])
//! für jedes vorkommende cluster_tag und doc_tag des Segments.
//! Dient der schnellen Ermittlung betroffener Segmente (falsch-positiv erlaubt, falsch-negativ ausgeschlossen).

use contextra_crypto::hierarchy::IdxKey;
use contextra_types::{ClusterTag, DocTag};

/// Berechnet den 64-Bit Mitgliedschafts-Hash m(t) nach SPEC D-121 Abs. 3:
/// `m(t) = u64_be(BLAKE3-keyed(K_idx, "ctx/member/v1" || t)[0..8])`
pub fn compute_member_hash(k_idx: &IdxKey, tag_bytes: &[u8; 16]) -> u64 {
    let mut hasher = blake3::Hasher::new_keyed(k_idx.0.as_bytes());
    hasher.update(b"ctx/member/v1");
    hasher.update(tag_bytes);
    let out = hasher.finalize();
    let bytes: [u8; 8] = out.as_bytes()[0..8].try_into().unwrap_or([0u8; 8]);
    u64::from_be_bytes(bytes)
}

/// Mitgliedschaftsblock zur schnellen Segmentauflösung bei Löschungen (D-121).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MembershipBlock {
    pub members: Vec<u64>,
}

impl MembershipBlock {
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
        }
    }

    /// Fügt ein Tag (Cluster- oder Dokument-Tag) dem Mitgliedschaftsblock hinzu.
    pub fn add_tag(&mut self, k_idx: &IdxKey, tag_bytes: &[u8; 16]) {
        let h = compute_member_hash(k_idx, tag_bytes);
        self.members.push(h);
    }

    /// Sortiert und dedupliziert die Mitgliederliste kanonisch.
    pub fn finalize(&mut self) {
        self.members.sort_unstable();
        self.members.dedup();
    }

    /// Prüft, ob ein gegebenes Tag im Mitgliedschaftsblock enthalten ist (SPEC D-121 Abs. 3).
    /// Falsch-negativ ist ausgeschlossen!
    pub fn contains_tag(&self, k_idx: &IdxKey, tag_bytes: &[u8; 16]) -> bool {
        let target = compute_member_hash(k_idx, tag_bytes);
        self.members.binary_search(&target).is_ok()
    }

    pub fn contains_cluster(&self, k_idx: &IdxKey, cluster: &ClusterTag) -> bool {
        self.contains_tag(k_idx, cluster.as_bytes())
    }

    pub fn contains_doc(&self, k_idx: &IdxKey, doc: &DocTag) -> bool {
        self.contains_tag(k_idx, doc.as_bytes())
    }

    pub fn encode(&self) -> Vec<u8> {
        let count = u32::try_from(self.members.len()).unwrap_or(u32::MAX);
        let mut out = Vec::with_capacity(4 + self.members.len() * 8);
        out.extend_from_slice(&count.to_be_bytes());
        for &m in &self.members {
            out.extend_from_slice(&m.to_be_bytes());
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 4 {
            return None;
        }
        let count_bytes: [u8; 4] = bytes.get(0..4)?.try_into().ok()?;
        let count = u32::from_be_bytes(count_bytes) as usize;
        let mut offset = 4;
        let mut members = Vec::with_capacity(count);

        for _ in 0..count {
            if offset + 8 > bytes.len() {
                return None;
            }
            let m_bytes: [u8; 8] = bytes.get(offset..offset + 8)?.try_into().ok()?;
            members.push(u64::from_be_bytes(m_bytes));
            offset += 8;
        }

        Some((Self { members }, offset))
    }
}
