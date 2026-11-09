//! Lokale Port-Traits für `contextra-infer-ollama` (B3-Konformität).

use crate::error::Result;
use std::future::Future;
use std::pin::Pin;

/// Typalias für gepinnte ge-boxte Futures ohne Fremdtypen.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Lokaler Trait für Embedding-Provider.
pub trait Embedder: Send + Sync {
    /// Liefert den Namen des Providers.
    fn provider_name(&self) -> &str;

    /// Erzeugt ein Embedding für den übergebenen Text.
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>>;

    /// Liefert die Embedding-Dimension des konfigurierten Modells.
    fn embedding_dim(&self) -> usize;

    /// Erzeugt Embeddings für einen Batch von Texten.
    fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, Result<Vec<Vec<f32>>>>;
}

/// Lokaler Trait für asynchrone Embedding-Provider.
pub trait AsyncEmbedder: Send + Sync {
    /// Liefert den Modellnamen des Providers.
    fn model_name(&self) -> &str;

    /// Erzeugt ein Embedding asynchron.
    fn embed_async<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>>;

    /// Erzeugt Embeddings für einen Batch asynchron.
    fn embed_batch_async<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, Result<Vec<Vec<f32>>>>;
}

/// Lokaler Trait für LLM-Textgeneratoren.
pub trait TextGenerator: Send + Sync {
    /// Generiert Text für den übergebenen Prompt.
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>>;
}
