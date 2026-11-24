//! Brücke für LlmTextGenerator und LlmTextGeneratorStreaming (Ports).

use crate::inference::core::CandleLlmClient;
use contextra_ports::{BoxFuture, BoxStream, ContextSegment};
use contextra_ports::{LlmTextGenerator, LlmTextGeneratorStreaming};
use contextra_types::{ContextraError, Result, TenantId};
use futures_util::stream;
use std::sync::Arc;

impl LlmTextGenerator for CandleLlmClient {
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>> {
        let model = Arc::clone(&self.model);
        let tokenizer = self.tokenizer.clone();
        let device = self.device.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let prompt_owned = prompt.to_string();

        Box::pin(async move {
            let _permit = semaphore.acquire().await.map_err(|_| {
                ContextraError::Internal("Candle inference semaphore closed".into())
            })?;

            tokio::task::spawn_blocking(move || {
                let mut guard = model.blocking_lock();
                guard
                    .generate(&prompt_owned, &tokenizer, &device)
                    .map_err(|e| ContextraError::Internal(e.to_string()))
            })
            .await
            .map_err(|e| {
                ContextraError::Internal(format!("Candle inference task join error: {e}"))
            })?
        })
    }

    #[cfg(feature = "kv-bridge")]
    fn generate_with_context<'a>(
        &'a self,
        tenant: TenantId,
        segments: &'a [ContextSegment<'a>],
    ) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            #[cfg(feature = "kv-stage-b")]
            if self.prefix_store.is_some() {
                return self.generate_with_prefix_store(tenant, segments).await;
            }

            if let Some(ref adapter) = self.kv_bridge {
                for segment in segments {
                    adapter.consult_segment(segment);
                    let fp = segment
                        .model_fingerprint
                        .cloned()
                        .unwrap_or_else(|| self.fingerprint.clone().into());
                    let key = crate::kv_bridge::KvCacheKey::new(
                        segment.chunk_id,
                        fp,
                        segment.rope_offset,
                    );
                    if adapter.try_get_cached_segment(tenant, &key).is_none() {
                        let fresh_bytes = format!(
                            "kv_cache_tensor:{}:{}:{}",
                            key.chunk_id, key.fingerprint.model_id, segment.text
                        )
                        .into_bytes();
                        adapter.store_segment(tenant, key, fresh_bytes);
                    }
                    self.prefill_count
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            } else {
                self.prefill_count
                    .fetch_add(segments.len() as u64, std::sync::atomic::Ordering::SeqCst);
            }

            let concatenated = segments
                .iter()
                .map(|s| s.text)
                .collect::<Vec<_>>()
                .join("\n\n");

            self.generate(&concatenated).await
        })
    }

    #[cfg(not(feature = "kv-bridge"))]
    fn generate_with_context<'a>(
        &'a self,
        tenant: TenantId,
        segments: &'a [ContextSegment<'a>],
    ) -> BoxFuture<'a, Result<String>> {
        Box::pin(async move {
            #[cfg(feature = "kv-stage-b")]
            if self.prefix_store.is_some() {
                return self.generate_with_prefix_store(tenant, segments).await;
            }

            let _ = tenant;
            self.prefill_count
                .fetch_add(segments.len() as u64, std::sync::atomic::Ordering::SeqCst);
            let concatenated = segments
                .iter()
                .map(|s| s.text)
                .collect::<Vec<_>>()
                .join("\n\n");

            self.generate(&concatenated).await
        })
    }
}

impl LlmTextGeneratorStreaming for CandleLlmClient {
    fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        _config: &'a contextra_types::ConfigFingerprint,
    ) -> BoxStream<'a, Result<String>> {
        let model = Arc::clone(&self.model);
        let tokenizer = self.tokenizer.clone();
        let device = self.device.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let prompt_owned = prompt.to_string();

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<String>>(32);

        tokio::spawn(async move {
            let permit = match semaphore.acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    let _ = tx
                        .send(Err(ContextraError::Internal(
                            "Candle inference semaphore closed".into(),
                        )))
                        .await;
                    return;
                }
            };

            let tx_thread = tx.clone();
            let join_res = tokio::task::spawn_blocking(move || {
                let _permit_guard = permit;
                let mut guard = model.blocking_lock();

                guard.generate_stream(
                    &prompt_owned,
                    &tokenizer,
                    &device,
                    &mut |token: String| {
                        let _ = tx_thread.blocking_send(Ok(token));
                        true
                    },
                )
            })
            .await;

            if let Err(e) = join_res {
                let _ = tx
                    .send(Err(ContextraError::Internal(format!(
                        "Candle streaming inference task join error: {e}"
                    ))))
                    .await;
            }
        });

        Box::pin(stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        }))
    }
}

#[cfg(feature = "kv-stage-b")]
impl CandleLlmClient {
    /// Configures optional `KvPrefixStore` context for stage B prefix reuse.
    pub fn with_prefix_store(mut self, store: Arc<dyn contextra_ports::kv::KvPrefixStore>) -> Self {
        self.prefix_store = Some(crate::compat::kv::KvPrefixContext { store });
        self
    }

    /// Generates completion with stage B KV prefix reuse and tenant isolation.
    pub fn generate_with_prefix_store<'a>(
        &'a self,
        tenant: TenantId,
        segments: &'a [ContextSegment<'a>],
    ) -> BoxFuture<'a, Result<String>> {
        let store_ctx = match self.prefix_store.clone() {
            Some(ctx) => ctx,
            None => return self.generate_with_context(tenant, segments),
        };

        let model = Arc::clone(&self.model);
        let tokenizer = self.tokenizer.clone();
        let device = self.device.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let fingerprint = self.fingerprint.clone();

        let prefill_count = Arc::clone(&self.prefill_count);
        let prefill_skip_count = Arc::clone(&self.prefill_skip_count);
        let prefill_skipped_tokens = Arc::clone(&self.prefill_skipped_tokens);

        let concatenated = segments
            .iter()
            .map(|s| s.text)
            .collect::<Vec<_>>()
            .join("\n\n");

        Box::pin(async move {
            let _permit = semaphore.acquire().await.map_err(|_| {
                ContextraError::Internal("Candle inference semaphore closed".into())
            })?;

            tokio::task::spawn_blocking(move || {
                let mut guard = model.blocking_lock();

                let prompt_tokens = tokenizer
                    .encode(concatenated.as_str(), true)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?
                    .get_ids()
                    .to_vec();

                let (layout, rope) = guard.kv_layout().unwrap_or_else(|| {
                    (
                        crate::kv_state::KvLayout {
                            n_layer: 0,
                            n_kv_head: 0,
                            head_dim: 0,
                            dtype: "f16".to_string(),
                        },
                        crate::kv_state::RopeConfig {
                            base: 10000.0,
                            scaling: None,
                        },
                    )
                });

                let key =
                    crate::compat::kv::build_prefix_key(&fingerprint, &tokenizer, layout, rope)?;
                let hit = store_ctx.store.lookup(tenant, &key, &prompt_tokens);

                let seed = if let Some(ref h) = hit {
                    match crate::compat::kv::seed_from_hit(h, prompt_tokens.len()) {
                        Ok(s) => {
                            prefill_skip_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            prefill_skipped_tokens.fetch_add(
                                s.matched_len as u64,
                                std::sync::atomic::Ordering::SeqCst,
                            );
                            Some(s)
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                };

                prefill_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                let run = guard
                    .generate_with_prefix(&concatenated, &tokenizer, &device, seed)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?;

                if let Some(block) = run.exported_prefix {
                    if !run.prompt_tokens.is_empty() {
                        let _ =
                            store_ctx
                                .store
                                .insert(tenant, &key, &run.prompt_tokens, vec![block.into()]);
                    }
                }

                Ok(run.output)
            })
            .await
            .map_err(|e| {
                ContextraError::Internal(format!("Candle inference task join error: {e}"))
            })?
        })
    }
}
