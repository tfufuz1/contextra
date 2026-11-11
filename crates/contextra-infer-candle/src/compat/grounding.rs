//! Brücken für GroundingValidator und ResponseGroundingValidator.

use crate::gasp::{ContextChunk as LocalContextChunk, GaspValidator};
use contextra_ports::{
    BoxFuture, GroundingAssessment, GroundingValidator, ResponseGroundingValidator,
};
use contextra_types::{ContextChunk, ContextraError, Result};

impl From<&ContextChunk> for LocalContextChunk {
    fn from(c: &ContextChunk) -> Self {
        LocalContextChunk {
            doc_id: c.doc_id.0,
            content: c.content.clone(),
            relevance: c.relevance,
            contextual_prefix: c.contextual_prefix.clone(),
        }
    }
}

impl ResponseGroundingValidator for GaspValidator {
    fn score_grounding(&self, response: &str, sources: &[&str]) -> Result<f32> {
        self.score_grounding_slices(response, sources)
            .map_err(Into::into)
    }
}

impl GroundingValidator for GaspValidator {
    fn validate_grounding<'a>(
        &'a self,
        response: &'a str,
        context_chunks: &'a [ContextChunk],
    ) -> BoxFuture<'a, Result<GroundingAssessment>> {
        Box::pin(async move {
            let local_chunks: Vec<LocalContextChunk> =
                context_chunks.iter().map(Into::into).collect();

            let final_score = self
                .validate_grounding_local(response, &local_chunks)
                .await
                .map_err(|e| -> ContextraError { e.into() })?;

            Ok(GroundingAssessment {
                score: final_score,
                is_grounded: true,
                reason: None,
            })
        })
    }
}

#[cfg(feature = "workspace")]
impl GaspValidator {
    /// Fassade nach H3: Akzeptiert `&[contextra_types::ContextChunk]` für Abwärtskompatibilität.
    pub fn compute_raw_grounding_score(
        &self,
        response: &str,
        context_chunks: &[ContextChunk],
    ) -> Result<f32> {
        let local_chunks: Vec<LocalContextChunk> = context_chunks.iter().map(Into::into).collect();
        self.compute_raw_grounding_score_local(response, &local_chunks)
            .map_err(Into::into)
    }
}
