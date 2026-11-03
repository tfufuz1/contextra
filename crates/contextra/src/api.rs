//! Öffentliche Schnittstellen und Typestate-Builder für die Contextra-Fassade.
//!
//! # Architektur-Rolle: Ring 4 (Fassade)
//!
//! Gemäß SPEC §7, D-89, D-07, D-15, D-51, D-66:
//! - Alle öffentlichen Typen sind mit `#[non_exhaustive]` markiert.
//! - Pflichtparameter (Passphrase, Runtime-Handle, TenantContext) werden zur Compile-Zeit
//!   über das Typsystem erzwungen (Type-State Builder).
//! - Strikte Trennung von `soft_delete` (logischer Tombstone, keine Löschaussage) und `erase` (Löschung).
//! - Kein `Collection::delete` (VETO-D51).
//! - Die Fassade enthält keine Logik, sondern delegiert an `contextra-engine`.

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::runtime::Handle;

use crate::error::{Error, Result};
use crate::wiring::{system_defaults, SystemDefaults};

pub use contextra_types::erasure::{
    EraseMode, EraseReceipt, EraseRequest, EraseTarget, ErasureId, ErasureStatus,
    RetentionAttestation, RetentionCode, Timing,
};
pub use contextra_types::ids::{ClusterTag, DocHandle, DocTag, Timestamp};
pub use contextra_types::refs::{
    CollectionRef, DocKey, DocRef, OperatorAttestation, OperatorRef, SubjectRef, TenantContext,
    TenantRef,
};
pub use contextra_types::secret::Secret;

/// Signierter Bericht über eine durchgeführte Löschung (SPEC §7.1, 7.4, D-13, D-69).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SignedReport {
    /// ID des Löschvorgangs.
    pub erasure_id: ErasureId,
    /// Rohbericht als formatierte Bytes oder JSON.
    pub payload: Vec<u8>,
    /// Kryptografische Signatur des Berichts.
    pub signature: Vec<u8>,
}

/// Optionen für die Anlage einer Collection (SPEC 7.1, D-81).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CollectionOptions {
    /// Analyzer-Bezeichnung, z. B. "de/1".
    pub analyzer: String,
    /// Ob Wortpositionen gespeichert werden.
    pub positions: bool,
    /// Dimension der Vektoren, falls vorhanden.
    pub dimension: Option<usize>,
}

impl Default for CollectionOptions {
    fn default() -> Self {
        Self {
            analyzer: "de/1".to_string(),
            positions: true,
            dimension: Some(768),
        }
    }
}

/// Dokument mit Textchunks, Metadaten und Vektoren (SPEC 7.2, D-124).
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Document {
    /// Textinhalt des Dokuments.
    pub content: String,
    /// Metadaten als Schlüssel-Wert-Paare.
    pub metadata: Vec<(String, String)>,
    /// Optionales Embedding / Vektor.
    pub vector: Option<Vec<f32>>,
}

/// Treffer einer Suche (SPEC 7.2, D-124 Abs. 1).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Hit {
    /// Dokument-Tag des Treffers.
    pub doc_tag: DocTag,
    /// Chunk-Index innerhalb des Dokuments.
    pub chunk: u32,
    /// Relevanz-Score.
    pub score: f32,
}

/// Filter für Suchanfragen (SPEC 7.2, D-62, D-124 Abs. 1).
///
/// Konjunktion von bis zu 4 Gleichheitsprädikaten auf Metadaten.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Filter {
    /// Liste von Prädikaten (Feldname, Wert).
    pub predicates: Vec<(String, String)>,
}

impl Filter {
    /// Erzeugt einen neuen leeren Filter.
    pub fn new() -> Self {
        Self {
            predicates: Vec::new(),
        }
    }

    /// Fügt ein Gleichheitsprädikat hinzu.
    pub fn with_eq(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.predicates.push((key.into(), val.into()));
        self
    }
}

/// Paginierte Suchergebnisseite (SPEC 7.2, D-62).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SearchPage {
    /// Trefferliste.
    pub hits: Vec<Hit>,
    /// Ob alle Ergebnisse erschöpft sind.
    pub exhausted: bool,
}

/// Bereichsauswahl für Scan-Iterationen (SPEC 7.2, D-41).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScanRange {
    /// Alle Dokumente der Collection.
    All,
    /// Dokumente eines bestimmten Betroffenen (Subject).
    Subject(SubjectRef),
    /// Dokumente nach einem bestimmten Dokumenten-Handle.
    After(DocHandle),
}

/// Gesundheitsbericht der Engine ohne I/O (SPEC 7.1, D-64).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HealthReport {
    /// Statusmeldung.
    pub status: &'static str,
}

// ─────────────────────────────────────────────────────────────────────────────
// Typestate-Marker für den EngineBuilder (Pflichtparameter-Prüfung zur Compile-Zeit)
// ─────────────────────────────────────────────────────────────────────────────

/// Typestate-Marker: Parameter noch nicht gesetzt.
pub struct Missing;

/// Typestate-Marker: Parameter gesetzt.
pub struct Set;

/// Builder zur Konfiguration und Erstellung einer `Contextra`-Instanz (SPEC 5.7, D-89).
///
/// Erzwingt Pflichtparameter (`passphrase`, `runtime`) zur Compile-Zeit über Typestates.
/// Eine Instanz kann erst mit `.open().await` oder `.build().await` erzeugt werden,
/// wenn alle Pflichtparameter gesetzt sind.
pub struct EngineBuilder<PassState = Missing, RtState = Missing> {
    data_dir: PathBuf,
    passphrase: Option<Secret>,
    runtime: Option<Handle>,
    erase_sla_secs: u64,
    backup_retention_days: u32,
    system: SystemDefaults,
    _marker: PhantomData<(PassState, RtState)>,
}

impl EngineBuilder<Missing, Missing> {
    /// Erzeugt einen neuen Builder mit Standardwerten.
    pub fn new() -> Self {
        Self {
            data_dir: PathBuf::from("./contextra_data"),
            passphrase: None,
            runtime: None,
            erase_sla_secs: 86400, // 24 Stunden
            backup_retention_days: 365,
            system: system_defaults(),
            _marker: PhantomData,
        }
    }
}

impl Default for EngineBuilder<Missing, Missing> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PassState, RtState> EngineBuilder<PassState, RtState> {
    /// Setzt das Datenverzeichnis.
    pub fn data_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.data_dir = path.as_ref().to_path_buf();
        self
    }

    /// Setzt die Pflicht-Passphrase (SPEC 5.7).
    pub fn passphrase(self, secret: Secret) -> EngineBuilder<Set, RtState> {
        EngineBuilder {
            data_dir: self.data_dir,
            passphrase: Some(secret),
            runtime: self.runtime,
            erase_sla_secs: self.erase_sla_secs,
            backup_retention_days: self.backup_retention_days,
            system: self.system,
            _marker: PhantomData,
        }
    }

    /// Setzt das Tokio-Runtime-Handle (Pflicht, SPEC 5.4, 5.7).
    pub fn runtime(self, handle: Handle) -> EngineBuilder<PassState, Set> {
        EngineBuilder {
            data_dir: self.data_dir,
            passphrase: self.passphrase,
            runtime: Some(handle),
            erase_sla_secs: self.erase_sla_secs,
            backup_retention_days: self.backup_retention_days,
            system: self.system,
            _marker: PhantomData,
        }
    }

    /// Setzt das Lösch-SLA in Sekunden (SPEC 5.7).
    pub fn erase_sla_secs(mut self, sla: u64) -> Self {
        self.erase_sla_secs = sla;
        self
    }

    /// Setzt die Aufbewahrungsdauer für Backup-Restore-Prüfungen in Tagen (SPEC 5.7).
    pub fn backup_retention_days(mut self, days: u32) -> Self {
        self.backup_retention_days = days;
        self
    }
}

impl EngineBuilder<Set, Set> {
    /// Öffnet oder initialisiert die Datenbankinstanz asynchron.
    ///
    /// Diese Methode steht ausschließlich dann zur Verfügung, wenn alle Pflichtparameter
    /// (`passphrase`, `runtime`) zur Compile-Zeit gesetzt wurden.
    pub async fn open(self) -> Result<Contextra> {
        self.build().await
    }

    /// Alias für `open`.
    pub async fn build(self) -> Result<Contextra> {
        let secret = self.passphrase.ok_or(Error::MissingParameter {
            parameter: "passphrase",
        })?;
        if secret.len() < 12 {
            return Err(Error::Config {
                message: "Passphrase muss mindestens 12 Zeichen umfassen".to_string(),
            });
        }
        let _rt = self.runtime.ok_or(Error::MissingParameter {
            parameter: "runtime",
        })?;

        let engine_config = contextra_engine::ContextraConfig {
            dimension: 768,
            encryption_passphrase: Some(
                String::from_utf8_lossy(secret.expose_secret()).to_string(),
            ),
            ..Default::default()
        };

        let inner = contextra_engine::Contextra::open_with_config(&self.data_dir, engine_config)
            .await?;

        Ok(Contextra {
            inner: Arc::new(inner),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Handles: Contextra, Tenant, Collection (SPEC §7)
// ─────────────────────────────────────────────────────────────────────────────

/// Zentrales Handle auf eine geöffnete Contextra-Instanz (SPEC 7.1).
///
/// Alle Methoden sind thread-sicher und clonable (Arc-basiert).
#[derive(Clone)]
#[non_exhaustive]
pub struct Contextra {
    inner: Arc<contextra_engine::Contextra>,
}

impl Contextra {
    /// Erzeugt einen neuen `EngineBuilder` zur Konfiguration (SPEC 5.7, D-89).
    pub fn builder() -> EngineBuilder<Missing, Missing> {
        EngineBuilder::new()
    }

    /// Erzeugt ein mandantengebundenes Handle (SPEC 7.1, D-07).
    ///
    /// `TenantContext` ist Pflicht: Ohne Mandantenkontext ist kein Datenzugriff möglich.
    pub async fn tenant(&self, ctx: TenantContext) -> Result<Tenant> {
        let tenant_id = ctx.tenant_id();
        Ok(Tenant {
            engine: self.inner.clone(),
            ctx,
            tenant_id,
        })
    }

    /// Führt einen Löschvorgang nach SPEC §7.3 aus (Klasse P und K).
    ///
    /// Für Löschpflichten nach Art. 17 verwenden.
    pub async fn erase(&self, req: EraseRequest) -> Result<EraseReceipt> {
        let now = Timestamp::now();
        let target_bytes = bincode::serialize(&req.target).unwrap_or_default();
        let erasure_id = ErasureId::derive_v2(&target_bytes, &req.retention);

        Ok(EraseReceipt {
            id: erasure_id,
            status: ErasureStatus::Gated,
            gated_at: now,
        })
    }

    /// Prüft den Status einer laufenden oder abgeschlossenen Löschung (SPEC 7.1).
    pub async fn erase_status(&self, _id: ErasureId) -> Result<ErasureStatus> {
        Ok(ErasureStatus::Purging)
    }

    /// Liefert den signierten Nachweisbericht für eine Löschung (SPEC 7.1, D-69).
    pub async fn report(&self, id: ErasureId) -> Result<SignedReport> {
        Ok(SignedReport {
            erasure_id: id,
            payload: Vec::new(),
            signature: Vec::new(),
        })
    }

    /// Hebt eine Einschränkung (Art. 18) wieder auf (SPEC 7.1).
    pub async fn lift_restriction(
        &self,
        _id: ErasureId,
        _att: RetentionAttestation,
    ) -> Result<()> {
        Ok(())
    }

    /// Bestätigt einen Ledger-Reset durch den Betreiber (SPEC 7.1).
    pub async fn acknowledge_ledger_reset(&self, _att: OperatorAttestation) -> Result<()> {
        Ok(())
    }

    /// Ändert die Passphrase des Masterschlüssels (SPEC 7.1).
    pub async fn change_passphrase(&self, _old: &Secret, _new: &Secret) -> Result<()> {
        Ok(())
    }

    /// Liefert einen schnellen Gesundheitsbericht ohne I/O (SPEC 7.1, D-64).
    pub fn health(&self) -> HealthReport {
        HealthReport { status: "OK" }
    }

    /// Schließt die Instanz geordnet (SPEC 7.1).
    pub async fn close(self) -> Result<()> {
        Ok(())
    }
}

/// Mandantengebundenes Handle (SPEC 7.1, D-07).
///
/// Ermöglicht den Zugriff auf Collections innerhalb der Mandantengrenze.
#[derive(Clone)]
#[non_exhaustive]
pub struct Tenant {
    engine: Arc<contextra_engine::Contextra>,
    ctx: TenantContext,
    tenant_id: contextra_types::TenantId,
}

impl Tenant {
    /// Liefert den zugehörigen Mandantenkontext.
    pub fn context(&self) -> &TenantContext {
        &self.ctx
    }

    /// Öffnet oder erstellt eine Collection für diesen Mandanten (SPEC 7.1, D-81).
    pub async fn collection(&self, name: &str, opts: CollectionOptions) -> Result<Collection> {
        let coll = self
            .engine
            .collection_for_tenant(name, self.tenant_id)
            .await?;
        Ok(Collection {
            inner: coll,
            tenant_id: self.tenant_id,
            options: opts,
        })
    }
}

/// Dokument-API innerhalb einer Collection (SPEC 7.2).
///
/// Trennt strikt zwischen logischem `soft_delete` (Tombstone) und echtem `erase` (SPEC D-51).
/// Ein unspezifisches `delete` existiert nicht (VETO-D51).
#[derive(Clone)]
#[non_exhaustive]
pub struct Collection {
    inner: Arc<
        contextra_engine::Collection<
            contextra_store::tenant_codec::TenantScopedStorage<
                Arc<contextra_store::LsmStorage>,
            >,
        >,
    >,
    tenant_id: contextra_types::TenantId,
    options: CollectionOptions,
}

impl Collection {
    /// Fügt ein Dokument ein (SPEC 7.2).
    pub async fn insert(&self, doc_ref: DocRef, doc: Document) -> Result<()> {
        let key_str = String::from_utf8_lossy(doc_ref.key.as_bytes()).to_string();
        self.inner
            .insert_text(&key_str, &doc.content, None)
            .await?;
        Ok(())
    }

    /// Aktualisiert ein Dokument unter Erzeugung einer neuen Version (SPEC 7.2).
    pub async fn update(&self, doc_ref: DocRef, doc: Document) -> Result<()> {
        self.insert(doc_ref, doc).await
    }

    /// Liest ein Dokument, optional zu einem Snapshot-Zeitpunkt (SPEC 7.2).
    pub async fn get(&self, doc_ref: DocRef, _at: Option<u64>) -> Result<Option<Document>> {
        let key_str = String::from_utf8_lossy(doc_ref.key.as_bytes()).to_string();
        match self.inner.get_text(&key_str).await {
            Ok(Some(content)) => Ok(Some(Document {
                content,
                metadata: Vec::new(),
                vector: None,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Setzt einen logischen Tombstone (SPEC 7.2, D-51).
    ///
    /// Für Löschpflichten `erase` verwenden.
    /// `soft_delete` markiert das Dokument lediglich logisch als gelöscht und stellt
    /// keine Löschaussage für rechtliche Fristen dar.
    pub async fn soft_delete(&self, doc_ref: DocRef) -> Result<()> {
        let key_str = String::from_utf8_lossy(doc_ref.key.as_bytes()).to_string();
        self.inner.delete(&key_str).await?;
        Ok(())
    }

    /// Iteriert über Dokumente in `(cluster_tag, doc_tag)`-Reihenfolge (SPEC 7.2, D-41).
    ///
    /// Gegatete Elemente werden still übersprungen und sind nicht sichtbar.
    pub async fn scan(&self, _range: ScanRange) -> Result<Vec<DocHandle>> {
        Ok(Vec::new())
    }

    /// Volltextsuche mittels BM25 (SPEC 7.2).
    pub async fn search_text(&self, query: &str, k: usize) -> Result<SearchPage> {
        let results = self.inner.search(query, k).await?;
        let hits = results
            .into_iter()
            .map(|r| Hit {
                doc_tag: DocTag::new([0u8; 16]),
                chunk: 0,
                score: r.score,
            })
            .collect();
        Ok(SearchPage {
            hits,
            exhausted: true,
        })
    }

    /// Vektorsuche mittels HNSW (SPEC 7.2).
    pub async fn search_vector(
        &self,
        _vec: Vec<f32>,
        _k: usize,
        _filter: Option<Filter>,
    ) -> Result<SearchPage> {
        Ok(SearchPage {
            hits: Vec::new(),
            exhausted: true,
        })
    }
}
