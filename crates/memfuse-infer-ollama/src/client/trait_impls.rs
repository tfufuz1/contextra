use super::core::OllamaClient;

impl memfuse_core::LlmTextGenerator for OllamaClient {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
    ) -> memfuse_core::traits::BoxFuture<'a, memfuse_core::Result<String>> {
        Box::pin(async move { self.generate_text(&self.config().model, prompt).await })
    }
}

impl memfuse_core::LlmTextGeneratorStreaming for OllamaClient {
    fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        _config: &'a memfuse_core::ConfigFingerprint,
    ) -> memfuse_core::traits::BoxStream<'a, memfuse_core::Result<String>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<memfuse_core::Result<String>>(32);
        let client = self.clone();
        let prompt_owned = prompt.to_string();

        tokio::spawn(async move {
            let res = client
                .generate_text_stream(&client.config().model, &prompt_owned, |token| {
                    let tx = tx.clone();
                    async move { tx.send(Ok(token)).await.is_ok() }
                })
                .await;

            if let Err(err) = res {
                let _ = tx.send(Err(err)).await;
            }
        });

        Box::pin(futures_util::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|item| (item, rx))
        }))
    }
}

impl memfuse_core::SegmentSynthesizer for OllamaClient {
    fn synthesize_segment<'a>(
        &'a self,
        segment_texts: &'a [&'a str],
    ) -> memfuse_core::BoxFuture<'a, memfuse_core::Result<String>> {
        Box::pin(async move {
            let combined = segment_texts.join("\n---\n");
            let prompt = format!(
                "Fasse die folgenden verwandten Erinnerungen zu einer einzigen, prägnanten \
                 abstrakten Wissensaussage zusammen (max. 2 Sätze, keine Detailwiederholungen):\n\n\
                 {combined}\n\nAbstrakte Zusammenfassung:"
            );
            self.generate_text(&self.config().model, &prompt).await
        })
    }

    fn model_id(&self) -> &str {
        &self.config().model
    }
}
