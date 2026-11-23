//! Produktive Kryptoprimitive des Speicherkerns (SPEC §4, §10.1, §16; IMPL IF-3, WP-2.4).
//!
//! Die Module liefern nur Primitive. Das Protokoll (Formate, Nonce-Reservierung,
//! Schlüsselhierarchie) liegt im Kern (SD-36).
//!
//! Port-Implementierungen stehen nur dort, wo der Port in `contextra-ports::persistence`
//! auf `main` bereits der Spec entspricht (`Mac`, `Kdf`). `Cipher` (24-Byte-Nonce, UE-10),
//! `Hasher` und `KeyProvider` fehlen in diesem Port-Stand; ihre Signaturen stehen hier als
//! inhärente Methoden, bis P1 den Port angleicht (siehe PR-Beschreibung, SA-4).

pub mod cipher;
pub mod hasher;
pub mod kdf;
pub mod mac;

pub use cipher::{XChaCha20Cipher, TAG_LEN};
pub use hasher::Blake3Hasher;
pub use kdf::HkdfSha256Kdf;
pub use mac::Blake3Mac;
