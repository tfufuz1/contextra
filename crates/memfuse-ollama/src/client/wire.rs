use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(super) struct ChatRequest {
    pub(super) model: String,
    pub(super) messages: Vec<ChatMessage>,
    pub(super) stream: bool,
}

#[derive(Serialize, Clone, Debug)]
pub(super) struct ChatMessage {
    pub(super) role: String,
    pub(super) content: String,
}

#[derive(Deserialize)]
pub(super) struct ChatStreamChunk {
    pub(super) message: Option<ChatMessageResponse>,
    #[serde(default)]
    pub(super) done: bool,
}

#[derive(Deserialize)]
pub(super) struct ChatMessageResponse {
    pub(super) content: String,
}

#[derive(Serialize)]
pub(super) struct GenerateRequest<'a> {
    pub(super) model: &'a str,
    pub(super) prompt: &'a str,
    pub(super) stream: bool,
}

#[derive(Deserialize)]
pub(super) struct GenerateResponse {
    pub(super) response: String,
}

#[derive(Serialize)]
pub(super) struct EmbedRequest<'a> {
    pub(super) model: &'a str,
    pub(super) prompt: &'a str,
}

#[derive(Deserialize)]
pub(super) struct EmbedResponse {
    pub(super) embedding: Vec<f32>,
}

/// Batch-Embedding Request für `/api/embed` (Ollama ≥ 0.3.9).
#[derive(Serialize)]
pub(super) struct BatchEmbedRequest<'a> {
    pub(super) model: &'a str,
    pub(super) input: Vec<&'a str>,
}

/// Batch-Embedding Response.
#[derive(Deserialize)]
pub(super) struct BatchEmbedResponse {
    pub(super) embeddings: Vec<Vec<f32>>,
}
