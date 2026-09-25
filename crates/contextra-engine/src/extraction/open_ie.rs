// FILE-CONTEXT
// ZWECK: LLM-basierte OpenIE-Entitätsextraktion aus Texten.
// INVARIANTEN: Zero panic / unwrap / expect in non-test paths; Harte Limitierung via max_llm_calls_per_cycle.
// STAND: 2026-09-07

#![forbid(unsafe_code)]

use contextra_ports::LlmTextGenerator;
use contextra_types::{ContextraError, Result};
use super::types::{EntityExtractionConfig, ExtractedTriple};

/// Zerlegt den Eingabetext in einfache satz- oder zeilenbasierte Chunks.
/// Dies ist eine einfache Erstversion (ohne komplexe NLP-Grammatik) für die Entitätsextraktion.
fn chunk_text_simple(text: &str) -> Vec<&str> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current_start = 0;

    for (idx, ch) in trimmed.char_indices() {
        if ch == '.' || ch == '!' || ch == '?' || ch == '\n' {
            let chunk = trimmed[current_start..=idx].trim();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
            current_start = idx + ch.len_utf8();
        }
    }

    if current_start < trimmed.len() {
        let chunk = trimmed[current_start..].trim();
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
    }

    if chunks.is_empty() && !trimmed.is_empty() {
        chunks.push(trimmed);
    }

    chunks
}

/// Helper zum Bereinigen eventueller Markdown Code-Blocks (` ```json ... ``` `) in der LLM-Antwort.
fn sanitize_json_response(raw: &str) -> &str {
    let trimmed = raw.trim();
    let after_start = if let Some(stripped) = trimmed.strip_prefix("```json") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("```") {
        stripped
    } else {
        trimmed
    };

    if let Some(stripped) = after_start.strip_suffix("```") {
        stripped.trim()
    } else {
        after_start.trim()
    }
}

/// Extrahiert Wissens-Tripel (Subjekt, Prädikat, Objekt) aus dem gegebenen Text mittels LLM.
///
/// # Fehlerverhalten
/// - Wenn `config.enabled == false`, wird sofort eine leere Liste zurückgegeben.
/// - Wenn die Anzahl der Textchunks `config.max_llm_calls_per_cycle` überschreitet,
///   wird abgebrochen und ein `ContextraError::LimitExceeded` zurückgegeben.
/// - Fehlerhafte LLM-Antworten werden in `ContextraError::ParseError` oder `ContextraError::Serialization`
///   umgewandelt. Es wird niemals gepanict.
pub async fn extract_triples(
    text: &str,
    generator: &dyn LlmTextGenerator,
    config: &EntityExtractionConfig,
) -> Result<Vec<ExtractedTriple>> {
    if !config.enabled || text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let chunks = chunk_text_simple(text);
    if chunks.len() > config.max_llm_calls_per_cycle {
        return Err(ContextraError::limit_exceeded(
            config.max_llm_calls_per_cycle,
            format!(
                "Entity extraction input generated {} chunks, which exceeds max_llm_calls_per_cycle limit of {}",
                chunks.len(),
                config.max_llm_calls_per_cycle
            ),
        ));
    }

    let mut all_triples = Vec::new();

    for chunk in chunks {
        let prompt = format!(
            "Extract factual knowledge triples (subject, predicate, object) from the following text.\n\
             Return ONLY a strict JSON array of objects with keys: \"subject\" (string), \"predicate\" (string), \"object\" (string), \"confidence\" (float 0.0-1.0), \"source_span\" (optional array [start_char, end_char] or null).\n\n\
             Text:\n{}\n\n\
             JSON Output:",
            chunk
        );

        let response = generator.generate(&prompt).await?;
        let clean_json = sanitize_json_response(&response);

        let triples: Vec<ExtractedTriple> = serde_json::from_str(clean_json).map_err(|e| {
            ContextraError::ParseError(format!(
                "Failed to parse LLM entity extraction response as JSON: {e}. Raw response was: {response:?}"
            ))
        })?;

        for triple in triples {
            if triple.confidence >= config.min_confidence {
                all_triples.push(triple);
            }
        }
    }

    Ok(all_triples)
}
