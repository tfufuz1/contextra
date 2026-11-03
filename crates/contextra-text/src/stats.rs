//! Exakte Segmentstatistiken und Neuberechnung nach Segment-Rewrite (SPEC E-L5).

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SegmentStats {
    pub doc_count: u32,
    pub total_len: u64,
    pub avg_doc_len: f32,
}

impl SegmentStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recompute(living_doc_lens: &[u32]) -> Self {
        let doc_count = u32::try_from(living_doc_lens.len()).unwrap_or(u32::MAX);
        let total_len: u64 = living_doc_lens.iter().map(|&len| len as u64).sum();
        let avg_doc_len = if doc_count > 0 {
            total_len as f32 / doc_count as f32
        } else {
            0.0
        };

        Self {
            doc_count,
            total_len,
            avg_doc_len,
        }
    }
}
