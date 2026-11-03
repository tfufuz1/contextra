//! Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O.
//!
//! # Architektur-Rolle: Ring 0 (Typen-Kern)
//!
//! Gemäß SPEC Abschnitt 5.1 und 8 sowie `docs/specs/01-contextra-types.md`.
//!
//! - Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O.
//! - Ring: 0
//! - Erlaubte Workspace-Abhängigkeiten: keine.
//! - Keine asynchronen Laufzeiten, kein I/O, kein `unsafe`.
//! - Wird eingebunden von: `contextra-engine`, `contextra`, `contextra-crypto`, `contextra-mvcc`,
//!   `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank`, `contextra-ports`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod erasure;
pub mod error;
pub mod error_dto;
pub mod file;
pub mod filter;
pub mod ids;
pub mod key;
pub mod labels;
pub mod limits;
pub mod model_fingerprint;
pub mod refs;
pub mod retrieval_strategy;
pub mod schema;
pub mod secret;
pub mod tags;
pub mod tenant_scope;
pub mod tombstone;
pub mod types;

pub use erasure::{
    Armed, ControlEvidence, EraseMode, EraseReceipt, EraseRequest, EraseTarget, ErasureStatus,
    GateVariant, KeyCoverage, LayerId, LayerWitness, MeasureClass, PendingReason, RefusalReason,
    ResidualRisk, Residue, RetentionAttestation, RetentionCode, RiskCondition, ScanMethod, Scanner,
    Timing, Unarmed,
};
pub use error::{
    CacheDirective, ConfigError, Conflict, ContextraError, Error, HnswDeletionError, Limit, Result,
    StepId,
};
pub use error_dto::ContextraErrorDto;
pub use file::{AlgIds, FileKind, FormatVersion};
pub use filter::FilterExpr;
pub use ids::{ErasureId, FileId, GateId, Seq, StoreId, Timestamp};
pub use key::{
    Family, KeyBound, KeyBuf, KeyRange, Matcher, MatcherItem, Region, KEY_BOUND_LEN, KEY_LEN,
};
pub use labels::{
    LABEL_COLL_TAG, LABEL_COMMIT, LABEL_DEK, LABEL_DEK_SALT, LABEL_DOC_TAG, LABEL_ENC, LABEL_FILE,
    LABEL_FKMASK, LABEL_HEADER_MAC, LABEL_IDX, LABEL_KEYSLOT_AAD, LABEL_LEDGER, LABEL_NAME_TAG,
    LABEL_RECORD_AAD, LABEL_SIGN, LABEL_SUBJECT_TAG, LABEL_TENANT_TAG, LABEL_TERM_TAG,
    LABEL_TEXT_BINDING_AAD, LABEL_UNIT, LABEL_VAULT_MAC, LABEL_WAL,
};
pub use limits::{
    MAX_DIMENSION, MAX_DOCS_PER_STORE, MAX_DOC_BYTES_DEFAULT, MAX_DOC_BYTES_HARD, MAX_K,
    MAX_QUERY_TERMS, MAX_SNAPSHOTS_PER_TENANT,
};
pub use model_fingerprint::ModelFingerprint;
pub use refs::{
    CollectionRef, DocHandle, DocKey, DocRef, OperatorRef, SubjectRef, TenantContext, TenantRef,
    MAX_REF_LEN,
};
pub use retrieval_strategy::RetrievalStrategy;
pub use schema::{DocIdWidth, ManifestSchemaVersion};
pub use secret::Secret;
pub use tags::{ClusterTag, CollTag, DocTag, NameTag, SubjectTag, TagPreimage, TenantTag, TermTag};
pub use tenant_scope::{TenantScopeViolation, TenantScoped};
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use types::{
    budget, domain, importance, saos, AutoExtractionMode, BudgetStrategy, CollectionId,
    ConfigFingerprint, ContextChunk, ContextWindow, DecayFunction, DistanceMetric, DocId, Edge,
    Embedding, Entity, EntityId, FusionStrategy, FusionWeights, GraphTraversalStrategy,
    HybridQuery, HybridQueryBuilder, ImportanceScore, LinkRelation, MemoryImportance, MemoryLink,
    MemoryType, OnSignalFailure, PprAlgorithm, PprConfig, QueryOptions, QueryOptionsBuilder,
    QueryRewriteOutput, RerankResult, Reservation, ResourceBudget, ResourceTracker, ScoredDocument,
    ScoredEntry, Signal, SignalFusionStrategies, TenantId, TlHfdParams, TokenBudget, TxId,
    WorkflowState, EXPIRY_METADATA_KEY, MAX_SEARCH_K, TOMBSTONE_BIT,
};
