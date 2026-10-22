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

/// Strukturierte Parameter für Dokumenteinfügung.
struct ParsedInsertParams<'a> {
    collection_name: &'a str,
    id: &'a str,
    vector_opt: Option<Vec<f32>>,
    text_opt: Option<&'a str>,
    base_metadata: serde_json::Map<String, Value>,
}

impl McpServer {
    /// Zentraler Tool-Dispatch des MCP-Servers.
    ///
    /// # Sicherheits- und Ausführungsreihenfolge:
    /// 1. **Sandbox-Validierung:** `self.sandbox.validate_tool_call(name, args)` erzwingt Rollen- und Policy-Prüfungen vor jedem Seiteneffekt.
    /// 2. **Tool-Name Dispatch:** Namensabgleich und Delegierung an dezidierte Handler-Funktion.
    /// 3. **Argument-Extraktion & Validierung:** Pflichtfelder, Typen, Längenbeschränkungen, Collection-Namen, Prompt-Injection-Erkennung sowie Confirm-Flags.
    /// 4. **Ausführung & Seiteneffekte:** Speicher- und Indexzugriffe, Einbettung, Konsolidierung, Cloud-Query oder Graph-Relationen.
    /// 5. **Ergebnis-Serialisierung & Sanitizing:** Anreicherung mit Inhaltsherkunft (`retrieved_untrusted_data`) und Injection-Guard-Prüfung.
    pub(crate) async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, McpError> {
        self.sandbox
            .validate_tool_call(name, args)
            .map_err(McpError::from)?;

        match name {
            "contextra_search" => self.handle_search(args).await,
            "contextra_insert" | "contextra_upsert" => self.handle_insert_upsert(args).await,
            "contextra_get" => self.handle_get(args).await,
            "contextra_forget" => self.handle_forget(args).await,
            "contextra_collections" => self.handle_collections().await,
            "contextra_cloud_query" => self.handle_cloud_query(args).await,
            "contextra_relate" => self.handle_relate(args).await,
            "contextra_relate_n_ary" => self.handle_relate_n_ary(args).await,
            "contextra_delete" => self.handle_delete(args).await,
            "contextra_create_collection" => self.handle_create_collection(args).await,
            "contextra_drop_collection" => self.handle_drop_collection(args).await,
            "contextra_plugin_status" => self.handle_plugin_status().await,
            other => Err(McpError::invalid_params(format!(
                "Unbekanntes Tool: {other}"
            ))),
        }
    }

    /// Handler für `contextra_search`.
    ///
    /// Vorbedingung: `args` enthält optional `collection`, `k`/`limit` und zwingend `query` (nicht leer, max `MAX_SEARCH_QUERY_BYTES`).
    /// Nachbedingung: Führt eine hybride Text-/Vektorsuche aus, konsultiert ggf. die KV-Bridge, reichert Ergebnisse mit Provenance
    /// (`retrieved_untrusted_data`) an und unterzieht sie dem Prompt-Injection-Guard.
    async fn handle_search(&self, args: &Value) -> Result<Value, McpError> {
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
            Some(Value::Number(n)) => n
                .as_u64()
                .ok_or_else(|| McpError::invalid_params("k/limit muss eine positive Ganzzahl sein"))?
                as usize,
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
                let chunk_id = DocId::from_key(&res.id).map(|d| d.inner()).unwrap_or(0);
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

    /// Hilfsfunktion zur Extraktion und Validierung der gemeinsamen Einfügeparameter (ID, Vektor, Text, Metadaten).
    fn parse_insert_params<'a>(
        &self,
        args: &'a Value,
    ) -> Result<ParsedInsertParams<'a>, McpError> {
        let collection_name = if let Some(col_val) = args.get("collection") {
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

        let vec_val = args.get("vector");
        let text_val = args.get("text");

        let vector_opt: Option<Vec<f32>> = if let Some(v) = vec_val {
            let arr = v.as_array().ok_or_else(|| {
                McpError::invalid_params("Invalid params: 'vector' must be an array of numbers")
            })?;
            if arr.is_empty() {
                return Err(McpError::invalid_params("vector cannot be empty"));
            }
            let mut vec = Vec::with_capacity(arr.len());
            for elem in arr {
                let num = elem.as_f64().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'vector' must contain numbers")
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

        Ok(ParsedInsertParams {
            collection_name,
            id,
            vector_opt,
            text_opt,
            base_metadata,
        })
    }

    /// Handler für `contextra_insert` und `contextra_upsert`.
    ///
    /// Vorbedingung: `args` enthält Pflichtfeld `id` (max 256 Zeichen) sowie mindestens eines aus `vector` (Gleitkomma-Array,
    /// keine NaN/Inf) oder `text` (nicht leer, max 10MB). Optional `collection` und `metadata`.
    /// Nachbedingung: Fügt das Dokument direkt (bei `vector`) oder nach automatischer Markdown-Chunkierung und Einbettung (bei `text`)
    /// in die Collection ein und liefert Status mit eingefügten `chunk_ids` zurück.
    async fn handle_insert_upsert(&self, args: &Value) -> Result<Value, McpError> {
        let params = self.parse_insert_params(args)?;
        let col_name = params.collection_name;
        let id = params.id;

        let col = self.db.collection(col_name).await.map_err(McpError::from)?;

        if let Some(vector) = params.vector_opt {
            let mut meta = json!({
                "source_id": id,
            });
            if let Some(obj) = meta.as_object_mut() {
                if let Some(text) = params.text_opt {
                    obj.insert("text".to_string(), json!(text));
                }
                for (k, v) in &params.base_metadata {
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

        let text = match params.text_opt {
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
                for (k, v) in &params.base_metadata {
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

    /// Handler für `contextra_get`.
    ///
    /// Vorbedingung: `args` enthält Pflichtfeld `id` (nicht leer, max 256 Zeichen) und optional `collection`.
    /// Nachbedingung: Liefert das Dokument (angereichert mit Provenance und Injection-Guard-Geprüft) oder `null` zurück.
    async fn handle_get(&self, args: &Value) -> Result<Value, McpError> {
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

    /// Handler für `contextra_forget`.
    ///
    /// Vorbedingung: `args` enthält Pflichtfeld `collection` und explizit `confirm: true`. Optional `id`.
    /// Nachbedingung: Bei Angabe von `id` wird ein einzelnes Dokument gelöscht (ohne Löschnachweis, Scope `"collection_only"`).
    /// Ohne `id` wird die gesamte Collection inkl. Erzeugung eines `DeletionProof` aus dem Umgebungsschlüssel gelöscht.
    async fn handle_forget(&self, args: &Value) -> Result<Value, McpError> {
        let session_tenant = crate::validation::resolve_session_tenant_from_env()
            .map_err(McpError::from)?;
        let _tenant_id = crate::validation::validate_request_tenant(session_tenant, Some(args))?;

        let col_name = match args.get("collection") {
            Some(col_val) => {
                let s = col_val.as_str().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'collection' must be a string")
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

        let assurance_requested = match args.get("assurance") {
            Some(v) => {
                let s = v.as_str().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'assurance' must be a string (S1, S2, S3, S4)")
                })?;
                match s.trim() {
                    "S1" => contextra_crypto::deletion_proof::claim_text::WitAssurance::S1,
                    "S2" => contextra_crypto::deletion_proof::claim_text::WitAssurance::S2,
                    "S3" => contextra_crypto::deletion_proof::claim_text::WitAssurance::S3,
                    "S4" => contextra_crypto::deletion_proof::claim_text::WitAssurance::S4,
                    invalid => {
                        return Err(McpError::invalid_params(format!(
                            "Invalid assurance level '{invalid}'. Expected one of: S1, S2, S3, S4"
                        )));
                    }
                }
            }
            None => contextra_crypto::deletion_proof::claim_text::WitAssurance::S1,
        };

        let purge_requested = match args.get("purge") {
            Some(v) => {
                let s = v.as_str().ok_or_else(|| {
                    McpError::invalid_params("Invalid params: 'purge' must be a string (None, Purge, PurgeWithScrub)")
                })?;
                match s.trim() {
                    "None" | "none" => contextra::ForgetOptions::default().purge,
                    "Purge" | "purge" => contextra::ForgetOptions::default().with_purge(contextra::PurgeMode::Purge).purge,
                    "PurgeWithScrub" | "purgewithscrub" => contextra::ForgetOptions::default().with_purge(contextra::PurgeMode::PurgeWithScrub).purge,
                    invalid => {
                        return Err(McpError::invalid_params(format!(
                            "Invalid purge mode '{invalid}'. Expected one of: None, Purge, PurgeWithScrub"
                        )));
                    }
                }
            }
            None => contextra::ForgetOptions::default().purge,
        };

        let opts = contextra::ForgetOptions::default()
            .with_assurance(assurance_requested)
            .with_purge(purge_requested);

        let exclusions = json!([
            "client_and_llm_context_not_covered",
            "external_backups_not_covered",
            "flash_remanence_not_covered_without_S4"
        ]);

        if let Some(id) = id_opt {
            let receipt = self.db.forget(id, opts).await.map_err(McpError::from)?;

            let proof_present = receipt.proof.is_some();
            let proof_val = match &receipt.proof {
                Some(p) => Some(serde_json::to_value(p).map_err(|e| McpError::internal_error(e.to_string()))?),
                None => None,
            };
            let layers_val = serde_json::to_value(&receipt.layers)
                .map_err(|e| McpError::internal_error(e.to_string()))?;

            let claim_text = match receipt.assurance_reached {
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S0 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S0.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S1 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S1.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S2 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S2V.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S3 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S3.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S4 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S4.text()
                }
                _ => contextra_crypto::deletion_proof::claim_text::WitClaimText::S0.text(),
            };

            let ok = receipt.assurance_reached >= receipt.assurance_requested;
            let reason = if !ok {
                Some(format!(
                    "Requested assurance {:?} was not reached (reached {:?})",
                    receipt.assurance_requested, receipt.assurance_reached
                ))
            } else {
                None
            };

            Ok(json!({
                "ok": ok,
                "collection": col_name,
                "id": id,
                "assurance_requested": format!("{:?}", receipt.assurance_requested),
                "assurance_reached": format!("{:?}", receipt.assurance_reached),
                "claim": claim_text,
                "proof_present": proof_present,
                "proof": proof_val,
                "layers": layers_val,
                "exclusions": exclusions,
                "reason": reason,
            }))
        } else {
            let receipt = self.db.forget_collection(col_name, opts).await.map_err(McpError::from)?;

            let proof_present = receipt.proof.is_some();
            let proof_val = match &receipt.proof {
                Some(p) => Some(serde_json::to_value(p).map_err(|e| McpError::internal_error(e.to_string()))?),
                None => None,
            };
            let layers_val = serde_json::to_value(&receipt.layers)
                .map_err(|e| McpError::internal_error(e.to_string()))?;

            let claim_text = match receipt.assurance_reached {
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S0 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S0.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S1 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S1.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S2 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S2V.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S3 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S3.text()
                }
                contextra_crypto::deletion_proof::claim_text::WitAssurance::S4 => {
                    contextra_crypto::deletion_proof::claim_text::WitClaimText::S4.text()
                }
                _ => contextra_crypto::deletion_proof::claim_text::WitClaimText::S0.text(),
            };

            let ok = receipt.assurance_reached >= receipt.assurance_requested;
            let reason = if !ok {
                Some(format!(
                    "Requested assurance {:?} was not reached (reached {:?})",
                    receipt.assurance_requested, receipt.assurance_reached
                ))
            } else {
                None
            };

            Ok(json!({
                "ok": ok,
                "collection": col_name,
                "assurance_requested": format!("{:?}", receipt.assurance_requested),
                "assurance_reached": format!("{:?}", receipt.assurance_reached),
                "claim": claim_text,
                "proof_present": proof_present,
                "proof": proof_val,
                "layers": layers_val,
                "exclusions": exclusions,
                "reason": reason,
            }))
        }
    }

    /// Handler für `contextra_collections`.
    ///
    /// Vorbedingung: Keine.
    /// Nachbedingung: Gibt die Liste aller vorhandenen Collection-Namen als JSON zurück.
    async fn handle_collections(&self) -> Result<Value, McpError> {
        let names = self.db.list_collections().await.map_err(McpError::from)?;
        Ok(json!({ "collections": names }))
    }

    /// Handler für `contextra_consolidate`.
    ///
    /// Vorbedingung: `args` enthält optional `collection`.
    /// Nachbedingung: Liest chronologisch turns aus der Collection, führt die Background-Konsolidierung aus
    /// und liefert den Konsolidierungsbericht zurück.
    async fn handle_consolidate(&self, args: &Value) -> Result<Value, McpError> {
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
            if let Ok(stored) = contextra::engine::collection::StoredDocument::from_slice(&v) {
                if let Ok(doc_id) = DocId::from_key(&stored.id) {
                    turns.push((doc_id, stored.embedding));
                }
            } else if let Ok(stored) = serde_json::from_slice::<serde_json::Value>(&v) {
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

    /// Handler für `contextra_cloud_query`.
    ///
    /// Vorbedingung: `args` enthält Pflichtfeld `query` (max `MAX_SEARCH_QUERY_BYTES`), optional `collection` und optional `max_results`/`k` (max `MAX_SEARCH_K`).
    /// Nachbedingung: Leitet die Anfrage über `egress_gateway::handle_cloud_query` mit Egress-Klassifikation weiter und gibt die Antwort zurück.
    async fn handle_cloud_query(&self, args: &Value) -> Result<Value, McpError> {
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

        let max_results = if let Some(k_val) = args.get("max_results").or_else(|| args.get("k")) {
            match k_val {
                Value::Number(n) => {
                    let k_raw = n.as_u64().ok_or_else(|| {
                        McpError::invalid_params("max_results muss eine positive Ganzzahl sein")
                    })? as usize;
                    Some(k_raw.min(MAX_SEARCH_K))
                }
                _ => return Err(McpError::invalid_params("max_results muss eine Zahl sein")),
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
            egress_gateway::handle_cloud_query(request, self.egress_classifier.as_ref()).await?;

        serde_json::to_value(&response).map_err(|e| {
            McpError::internal_error(format!("Response serialization error: {e}"))
        })
    }

    /// Handler für `contextra_relate`.
    ///
    /// Vorbedingung: `args` enthält Pflichtfelder `from`, `to`, `label` (jeweils max 256 Zeichen; `label` frei von Prompt Injections),
    /// optional `collection` und optional `bidirectional` Bool.
    /// Nachbedingung: Erstellt eine gerichtete oder bidirektionale Relation im Graph und liefert Erfolgsstatus.
    async fn handle_relate(&self, args: &Value) -> Result<Value, McpError> {
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

    /// Hilfsfunktion zur Validierung und Extraktion der n-ären Teilnehmer-Paare (`doc_id`, `role`).
    fn parse_n_ary_participants<'a>(
        &self,
        participants_arr: &'a [Value],
    ) -> Result<Vec<(&'a str, &'a str)>, McpError> {
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

        Ok(participant_pairs)
    }

    /// Handler für `contextra_relate_n_ary`.
    ///
    /// Vorbedingung: `args` enthält Pflichtfeld `predicate` (max 256 Zeichen, injection-frei) und `participants` Array
    /// (2 bis `MAX_RELATE_PARTICIPANTS` Objekte mit `doc_id` und injection-freiem `role`). Optional `source_doc_id` und `collection`.
    /// Nachbedingung: Erstellt eine n-äre Hyperkante im Graph und liefert `hyperedge_id` und Teilnehmeranzahl zurück.
    async fn handle_relate_n_ary(&self, args: &Value) -> Result<Value, McpError> {
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

        let participant_pairs = self.parse_n_ary_participants(participants_arr)?;

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

    /// Handler für `contextra_plugin_status`.
    ///
    /// Vorbedingung: Keine.
    /// Nachbedingung: Erstellt einen Plugin-Statusbericht über die `plugin_registry` (oder Fallback mit `OpenFastGate`)
    /// und liefert das serialisierte Ergebnis zurück.
    async fn handle_plugin_status(&self) -> Result<Value, McpError> {
        let registry = self.plugin_registry.as_ref().map(|r| r.as_ref());
        let empty_registry;
        let reg_ref = match registry {
            Some(r) => r,
            None => {
                empty_registry = contextra_ports::plugin::PluginRegistry::new(
                    std::sync::Arc::new(contextra_license::OpenFastGate),
                );
                &empty_registry
            }
        };
        let status = crate::plugin_status::handle_plugin_status(reg_ref).await?;
        serde_json::to_value(&status).map_err(|e| {
            McpError::internal_error(format!("Response serialization error: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_relate_participants_constant() {
        assert_eq!(MAX_RELATE_PARTICIPANTS, 64);
    }
}
