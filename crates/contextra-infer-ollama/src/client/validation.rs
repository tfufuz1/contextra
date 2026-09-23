use super::config::{MAX_BATCH_SIZE, MAX_TEXT_BYTES};
use contextra_core::{ContextraError, Result};

/// Escapes XML special characters in string inputs to prevent tag injection.
pub fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Halluzinationsprävention bei lokalen 7B-Modellen ohne Guard:
/// Lokale Sprachmodelle neigen ohne explizite Verhaltensanweisungen dazu, bei fehlendem Kontext Fakten frei zu erfinden (Halluzination).
/// Der Instruktionsblock wird VOR dem `<context>`-Block platziert, damit das Modell die Regeln zur Kontexttreue, Fallback-Antwort und Quellenangabe vor der Verarbeitung der Dokumentdaten liest.
///
/// Constructs a structurally isolated prompt encapsulating system instructions, RAG instructions, RAG context, and user query.
pub fn build_rag_prompt(system_context: &str, rag_context: &str, user_query: &str) -> String {
    format!(
        "<system>{}</system>\n<instructions>\nBeantworte die folgende Anfrage ausschließlich auf Basis der in &lt;context&gt; bereitgestellten Dokumentauszüge.\nVerwende kein Wissen außerhalb dieses Kontexts. Wenn die Antwort nicht eindeutig aus dem Kontext hervorgeht, antworte wörtlich: \"Diese Information ist in den importierten Dokumenten nicht enthalten.\"\nBelege jede sachliche Aussage am Satzende mit einem Verweis auf die Quelle in eckigen Klammern im Format [Quelle: &lt;Dateiname oder Chunk-Kennung&gt;], sofern diese Information in den Metadaten des Kontexts vorhanden ist.\n</instructions>\n<context>{}</context>\n<user_query>{}</user_query>",
        xml_escape(system_context),
        xml_escape(rag_context),
        xml_escape(user_query),
    )
}

/// Validates that input text length does not exceed `MAX_TEXT_BYTES`.
pub fn validate_text_length(text: &str, field_name: &str) -> Result<()> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(ContextraError::InvalidInput(format!(
            "Input field '{field_name}' size ({} bytes) exceeds maximum allowed limit of {MAX_TEXT_BYTES} bytes",
            text.len()
        )));
    }
    Ok(())
}

/// Validates that batch size does not exceed `MAX_BATCH_SIZE`.
pub fn validate_batch_size(count: usize) -> Result<()> {
    if count > MAX_BATCH_SIZE {
        return Err(ContextraError::InvalidInput(format!(
            "Batch size ({count}) exceeds maximum allowed limit of {MAX_BATCH_SIZE}"
        )));
    }
    Ok(())
}

/// Validates that model name does not contain invalid path traversal or whitespace control characters.
pub fn validate_model_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(ContextraError::InvalidInput(
            "Model name cannot be empty".into(),
        ));
    }
    if name.contains('/') || name.contains('\n') || name.contains('\r') {
        return Err(ContextraError::PolicyViolation(format!(
            "Model name '{name}' contains invalid characters"
        )));
    }
    Ok(())
}
