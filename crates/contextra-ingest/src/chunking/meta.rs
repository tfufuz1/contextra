// FILE-CONTEXT
// ZWECK: Erzeugung des Metadaten-Schemas contextra.chunk.v1 und BLAKE3 Chunk-IDs (WP M3).
// INVARIANTEN: Schema "contextra.chunk.v1", deterministische BLAKE3-Chunk-ID (32-Hex chars / 16 Byte), reservierte Schlüssel nie überschreiben.
// STAND: TS:2026-10-31T00:00:00Z

use crate::chunking::ChunkDraft;
use crate::format::ChunkStrategy;
use crate::ir::DocumentIr;
use blake3::Hasher;
use serde_json::{json, Map, Value};

/// Generiert das standardisierte JSON-Metadatenobjekt unter dem Schema `"contextra.chunk.v1"`.
pub fn build_chunk_metadata(
    canonical_hash: &str,
    cfg_fingerprint: &str,
    strategy: ChunkStrategy,
    draft: &ChunkDraft,
    chunk_index: u32,
    chunk_total: u32,
    ir: &DocumentIr,
) -> Value {
    // BLAKE3-Chunk-ID erzeugen: blake3(canonical_hash | cfg_fingerprint | strategy | span.start..span.end | chunk_index)
    let mut hasher = Hasher::new();
    hasher.update(canonical_hash.as_bytes());
    hasher.update(cfg_fingerprint.as_bytes());
    hasher.update(format!("{:?}", strategy).as_bytes());
    hasher.update(&draft.span.start.to_le_bytes());
    hasher.update(&draft.span.end.to_le_bytes());
    hasher.update(&chunk_index.to_le_bytes());

    let hash_bytes = hasher.finalize();
    let chunk_id = hex::encode(&hash_bytes.as_bytes()[..16]);

    let mut map = Map::new();
    map.insert("schema".into(), Value::String("contextra.chunk.v1".into()));
    map.insert(
        "chunker_version".into(),
        Value::String("ingest-chunk/1".into()),
    );
    map.insert("chunk_id".into(), Value::String(chunk_id));
    map.insert("chunk_index".into(), json!(chunk_index));
    map.insert("chunk_total".into(), json!(chunk_total));
    map.insert("format".into(), Value::String(ir.format.name().into()));
    map.insert(
        "strategy".into(),
        Value::String(format!("{:?}", strategy).to_lowercase()),
    );
    map.insert(
        "source_span".into(),
        json!({
            "start": draft.span.start,
            "end": draft.span.end,
        }),
    );

    // Locator nur gesetzte Felder aufnehmen
    let mut loc_map = Map::new();
    if let Some(p) = draft.locator.page {
        loc_map.insert("page".into(), json!(p));
    }
    if let Some(p) = draft.locator.page_end {
        loc_map.insert("page_end".into(), json!(p));
    }
    if let Some(s) = draft.locator.slide {
        loc_map.insert("slide".into(), json!(s));
    }
    if let Some(s) = draft.locator.slide_end {
        loc_map.insert("slide_end".into(), json!(s));
    }
    if let Some(ref s) = draft.locator.sheet {
        loc_map.insert("sheet".into(), Value::String(s.clone()));
    }
    if let Some(ref r) = draft.locator.range {
        loc_map.insert("range".into(), Value::String(r.clone()));
    }
    if let Some(m) = draft.locator.message {
        loc_map.insert("message".into(), json!(m));
    }
    if let Some(ref p) = draft.locator.path {
        loc_map.insert("path".into(), Value::String(p.clone()));
    }
    if let Some(ref a) = draft.locator.attachment {
        loc_map.insert("attachment".into(), Value::String(a.clone()));
    }
    map.insert("locator".into(), Value::Object(loc_map));

    if let Some(ref bc) = draft.breadcrumb {
        map.insert("breadcrumb".into(), Value::String(bc.clone()));
    }
    if let Some(hl) = draft.heading_level {
        map.insert("heading_level".into(), json!(hl));
    }
    if let Some(ref title) = ir.meta.title {
        map.insert("doc_title".into(), Value::String(title.clone()));
    }

    // Extra-Felder hinzufügen (ohne reservierte Schlüssel zu überschreiben)
    for (k, v) in &draft.extra {
        if !map.contains_key(k) {
            map.insert(k.clone(), v.clone());
        }
    }

    Value::Object(map)
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            use std::fmt::Write;
            write!(&mut s, "{:02x}", b).expect("writing to String cannot fail");
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FormatId;
    use crate::ir::Locator;

    #[test]
    fn test_build_chunk_metadata_schema() {
        let ir = DocumentIr::new(FormatId::Markdown);
        let draft = ChunkDraft {
            text: "Hello".into(),
            span: 0..5,
            locator: Locator::default(),
            breadcrumb: Some("# Title".into()),
            heading_level: Some(1),
            extra: Map::new(),
        };

        let meta = build_chunk_metadata(
            "dummy_canon_hash",
            "dummy_cfg_fp",
            ChunkStrategy::Prose,
            &draft,
            0,
            1,
            &ir,
        );

        assert_eq!(meta["schema"], "contextra.chunk.v1");
        assert_eq!(meta["chunk_id"].as_str().unwrap().len(), 32);
    }
}
