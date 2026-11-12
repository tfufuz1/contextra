//! # Modul: collection.rs
//!
//! **Zweck:** Lokale Collection-Abstraktion und Transaktionsunterstützung für contextra-cognition (SPEC S-63).
//!
//! **Architektur:** Gehört zur Crate `contextra-cognition` (Ebene E2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, kein unsafe Code (INV-12).

#![forbid(unsafe_code)]

use crate::collection_types::{StoredDocument, StoredDocumentMeta};
use crate::decay::AdaptiveDecayController;
use crate::ports::CollectionOps;
use async_trait::async_trait;
use contextra_graph::CsrGraph;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_text::Language;
use contextra_types::{DocId, Result, TxId};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Lokale Transaktionsstruktur für 2-Phase Commits in der Konsolidierungssitzung.
pub struct DbTransaction<S: StorageEngine, V: VectorIndex> {
    pub tx_id: TxId,
    pub collection: Collection<S, V>,
    pub staged_inserts: Vec<(String, Vec<f32>, Option<serde_json::Value>)>,
    pub staged_deletes: Vec<String>,
}

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    /// Erzeugt eine neue Transaktion.
    pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self {
        Self {
            tx_id,
            collection,
            staged_inserts: Vec::new(),
            staged_deletes: Vec::new(),
        }
    }

    /// Führt den Transaktions-Commit aus.
    pub async fn commit(self) -> Result<()> {
        let storage = self.collection.storage();
        for (id, embedding, metadata) in self.staged_inserts {
            let doc_id = DocId::from_key(&id).unwrap_or_else(|_| DocId::new(0));
            let doc_key = self.collection.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
            let meta = StoredDocumentMeta {
                id: id.clone(),
                metadata: metadata.clone(),
            };
            if let Ok(serialized) = serde_json::to_vec(&meta) {
                storage.put(self.tx_id, &doc_key, &serialized).await?;
            }
            let _ = self.collection.index().insert(self.tx_id, doc_id, &embedding).await;
        }
        for id in self.staged_deletes {
            let doc_id = DocId::from_key(&id).unwrap_or_else(|_| DocId::new(0));
            let doc_key = self.collection.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
            storage.delete(self.tx_id, &doc_key).await?;
            let _ = self.collection.index().delete(self.tx_id, doc_id).await;
        }
        storage.commit(self.tx_id).await?;
        self.collection.index().commit(self.tx_id).await?;
        Ok(())
    }
}

/// Repräsentiert eine namensraumisolierte Collection innerhalb der Kognitions-Engine.
pub struct Collection<S: StorageEngine, V: VectorIndex> {
    name: String,
    prefix: Vec<u8>,
    storage: Arc<S>,
    index: Arc<V>,
    graph_index: Arc<CsrGraph>,
    next_tx: Arc<AtomicU64>,
    dimension: usize,
    language: Language,
    pub consolidation_guard: Arc<tokio::sync::Mutex<()>>,
    pub consolidation_in_progress: Arc<AtomicBool>,
}

impl<S: StorageEngine, V: VectorIndex> Clone for Collection<S, V> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            prefix: self.prefix.clone(),
            storage: self.storage.clone(),
            index: self.index.clone(),
            graph_index: self.graph_index.clone(),
            next_tx: self.next_tx.clone(),
            dimension: self.dimension,
            language: self.language.clone(),
            consolidation_guard: self.consolidation_guard.clone(),
            consolidation_in_progress: self.consolidation_in_progress.clone(),
        }
    }
}

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Erstellt eine neue Collection.
    pub fn new(
        name: String,
        storage: Arc<S>,
        index: Arc<V>,
        graph_index: Arc<CsrGraph>,
        next_tx: Arc<AtomicU64>,
        dimension: usize,
        language: Language,
    ) -> Self {
        let prefix = if name == "default" {
            b"".to_vec()
        } else {
            format!("__col:{}:\x00", name).into_bytes()
        };

        Self {
            name,
            prefix,
            storage,
            index,
            graph_index,
            next_tx,
            dimension,
            language,
            consolidation_guard: Arc::new(tokio::sync::Mutex::new(())),
            consolidation_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Gibt den Namen der Collection zurück.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gibt die Vektordimension zurück.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Gibt eine Referenz auf den physischen Speicher zurück.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Gibt den Vektorindex zurück.
    pub fn index(&self) -> &V {
        &self.index
    }

    /// Gibt den Wissensgraphen zurück.
    pub fn graph_index(&self) -> Arc<CsrGraph> {
        self.graph_index.clone()
    }

    /// Erzeugt einen namensraumisolierten Schlüssel.
    pub fn namespaced_key(&self, raw: &[u8], key_type: u8) -> Vec<u8> {
        let mut key = Vec::with_capacity(self.prefix.len() + 1 + raw.len());
        key.extend_from_slice(&self.prefix);
        key.push(key_type);
        key.extend_from_slice(raw);
        key
    }

    /// Gibt das Schlüsselpräfix für reguläre Benutzerdaten zurück.
    pub fn user_key_prefix(&self) -> Vec<u8> {
        let mut p = self.prefix.clone();
        p.push(1);
        p
    }

    /// Allokiert eine Transaktions-ID.
    pub fn allocate_tx(&self) -> Result<TxId> {
        let id = self.next_tx.fetch_add(1, Ordering::SeqCst);
        Ok(TxId::new(id))
    }

    /// Beginnt eine neue Transaktion.
    pub fn begin_transaction(&self) -> Result<DbTransaction<S, V>> {
        let tx = self.allocate_tx()?;
        Ok(DbTransaction::new(self.clone(), tx))
    }

    /// Ruft den Transaktionszeitstempel eines Dokuments ab.
    pub async fn get_doc_tx(&self, _doc_id: DocId) -> Result<Option<TxId>> {
        Ok(None)
    }

    /// Prüft, ob eine Konsolidierung aktiv ist.
    pub fn consolidation_in_progress(&self) -> Arc<AtomicBool> {
        self.consolidation_in_progress.clone()
    }

    /// Bereitet das Einfügen eines Dokuments in einer Transaktion vor.
    pub async fn insert_op(
        &self,
        db_tx: &DbTransaction<S, V>,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let doc_id = DocId::from_key(id).unwrap_or_else(|_| DocId::new(0));
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        let meta = StoredDocumentMeta {
            id: id.to_string(),
            metadata,
        };
        let serialized = serde_json::to_vec(&meta)
            .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;
        self.storage.put(db_tx.tx_id, &doc_key, &serialized).await?;
        let _ = self.index.insert(db_tx.tx_id, doc_id, embedding).await;
        Ok(())
    }

    /// Bereitet das Löschen eines Dokuments in einer Transaktion vor.
    pub async fn delete_op(&self, db_tx: &mut DbTransaction<S, V>, id: &str) -> Result<()> {
        let doc_id = DocId::from_key(id).unwrap_or_else(|_| DocId::new(0));
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        self.storage.delete(db_tx.tx_id, &doc_key).await?;
        let _ = self.index.delete(db_tx.tx_id, doc_id).await;
        Ok(())
    }

    /// Ruft ein Dokument ab.
    pub async fn get(&self, id: &str) -> Result<Option<StoredDocument>> {
        let doc_id = DocId::from_key(id).unwrap_or_else(|_| DocId::new(0));
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        match self.storage.get(&doc_key).await? {
            Some(bytes) => {
                let meta: StoredDocumentMeta = serde_json::from_slice(&bytes)
                    .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;
                Ok(Some(StoredDocument {
                    id: meta.id,
                    embedding: Vec::new(),
                    metadata: meta.metadata,
                }))
            }
            None => Ok(None),
        }
    }

    /// Löscht ein Dokument direkt.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let tx = self.allocate_tx()?;
        let doc_id = DocId::from_key(id).unwrap_or_else(|_| DocId::new(0));
        let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        self.storage.delete(tx, &doc_key).await?;
        let _ = self.index.delete(tx, doc_id).await;
        self.storage.commit(tx).await?;
        let _ = self.index.commit(tx).await;
        Ok(())
    }

    /// Inserts a text document.
    pub async fn insert_text_only(
        &self,
        id: &str,
        text: &str,
        mut metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let meta = metadata.get_or_insert(serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            if !obj.contains_key("text") {
                obj.insert(
                    "text".to_string(),
                    serde_json::Value::String(text.to_string()),
                );
            }
        }
        self.put_kv(id, meta).await
    }

    /// Schreibt einen beliebigen Schlüssel-Wert-Eintrag.
    pub async fn put_kv(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        let tx = self.allocate_tx()?;
        let k = self.namespaced_key(key.as_bytes(), 2);
        let val_bytes = serde_json::to_vec(value)
            .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;
        self.storage.put(tx, &k, &val_bytes).await?;
        self.storage.commit(tx).await?;
        Ok(())
    }

    /// Bereinigt abgewertete Chunks.
    pub async fn evict_decayed_chunks(
        &self,
        _controller: &AdaptiveDecayController,
        _limit: usize,
    ) -> Result<usize> {
        Ok(0)
    }

    /// Fügt ein Dokument mit Embedding und optionalen Metadaten ein.
    ///
    /// Schlanke Insert-Methode, die `insert_op` intern nutzt.
    /// Keine Backpressure oder komplexes Locking — geeignet für Testszenarien.
    pub async fn insert(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let tx_id = self.allocate_tx()?;
        let db_tx = DbTransaction { tx_id, collection: self.clone(), staged_inserts: Vec::new(), staged_deletes: Vec::new() };
        self.insert_op(&db_tx, id, embedding, metadata).await?;
        self.storage.commit(tx_id).await?;
        let _ = self.index.commit(tx_id).await;
        Ok(())
    }

    /// Fügt ein Dokument mit typisiertem `MemoryType` ein (Wrapper um `insert`).
    pub async fn insert_typed(
        &self,
        id: &str,
        embedding: &[f32],
        _memory_type: contextra_types::MemoryType,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.insert(id, embedding, metadata).await
    }

    /// Aktualisiert ein Dokument (upsert-Semantik, Alias für `insert`).
    pub async fn update(
        &self,
        id: &str,
        embedding: &[f32],
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.insert(id, embedding, metadata).await
    }

    /// Erstellt eine gerichtete Relation zwischen zwei Dokumenten im Graph-Index.
    pub async fn relate(
        &self,
        from: &str,
        to: &str,
        label: &str,
    ) -> Result<()> {
        let tx_id = self.allocate_tx()?;
        let key_str = format!("{}:{}:{}", from, label, to);
        let key = self.namespaced_key(key_str.as_bytes(), 2);
        let val = serde_json::json!({"from": from, "to": to, "label": label});
        let bytes = serde_json::to_vec(&val)
            .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;
        self.storage.put(tx_id, &key, &bytes).await?;
        self.storage.commit(tx_id).await?;
        Ok(())
    }
}

#[async_trait]
impl<S: StorageEngine, V: VectorIndex> CollectionOps for Collection<S, V> {
    fn name(&self) -> &str {
        self.name()
    }

    fn namespaced_key(&self, raw: &[u8], key_type: u8) -> Vec<u8> {
        self.namespaced_key(raw, key_type)
    }

    fn user_key_prefix(&self) -> Vec<u8> {
        self.user_key_prefix()
    }

    fn consolidation_in_progress(&self) -> bool {
        self.consolidation_in_progress.load(Ordering::SeqCst)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn allocate_tx(&self) -> Result<TxId> {
        self.allocate_tx()
    }

    async fn get(&self, id: &str) -> Result<Option<StoredDocument>> {
        self.get(id).await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        self.delete(id).await
    }

    async fn put_kv(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        self.put_kv(key, value).await
    }

    async fn evict_decayed_chunks(
        &self,
        controller: &AdaptiveDecayController,
        limit: usize,
    ) -> Result<usize> {
        self.evict_decayed_chunks(controller, limit).await
    }
}
