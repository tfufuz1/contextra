// FILE-CONTEXT
// ZWECK: Cryptographic deletion proof for GDPR Article 17 compliance verification across storage layers.
// INVARIANTEN: INV-DELETION-1: DeletionProof::create() MUST only be invoked AFTER physical layer sanitization.
// NICHT-OFFENSICHTLICH: V1 signiert nur scope, key-hash und tx_id via HMAC-SHA256; V2 signiert scope, key-hash, tx_id, covered_layers, excluded_scopes und wal_receipt via HMAC-SHA256; V3 signiert mit Ed25519 und nutzt längenpräfixierte Key-Hashes. verify() nutzt bei HMAC-Versionen Constant-Time-Vergleiche.
// HOTSPOTS: [40-130]
// STAND: TS:2026-09-07T00:00:00Z

#![forbid(unsafe_code)]

//! Kryptographischer Löschbeweis für die Storage-Ebene.
//!
//! KRITISCHE DECKUNGSGRENZE (Pflicht in Enterprise-Doku und export_for_audit):
//! Dieser Proof deckt AUSSCHLIESSLICH die physischen Storage-Layer ab:
//! `LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`,
//! `KvCacheSegments` und `EmbeddingCache`.
//!
//! Explizit VOM PROOF AUSGESCHLOSSEN (`ExcludedScope`) sind:
//! `ConsolidatedAndDistilled` (Zusammenfassungen für LLM-Fine-Tuning) und
//! `LlmParameterMemory` (LLM-Modellparameter; arXiv:2505.16831).
//! Er KANN NICHT garantieren, dass Wissen aus Fine-Tuning-Zusammenfassungen
//! aus LLM-Parametern entfernbar ist. Referenz: arXiv:2505.16831.
//!
//! INVARIANTE INV-DELETION-1: DeletionProof::create() wird NUR nach
//! physischer Layer-Bereinigung aufgerufen. Proof vor Bereinigung = falsch.

pub mod audit_export;
pub mod builder;
pub mod canon;
pub mod claim_text;
pub mod evidence;
pub mod exclusions;
pub mod issuer;
pub mod keys;
pub mod layer_ids;
pub mod layer_proof;
pub mod policy;
pub mod probe;
pub mod proof;
pub mod proof_store;
pub mod proof_v2;
pub mod proof_v3;
pub mod proof_v4;
pub mod signer;
pub mod timestamp_anchor;
pub mod transparency_log;
pub mod types;
pub mod unverified;
pub mod verify;
pub mod wal_receipt;
pub mod wiring;

pub use self::builder::*;
pub use self::canon::*;
pub use self::claim_text::*;
pub use self::evidence::*;
pub use self::exclusions::*;
pub use self::issuer::*;
pub use self::keys::*;
pub use self::layer_proof::*;
pub use self::policy::*;
pub use self::probe::*;
pub use self::proof::{Coverage, DeletionProof, UncoveredLayer, UncoveredReason};
pub use self::proof_store::*;
pub use self::proof_v4::*;
pub use self::signer::*;
pub use self::types::*;
pub use self::unverified::*;
pub use self::wal_receipt::*;
pub use self::wiring::*;
