//! acta-crypto: AEAD (XChaCha20-Poly1305), HKDF, HMAC, Ed25519, `Sealed<P>`, Schlüsseltypen,
//! Hash-Kette, Größenklassen-Padding.
//!
//! Level L1 · Budget 4200 LOC · Spezifikation docs/SPEC.md Kap. 4.1, 4.2, 4.5, 6.2.
//!
//! Extraktionsarchiv: `archive/acta-crypto/MANIFEST.md`. Übernommen sind die RFC-Testvektoren
//! für HKDF und HMAC. Der Umschlag, die Ableitungen und die Hash-Kette sind neu nach Spec gebaut.
//! Argon2id (`FileAnchor`, Spec 4.1) gehört nicht hierher, sondern zu `acta-vault`.
#![forbid(unsafe_code)]

pub mod aead;
pub mod domain;
pub mod error;
pub mod hash;
pub mod kdf;
pub mod key;
pub mod mac;
pub mod pad;
pub mod sign;
