#![forbid(unsafe_code)]

//! `contextra-license`
//!
//! Lizenz- und Aktivierungs-Gate-Schicht für optionale Contextra-Feature-Ringe (§14.6, §15).
//!
//! # Ring-Zugehörigkeit und Architektur
//! - **Ring:** Ring 4 (Erweiterung / Enterprise-Features)
//! - **Klasse:** Erweiterung `[E]` (gemäß Spezifikation §5.2 / §6.0)
//! - **Status:** Archiv-Status laut Dokumentation, wird jedoch weiterhin als vollwertiges Workspace-Mitglied
//!   gepflegt und kompiliert.
//! - **Sicherheit:** `#![forbid(unsafe_code)]` – kein `unsafe` im gesamten Crate erlaubt oder vorhanden.
//!
//! # Abhängigkeiten
//! - **Eingehend (wird genutzt von):** `contextra` (Ring 4 Aggregator) und `contextra-mcp` (Model Context Protocol).
//! - **Ausgehend (nutzt):**
//!   - `contextra-types`: Gemeinsame Basis-Datentypen (z. B. [`contextra_types::TenantId`]).
//!   - `contextra-ports`: Port-Schnittstellen für Zeit ([`Clock`](contextra_ports::clock::Clock))
//!     und Lizenzierungs-Gates ([`LicenseGate`]).
//!   - Kryptografie: `ed25519-dalek` (Ed25519-Signaturprüfung) und `blake3` (kryptografische Hashfunktion).
//!   - Serialisierung: `serde` und `bincode`.
//!
//! # Hauptkomponenten
//! - [`SignedLicenseGate`]: Implementiert den [`LicenseGate`]-Port und prüft Lizenzrechte offline
//!   über ein 7-stufiges Fail-Closed-Verfahren.
//! - [`SignedActivation`]: Kryptografischer Aktivierungsdatensatz mit Ed25519-Signatur.
//! - [`LicensePayload`]: Strukturierter Lizenznutzlast-Datensatz (unterstützt auch Legacy-Formate).

// ANCHOR[contextra-license:LIB-REEXPORTS]
// Re-Exports der Port-Typen für Konsumenten des Crates
pub use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};

pub mod signed_gate;
pub use signed_gate::{
    constant_time_eq_32, derive_local_installation_id_hash, verify_step_1_fast_ring,
    verify_step_2_presence, verify_step_3_installation, verify_step_4_signature,
    verify_step_5_expiration, verify_step_6_ring_permission, verify_step_7_success, LicensePayload,
    SignedActivation, SignedLicenseGate, SignedLicenseOptions, StepVerificationError,
    MAX_PAYLOAD_BYTES,
};
