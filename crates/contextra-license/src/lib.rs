#![forbid(unsafe_code)]

//! `contextra-license`
//!
//! Lizenz- und Aktivierungs-Gate-Schicht für optionale Contextra-Feature-Ringe (§14.6, §15).
//!
//! # Ring-Zugehörigkeit und Architektur
//! - **Ring:** Ring 4 (Erweiterung / Enterprise-Features)
//! - **Klasse:** Baustein Klasse A (gemäß Spezifikation v4.0 §2, §5)
//! - **Status:** Autarkes Lizenz-Validierungsmodul (bleibt erhalten, E-08).
//! - **Sicherheit:** `#![forbid(unsafe_code)]` – kein `unsafe` im gesamten Crate erlaubt oder vorhanden.
//!
//! # Abhängigkeiten
//! - **Klasse A (B1):** Keine internen Pflichtabhängigkeiten in `[dependencies]`.
//! - **Transitionale Kompatibilität:** Optionales Feature `ports-compat` bindet `contextra-ports` und
//!   `contextra-types` an, um bestehende Workspace-Konsumenten bis Welle 6 zu unterstützen.
//! - **Kryptografie:** `ed25519-dalek` (Ed25519-Signaturprüfung) und `blake3` (kryptografische Hashfunktion).
//! - **Serialisierung:** `serde` und `bincode`.
//!
//! # Hauptkomponenten
//! - [`SignedLicenseGate`]: Implementiert den [`LicenseGate`]-Trait und prüft Lizenzrechte offline
//!   über ein 7-stufiges Fail-Closed-Verfahren.
//! - [`SignedActivation`]: Kryptografischer Aktivierungsdatensatz mit Ed25519-Signatur.
//! - [`LicensePayload`]: Strukturierter Lizenznutzlast-Datensatz (unterstützt auch Legacy-Formate).
//! - [`LicenseGate`]: Crate-lokaler Trait zur Lizenzautorisierung.
//! - [`OpenFastGate`]: Offenes Default-Gate für den freien Fast-Ring.

pub mod types;
pub use types::{
    AuthorizedRing, Clock, FeatureRing, FixedClock, LicenseError, LicenseGate, OpenFastGate,
    Result, SystemClock, TenantId,
};

pub mod signed_gate;
pub use signed_gate::{
    constant_time_eq_32, derive_local_installation_id_hash, verify_step_1_fast_ring,
    verify_step_2_presence, verify_step_3_installation, verify_step_4_signature,
    verify_step_5_expiration, verify_step_6_ring_permission, verify_step_7_success, LicensePayload,
    SignedActivation, SignedLicenseGate, SignedLicenseOptions, StepVerificationError,
    MAX_PAYLOAD_BYTES,
};
