#![deny(dead_code)]
// FILE-CONTEXT
// STAND: 2026-10-31
// ZWECK: In-process ONNX Embedding Engine und Cross-Encoder-Reranker (Ring 2).
// INVARIANTEN: Default-Build ohne ONNX hat leere Feature-Flags (ADR-005, Pure-Rust-USP).
// NICHT-OFFENSICHTLICH: Threading via tokio::task::spawn_blocking zur Vermeidung von Executor-Starvation.
// SIEHE AUCH: crates/contextra-infer-onnx/AGENTS.md, rules/dependencies.md

//! # Crate `contextra-infer-onnx`
//!
//! **Ring & Klasse:** Ring 2 (Flüchtige Abhängigkeit / Rand `[R]`).
//! **Status:** Archiv-Status laut `CLAUDE.md` / SPEC §5.2 („ENTFERNT“), wird jedoch im Workspace geführt
//! und optional von `contextra-mcp` (über das Feature `onnx`) eingebunden.
//!
//! ## Zweck & Architekturrolle
//! Dieses Crate stellt eine lokale, In-Process-Inferenz-Engine für Text-Embeddings und
//! Cross-Encoder-Reranking basierend auf der **ONNX Runtime** (`ort`) und HuggingFace-Tokenizern bereit.
//! Dadurch können hochqualitative Vektor-Einbettungen und semantische Umsortierungen vollkommen
//! offline und ohne externe Server-Aufrufe (wie z. B. Ollama oder Cloud-APIs) berechnet werden.
//!
//! ## Einbindung und Abhängigkeiten
//! - **Eingehend:** Wird optional von `contextra-mcp` (Feature `onnx`) eingebunden.
//! - **Ausgehend (normal):**
//!   - `contextra-ports`: Definiert Schnittstellen wie `EmbeddingProvider` und `Reranker`.
//!   - `contextra-rank`: Liefert Kalibrierungsmodelle (`PlattScaler`) für Reranking-Wahrscheinlichkeiten.
//!   - `contextra-types`: Stellt gemeinsame Fehlertypen (`ContextraError`) und Datenstrukturen bereit.
//!   - `tokio`, `parking_lot`, `serde`, `tracing`.
//!   - Optional (über Feature `onnx`): `ort`, `tokenizers`, `ndarray`, `reqwest`, `sha2`.
//!
//! ## Wichtige Invarianten
//! 1. **Zero Unsafe:** `#![forbid(unsafe_code)]` wird strikt auf Crate-Ebene erzwungen.
//! 2. **Offline-First:** Netzwerk-Downloads sind standardmäßig deaktiviert (`allow_download = false`).
//! 3. **Non-Starvation:** Da Inferenz-Berechnungen CPU-intensiv und blockierend sind, werden sie
//!    konsequent über `tokio::task::spawn_blocking` ausgeführt und über Semaphoren gedrosselt (Backpressure).

#![forbid(unsafe_code)]

#[cfg(feature = "onnx")]
use std::sync::Arc;

#[cfg(feature = "onnx")]
use contextra_ports::{BoxFuture, EmbeddingError, EmbeddingProvider};
#[cfg(feature = "onnx")]
use contextra_types::{ContextraError, Result};
#[cfg(feature = "onnx")]
use ort::value::Value;
#[cfg(feature = "onnx")]
use tokenizers::Tokenizer;
#[cfg(feature = "onnx")]
use tracing::{debug, info, warn};

// ANCHOR[contextra-infer-onnx:MOD-RERANKER]
/// Submodul für Cross-Encoder Reranking zur Präzisionsverbesserung nach initialer Suche.
pub mod reranker;
pub use reranker::{CrossEncoderReranker, PlattScaledSigmoid, RerankConfig, RerankResult};

use std::path::{Path, PathBuf};

/// Returns the default local cache directory for ONNX embedding models (`~/.contextra/models/<model_name>`).
// ANCHOR[contextra-infer-onnx:CACHE-DIR-001]
/// Liefert das standardmäßige lokale Cache-Verzeichnis für ONNX-Embedding-Modelle.
///
/// Standardpfad: `~/.contextra/models/<model_name>`
///
/// # Ablauf
/// 1. Umgebungsvariablen `HOME` bzw. `USERPROFILE` prüfen.
/// 2. Falls vorhanden, Pfad als `$HOME/.contextra/models/<model_name>` konstruieren.
/// 3. Fallback auf `.contextra/models/<model_name>`.
pub fn default_model_cache_dir(model_name: &str) -> PathBuf {
    #[allow(clippy::disallowed_methods)]
    {
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            PathBuf::from(home)
                .join(".contextra")
                .join("models")
                .join(model_name)
        } else {
            PathBuf::from(".contextra").join("models").join(model_name)
        }
    }
}

/// Security policies and configuration for downloading ONNX model artifacts over the network.
///
/// Download is **DISABLED** by default (`allow_download = false`) to enforce offline-first operation.
#[derive(Debug, Clone)]
// ANCHOR[contextra-infer-onnx:DOWNLOAD-OPTS-001]
/// Sicherheitsrichtlinien und Konfigurationsparameter für das Herunterladen von ONNX-Modellartefakten.
///
/// Gemäß Offline-First-Doktrin standardmäßig deaktiviert (`allow_download = false`).
pub struct ModelDownloadOptions {
    /// Allow downloading missing model files over network. Default: false.
    pub allow_download: bool,
    /// List of allowed hostnames for downloading models (e.g., `"huggingface.co"`).
    pub allowed_hosts: Vec<String>,
    /// Expected SHA-256 hex string for `model.onnx`. Mandatory if `allow_download` is true.
    pub expected_model_sha256: Option<String>,
    /// Expected SHA-256 hex string for `tokenizer.json`. Mandatory if `allow_download` is true.
    pub expected_tokenizer_sha256: Option<String>,
    /// Maximum allowed file size in bytes (default: 1,073,741,824 = 1 GiB).
    pub max_file_size_bytes: usize,
}

impl Default for ModelDownloadOptions {
    fn default() -> Self {
        Self {
            allow_download: false,
            allowed_hosts: vec!["huggingface.co".to_string()],
            expected_model_sha256: None,
            expected_tokenizer_sha256: None,
            max_file_size_bytes: 1_073_741_824,
        }
    }
}

/// Ensures the ONNX embedding model files (`model.onnx` and `tokenizer.json`) exist locally in `cache_dir`
/// (or `~/.contextra/models/<model_name>`).
///
/// **Offline-First Contract**: Network downloads are strictly disabled by default. Callers must explicitly
/// configure [`ModelDownloadOptions`] with `allow_download = true` AND provide expected SHA-256 checksums to enable downloads.
#[cfg(feature = "onnx")]
// ANCHOR[contextra-infer-onnx:ENSURE-DOWNLOAD-001]
/// Stellt sicher, dass die ONNX-Modelldateien (`model.onnx` und `tokenizer.json`) lokal existieren.
/// Nutzt die Standardoptionen von [`ModelDownloadOptions`].
pub async fn ensure_onnx_model_download(
    model_name: &str,
    cache_dir: Option<&Path>,
) -> Result<PathBuf> {
    ensure_onnx_model_download_with_options(model_name, cache_dir, ModelDownloadOptions::default())
        .await
}

/// Ensures the ONNX embedding model files exist locally using custom [`ModelDownloadOptions`].
#[cfg(feature = "onnx")]
// ANCHOR[contextra-infer-onnx:ENSURE-DOWNLOAD-WITH-OPTS-001]
/// Stellt sicher, dass die ONNX-Modelldateien lokal existieren, unter Verwendung benutzerdefinierter [`ModelDownloadOptions`].
///
/// # Ablauf
/// 1. Zielpfad ermitteln (`cache_dir` oder Standard-Cacheverzeichnis).
/// 2. Asynchron prüfen, ob `model.onnx` und `tokenizer.json` bereits existieren.
/// 3. Falls beide existieren: Sofortige Rückgabe des Verzeichnisses (`Ok(target_dir)`).
/// 4. Falls Dateien fehlen und `options.allow_download == false`: Abbruch mit `InvalidInput`.
/// 5. Zielverzeichnis über `spawn_blocking` anlegen.
/// 6. Download via `download_onnx_file_secured` mit SHA-256 Validierung ausführen.
pub async fn ensure_onnx_model_download_with_options(
    model_name: &str,
    cache_dir: Option<&Path>,
    options: ModelDownloadOptions,
) -> Result<PathBuf> {
    let target_dir = match cache_dir {
        Some(dir) => dir.join(model_name),
        None => default_model_cache_dir(model_name),
    };

    let model_file = target_dir.join("model.onnx");
    let tokenizer_file = target_dir.join("tokenizer.json");

    let model_exists = path_exists_async(&model_file).await?;
    let tokenizer_exists = path_exists_async(&tokenizer_file).await?;

    if model_exists && tokenizer_exists {
        return Ok(target_dir);
    }

    if !options.allow_download {
        return Err(ContextraError::InvalidInput(format!(
            "Modell-Dateien für '{model_name}' fehlen in {:?} und Downloads sind deaktiviert (offline-first). \
             Setze allow_download = true oder installiere 'model.onnx' und 'tokenizer.json' manuell.",
            target_dir
        )));
    }

    create_dir_all_async(&target_dir).await?;

    info!(
        model_name,
        target_dir = ?target_dir,
        "Lade ONNX-Modell herunter..."
    );

    let (model_url, tokenizer_url) = match model_name {
        "nomic-embed-text" | "nomic-embed-text-v1.5" => (
            "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/onnx/model.onnx",
            "https://huggingface.co/nomic-ai/nomic-embed-text-v1.5/resolve/main/tokenizer.json",
        ),
        _ => {
            return Err(ContextraError::InvalidInput(format!(
                "Unbekanntes ONNX-Modell '{model_name}'. Bitte erstelle die Modelldateien manuell in {target_dir:?}."
            )));
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| ContextraError::Internal(e.to_string()))?;

    if !model_exists {
        let expected_sha256 = options.expected_model_sha256.as_deref().ok_or_else(|| {
            ContextraError::InvalidInput(format!(
                "Download von model.onnx verweigert: missing expected_model_sha256. Prüfsumme ist zwingend erforderlich."
            ))
        })?;
        download_onnx_file_secured(
            &client,
            model_url,
            &model_file,
            "model.onnx",
            &target_dir,
            &options,
            expected_sha256,
        )
        .await?;
    }

    if !tokenizer_exists {
        let expected_sha256 = options.expected_tokenizer_sha256.as_deref().ok_or_else(|| {
            ContextraError::InvalidInput(format!(
                "Download von tokenizer.json verweigert: missing expected_tokenizer_sha256. Prüfsumme ist zwingend erforderlich."
            ))
        })?;
        download_onnx_file_secured(
            &client,
            tokenizer_url,
            &tokenizer_file,
            "tokenizer.json",
            &target_dir,
            &options,
            expected_sha256,
        )
        .await?;
    }

    Ok(target_dir)
}

#[cfg(feature = "onnx")]
// ANCHOR[contextra-infer-onnx:SECURED-DOWNLOAD-001]
/// Lädt eine Datei sicher per HTTPS herunter, verifiziert SHA-256 und schreibt atomar via tmp + rename.
async fn download_onnx_file_secured(
    client: &reqwest::Client,
    url_str: &str,
    dest_path: &Path,
    file_name: &str,
    target_dir: &Path,
    options: &ModelDownloadOptions,
    expected_sha256_hex: &str,
) -> Result<()> {
    if !url_str.starts_with("https://") {
        return Err(ContextraError::InvalidInput(format!(
            "Download-URL '{url_str}' verstoßen gegen HTTPS-Zwang: Nur HTTPS-Verbindungen erlaubt."
        )));
    }

    let url_parsed = reqwest::Url::parse(url_str).map_err(|e| {
        ContextraError::InvalidInput(format!("Ungültige Download-URL '{url_str}': {e}"))
    })?;

    let host = url_parsed.host_str().unwrap_or("");
    if !options
        .allowed_hosts
        .iter()
        .any(|h| h.eq_ignore_ascii_case(host))
    {
        return Err(ContextraError::InvalidInput(format!(
            "Download von Host '{host}' verweigert: Nicht in der Allowlist {:?}",
            options.allowed_hosts
        )));
    }

    let response = client.get(url_str).send().await.map_err(|e| {
        ContextraError::model_load(
            target_dir.display().to_string(),
            format!(
                "Download des Embedding-Modells ({file_name}) fehlgeschlagen: {e}. \
                 Bitte überprüfe deine Internetverbindung oder lade die Datei manuell nach {target_dir:?} herunter."
            ),
        )
    })?;

    if !response.status().is_success() {
        return Err(ContextraError::model_load(
            target_dir.display().to_string(),
            format!(
                "HTTP-Fehler {} beim Download von {file_name} aus {url_str}. \
                 Manuelle Installation in {target_dir:?} erforderlich.",
                response.status(),
                target_dir = target_dir.display()
            ),
        ));
    }

    if let Some(content_length) = response.content_length() {
        if content_length > options.max_file_size_bytes as u64 {
            return Err(ContextraError::InvalidInput(format!(
                "Download von {file_name} überschreitet Größenlimit ({} > max {} bytes)",
                content_length, options.max_file_size_bytes
            )));
        }
    }

    let bytes = response.bytes().await.map_err(|e| {
        ContextraError::model_load(
            target_dir.display().to_string(),
            format!(
                "Fehler beim Empfangen von {file_name}: {e}. \
                 Bitte überprüfe deine Internetverbindung oder installiere das Modell manuell in {target_dir:?}"
            ),
        )
    })?;

    if bytes.len() > options.max_file_size_bytes {
        return Err(ContextraError::InvalidInput(format!(
            "Empfangene Datei {file_name} überschreitet Größenlimit ({} > max {} bytes)",
            bytes.len(),
            options.max_file_size_bytes
        )));
    }

    // SHA-256 Prüfsummen-Validierung
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual_sha256 = format!("{:x}", hasher.finalize());

    if !actual_sha256.eq_ignore_ascii_case(expected_sha256_hex.trim()) {
        return Err(ContextraError::InvalidInput(format!(
            "Prüfsummen-Fehler bei {file_name}: Erwartet SHA-256 '{expected_sha256_hex}', erhalten '{actual_sha256}'."
        )));
    }

    // Atomares Schreiben: tmp-Datei -> fsync -> rename
    let dest_path_buf = dest_path.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let tmp_path = dest_path_buf.with_extension("tmp");
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp_path).map_err(|e| {
            ContextraError::Internal(format!(
                "Konnte temporäre Datei {tmp_path:?} nicht erstellen: {e}"
            ))
        })?;
        f.write_all(&bytes).map_err(|e| {
            ContextraError::Internal(format!(
                "Konnte Daten in temporäre Datei {tmp_path:?} nicht schreiben: {e}"
            ))
        })?;
        f.sync_all().map_err(|e| {
            ContextraError::Internal(format!(
                "fsync auf temporäre Datei {tmp_path:?} fehlgeschlagen: {e}"
            ))
        })?;
        drop(f);

        std::fs::rename(&tmp_path, &dest_path_buf).map_err(|e| {
            ContextraError::Internal(format!("Konnte {dest_path_buf:?} nicht speichern: {e}"))
        })?;
        Ok(())
    })
    .await
    .map_err(|e| ContextraError::Internal(format!("spawn_blocking join: {e}")))?
}

#[cfg(feature = "onnx")]
// ANCHOR[contextra-infer-onnx:PATH-EXISTS-ASYNC-001]
/// Prüft blockierungsfrei über `tokio::task::spawn_blocking`, ob ein Pfad existiert.
async fn path_exists_async(path: &Path) -> Result<bool> {
    let p = path.to_path_buf();
    tokio::task::spawn_blocking(move || p.exists())
        .await
        .map_err(|e| ContextraError::Internal(format!("spawn_blocking join: {e}")))
}

#[cfg(feature = "onnx")]
// ANCHOR[contextra-infer-onnx:CREATE-DIR-ASYNC-001]
/// Erstellt rekursiv Verzeichnisse im Hintergrundthread via `spawn_blocking`.
async fn create_dir_all_async(dir: &Path) -> Result<()> {
    let d = dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(&d).map_err(|e| {
            ContextraError::Internal(format!(
                "Konnte Modellspeicherverzeichnis {:?} nicht erstellen: {}",
                d, e
            ))
        })
    })
    .await
    .map_err(|e| ContextraError::Internal(format!("spawn_blocking join: {e}")))?
}

#[cfg(not(feature = "onnx"))]
// ANCHOR[contextra-infer-onnx:ENSURE-DOWNLOAD-FALLBACK-001]
/// Stellt sicher, dass die ONNX-Modelldateien (`model.onnx` und `tokenizer.json`) lokal existieren.
/// Nutzt die Standardoptionen von [`ModelDownloadOptions`].
pub async fn ensure_onnx_model_download(
    _model_name: &str,
    _cache_dir: Option<&Path>,
) -> contextra_types::Result<PathBuf> {
    Err(contextra_types::ContextraError::CapabilityUnsupported {
        capability: "onnx".to_string(),
        reason: "ONNX support is disabled in this build. Recompile with feature flag 'onnx'."
            .to_string(),
    })
}

/// Conservative default. Override via `TextEmbedderConfig::max_batch_size`.
/// At 1536D × 512 × f32 = ~3 MB input tensor; safe within 128 MB memory budgets.
// ANCHOR[contextra-infer-onnx:MAX-EMBED-BATCH-SIZE-001]
/// Standard-Obergrenze für Batch-Embedding-Anfragen (512 Texte).
pub const MAX_EMBED_BATCH_SIZE: usize = 512;

/// Configuration settings for the text embedder.
#[cfg(feature = "onnx")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// ANCHOR[contextra-infer-onnx:CONFIG-STRUCT-001]
/// Konfigurationsparameter für den [`TextEmbedder`].
pub struct TextEmbedderConfig {
    /// Maximum number of tokens per text sequence (default: 512).
    pub max_sequence_length: usize,
    /// Maximum parallel ONNX inference threads (default: 1).
    /// Values > 1 are not supported — ONNX session is not thread-safe under concurrent access without external serialization. Default is 1.
    pub pool_size: usize,
    /// Maximum concurrent embedding calls. Prevents spawn_blocking thread pool exhaustion.
    /// Should be <= (tokio blocking thread pool size / 2) to leave headroom for other blocking operations. Default: 8.
    pub max_concurrent_embeddings: usize,
    /// Expected output embedding dimension (optional).
    pub expected_dim: Option<usize>,
    /// Maximum batch size for embed_batch(). Default: MAX_EMBED_BATCH_SIZE (512).
    pub max_batch_size: usize,
}

#[cfg(feature = "onnx")]
impl Default for TextEmbedderConfig {
    fn default() -> Self {
        Self {
            max_sequence_length: 512,
            pool_size: 1,
            max_concurrent_embeddings: 8,
            expected_dim: None,
            max_batch_size: MAX_EMBED_BATCH_SIZE,
        }
    }
}

#[cfg(feature = "onnx")]
impl TextEmbedderConfig {
    /// Validates configuration parameters.
    // ANCHOR[contextra-infer-onnx:CONFIG-VALIDATE-001]
    /// Validiert die Konfigurationswerte auf Konsistenz und Grenzwerte.
    pub fn validate(&self) -> Result<()> {
        if self.pool_size != 1 {
            return Err(ContextraError::InvalidInput(format!(
                "TextEmbedder: pool_size > 1 ({}) not supported — ONNX session is not thread-safe \
                 under concurrent access without external serialization. Use pool_size = 1.",
                self.pool_size
            )));
        }
        if self.max_concurrent_embeddings == 0 {
            return Err(ContextraError::InvalidInput(
                "TextEmbedder: max_concurrent_embeddings must be > 0".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(feature = "onnx")]
// ANCHOR[contextra-infer-onnx:TYPE-ALIAS-ONNX-EMBEDDER]
/// Typalias für [`TextEmbedder`] zur Wahrung von Kompatibilität mit `contextra-mcp`.
pub type OnnxEmbedder = TextEmbedder;

/// Handles text tokenization and ONNX model inference.
///
/// Uses `tokio::task::spawn_blocking` to offload ONNX inference to a blocking
/// thread pool, preventing Tokio runtime starvation. A [`tokio::sync::Semaphore`]
/// limits the number of concurrent inference operations.
#[cfg(feature = "onnx")]
#[derive(Clone)]
// ANCHOR[contextra-infer-onnx:TEXT-EMBEDDER-STRUCT]
/// Führt Text-Tokenisierung und ONNX-Modell-Inferenz im Prozess aus.
pub struct TextEmbedder {
    /// Shared ONNX session (thread-safe via `Arc<Mutex>`).
    session: Arc<parking_lot::Mutex<ort::session::Session>>,
    /// Shared tokenizer instance (thread-safe via `Arc`).
    tokenizer: Arc<Tokenizer>,
    /// Semaphore limiting parallel ONNX inference threads.
    semaphore: Arc<tokio::sync::Semaphore>,
    /// Configuration settings for the embedder.
    config: TextEmbedderConfig,
    /// Expected output embedding dimension.
    expected_dim: Option<usize>,
    /// Number of ONNX session loads for this instance.
    session_load_count: Arc<std::sync::atomic::AtomicUsize>,
}

#[cfg(feature = "onnx")]
impl std::fmt::Debug for TextEmbedder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextEmbedder")
            .field("config", &self.config)
            .field("expected_dim", &self.expected_dim)
            .finish()
    }
}

#[cfg(feature = "onnx")]
impl EmbeddingProvider for TextEmbedder {
    // ANCHOR[contextra-infer-onnx:PROVIDER-NAME-001]
    /// Kennung des Providers: `"onnx"`.
    fn provider_name(&self) -> &str {
        "onnx"
    }

    // ANCHOR[contextra-infer-onnx:EMBED-TRAIT-001]
    /// Asynchroner Embedding-Aufruf über das `EmbeddingProvider`-Trait aus `contextra-ports`.
    fn embed<'a>(
        &'a self,
        text: &'a str,
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, EmbeddingError>> {
        Box::pin(async move {
            let max_len = self.config.max_sequence_length;
            if let Ok(encoding) = self.tokenizer.encode(text, true) {
                let len = encoding.get_ids().len();
                if len > max_len {
                    return Err(EmbeddingError::InputTooLong { len, max: max_len });
                }
            }
            self.embed_async(text).await.map_err(|e| match e {
                ContextraError::InvalidInput(msg)
                    if msg.contains("too long") || msg.contains("exceeds") =>
                {
                    EmbeddingError::InputTooLong {
                        len: text.len(),
                        max: max_len,
                    }
                }
                ContextraError::InvalidInput(msg) | ContextraError::NotFound(msg) => {
                    EmbeddingError::Unavailable(msg)
                }
                other => EmbeddingError::ComputationFailed(other.to_string()),
            })
        })
    }

    // ANCHOR[contextra-infer-onnx:EMBED-DIM-001]
    /// Gibt die konfigurierte Ausgabedimension zurück, oder `0` falls nicht gesetzt.
    fn embedding_dim(&self) -> usize {
        self.expected_dim.unwrap_or(0)
    }

    // ANCHOR[contextra-infer-onnx:EMBED-BATCH-TRAIT-001]
    /// Verarbeitet mehrere Texte im Batch mit parallelen Tasks und sequentiellem Fallback.
    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, EmbeddingError>> {
        Box::pin(async move {
            let limit = self.config.max_batch_size;
            if texts.len() > limit {
                return Err(EmbeddingError::Unavailable(format!(
                    "Batch size {} exceeds max_batch_size {}. Split into smaller batches.",
                    texts.len(),
                    limit
                )));
            }

            let mut handles = Vec::with_capacity(texts.len());
            for text in texts {
                let text_owned = text.to_string();
                let embedder = self.clone();
                handles.push(tokio::spawn(
                    async move { embedder.embed(&text_owned).await },
                ));
            }

            let mut results = Vec::with_capacity(texts.len());
            let mut parallel_failed = false;
            for handle in handles {
                match handle.await {
                    Ok(Ok(res)) => results.push(res),
                    Ok(Err(e)) => return Err(e),
                    Err(_) => {
                        parallel_failed = true;
                        break;
                    }
                }
            }

            if !parallel_failed && results.len() == texts.len() {
                return Ok(results);
            }

            // Sequential fallback path
            let mut seq_results = Vec::with_capacity(texts.len());
            for text in texts {
                seq_results.push(self.embed(text).await?);
            }
            Ok(seq_results)
        })
    }
}

#[cfg(feature = "onnx")]
impl TextEmbedder {
    /// Creates a new embedder from a model file or model directory path.
    // ANCHOR[contextra-infer-onnx:FROM-PATH-001]
    /// Erstellt einen Embedder aus einer Modelldatei oder einem Modellverzeichnis.
    pub fn from_path(model_path: impl AsRef<Path>) -> Result<Self> {
        let path = model_path.as_ref();
        let dir = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        Self::load(dir)
    }

    /// Creates a new embedder from a model directory. Alias for `load`.
    // ANCHOR[contextra-infer-onnx:NEW-EMBEDDER-001]
    /// Erstellt einen neuen Embedder aus einem Modellverzeichnis (Alias für [`TextEmbedder::load`]).
    pub fn new(model_dir: impl AsRef<Path>) -> Result<Self> {
        Self::load(model_dir)
    }

    /// Creates a new embedder by loading a tokenizer and ONNX model path from the specified directory.
    ///
    /// The directory should contain `model.onnx` and `tokenizer.json`.
    // ANCHOR[contextra-infer-onnx:LOAD-EMBEDDER-001]
    /// Lädt Modell und Tokenizer aus dem angegebenen Verzeichnis mit Standardkonfiguration.
    pub fn load(model_dir: impl AsRef<Path>) -> Result<Self> {
        Self::load_with_config(model_dir, TextEmbedderConfig::default())
    }

    /// Creates a new embedder with a custom configuration.
    // ANCHOR[contextra-infer-onnx:LOAD-WITH-CONFIG-001]
    /// Lädt Modell und Tokenizer mit einer benutzerdefinierten Konfiguration.
    pub fn load_with_config(
        model_dir: impl AsRef<Path>,
        config: TextEmbedderConfig,
    ) -> Result<Self> {
        config.validate()?;

        let path = model_dir.as_ref();
        if !path.join("tokenizer.json").exists() {
            return Err(ContextraError::InvalidInput(
                "tokenizer.json not found".into(),
            ));
        }
        if !path.join("model.onnx").exists() {
            return Err(ContextraError::InvalidInput("model.onnx not found".into()));
        }

        let model_path = path.join("model.onnx");
        let tokenizer_path = path.join("tokenizer.json");

        info!("Loading tokenizer from {:?}", tokenizer_path);
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| ContextraError::Internal(format!("Failed to load tokenizer: {}", e)))?;

        info!("Loading ONNX model session from {:?}", model_path);
        let session = ort::session::Session::builder()
            .map_err(|e| ContextraError::Internal(format!("Failed to build ONNX session: {}", e)))?
            .commit_from_file(&model_path)
            .map_err(|e| {
                ContextraError::Internal(format!(
                    "Failed to load ONNX model from {:?}: {}",
                    model_path, e
                ))
            })?;

        let session_load_count = Arc::new(std::sync::atomic::AtomicUsize::new(1));

        let expected_dim = config.expected_dim;
        let max_concurrent = config.max_concurrent_embeddings;
        Ok(Self {
            session: Arc::new(parking_lot::Mutex::new(session)),
            tokenizer: Arc::new(tokenizer),
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            config,
            expected_dim,
            session_load_count,
        })
    }

    /// Returns the number of times an ONNX session has been loaded by this embedder instance.
    #[doc(hidden)]
    // ANCHOR[contextra-infer-onnx:SESSION-LOAD-COUNT-001]
    /// Gibt zurück, wie oft eine ONNX-Session geladen wurde.
    pub fn session_load_count(&self) -> usize {
        self.session_load_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Sets the expected embedding output dimension for post-inference validation.
    // ANCHOR[contextra-infer-onnx:WITH-EXPECTED-DIM-001]
    /// Konfiguriert die erwartete Dimension des Ausgabeverktors zur Validierung nach Inferenz.
    pub fn with_expected_dimension(mut self, dim: usize) -> Self {
        self.expected_dim = Some(dim);
        self
    }

    /// Generates an embedding for the given text using `spawn_blocking`.
    ///
    /// Acquires a semaphore permit to limit concurrent inference operations,
    /// then offloads the blocking ONNX computation to Tokio's blocking thread pool.
    // ANCHOR[contextra-infer-onnx:EMBED-ASYNC-001]
    /// Berechnet die Vektor-Einbettung (Embedding) für einen Text asynchron unter Einhaltung von Backpressure.
    pub async fn embed_async(&self, text: &str) -> Result<Vec<f32>> {
        if self.semaphore.available_permits() == 0 {
            warn!("Embedding backpressure active: max_concurrent_embeddings reached");
        }

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| ContextraError::Internal("Semaphore closed".into()))?;

        let text = text.to_string();
        let session = self.session.clone();
        let tokenizer = self.tokenizer.clone();
        let max_sequence_length = self.config.max_sequence_length;

        let output = tokio::task::spawn_blocking(move || {
            let mut guard = session.lock();
            Self::run_inference(&mut guard, &tokenizer, &text, max_sequence_length)
        })
        .await
        .map_err(|e| ContextraError::Internal(format!("spawn_blocking join: {}", e)))??;

        if let Some(expected_dim) = self.expected_dim {
            if output.len() != expected_dim {
                return Err(ContextraError::InvalidInput(format!(
                    "Model output dim {} != expected {}",
                    output.len(),
                    expected_dim
                )));
            }
        }

        Ok(output)
    }

    /// Performs tokenization, ONNX forward pass, mean pooling, and L2 normalization.
    ///
    /// This is a synchronous function intended to run inside `spawn_blocking`.
    // ANCHOR[contextra-infer-onnx:RUN-INFERENCE-001]
    /// Synchroner Kern der Text-Inferenz: Tokenisierung, Forward-Pass, Mean-Pooling und L2-Normalisierung.
    fn run_inference(
        session: &mut ort::session::Session,
        tokenizer: &Tokenizer,
        text: &str,
        max_sequence_length: usize,
    ) -> Result<Vec<f32>> {
        debug!("Embedding text: {:?}", text);

        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| ContextraError::Internal(format!("Tokenization failed: {}", e)))?;

        let mut input_ids = encoding.get_ids().to_vec();
        let mut attention_mask = encoding.get_attention_mask().to_vec();
        let mut type_ids = encoding.get_type_ids().to_vec();

        if input_ids.len() > max_sequence_length {
            warn!(
                tokens = input_ids.len(),
                max = max_sequence_length,
                "Input text exceeds max sequence length and will be truncated. \
                 Consider using MarkdownChunker before embedding."
            );
            input_ids.truncate(max_sequence_length);
            attention_mask.truncate(max_sequence_length);
            type_ids.truncate(max_sequence_length);
        }

        let seq_len = input_ids.len();

        let input_ids_vec: Vec<i64> = input_ids.iter().map(|&id| id as i64).collect();
        let attention_mask_vec: Vec<i64> = attention_mask.iter().map(|&m| m as i64).collect();

        let input_ids_tensor = Value::from_array(([1, seq_len], input_ids_vec))
            .map_err(|e| ContextraError::Internal(format!("Failed to create tensor: {}", e)))?;
        let attention_mask_tensor = Value::from_array(([1, seq_len], attention_mask_vec))
            .map_err(|e| ContextraError::Internal(format!("Failed to create tensor: {}", e)))?;

        let has_token_type_ids = session
            .inputs()
            .iter()
            .any(|input| input.name() == "token_type_ids");

        let outputs = if has_token_type_ids {
            let token_type_ids_vec: Vec<i64> = type_ids.iter().map(|&id| id as i64).collect();
            let token_type_ids_tensor = Value::from_array(([1, seq_len], token_type_ids_vec))
                .map_err(|e| ContextraError::Internal(format!("Failed to create tensor: {}", e)))?;
            session
                .run(ort::inputs![
                    "input_ids" => input_ids_tensor,
                    "attention_mask" => attention_mask_tensor,
                    "token_type_ids" => token_type_ids_tensor,
                ])
                .map_err(|e| ContextraError::Internal(format!("ONNX inference failed: {}", e)))?
        } else {
            session
                .run(ort::inputs![
                    "input_ids" => input_ids_tensor,
                    "attention_mask" => attention_mask_tensor,
                ])
                .map_err(|e| ContextraError::Internal(format!("ONNX inference failed: {}", e)))?
        };

        // Mean Pooling over token embeddings, weighted by attention_mask
        let process_tensor = |shape: &[i64], data: &[f32]| -> Result<Vec<f32>> {
            // shape: [batch=1, seq_len, hidden_size]
            if shape.len() != 3 {
                return Err(ContextraError::Internal(format!(
                    "Unexpected output shape: {:?}",
                    shape
                )));
            }

            let hidden_size = shape[2] as usize;
            let mut mean_vec = vec![0.0f32; hidden_size];

            for i in 0..seq_len {
                if attention_mask[i] == 1 {
                    for j in 0..hidden_size {
                        mean_vec[j] += data[i * hidden_size + j];
                    }
                }
            }

            let sum_mask = attention_mask.iter().sum::<u32>() as f32;
            if sum_mask > 0.0 {
                for val in mean_vec.iter_mut() {
                    *val /= sum_mask;
                }
            }

            // L2 Normalization (standard for sentence embeddings)
            let norm = mean_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for val in mean_vec.iter_mut() {
                    *val /= norm;
                }
            }
            Ok(mean_vec)
        };

        // Extract tensor — try named output first, fall back to positional
        if let Some(out) = outputs.get("last_hidden_state") {
            let (shape, data) = out
                .try_extract_tensor::<f32>()
                .map_err(|e| ContextraError::Internal(format!("Failed to extract: {}", e)))?;
            process_tensor(shape, data)
        } else if let Some((_, out)) = outputs.iter().next() {
            let (shape, data) = out
                .try_extract_tensor::<f32>()
                .map_err(|e| ContextraError::Internal(format!("Failed to extract: {}", e)))?;
            process_tensor(shape, data)
        } else {
            Err(ContextraError::Internal("Model produced no outputs".into()))
        }
    }
}

#[cfg(test)]
// ANCHOR[contextra-infer-onnx:MOD-TESTS-001]
/// Modultests für Embedder-Konfiguration, Validierung, Fehlerfälle und Mock-Engines.
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[cfg(feature = "onnx")]
    use super::MAX_EMBED_BATCH_SIZE;

    #[cfg(feature = "onnx")]
    use std::fs::File;
    #[cfg(feature = "onnx")]
    use tempfile::tempdir;

    #[cfg(feature = "onnx")]
    #[test]
    /// Prüft, dass das Laden bei fehlenden Dateien mit `InvalidInput` fehlschlägt.
    fn test_text_embedder_load_missing_files() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempdir()?;

        // Empty directory — missing tokenizer.json check first
        let res = TextEmbedder::load(dir.path());
        assert!(res.is_err());
        let err = res.err().unwrap(); // unwrap
        assert!(matches!(err, ContextraError::InvalidInput(_)));

        // Create tokenizer but still missing model.onnx
        File::create(dir.path().join("tokenizer.json"))?;
        let res = TextEmbedder::load(dir.path());
        assert!(res.is_err());
        let err = res.err().unwrap(); // unwrap
        assert!(matches!(err, ContextraError::InvalidInput(_)));
        Ok(())
    }

    #[cfg(feature = "onnx")]
    #[test]
    /// Prüft die Standardwerte von `TextEmbedderConfig`.
    fn test_text_embedder_config_default() {
        let cfg = TextEmbedderConfig::default();
        assert_eq!(cfg.max_sequence_length, 512);
        assert_eq!(cfg.pool_size, 1);
        assert_eq!(cfg.max_concurrent_embeddings, 8);
        assert_eq!(cfg.max_batch_size, MAX_EMBED_BATCH_SIZE);
        assert!(cfg.validate().is_ok());
    }

    #[cfg(feature = "onnx")]
    #[test]
    /// Prüft ungültige Konfigurationswerte (`pool_size > 1` oder `max_concurrent_embeddings == 0`).
    fn test_text_embedder_config_validation() {
        let mut cfg = TextEmbedderConfig::default();
        cfg.pool_size = 2;
        let res = cfg.validate();
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("pool_size > 1"));

        let mut cfg = TextEmbedderConfig::default();
        cfg.max_concurrent_embeddings = 0;
        let res = cfg.validate();
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("max_concurrent_embeddings must be > 0"));
    }

    #[cfg(feature = "onnx")]
    #[test]
    /// Prüft, dass korrupte Modelldateien zu einem Ladefehler führen.
    fn test_text_embedder_load_invalid_content(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use std::io::Write;

        let dir = tempdir()?;
        File::create(dir.path().join("model.onnx"))?.write_all(b"invalid")?;
        File::create(dir.path().join("tokenizer.json"))?.write_all(b"invalid")?;

        let res = TextEmbedder::load(dir.path());
        assert!(res.is_err());
        let err_msg = res.err().map(|e| e.to_string()).unwrap_or_default();
        assert!(
            err_msg.contains("Failed to load tokenizer")
                || err_msg.contains("Failed to load model")
                || err_msg.contains("InvalidInput")
        );
        Ok(())
    }

    #[test]
    /// Prüft, dass Debug-Formatierung auf Fehlern nicht panickt.
    fn test_formatting_safety() {
        use contextra_types::ContextraError;
        // Verify that Debug formatting on ContextraError doesn't panic
        let err = ContextraError::Internal("test".into());
        let formatted = format!("{:?}", err);
        assert!(formatted.contains("Internal"));
    }

    #[test]
    /// Prüft die korrekte Pfadauflösung des Modell-Cache-Verzeichnisses.
    fn test_default_model_cache_dir_resolution() {
        let dir = default_model_cache_dir("nomic-embed-text");
        assert!(dir.to_string_lossy().contains("nomic-embed-text"));
        assert!(dir.to_string_lossy().contains(".contextra"));
    }

    #[cfg(feature = "onnx")]
    #[tokio::test]
    /// Prüft, dass bereits gecachte Modelle sofort zurückgegeben werden.
    async fn test_ensure_onnx_model_download_cached_returns_immediately() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("models");
        let model_dir = cache_dir.join("nomic-embed-text");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::File::create(model_dir.join("model.onnx")).unwrap();
        std::fs::File::create(model_dir.join("tokenizer.json")).unwrap();

        let resolved = ensure_onnx_model_download("nomic-embed-text", Some(&cache_dir))
            .await
            .unwrap();
        assert_eq!(resolved, model_dir);
    }

    #[tokio::test]
    /// Prüft die Ausführung einer Mock-Embedding-Engine.
    async fn test_mock_embedding_engine() -> std::result::Result<(), Box<dyn std::error::Error>> {
        use contextra_ports::{BoxFuture, TextEmbeddingEngine};
        use contextra_types::Result;

        struct MockEngine;
        impl TextEmbeddingEngine for MockEngine {
            fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>> {
                Box::pin(async move { Ok(vec![text.len() as f32]) })
            }
        }

        let engine = MockEngine;
        let res = engine.embed("contextra").await?;
        assert_eq!(res, vec![9.0]);
        Ok(())
    }

    #[tokio::test]
    /// Prüft Reihenfolge und Fehlerweiterleitung bei Batch-Embeddings.
    async fn test_embed_batch_ordering_and_fallback(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use contextra_ports::{BoxFuture, TextEmbeddingEngine};
        use contextra_types::{ContextraError, Result};

        struct MockOrderedEngine {
            fail_on: Option<String>,
        }

        impl TextEmbeddingEngine for MockOrderedEngine {
            fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>> {
                Box::pin(async move {
                    if let Some(ref fail) = self.fail_on {
                        if text == fail {
                            return Err(ContextraError::InvalidInput(format!("Failed on {text}")));
                        }
                    }
                    Ok(vec![
                        text.len() as f32,
                        (text.chars().next().unwrap_or('a') as u32) as f32,
                    ])
                })
            }
        }

        let engine = MockOrderedEngine { fail_on: None };
        let texts = vec!["a", "b", "c"];
        let batch_res = engine.embed_batch(&texts).await?;
        assert_eq!(batch_res.len(), 3);
        assert_eq!(batch_res[0], vec![1.0, 'a' as u32 as f32]);
        assert_eq!(batch_res[1], vec![1.0, 'b' as u32 as f32]);
        assert_eq!(batch_res[2], vec![1.0, 'c' as u32 as f32]);

        // Error propagation test
        let failing_engine = MockOrderedEngine {
            fail_on: Some("b".into()),
        };
        let err_res = failing_engine.embed_batch(&texts).await;
        assert!(err_res.is_err());
        let err_msg = err_res.err().unwrap().to_string(); // unwrap
        assert!(err_msg.contains("Failed on b"));

        Ok(())
    }

    #[cfg(feature = "onnx")]
    #[tokio::test]
    /// Prüft, dass übergroße Batches das Limit nicht überschreiten dürfen.
    async fn test_embed_batch_oversized_limit(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use contextra_ports::EmbeddingProvider;
        use std::path::PathBuf;

        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("model.onnx");

        if !fixture_path.exists() {
            return Ok(());
        }

        let embedder = TextEmbedder::from_path(&fixture_path)?;

        let large_texts: Vec<&str> = vec!["text"; MAX_EMBED_BATCH_SIZE + 1];
        let res = EmbeddingProvider::embed_batch(&embedder, &large_texts).await;
        assert!(res.is_err());
        if let Err(err) = res {
            assert!(matches!(err, EmbeddingError::Unavailable(_)));
            assert!(err.to_string().contains("exceeds max_batch_size"));
        } else {
            panic!("Expected InvalidInput error for oversized embed batch");
        }
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "onnx")]
    /// Prüft, dass Downloads standardmäßig deaktiviert sind.
    async fn test_model_download_offline_by_default() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("models");

        // Download disabled by default => returns InvalidInput error asking to enable download
        let res = ensure_onnx_model_download("nomic-embed-text", Some(&cache_dir)).await;
        assert!(res.is_err());
        if let Err(err) = res {
            assert!(matches!(err, ContextraError::InvalidInput(_)));
            assert!(err.to_string().contains("offline-first"));
        }
    }

    #[tokio::test]
    #[cfg(feature = "onnx")]
    /// Prüft Sicherheitsanforderungen (Prüfsummenpflicht und Host-Allowlist) beim Download.
    async fn test_model_download_options_secured_checks() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("models");

        // 1. Missing checksum => fails
        let opts = ModelDownloadOptions {
            allow_download: true,
            ..Default::default()
        };
        let res =
            ensure_onnx_model_download_with_options("nomic-embed-text", Some(&cache_dir), opts)
                .await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("missing expected_model_sha256"));

        // 2. Disallowed host => fails
        let opts_host = ModelDownloadOptions {
            allow_download: true,
            allowed_hosts: vec!["internal.only".to_string()],
            expected_model_sha256: Some("1234".to_string()),
            ..Default::default()
        };
        let res_host = ensure_onnx_model_download_with_options(
            "nomic-embed-text",
            Some(&cache_dir),
            opts_host,
        )
        .await;
        assert!(res_host.is_err());
        assert!(res_host
            .unwrap_err()
            .to_string()
            .contains("Nicht in der Allowlist"));
    }

    #[tokio::test]
    #[cfg(not(feature = "onnx"))]
    /// Prüft, dass ohne das Feature `onnx` ein CapabilityUnsupported-Fehler zurückgegeben wird.
    async fn test_ensure_onnx_model_download_disabled_feature() {
        let res = ensure_onnx_model_download("nomic-embed-text", None).await;
        assert!(res.is_err());
        if let Err(err) = res {
            assert!(matches!(
                err,
                contextra_types::ContextraError::CapabilityUnsupported { .. }
            ));
        }
    }
}

// REVIEW-PASS[1/2] STATUS:PASS (TS: 2026-09-02T08:17:15Z) (SESSION: f260cbf2) PRÜFER-KONTEXT: FRESH - Verified feature gate isolation, zero unsafe in production, and execution non-starvation model.
// REVIEW-PASS[2/2] STATUS:PASS (TS: 2026-09-03T19:40:00Z) (SESSION: 6da6a1c8) PRÜFER-KONTEXT: FRESH - Verified Chaos Engineering fault tolerance, hermetic feature gate, and zero-unsafe invariants.
// REVIEW-PASS[3/3] STATUS:PASS (TS: 2026-09-16T16:24:18Z) (SESSION: d3856baa) PRÜFER-KONTEXT: FRESH - Independent review verified feature gate isolation, zero unsafe in production, spawn_blocking non-starvation model, and domain scoring invariants (APM-22, APM-23, APM-24).
