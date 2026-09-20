// FILE-CONTEXT
// ZWECK: Layer-4 EgressGuard (Re-export from memfuse-privacy)

pub use memfuse_privacy::egress_guard::*;
use memfuse_privacy::BoxFuture;
use std::sync::Arc;

pub struct CollectionSearchEngine {
    collection: Arc<memfuse_db::Collection>,
}

impl CollectionSearchEngine {
    pub fn new(collection: Arc<memfuse_db::Collection>) -> Self {
        Self { collection }
    }
}

impl TextSearchEngine for CollectionSearchEngine {
    fn search_text<'a>(
        &'a self,
        text: &'a str,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
        #[allow(deprecated)]
        let search_fut = self.collection.search_text(text, limit);
        Box::pin(async move {
            match search_fut.await {
                Ok(results) => Ok(results
                    .into_iter()
                    .map(|r| TextSearchResult {
                        id: r.id.to_string(),
                        score: r.score,
                    })
                    .collect()),
                Err(err) => Err(err.to_string()),
            }
        })
    }
}
