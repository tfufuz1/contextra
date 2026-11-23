//! Referenzimplementierung Cipher/Mac/Kdf/Checksum (SP-KEYS §1). Nur geprüfte Bibliotheken.
#![forbid(unsafe_code)]
pub mod checksum;
pub mod cipher;
pub mod kdf;
pub mod mac;
#[cfg(feature = "testing")]
pub mod mem_key_provider;
