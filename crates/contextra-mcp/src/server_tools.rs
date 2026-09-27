use crate::egress_gateway::{self, CloudQueryRequest};
use crate::io::MAX_SEARCH_QUERY_BYTES;
use crate::protocol::McpError;
use crate::server::McpServer;
use crate::validation::validate_collection_name;
use contextra::chunker::{ChunkerConfig, MarkdownChunker};
use contextra_ports::StorageEngine;
use contextra_types::{DocId, MAX_SEARCH_K};
use serde_json::{json, Value};

/// Maximale Anzahl von Teilnehmern an einer n-ären Hyperkante (`contextra_relate_n_ary`).
pub(crate) const MAX_RELATE_PARTICIPANTS: usize = 64;

impl McpServer {
    pub(crate) async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, McpError> {
        self.sandbox
            .validate_tool_call(name, args)
            .map_err(McpError::from)?;
        match name {
            "contextra_search" => {
                let query = match args.get("query") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'query' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("query cannot be empty"));
                        }
                        if s.len() > MAX_SEARCH_QUERY_BYTES {
                            return Err(McpError::invalid_params(format!(
                                "query size exceeds limit: {} bytes > {} limit",
                                s.len(),
                                MAX_SEARCH_QUERY_BYTES
                            )));
                        }
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params("missing required field: 'query'"));
                    }
                };

                let col_name = if let Some(col_val) = args.get("collection") {
                    let s = col_val.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'collection' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        "default"
                    } else {
                        validate_collection_name(s)?;
                        s
                    }
                } else {
                    "default"
                };

                let k_val = args.get("k").or_else(|| args.get("limit"));
                let k_raw = match k_val {
                    Some(Value::Number(n)) => n.as_u64().ok_or_else(|| {
                        McpError::invalid_params("k/limit muss eine positive Ganzzahl sein")
                    })? as usize,
                    Some(_) => return Err(McpError::invalid_params("k/limit muss eine Zahl sein")),
                    None => 10,
                };
                let k = k_raw.min(MAX_SEARCH_K);
                if k_raw > MAX_SEARCH_K {
                    tracing::warn!(
                        requested_k = k_raw,
                        capped_k = MAX_SEARCH_K,
                        "Client k capped to MAX_SEARCH_K"
                    );
                }

                let col = self.db.collection(col_name).await.map_err(McpError::from)?;
                let vec = self
                    .embedder
                    .embed(query)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string()))?;
                let results = col
                    .query()
                    .text(query)
                    .vector(&vec)
                    .k(k)
                    .execute()
                    .await
                    .map_err(McpError::from)?;

                #[cfg(feature = "kv-bridge")]
                if let Some(ref bridge) = self.kv_bridge {
                    for res in &results {
                        let chunk_id = DocId::from_key(&res.id).map(|d| d.as_u64()).unwrap_or(0);
                        let text = res
                            .metadata
                            .as_ref()
                            .and_then(|m| m.get("text").or_else(|| m.get("content")))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let segment = contextra_ports::ContextSegment::new(chunk_id, text);
                        bridge.consult_segment(&segment);
                    }
                }

                let mut enriched_results = Vec::with_capacity(results.len());
                for res in results {
                    let mut val = serde_json::to_value(&res).map_err(|e| {
                        McpError::internal_error(format!("Result serialization error: {e}"))
                    })?;
                    if let Some(obj) = val.as_object_mut() {
                        obj.insert(
                            "content_provenance".to_string(),
                            json!("retrieved_untrusted_data"),
                        );
                        self.injection_guard.process_result(&res.id, col_name, obj);
                    }
                    enriched_results.push(val);
                }

                Ok(json!(enriched_results))
            }

            "contextra_insert" => {
                // Validate collection parameter if present
                let col_name = if let Some(col_val) = args.get("collection") {
                    let s = col_val.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'collection' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        "default"
                    } else {
                        validate_collection_name(s)?;
                        s
                    }
                } else {
                    "default"
                };

                // Validate id parameter
                let id = match args.get("id") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'id' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("id cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "id length exceeds limit: max 256 chars",
                            ));
                        }
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params(
                            "id fehlt: missing required field 'id'",
                        ));
                    }
                };

                // Validate vector / text parameters
                let vec_val = args.get("vector");
                let text_val = args.get("text");

                let vector_opt: Option<Vec<f32>> = if let Some(v) = vec_val {
                    let arr = v.as_array().ok_or_else(|| {
                        McpError::invalid_params(
                            "Invalid params: 'vector' must be an array of numbers",
                        )
                    })?;
                    if arr.is_empty() {
                        return Err(McpError::invalid_params("vector cannot be empty"));
                    }
                    let mut vec = Vec::with_capacity(arr.len());
                    for elem in arr {
                        let num = elem.as_f64().ok_or_else(|| {
                            McpError::invalid_params(
                                "Invalid params: 'vector' must contain numbers",
                            )
                        })?;
                        let f = num as f32;
                        if f.is_nan() || f.is_infinite() {
                            return Err(McpError::invalid_params("NaN or Inf in vector"));
                        }
                        vec.push(f);
                    }
                    Some(vec)
                } else {
                    None
                };

                let text_opt = if let Some(t) = text_val {
                    let s = t.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'text' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        return Err(McpError::invalid_params("text cannot be empty"));
                    }
                    const MAX_INSERT_TEXT_BYTES: usize = 10 * 1024 * 1024; // 10MB
                    if s.len() > MAX_INSERT_TEXT_BYTES {
                        return Err(McpError::invalid_params(format!(
                            "text too large: {}MB > 10MB limit",
                            s.len() / 1_048_576
                        )));
                    }
                    Some(s)
                } else {
                    None
                };

                if vector_opt.is_none() && text_opt.is_none() {
                    return Err(McpError::invalid_params(
                        "text/vector fehlt: missing required field 'vector' or 'text'",
                    ));
                }

                let base_metadata = args
                    .get("metadata")
                    .and_then(|v| v.as_object())
                    .cloned()
                    .unwrap_or_default();

                let col = self.db.collection(col_name).await.map_err(McpError::from)?;

                if let Some(vector) = vector_opt {
                    let mut meta = json!({
                        "source_id": id,
                    });
                    if let Some(obj) = meta.as_object_mut() {
                        if let Some(text) = text_opt {
                            obj.insert("text".to_string(), json!(text));
                        }
                        for (k, v) in &base_metadata {
                            obj.entry(k.clone()).or_insert_with(|| v.clone());
                        }
                    }
                    col.insert(id, &vector, Some(meta))
                        .await
                        .map_err(McpError::from)?;

                    return Ok(json!({
                        "ok": true,
                        "id": id,
                        "chunks_inserted": 1,
                        "chunk_ids": [id],
                        "collection": col_name
                    }));
                }

                // AUTO-CHUNKING: Text in semantische Einheiten aufteilen mit MarkdownChunker (~512 Tokens)
                let text = match text_opt {
                    Some(t) => t,
                    None => {
                        return Err(McpError::invalid_params(
                            "text/vector fehlt: missing required field 'text'",
                        ));
                    }
                };
                let chunker = MarkdownChunker::new(ChunkerConfig::default());
                let doc_id = DocId::from_key(id).map_err(|e| {
                    McpError::invalid_params(format!("Invalid document ID '{}': {}", id, e))
                })?;
                let chunks = chunker.chunk(doc_id, text);

                if chunks.is_empty() {
                    return Ok(json!({
                        "ok": false,
                        "error": "Text konnte nicht in Chunks aufgeteilt werden (leer?)"
                    }));
                }

                let total = chunks.len();
                let mut inserted_chunk_ids = Vec::new();

                for (i, chunk) in chunks.iter().enumerate() {
                    let chunk_id = if total == 1 {
                        id.to_string()
                    } else {
                        format!("{id}:chunk:{i}")
                    };

                    let embedding = self
                        .embedder
                        .embed(&chunk.content)
                        .await
                        .map_err(|e| McpError::internal_error(e.to_string()))?;

                    let mut chunk_meta = json!({
                        "text": &chunk.content,
                        "source_id": id,
                        "chunk_index": i,
                        "chunk_total": total
                    });

                    if let Some(obj) = chunk_meta.as_object_mut() {
                        if let Some(meta) = &chunk.metadata {
                            if let Some(m_obj) = meta.as_object() {
                                for (k, v) in m_obj {
                                    obj.insert(k.clone(), v.clone());
                                }
                            }
                        }
                        for (k, v) in &base_metadata {
                            obj.entry(k.clone()).or_insert_with(|| v.clone());
                        }
                    }

                    col.insert(&chunk_id, &embedding, Some(chunk_meta))
                        .await
                        .map_err(McpError::from)?;

                    inserted_chunk_ids.push(chunk_id);
                }

                Ok(json!({
                    "ok": true,
                    "id": id,
                    "chunks_inserted": inserted_chunk_ids.len(),
                    "chunk_ids": inserted_chunk_ids,
                    "collection": col_name
                }))
            }

            "contextra_get" => {
                let id = match args.get("id") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'id' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("id cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "id length exceeds limit: max 256 chars",
                            ));
                        }
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params(
                            "id fehlt: missing required field 'id'",
                        ));
                    }
                };

                let col_name = if let Some(col_val) = args.get("collection") {
                    let s = col_val.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'collection' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        "default"
                    } else {
                        validate_collection_name(s)?;
                        s
                    }
                } else {
                    "default"
                };

                let col = self.db.collection(col_name).await.map_err(McpError::from)?;
                match col.get(id).await.map_err(McpError::from)? {
                    Some(doc) => {
                        let mut val = serde_json::to_value(&doc)
                            .map_err(|e| McpError::internal_error(e.to_string()))?;
                        if let Some(obj) = val.as_object_mut() {
                            obj.insert(
                                "content_provenance".to_string(),
                                json!("retrieved_untrusted_data"),
                            );
                            self.injection_guard.process_result(&doc.id, col_name, obj);
                        }
                        Ok(val)
                    }
                    None => Ok(json!(null)),
                }
            }

            "contextra_forget" => {
                let col_name = match args.get("collection") {
                    Some(col_val) => {
                        let s = col_val.as_str().ok_or_else(|| {
                            McpError::invalid_params(
                                "Invalid params: 'collection' must be a string",
                            )
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("collection cannot be empty"));
                        }
                        validate_collection_name(s)?;
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params(
                            "collection fehlt: missing required field 'collection'",
                        ));
                    }
                };

                let confirm = match args.get("confirm") {
                    Some(Value::Bool(b)) => *b,
                    _ => false,
                };
                if !confirm {
                    return Err(McpError::invalid_params(
                        "confirm parameter must be explicitly set to true for contextra_forget",
                    ));
                }

                let id_opt = match args.get("id") {
                    Some(v) if !v.is_null() => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'id' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("id cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "id length exceeds limit: max 256 chars",
                            ));
                        }
                        Some(s)
                    }
                    _ => None,
                };

                if let Some(id) = id_opt {
                    let col = self.db.collection(col_name).await.map_err(McpError::from)?;
                    col.delete(id).await.map_err(McpError::from)?;

                    // Single document deletion in Contextra does not generate a DeletionProof.
                    // In accordance with honesty rules, we return proof: null and proof_scope: "collection_only".
                    Ok(json!({
                        "ok": true,
                        "collection": col_name,
                        "id": id,
                        "proof": null,
                        "proof_scope": "collection_only"
                    }))
                } else {
                    let proof_key_str = std::env::var("CONTEXTRA_DELETION_PROOF_KEY")
                        .or_else(|_| std::env::var("CONTEXTRA_PROOF_KEY"))
                        .map_err(|_| {
                            McpError::invalid_params("deletion proof key not configured")
                        })?;
                    let trimmed_key = proof_key_str.trim();
                    if trimmed_key.is_empty() {
                        return Err(McpError::invalid_params(
                            "deletion proof key not configured",
                        ));
                    }

                    let tenant_id = contextra_types::TenantId::try_new(1)
                        .unwrap_or(contextra_types::TenantId::SYSTEM);

                    let proof = self
                        .db
                        .drop_collection(col_name, tenant_id, trimmed_key.as_bytes())
                        .await
                        .map_err(McpError::from)?;

                    let proof_json_str = proof
                        .export_for_audit()
                        .map_err(|e| McpError::internal_error(e.to_string()))?;
                    let proof_val: Value = serde_json::from_str(&proof_json_str)
                        .map_err(|e| McpError::internal_error(e.to_string()))?;

                    Ok(json!({
                        "ok": true,
                        "collection": col_name,
                        "proof": proof_val,
                        "proof_scope": "collection"
                    }))
                }
            }

            "contextra_collections" => {
                let names = self.db.list_collections().await.map_err(McpError::from)?;
                Ok(json!({ "collections": names }))
            }

            "contextra_consolidate" => {
                let col_name = if let Some(col_val) = args.get("collection") {
                    let s = col_val.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'collection' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        "default"
                    } else {
                        validate_collection_name(s)?;
                        s
                    }
                } else {
                    "default"
                };

                let col = self.db.collection(col_name).await.map_err(McpError::from)?;

                // Turns chronologisch aus der Collection lesen (analog zur ConsolidationEngine)
                let user_key_prefix = col.user_key_prefix();
                let entries = col
                    .storage()
                    .scan_prefix(&user_key_prefix)
                    .await
                    .map_err(McpError::from)?;

                let mut turns: Vec<(DocId, Vec<f32>)> = Vec::new();
                for (k, v) in entries {
                    if col_name == "default" && k.starts_with(b"__") {
                        continue;
                    }
                    if let Ok(stored) = serde_json::from_slice::<serde_json::Value>(&v) {
                        if let (Some(id_str), Some(arr)) = (
                            stored.get("id").and_then(|i| i.as_str()),
                            stored.get("embedding").and_then(|e| e.as_array()),
                        ) {
                            if let Ok(doc_id) = DocId::from_key(id_str) {
                                let mut vec = Vec::with_capacity(arr.len());
                                for elem in arr {
                                    if let Some(f) = elem.as_f64() {
                                        vec.push(f as f32);
                                    }
                                }
                                turns.push((doc_id, vec));
                            }
                        }
                    }
                }

                let turns_scanned = turns.len();
                let (consolidation_res, synthesis_res) =
                    contextra::execute_background_consolidation(
                        col.as_ref(),
                        &turns,
                        &contextra::memory_consolidation::ConsolidationConfig::default(),
                        None,
                        None,
                        None,
                        None,
                    )
                    .await
                    .map_err(McpError::from)?;

                let duplicates_tombstoned_count = consolidation_res.duplicates_tombstoned.len();
                let cascade_tombstones_count =
                    consolidation_res.cascade_edge_tombstones_needed.len();
                let synthesized_count = synthesis_res
                    .as_ref()
                    .map(|s| s.synthesized.len())
                    .unwrap_or(0);

                Ok(json!({
                    "ok": true,
                    "collection": col_name,
                    "turns_scanned": turns_scanned,
                    "segments_created": consolidation_res.segments_created,
                    "duplicates_tombstoned": duplicates_tombstoned_count,
                    "synthesized_chunks": synthesized_count,
                    "cascade_edge_tombstones_needed": cascade_tombstones_count,
                    "cascade_errors": consolidation_res.cascade_errors,
                }))
            }

            "contextra_cloud_query" => {
                let query = match args.get("query") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'query' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("query cannot be empty"));
                        }
                        if s.len() > MAX_SEARCH_QUERY_BYTES {
                            return Err(McpError::invalid_params(format!(
                                "query size exceeds limit: {} bytes > {} limit",
                                s.len(),
                                MAX_SEARCH_QUERY_BYTES
                            )));
                        }
                        s.to_string()
                    }
                    None => {
                        return Err(McpError::invalid_params("missing required field: 'query'"));
                    }
                };

                let collection = if let Some(col_val) = args.get("collection") {
                    let s = col_val.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'collection' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        Some("default".to_string())
                    } else {
                        validate_collection_name(s)?;
                        Some(s.to_string())
                    }
                } else {
                    Some("default".to_string())
                };

                let max_results = if let Some(k_val) =
                    args.get("max_results").or_else(|| args.get("k"))
                {
                    match k_val {
                        Value::Number(n) => {
                            let k_raw = n.as_u64().ok_or_else(|| {
                                McpError::invalid_params(
                                    "max_results muss eine positive Ganzzahl sein",
                                )
                            })? as usize;
                            Some(k_raw.min(MAX_SEARCH_K))
                        }
                        _ => {
                            return Err(McpError::invalid_params("max_results muss eine Zahl sein"))
                        }
                    }
                } else {
                    Some(10)
                };

                let request = CloudQueryRequest {
                    query,
                    collection,
                    max_results,
                };

                let response =
                    egress_gateway::handle_cloud_query(request, self.egress_classifier.as_ref())
                        .await?;

                serde_json::to_value(&response).map_err(|e| {
                    McpError::internal_error(format!("Response serialization error: {e}"))
                })
            }

            "contextra_relate" => {
                let from = match args.get("from") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'from' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("from cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "from length exceeds limit: max 256 chars",
                            ));
                        }
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params("missing required field: 'from'"));
                    }
                };

                let to = match args.get("to") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'to' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("to cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "to length exceeds limit: max 256 chars",
                            ));
                        }
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params("missing required field: 'to'"));
                    }
                };

                let label = match args.get("label") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'label' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("label cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "label length exceeds limit: max 256 chars",
                            ));
                        }
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params("missing required field: 'label'"));
                    }
                };

                let col_name = if let Some(col_val) = args.get("collection") {
                    let s = col_val.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'collection' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        "default"
                    } else {
                        validate_collection_name(s)?;
                        s
                    }
                } else {
                    "default"
                };

                let bidirectional = match args.get("bidirectional") {
                    Some(Value::Bool(b)) => *b,
                    Some(_) => {
                        return Err(McpError::invalid_params(
                            "Invalid params: 'bidirectional' must be a boolean",
                        ));
                    }
                    None => false,
                };

                if let Some(matched_pattern) = self.injection_guard.detect(label) {
                    return Err(McpError::invalid_params(format!(
                        "Prompt injection detected in label: {matched_pattern}"
                    )));
                }

                let col = self.db.collection(col_name).await.map_err(McpError::from)?;
                if bidirectional {
                    col.relate_bidirectional(from, to, label)
                        .await
                        .map_err(McpError::from)?;
                } else {
                    col.relate(from, to, label).await.map_err(McpError::from)?;
                }

                Ok(json!({
                    "status": "ok",
                    "from": from,
                    "to": to,
                    "label": label,
                    "collection": col_name,
                    "bidirectional": bidirectional
                }))
            }

            "contextra_relate_n_ary" => {
                let predicate = match args.get("predicate") {
                    Some(v) => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params("Invalid params: 'predicate' must be a string")
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("predicate cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "predicate length exceeds limit: max 256 chars",
                            ));
                        }
                        s
                    }
                    None => {
                        return Err(McpError::invalid_params(
                            "missing required field: 'predicate'",
                        ));
                    }
                };

                if let Some(matched_pattern) = self.injection_guard.detect(predicate) {
                    return Err(McpError::invalid_params(format!(
                        "Prompt injection detected in predicate: {matched_pattern}"
                    )));
                }

                let participants_arr = match args.get("participants") {
                    Some(v) => v.as_array().ok_or_else(|| {
                        McpError::invalid_params(
                            "Invalid params: 'participants' must be an array of objects",
                        )
                    })?,
                    None => {
                        return Err(McpError::invalid_params(
                            "missing required field: 'participants'",
                        ));
                    }
                };

                if participants_arr.len() < 2 {
                    return Err(McpError::invalid_params(format!(
                        "participants must contain at least 2 items, found {}",
                        participants_arr.len()
                    )));
                }
                if participants_arr.len() > MAX_RELATE_PARTICIPANTS {
                    return Err(McpError::invalid_params(format!(
                        "participants count exceeds limit: {} > {} limit",
                        participants_arr.len(),
                        MAX_RELATE_PARTICIPANTS
                    )));
                }

                let mut participant_pairs: Vec<(&str, &str)> =
                    Vec::with_capacity(participants_arr.len());

                for (idx, elem) in participants_arr.iter().enumerate() {
                    let obj = elem.as_object().ok_or_else(|| {
                        McpError::invalid_params(format!(
                            "Invalid params: participant at index {idx} must be an object"
                        ))
                    })?;

                    let doc_id = match obj.get("doc_id") {
                        Some(v) => {
                            let s = v.as_str().ok_or_else(|| {
                                McpError::invalid_params(format!(
                                    "Invalid params: participant[{idx}].doc_id must be a string"
                                ))
                            })?;
                            if s.trim().is_empty() {
                                return Err(McpError::invalid_params(format!(
                                    "participant[{idx}].doc_id cannot be empty"
                                )));
                            }
                            if s.len() > 256 {
                                return Err(McpError::invalid_params(format!(
                                    "participant[{idx}].doc_id length exceeds limit: max 256 chars"
                                )));
                            }
                            s
                        }
                        None => {
                            return Err(McpError::invalid_params(format!(
                                "missing required field: participant[{idx}].doc_id"
                            )));
                        }
                    };

                    let role = match obj.get("role") {
                        Some(v) => {
                            let s = v.as_str().ok_or_else(|| {
                                McpError::invalid_params(format!(
                                    "Invalid params: participant[{idx}].role must be a string"
                                ))
                            })?;
                            if s.trim().is_empty() {
                                return Err(McpError::invalid_params(format!(
                                    "participant[{idx}].role cannot be empty"
                                )));
                            }
                            if s.len() > 256 {
                                return Err(McpError::invalid_params(format!(
                                    "participant[{idx}].role length exceeds limit: max 256 chars"
                                )));
                            }
                            s
                        }
                        None => {
                            return Err(McpError::invalid_params(format!(
                                "missing required field: participant[{idx}].role"
                            )));
                        }
                    };

                    if let Some(matched_pattern) = self.injection_guard.detect(role) {
                        return Err(McpError::invalid_params(format!(
                            "Prompt injection detected in participant[{idx}].role: {matched_pattern}"
                        )));
                    }

                    participant_pairs.push((doc_id, role));
                }

                let source_doc_id_opt = match args.get("source_doc_id") {
                    Some(v) if !v.is_null() => {
                        let s = v.as_str().ok_or_else(|| {
                            McpError::invalid_params(
                                "Invalid params: 'source_doc_id' must be a string",
                            )
                        })?;
                        if s.trim().is_empty() {
                            return Err(McpError::invalid_params("source_doc_id cannot be empty"));
                        }
                        if s.len() > 256 {
                            return Err(McpError::invalid_params(
                                "source_doc_id length exceeds limit: max 256 chars",
                            ));
                        }
                        Some(s)
                    }
                    _ => None,
                };

                let col_name = if let Some(col_val) = args.get("collection") {
                    let s = col_val.as_str().ok_or_else(|| {
                        McpError::invalid_params("Invalid params: 'collection' must be a string")
                    })?;
                    if s.trim().is_empty() {
                        "default"
                    } else {
                        validate_collection_name(s)?;
                        s
                    }
                } else {
                    "default"
                };

                let col = self.db.collection(col_name).await.map_err(McpError::from)?;
                let hyperedge_id = col
                    .relate_n_ary(predicate, &participant_pairs, source_doc_id_opt)
                    .await
                    .map_err(McpError::from)?;

                Ok(json!({
                    "status": "ok",
                    "hyperedge_id": hyperedge_id.inner(),
                    "participants": participant_pairs.len(),
                    "collection": col_name
                }))
            }

            "contextra_plugin_status" => {
                let registry = self.plugin_registry.as_ref().map(|r| r.as_ref());
                let empty_registry;
                let reg_ref = match registry {
                    Some(r) => r,
                    None => {
                        empty_registry = contextra_ports::plugin::PluginRegistry::default();
                        &empty_registry
                    }
                };
                let status = crate::plugin_status::handle_plugin_status(reg_ref).await?;
                serde_json::to_value(&status).map_err(|e| {
                    McpError::internal_error(format!("Response serialization error: {e}"))
                })
            }

            other => Err(McpError::invalid_params(format!(
                "Unbekanntes Tool: {other}"
            ))),
        }
    }
}
