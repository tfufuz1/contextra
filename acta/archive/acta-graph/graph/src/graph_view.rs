//! Minimal read-only graph view trait for traversal and ranking (N5).

use crate::error::Result;
use crate::storage::GraphFuture;
use crate::types::{EntityId, PprConfig};

pub trait GraphView: Send + Sync {
    fn neighbors<'a>(&'a self, entity: EntityId) -> GraphFuture<'a, Result<Vec<EntityId>>>;
    fn personalized_page_rank<'a>(
        &'a self,
        seeds: &'a [EntityId],
        config: &'a PprConfig,
    ) -> GraphFuture<'a, Result<Vec<(EntityId, f32)>>>;
}

impl GraphView for crate::csr::CsrGraph {
    fn neighbors<'a>(&'a self, entity: EntityId) -> GraphFuture<'a, Result<Vec<EntityId>>> {
        Box::pin(async move {
            self.neighbors(entity).await.map_err(Into::into)
        })
    }

    fn personalized_page_rank<'a>(
        &'a self,
        seeds: &'a [EntityId],
        config: &'a PprConfig,
    ) -> GraphFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let mut ctx = crate::PprContext::new();
            let res = self.personalized_page_rank_with_context_async(seeds, config, &mut ctx).await;
            Ok(res)
        })
    }
}
