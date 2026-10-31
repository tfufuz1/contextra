// FILE-CONTEXT
// ZWECK: Thin adapter re-exporting contextra_privacy::egress_guard and adapting contextra::Collection.

pub use contextra_privacy::egress_guard::*;

use contextra::Collection;
use contextra_privacy::egress_vault::BoxFuture;
use std::sync::Arc;

/// Adapter implementing `TextSearchEngine` for `contextra::Collection`.
pub struct CollectionSearchEngine(pub Arc<Collection>);

impl TextSearchEngine for CollectionSearchEngine {
    fn search_text<'a>(
        &'a self,
        text: &'a str,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
        let collection = self.0.clone();
        let text_owned = text.to_string();
        Box::pin(async move {
            let res = collection
                .query()
                .text(&text_owned)
                .k(limit)
                .execute()
                .await;
            match res {
                Ok(results) => Ok(results
                    .into_iter()
                    .map(|r| TextSearchResult {
                        id: r.id,
                        score: r.score,
                    })
                    .collect()),
                Err(e) => Err(e.to_string()),
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_egress_guard_defaults() {
        assert_eq!(DEFAULT_EGRESS_GUARD_TIMEOUT, Duration::from_millis(200));
        assert_eq!(DEFAULT_EGRESS_GUARD_THRESHOLD, 0.85);
        assert_eq!(DEFAULT_EGRESS_GUARD_MIN_BYTES, 128);
    }
}
