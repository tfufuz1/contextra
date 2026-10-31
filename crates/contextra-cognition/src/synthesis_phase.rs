//! # Generative Synthesephase (`synthesis_phase.rs`)
//!
//! Führt die LLM-basierte Wissenssynthese auf Segment-Ebene aus.
//! Abstrahiert ganze Turn-Segmente zu übergeordneten Wissens-Chunks ([`SynthesizedChunk`]).
//! Dies ist bewusst vom rein deterministischen strukturellen Konsolidierungslauf getrennt,
//! da hier stochastische Sprachmodelle über den [`SegmentSynthesizer`]-Port aufgerufen werden.
//!
//! ## Ring-Zugehörigkeit
//! Teil der Kognitionsschicht (Ring 3). Nutzt den Port `SegmentSynthesizer` aus `contextra-ports`.
//!
//! ## Invarianten
//! - **Fehlertoleranz:** Ein Einzelfehler bei der LLM-Synthese führt niemals zum Abbruch des gesamten Laufs,
//!   sondern inkrementiert lediglich `skipped_segments` (Fail-Soft).
//! - **Provenienzerhalt:** Jeder synthetisierte Chunk speichert exakt die Quell-`DocId`s der Ausgangs-Turns.


use crate::memory_consolidation::TurnSegment;
use contextra_ports::SegmentSynthesizer;
use contextra_types::DocId;

/// Ergebnis des Generative Synthesis Pass auf Segment-Ebene.
#[derive(Debug, Clone, PartialEq, Eq)]
// ANCHOR[contextra-cognition:SYNTH-001]
/// Ergebnis des Generative Synthesis Pass auf Segment-Ebene.
pub struct SegmentSynthesisResult {
    /// Neu synthetisierte Chunks (einer pro konsolidiertem Segment).
    pub synthesized_chunks: Vec<SynthesizedChunk>,
    /// Anzahl der Segmente, für die keine Synthese möglich war (LLM-Fehler / zu kurz).
    pub skipped_segments: usize,
}

#[deprecated(note = "use SegmentSynthesisResult")]
// ANCHOR[contextra-cognition:SYNTH-002]
/// Veralteter Typalias für SegmentSynthesisResult.
pub type SynthesisPhaseResult = SegmentSynthesisResult;

/// Ein generativ synthetisierter Wissens-Chunk mit Provenienz.
#[derive(Debug, Clone, PartialEq, Eq)]
// ANCHOR[contextra-cognition:SYNTH-003]
/// Ein generativ synthetisierter Wissens-Chunk mit Provenienz.
pub struct SynthesizedChunk {
    /// Der neue, abstrakte Inhalt.
    pub content: String,
    /// DocIds der Quell-Turns aus dem ursprünglichen Segment.
    pub source_turn_ids: Vec<DocId>,
    /// Modell, das zur Synthese verwendet wurde.
    pub model_id: String,
}

/// Trait-Abstraktion für den LLM-Synthesizer (testbar via Mock).
pub use contextra_ports::SegmentSynthesizer as TraitSegmentSynthesizer;

/// Dies ist der LLM-BASIERTE Generative Synthesis Pass (erzeugt `SynthesizedChunk`s via `SegmentSynthesizer`-Trait). Für den deterministischen Structural Consolidation Pass siehe `memory_consolidation::run_structural_synthesis_pass()`.
///
/// Führt den Generative Synthesis Pass aus: synthetisiert pro Segment einen abstrakten Chunk.
///
/// # Fehlerverhalten
/// Einzelne Segment-Fehler werden übersprungen (`skipped_segments` inkrementiert),
/// nie als Err propagiert. Globale Fehler (Trait-Fehler im Setup) propagieren als Err.
// ANCHOR[contextra-cognition:SYNTH-004]
/// Führt den generativen Syntheselauf über Turn-Segmente aus.
pub async fn run_synthesis_pass(
    segments: &[TurnSegment],
    segment_texts: &[Vec<String>], // Texte der Turns pro Segment
    synthesizer: &dyn SegmentSynthesizer,
    min_turns_for_synthesis: usize, // Default: 3 — kurze Segmente überspringen
) -> SegmentSynthesisResult {
    let mut synthesized_chunks = Vec::new();
    let mut skipped_segments = 0;

    for (i, segment) in segments.iter().enumerate() {
        if segment.turn_ids.len() < min_turns_for_synthesis {
            skipped_segments += 1;
            continue;
        }

        let texts = match segment_texts.get(i) {
            Some(t) => t,
            None => {
                skipped_segments += 1;
                continue;
            }
        };

        if texts.is_empty() {
            skipped_segments += 1;
            continue;
        }

        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        match synthesizer.synthesize_segment(&text_refs).await {
            Ok(content) => {
                synthesized_chunks.push(SynthesizedChunk {
                    content,
                    source_turn_ids: segment.turn_ids.clone(),
                    model_id: synthesizer.model_id().to_string(),
                });
            }
            Err(e) => {
                tracing::warn!(
                    segment_idx = i,
                    error = %e,
                    "Generative synthesis pass segment synthesis failed; skipping segment"
                );
                skipped_segments += 1;
            }
        }
    }

    SegmentSynthesisResult {
        synthesized_chunks,
        skipped_segments,
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::get_first
)]
mod tests {
    use super::*;
    use contextra_ports::BoxFuture;
    use contextra_types::Result;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockSynthesizer {
        model: String,
        should_fail: AtomicBool,
    }

    impl MockSynthesizer {
        fn new(model: &str, should_fail: bool) -> Self {
            Self {
                model: model.to_string(),
                should_fail: AtomicBool::new(should_fail),
            }
        }
    }

    impl SegmentSynthesizer for MockSynthesizer {
        fn synthesize_segment<'a>(
            &'a self,
            segment_texts: &'a [&'a str],
        ) -> BoxFuture<'a, Result<String>> {
            Box::pin(async move {
                if self.should_fail.load(Ordering::SeqCst) {
                    Err(contextra_types::ContextraError::Internal(
                        "Mock LLM synthesis error".into(),
                    ))
                } else {
                    Ok(format!("Synthesized: {}", segment_texts.join(" + ")))
                }
            })
        }

        fn model_id(&self) -> &str {
            &self.model
        }
    }

// ANCHOR[contextra-cognition:TEST-SYNTHESI-022]
// Prüft Testfall `test_synthesis_pass_empty_segments_returns_empty`.
    #[tokio::test]
    async fn test_synthesis_pass_empty_segments_returns_empty() {
        let synthesizer = MockSynthesizer::new("test-model", false);
        let res = run_synthesis_pass(&[], &[], &synthesizer, 3).await;

        assert_eq!(res.synthesized_chunks.len(), 0);
        assert_eq!(res.skipped_segments, 0);
    }

// ANCHOR[contextra-cognition:TEST-SYNTHESI-023]
// Prüft Testfall `test_synthesis_pass_too_short_segments_skipped`.
    #[tokio::test]
    async fn test_synthesis_pass_too_short_segments_skipped() {
        let synthesizer = MockSynthesizer::new("test-model", false);
        let segment = TurnSegment {
            turn_ids: vec![DocId::new(1), DocId::new(2)], // 2 turns < min_turns_for_synthesis=3
            representative_embedding: vec![1.0, 0.0],
        };
        let texts = vec![vec!["text 1".to_string(), "text 2".to_string()]];

        let res = run_synthesis_pass(&[segment], &texts, &synthesizer, 3).await;

        assert_eq!(res.synthesized_chunks.len(), 0);
        assert_eq!(res.skipped_segments, 1);
    }

// ANCHOR[contextra-cognition:TEST-SYNTHESI-024]
// Prüft Testfall `test_synthesis_pass_llm_error_increments_skipped_not_err`.
    #[tokio::test]
    async fn test_synthesis_pass_llm_error_increments_skipped_not_err() {
        let synthesizer = MockSynthesizer::new("test-model", true); // should_fail = true
        let segment = TurnSegment {
            turn_ids: vec![DocId::new(1), DocId::new(2), DocId::new(3)],
            representative_embedding: vec![1.0, 0.0],
        };
        let texts = vec![vec![
            "text 1".to_string(),
            "text 2".to_string(),
            "text 3".to_string(),
        ]];

        let res = run_synthesis_pass(&[segment], &texts, &synthesizer, 3).await;

        assert_eq!(res.synthesized_chunks.len(), 0);
        assert_eq!(res.skipped_segments, 1);
    }

// ANCHOR[contextra-cognition:TEST-SYNTHESI-025]
// Prüft Testfall `test_synthesis_pass_synthesized_chunk_has_correct_source_ids`.
    #[tokio::test]
    async fn test_synthesis_pass_synthesized_chunk_has_correct_source_ids() {
        let synthesizer = MockSynthesizer::new("test-model", false);
        let segment = TurnSegment {
            turn_ids: vec![DocId::new(10), DocId::new(20), DocId::new(30)],
            representative_embedding: vec![1.0, 0.0],
        };
        let texts = vec![vec!["A".to_string(), "B".to_string(), "C".to_string()]];

        let res = run_synthesis_pass(&[segment], &texts, &synthesizer, 3).await;

        assert_eq!(res.synthesized_chunks.len(), 1);
        assert_eq!(res.skipped_segments, 0);

        let chunk = &res.synthesized_chunks[0];
        assert_eq!(chunk.content, "Synthesized: A + B + C");
        assert_eq!(
            chunk.source_turn_ids,
            vec![DocId::new(10), DocId::new(20), DocId::new(30)]
        );
        assert_eq!(chunk.model_id, "test-model");
    }
}
