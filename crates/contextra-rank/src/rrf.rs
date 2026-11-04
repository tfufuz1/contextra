//! Deterministische Reciprocal Rank Fusion (RRF) ohne Gleitkommaarithmetik.

use crate::error::RankError;

/// Glättungsfaktor $k = 60$ für Reciprocal Rank Fusion (SPEC §8.11, Anhang C, RK-01).
pub const K_RRF: u32 = 60;

/// Alias für eine Kandidatenliste (SPEC §3.1).
pub type RankedList<'a> = &'a [Candidate];

/// Alias für das fusionierte Gesamtergebnis (SPEC §3.1).
pub type Ranked = Vec<Fused>;

/// Alias für [`rrf_fuse`] gemäß SPEC §3.1 (`fuse(&[RankedList], k) -> Ranked`).
pub use rrf_fuse as fuse;

/// Höchstgrenze für Kandidaten je Liste (SPEC D-87, RK-06).
const MAX_CANDIDATES_PER_LIST: usize = 1000;

/// Skalierungsfaktor für Festkomma-Arithmetik: $2^{40}$ (RK-02).
const FIXED_POINT_SCALE: u64 = 1u64 << 40;

/// Opaker Dokument-Handle für Suchtreffer aus Ring 0 (`types`, D-70).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocHandle {
    /// Betroffenen- / Cluster-Kennung (`ClusterTag`).
    pub cluster_tag: [u8; 16],
    /// Gehashte Dokumentkennung (`DocTag`).
    pub doc_tag: [u8; 16],
}

/// Ein Kandidat einer bereits gefilterten Teilliste (RK-04, RK-05, D-124).
///
/// Der Rang entspricht der 1-basierten Position innerhalb der Liste.
/// Die Identität eines Treffers ist `(doc_tag, chunk)` (SPEC D-124, T-HYB-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Candidate {
    /// Eindeutiger Dokument-Handle.
    pub handle: DocHandle,
    /// Chunk-Index innerhalb des Dokuments (D-124).
    pub chunk: u32,
}

/// Festkomma-Ganzzahl-Repräsentation des fusionierten RRF-Scores (RK-02).
///
/// Vermeidet jegliche Gleitkomma-Nichtassoziativität und garantiert
/// bit-genaue Permutationsinvarianz über beliebig viele Teillisten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RrfScore(pub u64);

/// Ein fusionierter Treffer mit Rang und berechnetem Festkomma-Score.
/// Die Identität eines Treffers ist `(doc_tag, chunk)` (SPEC D-124, T-HYB-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fused {
    /// Eindeutiger Dokument-Handle des Treffers.
    pub handle: DocHandle,
    /// Chunk-Index innerhalb des Dokuments (D-124).
    pub chunk: u32,
    /// Akkumulierter Festkomma-Score.
    pub score: RrfScore,
}

/// Fusioniert beliebig viele Kandidatenlisten deterministisch via Reciprocal Rank Fusion.
///
/// - Glättungsfaktor: $k = 60$ (fest).
/// - Beitrag je Teilliste: $\lfloor 2^{40} / (60 + \text{Rang}) \rfloor$.
/// - Tie-Break (RK-03): Score absteigend, dann `doc_tag` aufsteigend, dann `chunk` aufsteigend,
///   dann `cluster_tag` aufsteigend.
/// - Validierung (RK-06):
///   - `top == 0` $\Rightarrow$ leere Rückgabe.
///   - Liste länger als 1.000 $\Rightarrow$ `Err(RankError::TooLarge)`.
///   - Duplikate innerhalb einer Teilliste $\Rightarrow$ `Err(RankError::Duplicate)`.
///   - Überlauf bei Addition $\Rightarrow$ `Err(RankError::Overflow)`.
///
/// # Errors
/// Gibt `RankError` zurück, falls eine Liste zu lang ist, interne Duplikate enthält oder
/// ein arithmetischer Überlauf auftritt.
pub fn rrf_fuse(lists: &[&[Candidate]], top: usize) -> Result<Vec<Fused>, RankError> {
    if top == 0 {
        return Ok(Vec::new());
    }

    // Phase 1: Validierung aller Teillisten (RK-06, D-124 Identität (doc_tag, chunk))
    for list in lists {
        if list.len() > MAX_CANDIDATES_PER_LIST {
            return Err(RankError::TooLarge);
        }
        if list.len() > 1 {
            let mut sorted_keys: Vec<(DocHandle, u32)> =
                list.iter().map(|c| (c.handle, c.chunk)).collect();
            sorted_keys.sort_unstable();
            for window in sorted_keys.windows(2) {
                if let [a, b] = window {
                    if a == b {
                        return Err(RankError::Duplicate);
                    }
                }
            }
        }
    }

    // Phase 2: Akkumulation der Beiträge über alle Listen (RK-02)
    let mut accumulator: Vec<(DocHandle, u32, u64)> = Vec::new();

    for list in lists {
        for (idx, candidate) in list.iter().enumerate() {
            let rank = (idx as u64).checked_add(1).ok_or(RankError::Overflow)?;
            let denom = (K_RRF as u64)
                .checked_add(rank)
                .ok_or(RankError::Overflow)?;
            let contrib = FIXED_POINT_SCALE
                .checked_div(denom)
                .ok_or(RankError::Overflow)?;

            if let Some(pos) = accumulator.iter().position(|(handle, chunk, _)| {
                *handle == candidate.handle && *chunk == candidate.chunk
            }) {
                if let Some(entry) = accumulator.get_mut(pos) {
                    entry.2 = entry.2.checked_add(contrib).ok_or(RankError::Overflow)?;
                }
            } else {
                accumulator.push((candidate.handle, candidate.chunk, contrib));
            }
        }
    }

    // Phase 3: Totale Ordnung herstellen (RK-03, SPEC §8.11, D-124)
    accumulator.sort_unstable_by(
        |(handle_a, chunk_a, score_a), (handle_b, chunk_b, score_b)| {
            score_b
                .cmp(score_a)
                .then_with(|| handle_a.doc_tag.cmp(&handle_b.doc_tag))
                .then_with(|| chunk_a.cmp(chunk_b))
                .then_with(|| handle_a.cluster_tag.cmp(&handle_b.cluster_tag))
        },
    );

    // Phase 4: Begrenzung auf `top`
    let result_len = accumulator.len().min(top);
    let mut fused = Vec::with_capacity(result_len);
    for &(handle, chunk, score) in accumulator.iter().take(result_len) {
        fused.push(Fused {
            handle,
            chunk,
            score: RrfScore(score),
        });
    }

    Ok(fused)
}
