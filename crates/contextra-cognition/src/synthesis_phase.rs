//! Generative Synthesis Pass — LLM-basierte, stochastische Wissenssynthese pro konsolidiertem Segment.
//! Bewusst getrennt vom Structural Consolidation Pass, da hier LLM-API-Abhängigkeiten bestehen, die dort verboten sind.
//!
//! SEGMENT-LEVEL (nicht Turn-Level): Jeder SynthesizedChunk abstrahiert ein ganzes
//! TurnSegment aus dem Structural Consolidation Pass. Dies entspricht LycheeMemory V2 (arXiv:2608.12990).

use crate::memory_consolidation::TurnSegment;
use contextra_ports::SegmentSynthesizer;
use contextra_types::DocId;

/// Ergebnis des Generative Synthesis Pass auf Segment-Ebene.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SegmentSynthesisResult {
    /// Neu synthetisierte Chunks (einer pro konsolidiertem Segment).
    pub synthesized_chunks: Vec<SynthesizedChunk>,
    /// Anzahl der Segmente, für die keine Synthese möglich war (LLM-Fehler / zu kurz).
    pub skipped_segments: usize,
    /// Detaillierte Fehlermeldungen fehlgeschlagener Segment-Synthesen.
    pub errors: Vec<String>,
}

#[deprecated(note = "use SegmentSynthesisResult")]
pub type SynthesisPhaseResult = SegmentSynthesisResult;

/// Ein generativ synthetisierter Wissens-Chunk mit Provenienz.
#[derive(Debug, Clone, PartialEq, Eq)]
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
pub async fn run_synthesis_pass(
    segments: &[TurnSegment],
    segment_texts: &[Vec<String>], // Texte der Turns pro Segment
    synthesizer: &dyn SegmentSynthesizer,
    min_turns_for_synthesis: usize, // Default: 3 — kurze Segmente überspringen
) -> SegmentSynthesisResult {
    let mut synthesized_chunks = Vec::new();
    let mut skipped_segments = 0;
    let mut errors = Vec::new();

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
                let err_msg = format!("Segment {i} synthesis failed: {e}");
                tracing::warn!(
                    segment_idx = i,
                    error = %e,
                    "Generative synthesis pass segment synthesis failed; skipping segment"
                );
                skipped_segments += 1;
                errors.push(err_msg);
            }
        }
    }

    SegmentSynthesisResult {
        synthesized_chunks,
        skipped_segments,
        errors,
    }
}

#[cfg(test)]
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

    #[tokio::test]
    async fn test_synthesis_pass_empty_segments_returns_empty() {
        let synthesizer = MockSynthesizer::new("test-model", false);
        let res = run_synthesis_pass(&[], &[], &synthesizer, 3).await;

        assert_eq!(res.synthesized_chunks.len(), 0);
        assert_eq!(res.skipped_segments, 0);
    }

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

    #[tokio::test]
    async fn test_synthesis_pass_segment_2_failure_captures_error() {
        struct MockSegment2FailingSynthesizer {
            model: String,
        }

        impl SegmentSynthesizer for MockSegment2FailingSynthesizer {
            fn synthesize_segment<'a>(
                &'a self,
                segment_texts: &'a [&'a str],
            ) -> BoxFuture<'a, Result<String>> {
                Box::pin(async move {
                    if segment_texts.iter().any(|t| t.contains("segment_2")) {
                        Err(contextra_types::ContextraError::Internal(
                            "Segment 2 failed synthesis".into(),
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

        let synthesizer = MockSegment2FailingSynthesizer {
            model: "test-model".to_string(),
        };

        let seg1 = TurnSegment {
            turn_ids: vec![DocId::new(1), DocId::new(2), DocId::new(3)],
            representative_embedding: vec![1.0, 0.0],
        };
        let seg2 = TurnSegment {
            turn_ids: vec![DocId::new(4), DocId::new(5), DocId::new(6)],
            representative_embedding: vec![0.0, 1.0],
        };

        let texts = vec![
            vec![
                "seg1_turn1".into(),
                "seg1_turn2".into(),
                "seg1_turn3".into(),
            ],
            vec![
                "segment_2_turn1".into(),
                "segment_2_turn2".into(),
                "segment_2_turn3".into(),
            ],
        ];

        let res = run_synthesis_pass(&[seg1, seg2], &texts, &synthesizer, 3).await;

        assert_eq!(res.synthesized_chunks.len(), 1);
        assert_eq!(res.skipped_segments, 1);
        assert_eq!(res.errors.len(), 1);
        assert!(res.errors[0].contains("Segment 1 synthesis failed"));
        assert!(res.errors[0].contains("Segment 2 failed synthesis"));
    }

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
