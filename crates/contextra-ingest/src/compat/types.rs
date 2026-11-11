//! Kompatibilitätsbrücke zwischen lokalen Ingest-Typen und `contextra-types`.

use crate::chunking::types::{
    ContextChunk as LocalContextChunk, ContextChunkLink as LocalContextChunkLink,
    DocId as LocalDocId,
};
use crate::error::IngestError;

impl From<LocalDocId> for contextra_types::DocId {
    #[inline]
    fn from(id: LocalDocId) -> Self {
        contextra_types::DocId::new(id.as_u64())
    }
}

impl TryFrom<contextra_types::DocId> for LocalDocId {
    type Error = IngestError;

    #[inline]
    fn try_from(id: contextra_types::DocId) -> Result<Self, Self::Error> {
        let val = id.as_u64();
        Ok(LocalDocId::new(val))
    }
}

impl From<LocalContextChunk> for contextra_types::ContextChunk {
    fn from(chunk: LocalContextChunk) -> Self {
        let links = chunk
            .links
            .into_iter()
            .filter_map(|link| {
                if let Ok(target_u64) = link.target.parse::<u64>() {
                    let rel = match link.kind.as_str() {
                        "elaborates" => contextra_types::LinkRelation::Elaborates,
                        "contradicts" => contextra_types::LinkRelation::Contradicts,
                        "supersedes" => contextra_types::LinkRelation::Supersedes,
                        _ => contextra_types::LinkRelation::References,
                    };
                    Some(contextra_types::MemoryLink {
                        target: contextra_types::DocId::new(target_u64),
                        relation: rel,
                        created_at_tx: contextra_types::TxId::new(0),
                    })
                } else {
                    None
                }
            })
            .collect();

        contextra_types::ContextChunk {
            doc_id: chunk.doc_id.into(),
            content: chunk.content,
            relevance: chunk.relevance,
            token_count: chunk.token_count,
            metadata: chunk.metadata,
            contextual_prefix: chunk.contextual_prefix,
            links,
        }
    }
}

impl From<contextra_types::ContextChunk> for LocalContextChunk {
    fn from(chunk: contextra_types::ContextChunk) -> Self {
        let links = chunk
            .links
            .into_iter()
            .map(|link| {
                let kind = match link.relation {
                    contextra_types::LinkRelation::Elaborates => "elaborates".to_string(),
                    contextra_types::LinkRelation::Contradicts => "contradicts".to_string(),
                    contextra_types::LinkRelation::Supersedes => "supersedes".to_string(),
                    contextra_types::LinkRelation::References => "references".to_string(),
                };
                LocalContextChunkLink {
                    kind,
                    target: link.target.as_u64().to_string(),
                }
            })
            .collect();

        LocalContextChunk {
            doc_id: LocalDocId::new(chunk.doc_id.as_u64()),
            content: chunk.content,
            relevance: chunk.relevance,
            token_count: chunk.token_count,
            metadata: chunk.metadata,
            contextual_prefix: chunk.contextual_prefix,
            links,
        }
    }
}
